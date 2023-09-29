use std::env;
use std::sync::Arc;

use aws_sdk_s3::Client as S3Client;
use aws_sdk_sqs::Client as SqsClient;
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use diesel_async::AsyncPgConnection;
use dotenvy::dotenv;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
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
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);
    let manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(database_config.url.to_string(), config);
    let pool = Pool::builder(manager).build().unwrap();

    Config { server: server_config, sqs_client, s3_client, pool: Arc::new(pool), db_config: database_config }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}

pub fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        // We first set up the way we want rustls to work.
        let rustls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_certs())
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) =
            tokio_postgres::connect(config, tls).await.map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                tracing::error!("Database connection: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

/// This function is used to load the cert file from the platform.
/// The certs being loaded here are not the certs on AWS RDS. The signing
/// over there is handled by the RDS proxy created on AWS. However, our connection
/// to the proxy also needs certs otherwise we get the UnknownIssuer error. The code
/// below loads the native certs in the system.
fn root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs().expect("Certs not loadable!");
    let certs: Vec<_> = certs.into_iter().map(|cert| cert.0).collect();
    roots.add_parsable_certificates(&certs);
    roots
}
