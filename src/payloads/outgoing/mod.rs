pub mod discord;
pub mod deploy;
pub mod sql;
pub mod frontend;

use std::collections::HashMap;

use actix_web::HttpResponse;
use reqwest::StatusCode;
use schemars::schema::SchemaObject;
use serde::Serialize;
use serde_json::json;

use crate::handlers::{ResponseFrom, OutgoingErr};
use crate::logging::*;

use super::incoming::{
    ToSql,
    ToDiscord,
    ToFrontend,
    ToDeploy,
};

#[derive(Debug, Clone)]
pub struct Outgoing {
    pub (crate) depl: Option<ResponseFrom<ToDeploy>>,
    pub (crate) disc: Option<ResponseFrom<ToDiscord>>,
    pub (crate) fron: Option<ResponseFrom<ToFrontend>>,
    pub (crate) sqll: Option<ResponseFrom<ToSql>>,
}

struct Details {
    code: u16,
    json: serde_json::Value,
}
impl Outgoing {
    fn get_details<Ok: Serialize, Err: OutgoingErr>(opt_res: Option<Result<Ok, Err>>) -> (bool, Option<Details>) {
        match opt_res {
            Some(Ok(res)) => {
                let Ok(json) = serde_json::to_value(res) else {
                    return (false, Some(Details { code: 500, json: json!("failed to send response") }));
                };
                let details = Details {
                    code: StatusCode::OK.as_u16(),
                    json
                };

                (true, Some(details))
            },
            Some(Err(e)) => {
                let status_code = e.status_code();
                let body = e.body();
                let details = match body {
                    Ok(json) => Details {
                        code: status_code,
                        json,
                    },
                    Err(err_str) => Details {
                        code: 500,
                        json: json!(err_str),
                    },
                };
                (false, Some(details))
            }
            None => (true, None),
        }
    }

    fn update_codes_and_map(
        name: &'static str,
        (ok, data): (bool, Option<Details>),
        map: &mut HashMap<&'static str, serde_json::Value>,
        bad_code_list: &mut Vec<u16>,
    ) -> Result<(), HttpResponse> {        
        if !ok {

            let Some(data) = data else {
                error!("The combination of error and no details should never occur!");
                info!("The error occurred in the {name} part of the response");
                return Err(HttpResponse::InternalServerError().body("Major server issue encountered. See logs for more info."));
            };
            
            info!("The {name} part of this response is a failure (code {})", data.code);

            bad_code_list.push(data.code);
            map.insert(name, data.json);
        } else if let Some(data) = data {
            map.insert(name, data.json);
        }
        Ok(())
    }

    pub fn response(self) -> HttpResponse {
        let mut bad_status_code_list = vec![];
        let mut json_map = HashMap::new();
        

        let depl = Self::update_codes_and_map(
            "deploy", Self::get_details(self.depl),
            &mut json_map, &mut bad_status_code_list,
        );
        let disc = Self::update_codes_and_map(
            "discord", Self::get_details(self.disc),
            &mut json_map, &mut bad_status_code_list,
        );
        let fron = Self::update_codes_and_map(
            "frontend", Self::get_details(self.fron),
            &mut json_map, &mut bad_status_code_list,
        );
        let sqll = Self::update_codes_and_map(
            "sql", Self::get_details(self.sqll),
            &mut json_map, &mut bad_status_code_list,
        );
        match (depl, disc, fron, sqll) {
            (Ok(()), Ok(()), Ok(()), Ok(())) => (),
            | (Err(e), _, _, _) 
            | (_, Err(e), _, _) 
            | (_, _, Err(e), _) 
            | (_, _, _, Err(e)) => return e,
        }

        if bad_status_code_list.is_empty() {
            info!("Response had no errors");
            HttpResponse
                ::build(StatusCode::OK)
                .json(json_map)
        } else {
            info!("Response had errors in {:?}", json_map.keys());

            let (client, server) = bad_status_code_list
                .iter()
                .fold((None, None), |(client, server), &code| {
                    (
                        client.or((400..500).contains(&code).then_some(code)),
                        server.or((500..600).contains(&code).then_some(code)),
                    )
                });

            let code = StatusCode
                ::from_u16(server.or(client).unwrap_or(500))
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            info!("Finalizing response with error code {code}");

            HttpResponse
                ::build(code)
                .json(json_map)
        }
    }
}


mod json_schema {
    use super::*;

    macro_rules! result_enum {
        (enum $name:ident { Ok($success_type:path), Err($err_type:path) }) => {
            #[derive(serde::Serialize, schemars::JsonSchema)]
            #[serde(tag = "ok", content = "data")]
            #[allow(unused, clippy::large_enum_variant)]
            enum $name {
                #[serde(rename = "success")]
                Ok($success_type),
                #[serde(rename = "err")]
                Err($err_type),
            }
        };
    }

    result_enum!(enum DeployResult { Ok(deploy::FromDeploy), Err(deploy::FromDeployErr) });
    result_enum!(enum DiscordResult { Ok(discord::FromDiscord), Err(discord::FromDiscordErr) });
    result_enum!(enum FrontendResult { Ok(frontend::FromFrontend), Err(frontend::FromFrontendErr) });
    result_enum!(enum SqlResult { Ok(sql::FromSql), Err(sql::FromSqlErr) });
    
    #[derive(schemars::JsonSchema)]
    #[allow(unused)]
    pub struct OutgoingSchemaShape {
        deploy: Option<DeployResult>,
        discord: Option<DiscordResult>,
        frontend: Option<FrontendResult>,
        sql: Option<SqlResult>,
    }
    
    impl schemars::JsonSchema for Outgoing {
        fn schema_name() -> String {
            "Outgoing".to_owned()
        }
    
        fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
            let schema = OutgoingSchemaShape::json_schema(gen);
            let mut schema = SchemaObject::from(schema);
    
            let mut metadata = schema.metadata().clone();
            metadata.title = Some("Outgoing".to_owned());
            schema.metadata = Some(Box::new(metadata));
    
            schema.into()
        }
    }
}
