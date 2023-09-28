use std::env;
use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;
use aws_sdk_sqs::Client as SqsClient;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
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
    pool: Arc<Pool<AsyncPgConnection>>,
    db_config: DatabaseConfig,
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

    pub fn pool(&self) -> &Arc<Pool<AsyncPgConnection>> {
        &self.pool
    }

    pub fn db_url(&self) -> &str {
        &self.db_config.url
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

    // create a new connection pool with the default config
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(database_config.url.to_string());
    let pool = Pool::builder(manager).build().unwrap();

    Config { server: server_config, sqs_client, s3_client, pool: Arc::new(pool), db_config: database_config }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}
