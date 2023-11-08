pub mod discord;
pub mod deploy;
pub mod sql;
pub mod frontend;

use actix_web::HttpResponse;
use reqwest::StatusCode;
use schemars::schema::SchemaObject;
use serde::Serialize;

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

impl Outgoing {
    fn get_code<T, Err: OutgoingErr>(opt_res: &Option<Result<T, Err>>) -> Option<u16> {
        match opt_res {
            Some(Ok(res)) => {
                Some(200)
            },
            Some(Err(e)) => {
                let status_code = e.status_code();
                Some(status_code)
            }
            None => None,
        }
    }

    pub fn response(self) -> HttpResponse {
        let mut bad_status_code_list = vec![];
        

        if let Some(deploy_code) = Self::get_code(&self.depl) {
            if !(200..300).contains(&deploy_code) {
                bad_status_code_list.push(deploy_code);
            }
        };
        if let Some(discord_code) = Self::get_code(&self.disc) {
            if !(200..300).contains(&discord_code) {
                bad_status_code_list.push(discord_code);
            }
        };
        if let Some(frontend_code) = Self::get_code(&self.fron) {
            if !(200..300).contains(&frontend_code) {
                bad_status_code_list.push(frontend_code);
            }
        };
        if let Some(sql_code) = Self::get_code(&self.sqll) {
            if !(200..300).contains(&sql_code) {
                bad_status_code_list.push(sql_code);
            }
        };

        if bad_status_code_list.is_empty() {
            info!("Response had no errors");
            HttpResponse
                ::build(StatusCode::OK)
                .json(json_schema::OutgoingSchemaShape::from(self))
        } else {
            info!("Response had errors");

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
                .json(json_schema::OutgoingSchemaShape::from(self))
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
            pub(super) enum $name {
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
    
    #[derive(Serialize, schemars::JsonSchema)]
    #[allow(unused)]
    pub struct OutgoingSchemaShape {
        pub(super) deploy: Option<DeployResult>,
        pub(super) discord: Option<DiscordResult>,
        pub(super) frontend: Option<FrontendResult>,
        pub(super) sql: Option<SqlResult>,
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


    impl From<Outgoing> for OutgoingSchemaShape {
        fn from(out: Outgoing) -> Self {
            Self {
                deploy: out.depl.map(|res| match res {
                    Ok(ok) => DeployResult::Ok(ok),
                    Err(err) => DeployResult::Err(err),
                }),
                discord: out.disc.map(|res| match res {
                    Ok(ok) => DiscordResult::Ok(ok),
                    Err(err) => DiscordResult::Err(err),
                }),
                frontend: out.fron.map(|res| match res {
                    Ok(ok) => FrontendResult::Ok(ok),
                    Err(err) => FrontendResult::Err(err),
                }),
                sql: out.sqll.map(|res| match res {
                    Ok(ok) => SqlResult::Ok(ok),
                    Err(err) => SqlResult::Err(err),
                }),
            }
        }
    }
}
