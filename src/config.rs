use std::env;

use aws_sdk_s3::Client as S3Client;
use aws_sdk_sqs::Client as SqsClient;
use deadpool_diesel::postgres::{Manager, Pool};
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
    sqs_client: SqsClient,
    s3_client: S3Client,
    pool: Pool,
}

impl Config {
    pub fn server_host(&self) -> &str {
        &self.server.host
    }

    pub fn server_port(&self) -> u16 {
        self.server.port
    }

    pub fn sqs_client(&self) -> &SqsClient {
        &self.sqs_client
    }

    pub fn s3_client(&self) -> &S3Client {
        &self.s3_client
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
    let sqs_client = SqsClient::new(&shared_config);

    // init AWS S3 client
    let s3_client = S3Client::new(&shared_config);

    let manager = Manager::new(database_config.url.to_string(), deadpool_diesel::Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    Config { server: server_config, sqs_client, s3_client, pool }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}
