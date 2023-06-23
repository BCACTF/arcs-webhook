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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().unwrap();
    set_up_logging(&DEFAULT_LOGGGING_TARGETS, "Webhook").unwrap();
    start_db_connection().await;

    
    if let Err(e) = env::check_env_vars() {
        error!("Failed to find env variables {e}. Aborting...");
        std::process::exit(1);
    }
    if let Err(e) = env::discord::check_env_vars() {
        error!("Failed to find discord env variables {e}. Aborting...");
        std::process::exit(1);
    }


    HttpServer::new(|| {
        App::new()
            .service(main_route)
    })
        .bind(("127.0.0.1", env::port().parse().unwrap()))?
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
