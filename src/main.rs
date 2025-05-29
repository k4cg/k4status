use badge::Badges;
use clap::Parser;
use configuration::Configuration;
use database::Database;
use icon::Icons;
use serde::Deserialize;
use simple_logger::SimpleLogger;
use spaceapi::SpaceApi;
use thiserror::Error;

mod badge;
mod configuration;
mod database;
mod icon;
mod server;
mod spaceapi;

const FILE_CONFIG: &str = "config.json";
const FILE_TEMPLATE: &str = "template.json";
const DIR_BADGES: &str = "assets/badges/";
const DIR_ICONS: &str = "assets/icons/";

async fn read_file<T>(fname: &str) -> Result<T, StatusError>
where
    T: for<'a> Deserialize<'a>,
{
    tokio::fs::read_to_string(fname)
        .await
        .map(|s| serde_json::from_str(&s))
        .map_err(|e| StatusError::File(format!("Failed to read {} ({})", fname, e)))?
        .map_err(|e| StatusError::File(format!("Failed to parse {} ({})", fname, e)))
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long, env = "K4S_CONFIG", default_value = FILE_CONFIG)]
    config: String,

    #[arg(short, long, env = "K4S_TEMPLATE", default_value = FILE_TEMPLATE)]
    template: String,

    #[arg(short, long, env = "K4S_BADGES", default_value = DIR_BADGES)]
    badges: String,

    #[arg(short, long, env = "K4S_ICONS", default_value = DIR_ICONS)]
    icons: String,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum StatusError {
    #[error("Failed to interact with database: {0}")]
    Database(String),

    #[error("Failed to read/parse file: {0}")]
    File(String),

    #[error("Failed to start server: {0}")]
    Server(String),
}

async fn app() -> Result<(), StatusError> {
    let args = Args::parse();

    log::info!("Parse configuration ({})", args.config);
    let config: Configuration = read_file(&args.config).await?;

    log::info!("Parse status template ({})", args.template);
    let template: SpaceApi = read_file(&args.template).await?;

    log::info!("Read badges ({})", args.badges);
    let badges = Badges::new(&args.badges).await?;

    log::info!("Read icons ({})", args.icons);
    let icons = Icons::new(&args.icons).await?;

    log::info!("Initialize database connection");
    let database = Database::new(&config.database);

    log::info!("Start server");
    server::run(&config, &database, &template, &badges, &icons).await
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
        read_file::<SpaceApi>(FILE_TEMPLATE).await.unwrap();
    }
}
