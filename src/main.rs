use arcs_logging_rs::{DEFAULT_LOGGGING_TARGETS, set_up_logging};

use webhook_rs::handlers::Handle;
use webhook_rs::payloads::incoming::Incoming;

use webhook_rs::env;

use webhook_rs::logging::*;

use actix_web::{
    HttpServer, App, Responder,
    web::Json,
};
use webhook_rs::start_db_connection;

macro_rules! verify_env {
    ($fn:path: $name:literal) => {
        if let Err(e) = $fn() {
            error!("Failed to find {} env variables {e}. Aborting...", $name);
            std::process::exit(1);
        }
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().unwrap();
    set_up_logging(&DEFAULT_LOGGGING_TARGETS, "Webhook").unwrap();

    {
        use env::checks::*;
        verify_env!(main: "main");
        verify_env!(sql: "sql");
        verify_env!(discord: "discord");
        verify_env!(auth: "auth");
    }


    start_db_connection().await;


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

#[actix_web::post("/")]
async fn main_route(json: Json<Incoming>) -> impl Responder {
    json
        .into_inner()
        .handle()
        .await
        .unwrap()
        .response()
}
