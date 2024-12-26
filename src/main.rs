use clap::Parser;
use simple_logger::SimpleLogger;
use thiserror::Error;

mod configuration;
mod database;
mod server;
mod template;

const FILE_CONFIG: &str = "config.json";
const FILE_TEMPLATE: &str = "template.json";

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, env = "K4S_CONFIG", default_value = FILE_CONFIG)]
    config: String,

    #[arg(short, long, env = "K4S_TEMPLATE", default_value = FILE_TEMPLATE)]
    template: String,
}

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

async fn app() -> Result<(), StatusError> {
    let args = Args::parse();

    log::info!("Parse configuration ({})", args.config);
    let config = configuration::read_config(&args.config).await?;

    log::info!("Parse status template ({})", args.template);
    let template = template::read_template(&args.template).await?;

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

    app().await
}
