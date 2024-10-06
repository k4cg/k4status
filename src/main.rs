use simple_logger::SimpleLogger;
use thiserror::Error;

mod configuration;
mod database;
mod server;
mod template;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum StatusError {
    #[error("Failed to interact with database: {0}")]
    Database(String),

    #[error("Invalid template: {0}")]
    Template(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("Failed to start server: {0}")]
    Server(String),
}

const FILE_CONFIG: &str = "config.json";
const FILE_TEMPLATE: &str = "status.json";

async fn app() -> Result<(), StatusError> {
    log::info!("Parse configuration ({})", FILE_CONFIG);
    let config = configuration::read_config(FILE_CONFIG).await?;

    log::info!("Parse status template ({})", FILE_TEMPLATE);
    let template = template::read_template(FILE_TEMPLATE).await?;

    log::info!("Initialize database connection");
    let database = database::Database::new(&config.database);

    log::info!("Start server");
    server::run(&config, &database, &template).await
}

#[tokio::main]
async fn main() -> Result<(), StatusError> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .expect("Logger already initialized");

    match app().await {
        Ok(_) => panic!(),
        Err(err) => {
            log::error!("{}", err);
            Err(err)
        }
    }
}
