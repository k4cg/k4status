use clap::Parser;
use configuration::Configuration;
use serde::Deserialize;
use simple_logger::SimpleLogger;
use thiserror::Error;

mod configuration;
mod database;
mod server;

const FILE_CONFIG: &str = "config.json";
const FILE_TEMPLATE: &str = "template.json";

async fn read_file<T>(fname: &str) -> Result<T, StatusError>
where
    T: for<'a> Deserialize<'a>,
{
    tokio::fs::read_to_string(fname)
        .await
        .map(|s| serde_json::from_str(&s))
        .map_err(|e| StatusError::Template(format!("Failed to read {} ({})", fname, e)))?
        .map_err(|e| StatusError::Template(format!("Failed to parse {} ({})", fname, e)))
}

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
    let config: Configuration = read_file(&args.config).await?;

    log::info!("Parse status template ({})", args.template);
    let template: spaceapi::Status = read_file(&args.template).await?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_config() {
        read_file::<Configuration>(FILE_CONFIG).await.unwrap();
    }

    #[tokio::test]
    async fn parse_template() {
        read_file::<spaceapi::Status>(FILE_TEMPLATE).await.unwrap();
    }
}
