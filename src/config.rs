use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;
use aws_sdk_sqs::config::Region;
use aws_sdk_sqs::Client;
use std::env;

use dotenvy::dotenv;
use tokio::sync::OnceCell;

#[derive(Debug)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug)]
struct DatabaseConfig {
    url: String,
}

#[derive(Debug)]
pub struct Config {
    server: ServerConfig,
    db: DatabaseConfig,
    sqs_client: Client,
}

impl Config {
    pub fn db_url(&self) -> &str {
        &self.db.url
    }

    pub fn server_host(&self) -> &str {
        &self.server.host
    }

    pub fn server_port(&self) -> u16 {
        self.server.port
    }

    pub fn sqs_client(&self) -> &Client {
        &self.sqs_client
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn init_config() -> Config {
    dotenv().ok();
    // init server config
    let server_config = ServerConfig {
        host: env::var("HOST").unwrap_or_else(|_| String::from("127.0.0.1")),
        port: env::var("PORT").unwrap_or_else(|_| String::from("3000")).parse::<u16>().unwrap(),
    };

    // init database config
    let database_config = DatabaseConfig { url: env::var("DATABASE_URL").expect("DATABASE_URL must be set") };

    // init AWS config
    let shared_config = aws_config::from_env().load().await;

    // init AWS SQS
    let sqs_client = Client::new(&shared_config);

    Config { server: server_config, db: database_config, sqs_client }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}
