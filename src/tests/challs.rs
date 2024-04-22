use actix_web::App;
use assert_json::{ assert_json, validators, Validator };
use serde_json::{ json, Value };
use super::{
    json::{ JsonAccessors, PseudoIndex },
    utils::{ setup_env, rand_chars },
    req::exec_post,
    defaults::chall as chall_default,
};

use crate::{setup::{ main_route, setup }, sql::connection};


fn arr_eq<
    const LEN: usize,
    T: Into<Value> + Clone + std::fmt::Debug + 'static,
>(expected: [T; LEN]) -> impl Validator {
    validators::array(
        expected
            .into_iter()
            .map(|val| -> Box<dyn Validator> {
                Box::new(validators::eq(val))
            })
            .collect(),
    )
}

#[actix_web::test]
async fn create_update_get_delete() {
    setup_env();
    (setup(false, false).await)();
    let app = actix_web::test::init_service::<_, _, _, _>(App::new().service(main_route)).await;

    // Create chall
    let create_res = exec_post(
        json!({
            "sql": {
                "__type": "chall",
                "details": {
                    "__query_name": "create",
                    "params": {
                        "name": chall_default::NAME,
                        "description": chall_default::DESCRIPTION,
                        "points": chall_default::POINTS,
                        "authors": chall_default::AUTHORS,
                        "hints": chall_default::HINTS,
                        "categories": chall_default::CATEGORIES,
                        "tags": chall_default::TAGS,
                        "links": [],
                
                        "visible": chall_default::VISIBLE,
                        "source_folder": chall_default::SOURCE_FOLDER,
                
                        "flag": chall_default::FLAG,
                    },
                },
            },
        }),
        true,
        &app,
        "Create Challenge by ID",
    ).await;

    let id = access!(create_res.sql.data.data.id (str)).map(str::to_string);
    let id = id.and_then(|s| uuid::Uuid::parse_str(&s).ok());
    assert_json!(
        create_res,
        {
            "sql": {
                "data": {
                    "__type": "chall",
                    "data": {
                        "id": validators::string(|s| {
                            let uuid = uuid::Uuid::parse_str(s).map_err(|e| format!("Failed to parse UUID: {e}"))?;
                            if uuid.is_nil() {
                                Err("UUID is nil".to_string())
                            } else {
                                Ok(())
                            }
                        }),
                        "name": chall_default::NAME,
                        "points": chall_default::POINTS,
                        "description": chall_default::DESCRIPTION,
                        "solve_count": 0,
                        "source_folder": chall_default::SOURCE_FOLDER,
                        "visible": chall_default::VISIBLE,
                        "categories": arr_eq(chall_default::CATEGORIES),
                        "hints": arr_eq(chall_default::HINTS),
                        "authors": arr_eq(chall_default::AUTHORS),
                        "tags": arr_eq(chall_default::TAGS),
                    },
                },
            },
        }
    );

    //
    // Update chall
    //
    let new_name = format!("New Name {}", rand_chars(10, "aeiouybrtlsdch"));
    let update_res = exec_post(
        json!({
            "sql": {
                "__type": "chall",
                "details": {
                    "__query_name": "update",
                    "params": {
                        "id": id,
                        "name": new_name,
                        "visible": !chall_default::VISIBLE,
                    },
                },
            },
        }),
        true,
        &app,
        "Update Challenge by ID",
    ).await;

    assert_json!(
        update_res.clone(),
        {
            "sql": {
                "data": {
                    "__type": "chall",
                    "data": {
                        "id": id.unwrap_or(uuid::Uuid::nil()).to_string(),
                        "name": new_name,
                        "visible": !chall_default::VISIBLE,
                    },
                },
            },
        }
    );

    //
    // Get chall
    //
    let get_res = exec_post(
        json!({
            "sql": {
                "__type": "chall",
                "details": {
                    "__query_name": "get",
                    "params": {
                        "id": id,
                    },
                },
            },
        }),
        true,
        &app,
        "Get Challenge by ID",
    ).await;

    assert_eq!(get_res, update_res);

    // Cleanup
    let id = id.expect("Failed to get ID of test chall to clean up");
    use sqlx::prelude::Executor;
    connection()
        .await
        .expect("Failed to get DB handle to clean up test chall").execute(
            sqlx::query("DELETE FROM challenges WHERE id = $1").bind(id),
        ).await.expect("Failed to clean up test chall");
}
