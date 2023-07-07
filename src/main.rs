use arcs_logging_rs::{DEFAULT_LOGGGING_TARGETS, set_up_logging};

use webhook_rs::handlers::Handle as _;

use webhook_rs::env;

use webhook_rs::logging::*;

use actix_web::{
    HttpServer, App, Responder,
    web::Json,
};
use webhook_rs::start_db_connection;

macro_rules! verify_envs {
    ($($fn:path: $name:literal),+ $(,)?) => {
        {
            let mut failed = false;
            $(
                if let Err(e) = $fn() {
                    error!("Failed to find {} env variables {e}", $name);
                    failed |= true;
                }
            )+

            if failed {
                error!("Aborting...");
                std::process::exit(1);
            }
        }
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().unwrap();
    set_up_logging(&DEFAULT_LOGGGING_TARGETS, "Webhook").unwrap();

    {
        use env::checks::*;
        verify_envs!(
            main: "main",
            sql: "sql",
            discord: "discord",
            auth: "auth",
        );
    }
    
    if let Err(e) = start_db_connection().await {
        error!("Failed to initialize database connection.");
        error!("Error: {e}");
        error!("Aborting...");
        std::process::exit(1);
    }


    let ip = "0.0.0.0";
    let port = env::port().parse().unwrap();

    HttpServer::new(|| {
        App::new()
            .service(main_route)
    })
        .bind((ip, port))?
        .run()
        .await
}


use actix_web::{ web::Header, HttpResponse };
use serde_json::json;
use webhook_rs::{
    AuthHeader, Token,
    payloads::incoming::Incoming,
};


#[actix_web::post("/")]
async fn main_route(json: Json<Incoming>, authorization: Header<AuthHeader>) -> impl Responder {
    if authorization.0.check_matches(&[ Token::Frontend, Token::Deploy ]) {
        json
            .into_inner()
            .handle()
            .await
            .unwrap()
            .response()
    } else {
        // TODO: More accurate error messages
        HttpResponse::Unauthorized()
            .json(json!({ "error": "Improper bearer authentication" }))
    }

}
