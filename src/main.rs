use arcs_logging_rs::{DEFAULT_LOGGING_TARGETS, set_up_logging};

use webhook_rs::handlers::Handle as _;

use webhook_rs::env;

use webhook_rs::logging::*;

use actix_web::{
    HttpServer, App, Responder,
    web::Json,
};
use webhook_rs::setup::setup;
use webhook_rs::setup::main_route;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let clean_up_logging = setup(true, true).await;

    let ip = "0.0.0.0";
    let port = env::port().parse().unwrap();

    let res = HttpServer::new(|| {
        App::new()
            .service(main_route)
    })
        .bind((ip, port))?
        .run()
        .await;


    if let Err(e) = &res {
        error!("Failed to start server.");
        error!("Error: {e}");
    }
    clean_up_logging();

    res
}
