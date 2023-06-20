pub mod discord;
pub mod deploy;
pub mod sql;
pub mod frontend;

use std::collections::HashMap;

use actix_web::HttpResponse;
use reqwest::StatusCode;
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

#[derive(Debug, Clone, Serialize)]
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
        pair: (bool, Option<Details>),
        map: &mut HashMap<&'static str, serde_json::Value>,
        bad_code_list: &mut Vec<u16>,
    ) -> Result<(), HttpResponse> {
        if !pair.0 {
            let Some(data) = pair.1 else {
                return Err(HttpResponse::InternalServerError().body("Major server issue encountered. See logs for more info."));
            };
            bad_code_list.push(data.code);
            map.insert(name, data.json);
        } else if let Some(data) = pair.1 {
            map.insert(name, data.json);
        }
        Ok(())
    }

    pub fn response(self) -> HttpResponse {
        let mut bad_status_code_list = vec![];
        let mut json_map = HashMap::new();
        
        debug!("Outgoing: {self:#?}");
        

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
            HttpResponse
                ::build(StatusCode::OK)
                .json(json_map)
        } else {
            let (client, server) = bad_status_code_list
                .iter()
                .fold((None, None), |(client, server), &code| {
                    (
                        client.or((400 <= code && code < 500).then_some(code)),
                        server.or((500 <= code && code < 600).then_some(code)),
                    )
                });

            let code = StatusCode
                ::from_u16(server.or(client).unwrap_or(500))
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            HttpResponse
                ::build(code)
                .json(json_map)
        }
    }
}
