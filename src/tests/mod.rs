#![allow(clippy::unwrap_used)]

use actix_web::{body::{to_bytes, BoxBody}, dev::ServiceResponse};
use serde::Serialize;
use serde_json::Value;

fn setup_env() {
    #[allow(clippy::unwrap_used)]
    dotenvy::dotenv().unwrap();

    std::env::set_var("PORT", "3000");

    std::env::set_var("FRONTEND_AUTH_TOKEN", "F".repeat(64));
    std::env::set_var("WEBHOOK_AUTH_TOKEN", "W".repeat(64));
    std::env::set_var("DEPLOY_AUTH_TOKEN", "D".repeat(64));
    std::env::set_var("ALLOWED_OAUTH_TOKEN", "A".repeat(64));

    std::env::set_var("FRONTEND_ADDRESS", "");
    std::env::set_var("WEBHOOK_ADDRESS", "");
    std::env::set_var("DEPLOY_ADDRESS", "");

    std::env::set_var("DEPLOY_ADDRESS", "");
}

async fn try_parse_json(res: ServiceResponse<BoxBody>) -> Result<Value, String> {
    let body = res.into_body();
    let body = to_bytes(body).await;
    assert!(body.is_ok());

    let body = body.map_err(|e| e.to_string())?;
    let body: Vec<u8> = body.into_iter().collect();
    let body = String::from_utf8_lossy(&body).trim().to_string();
    
    serde_json::from_str::<Value>(&body).ok().ok_or(body)
}

fn post_req(body: impl Serialize, from_deploy: bool) -> actix_web::test::TestRequest {
    use actix_web::test::TestRequest;

    let auth = (
        "Authorization",
        format!(
            "Bearer {}",
            if from_deploy {
                "D".repeat(64)
            } else {
                "F".repeat(64)
            }
        ),
    );

    TestRequest::post()
        .uri("/")
        .insert_header(auth)
        .set_json(body)
}

async fn exec_post(
    body: impl Serialize,
    from_deploy: bool,
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
) -> Value {
    let req = post_req(body, from_deploy);
    let res = req.send_request(app).await;
    assert_eq!(res.status().as_u16(), 200, "Response body: {:?}", try_parse_json(res).await);
    
    let res = try_parse_json(res).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    res.unwrap()
}

mod json_trait {
    use serde_json::{Map, Value};

    pub trait JsonAccessors<'a> {
        fn null(self) -> Option<()>;
        fn bool(self) -> Option<bool>;
    
        fn float(self) -> Option<f64>;
        fn int(self) -> Option<u64>;
        fn str(self) -> Option<&'a str>;
        
        fn arr(self) -> Option<&'a [Value]>;
        fn obj(self) -> Option<&'a Map<String, Value>>;
    }
    
    impl<'a> JsonAccessors<'a> for &'a Value {
        fn null(self) -> Option<()> { Some(self).null() }
        fn bool(self) -> Option<bool> { Some(self).bool() }
        fn float(self) -> Option<f64> { Some(self).float() }
        fn int(self) -> Option<u64> { Some(self).int() }
        fn str(self) -> Option<&'a str> { Some(self).str() }
        fn arr(self) -> Option<&'a [Value]> { Some(self).arr() }
        fn obj(self) -> Option<&'a Map<String, Value>> { Some(self).obj() }
    }
    impl<'a> JsonAccessors<'a> for Option<&'a Value> {
        fn null(self) -> Option<()> { self?.as_null() }
        fn bool(self) -> Option<bool> { self?.as_bool() }
        fn float(self) -> Option<f64> { self?.as_f64() }
        fn int(self) -> Option<u64> { self?.as_u64() }
        fn str(self) -> Option<&'a str> { self?.as_str() }

        fn arr(self) -> Option<&'a [Value]> {
            self?.as_array().map(Vec::as_slice)
        }
        fn obj(self) -> Option<&'a Map<String, Value>> { self?.as_object() }
    }

    pub trait PseudoIndex<'a, T> {
        fn index(&self, index: T) -> Option<&'a Value>;
    }


    impl<'a> PseudoIndex<'a, usize> for &'a [Value] {
        fn index(&self, index: usize) -> Option<&'a Value> {
            Some(*self).index(index)
        }
    }
    impl<'a> PseudoIndex<'a, usize> for Option<&'a [Value]> {
        fn index(&self, index: usize) -> Option<&'a Value> {
            (*self)?.get(index)
        }
    }

    impl<'a> PseudoIndex<'a, &str> for &'a Map<String, Value> {
        fn index(&self, index: &str) -> Option<&'a Value> {
            Some(*self).index(index)
        }
    }
    impl<'a> PseudoIndex<'a, &str> for Option<&'a Map<String, Value>> {
        fn index(&self, index: &str) -> Option<&'a Value> {
            (*self)?.get(index)
        }
    }
}

#[macro_export]
macro_rules! access {
    ($value:ident$($rest:tt)*) => {
        access!(@impl ($value) | $($rest)*)
    };
    (@impl ($curr:expr) | .$prop:ident$($rest:tt)*) => {
        access!(@impl (($curr).obj().index(stringify!($prop))) | $($rest)*)
    };
    (@impl ($curr:expr) | [$idx:literal]$($rest:tt)*) => {
        access!(@impl (($curr).arr().index($idx)) | $($rest)*)
    };
    (@impl ($curr:expr) | $(($final_fn:ident))?) => {
        $curr$(.$final_fn())?
    };
}


