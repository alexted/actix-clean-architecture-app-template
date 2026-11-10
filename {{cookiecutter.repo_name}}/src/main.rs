//! Process entrypoint: reads the settings, initializes logging and the database pool,
//! then serves the application factory with the composition root.

use std::sync::Arc;

use actix_web::HttpServer;

use actix_clean_architecture::container::Container;
use actix_clean_architecture::infrastructure::clients::postgres::engine::init_database;
use actix_clean_architecture::infrastructure::core::application::create_app;
use actix_clean_architecture::infrastructure::core::logging_config::init_logging;
use actix_clean_architecture::infrastructure::core::settings::get_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = get_config();
    init_logging(config);
    init_database(config);

    let container = Arc::new(Container::new());

    HttpServer::new(move || create_app(container.clone()))
        .bind((config.server_host.as_str(), config.server_port))?
        .run()
        .await
}
