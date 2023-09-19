use aws_config::meta::region::RegionProviderChain;
use aws_config::SdkConfig;
use aws_sdk_sqs::config::Region;
use aws_sdk_sqs::Client;
use deadpool_diesel::postgres::{Manager, Object, Pool};
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

pub struct Config {
    server: ServerConfig,
    db: DatabaseConfig,
    sqs_client: Client,
    aws_config: SdkConfig,
    pool: Pool,
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

    pub fn aws_config(&self) -> &SdkConfig {
        &self.aws_config
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
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

    let manager = Manager::new(database_config.url.to_string(), deadpool_diesel::Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    Config { server: server_config, db: database_config, sqs_client, aws_config: shared_config, pool }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}