#[macro_export]
macro_rules! assert_json_shape {
    (
        $value:ident
        $(
            {$($accessors:tt)+} ($($matcher:tt)+),
        )+
    ) => {
        #[allow(clippy::redundant_pattern_matching)]
        {
            $(
                assert_json_shape!(@impl ($value $($accessors)+) | $($matcher)+);
            )+
        }
    };
    (@impl ($value:ident $($accessors:tt)+) | do |$closure_ident:ident| $closure_body:expr) => {
        let res = (|$closure_ident| $closure_body)(
            access!($value $($accessors)+)
        );
        assert!(res.is_some());
        assert!(res.unwrap());
    };

    (@impl ($value:ident $($accessors:tt)+) | == $to_match_against:expr) => {
        assert_json_shape!(@impl ($value $($accessors)+) | =? Some($to_match_against))
    };
    (@impl ($value:ident $($accessors:tt)+) | =? $to_match_against:expr) => {
        assert_eq!(
            access!($value $($accessors)+),
            $to_match_against,
        )
    };

    (@impl ($value:ident $($accessors:tt)+) | match $to_match_against:pat) => {
        assert!(
            matches!(
                access!($value $($accessors)+),
                $to_match_against,
            )
            // "Failed to match pattern",
        )
    };
}

mod challs {
    use actix_web::App;
    use serde_json::json;
    use super::{
        json_trait::JsonAccessors,
        json_trait::PseudoIndex,
        setup_env,
        exec_post,
    };

    use crate::{setup::{ main_route, setup }, sql::connection};

    mod defaults {
        pub const NAME: &str = "Test Challenge";
        pub const DESCRIPTION: &str = "Test Desc";
        pub const POINTS: u64 = 25;
        pub const VISIBLE: bool = true;
        pub const SOURCE_FOLDER: &str = "test_chall";
        pub const FLAG: &str = "test_ctf{t35t_f18g}";

        pub const AUTHORS: [&str; 1] = [ "Example Author" ];
        pub const HINTS: [&str; 0] = [ ];
        pub const CATEGORIES: [&str; 1] = [ "webex" ];
        pub const TAGS: [&str; 1] = [ "example-tag" ];
    }

    #[actix_web::test]
    async fn e2e() {
        setup_env();
        (setup(false, false).await)();
        let app = actix_web::test::init_service::<_, _, _, _>(App::new().service(main_route)).await;

        let create_res = exec_post(
            json!({
                "sql": {
                    "__type": "chall",
                    "details": {
                        "__query_name": "create",
                        "params": {
                            "name": defaults::NAME,
                            "description": defaults::DESCRIPTION,
                            "points": defaults::POINTS,
                            "authors": defaults::AUTHORS,
                            "hints": defaults::HINTS,
                            "categories": defaults::CATEGORIES,
                            "tags": defaults::TAGS,
                            "links": [],
                    
                            "visible": defaults::VISIBLE,
                            "source_folder": defaults::SOURCE_FOLDER,
                    
                            "flag": defaults::FLAG,
                        },
                    },
                },
            }),
            true,
            &app,
        ).await;

        assert_json_shape!(
            create_res
            {.sql} (match Some(_)),
            {.sql.data} (match Some(_)),
            {.sql.data.__type (str)} (== "chall"),
        );

        let create_chall = access!(create_res.sql.data.data);
        assert_json_shape!(
            create_chall

            {.id            (str) } (do |id| Some(!uuid::Uuid::parse_str(id?).ok()?.is_nil())),
            {.name          (str) } (== defaults::NAME),
            {.points        (int) } (== defaults::POINTS),
            {.description   (str) } (== defaults::DESCRIPTION),
            {.solve_count   (int) } (== 0),
            {.source_folder (str) } (== defaults::SOURCE_FOLDER),
            {.visible       (bool)} (== defaults::VISIBLE),
            {.flag          (str) } (match None), // Shouldn't return the flag

            {.categories[0] (str) } (=? defaults::CATEGORIES.first().copied()),
            {.hints[0]      (str) } (=? defaults::HINTS.first().copied()),
            {.authors[0]    (str) } (=? defaults::AUTHORS.first().copied()),
            {.tags[0]       (str) } (=? defaults::TAGS.first().copied()),
        );

        
        let mut chars = ['\0'; 10];
        rand::Rng::fill(&mut rand::thread_rng(), &mut chars);
        let rand_name: String = chars
            .into_iter()
            .map(|c| ((c as u32 % 26) as u8 + b'a') as char)
            .collect();
        let new_name = format!("Random Name {rand_name}");

        let update_res = exec_post(
            json!({
                "sql": {
                    "__type": "chall",
                    "details": {
                        "__query_name": "update",
                        "params": {
                            "id": access!(create_chall.id (str)),
                            "name": new_name,
                            "visible": !defaults::VISIBLE,
                        },
                    },
                },
            }),
            true,
            &app,
        ).await;

        assert_json_shape!(
            update_res
            {.sql} (match Some(_)),
            {.sql.data} (match Some(_)),
            {.sql.data.__type (str)} (== "chall"),
        );

        let update_chall = access!(update_res.sql.data.data);
        assert_json_shape!(
            update_chall
            {.id      (str) } (=? access!(create_chall.id (str))),
            {.name    (str) } (== new_name.as_str()),
            {.visible (bool)} (== !defaults::VISIBLE),
        );

        use sqlx::prelude::Executor;
        connection().await.unwrap().execute(
            sqlx::query(
                "DELETE FROM challenges WHERE id = $1",
            ).bind(
                uuid::Uuid::parse_str(access!(create_chall.id (str)).unwrap()).unwrap(),
            ),
        ).await.unwrap();
    }
}
