use simple_logger::SimpleLogger;
use thiserror::Error;
use clap::Parser;

mod configuration;
mod database;
mod server;
mod template;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    config: String,

    /// Number of times to greet
    #[arg(short, long)]
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

const FILE_CONFIG: &str = "config.json";
const FILE_TEMPLATE: &str = "status.json";

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

    match app().await {
        Ok(_) => panic!(),
        Err(err) => {
            log::error!("{}", err);
            Err(err)
        }
    }
}
