use std::env;
use std::sync::Arc;

use arc_swap::{ArcSwap, Guard};
//use aws_sdk_s3::Client as S3Client;
#[cfg(test)]
use diesel::{Connection, PgConnection, RunQueryDsl};
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use diesel_async::AsyncPgConnection;
#[cfg(test)]
use diesel_async::RunQueryDsl as AsyncRunQueryDsl;
use dotenvy::dotenv;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use tokio::sync::OnceCell;

use object_store::ObjectStore;

#[cfg(feature = "aws")]
use object_store::aws::AmazonS3Builder;

#[cfg(feature = "gcp")]
use object_store::gcp::GoogleCloudStorageBuilder;


use crate::utils::env::get_environment_variable;


#[cfg(test)]
use crate::run_migrations;
#[cfg(test)]
use crate::tests::common::constants::TEST_DB_NAME;
#[cfg(test)]
use crate::tests::common::utils::clear_db;

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
    //s3_client: S3Client,
    object_store: Arc<dyn ObjectStore>,
    pool: Arc<Pool<AsyncPgConnection>>,
    db_config: DatabaseConfig,
    is_dev: bool,
}

impl Config {
    pub fn server_host(&self) -> &str {
        &self.server.host
    }

    pub fn server_port(&self) -> u16 {
        self.server.port
    }

    // pub fn s3_client(&self) -> &S3Client {
    //     &self.s3_client
    // }

    pub fn object_store(&self) -> &Arc<dyn ObjectStore> {
        &self.object_store
    }

    pub fn pool(&self) -> &Arc<Pool<AsyncPgConnection>> {
        &self.pool
    }

    pub fn db_url(&self) -> &str {
        &self.db_config.url
    }

    pub fn is_dev(&self) -> bool {
        self.is_dev
    }
}

/// We are using `ArcSwap` as it allow us to replace the new `Config` with
/// a new one which is required when running test cases. This approach was
/// inspired from here - https://github.com/matklad/once_cell/issues/127
pub static CONFIG: OnceCell<ArcSwap<Config>> = OnceCell::const_new();

#[cfg(not(test))]
async fn init_config() -> Config {
    dotenv().ok();
    // init server config
    let server_config = ServerConfig {
        host: env::var("HOST").unwrap_or_else(|_| String::from("127.0.0.1")),
        port: env::var("PORT").unwrap_or_else(|_| String::from("3000")).parse::<u16>().unwrap(),
    };

    // init database config
    let database_config = DatabaseConfig { url: env::var("DATABASE_URL").expect("DATABASE_URL must be set") };

    // create a new connection pool with the default config
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);
    let manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(database_config.url.to_string(), config);
    let pool = Pool::builder(manager).build().unwrap();

    let is_dev = env::var("DEV_ENV").unwrap_or_else(|_| String::from("false")).parse::<bool>().unwrap_or(false);

    tracing::info!("DEV environment: {}", is_dev);

    // if !is_dev {
    //     // init AWS config
    //     let shared_config = aws_config::from_env().load().await;

    //     // init AWS S3 client
    //     let s3_client = S3Client::new(&shared_config);

    //     Config { server: server_config, s3_client, pool: Arc::new(pool), db_config: database_config, is_dev }
    // } else {
    //     let localstack_endpoint = env::var("LOCALSTACK_ENDPOINT").expect("LOCALSTACK_ENDPOINT must be set");
    //     let shared_config = aws_config::from_env().load().await;

    //     // init AWS S3 client
    //     let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
    //         .endpoint_url(localstack_endpoint)
    //         .force_path_style(true)
    //         .build();
    //     let s3_client = S3Client::from_conf(s3_config);

    //     Config { server: server_config, s3_client, pool: Arc::new(pool), db_config: database_config, is_dev }
    // }

    #[cfg(feature = "gcp")]
    let object_store = create_gcs_client().await;

    #[cfg(feature = "aws")]
    let object_store = create_s3_client().await;

    Config { server: server_config, /* s3_client, */  object_store, pool: Arc::new(pool), db_config: database_config, is_dev }
}

#[cfg(test)]
pub async fn init_config() -> Config {
    dotenv().ok();
    // init server config
    let server_config = ServerConfig {
        host: env::var("HOST").unwrap_or_else(|_| String::from("127.0.0.1")),
        port: env::var("PORT").unwrap_or_else(|_| String::from("3000")).parse::<u16>().unwrap(),
    };

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // First, connect to the db to be able to create our test database.
    let mut conn = PgConnection::establish(&database_url).expect("Cannot connect to the database.");

    // Clear the test database if it already exists. This can happen if the previous run panicked
    clear_db(database_url.as_str(), TEST_DB_NAME);

    let query = diesel::sql_query(format!("CREATE DATABASE {}", TEST_DB_NAME).as_str());
    RunQueryDsl::execute(query, &mut conn)
        .unwrap_or_else(|e| panic!("Could not create database {}, error: {}", TEST_DB_NAME, e));

    // init database config
    let database_config = DatabaseConfig { url: format!("{}/{}", database_url, TEST_DB_NAME) };

    // create a new connection pool with the default config
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);
    let manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(database_config.url.to_string(), config);
    let pool = Pool::builder(manager).build().unwrap();

    // Add uuid-ossp extension to the test database
    let mut conn = pool.get().await.expect("Failed to get connection from pool");
    let query = diesel::sql_query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"");
    AsyncRunQueryDsl::execute(query, &mut conn).await.expect("Failed to create uuid-ossp extension");

    // init tables
    run_migrations(database_config.url.clone()).await.expect("Failed to run migrations");

    // // init AWS config
    // let shared_config = aws_config::from_env().load().await;

    // let localstack_endpoint = get_environment_variable("LOCALSTACK_ENDPOINT");

    // // init AWS S3 client
    // let s3_config = aws_sdk_s3::config::Builder::from(&shared_config)
    //     .endpoint_url(localstack_endpoint)
    //     .force_path_style(true)
    //     .build();
    // let s3_client = S3Client::from_conf(s3_config);

    #[cfg(feature = "gcp")]
    let object_store = create_gcs_client().await;

    #[cfg(feature = "aws")]
    let object_store = create_s3_client().await;

    Config { server: server_config, /* s3_client, */ object_store, pool: Arc::new(pool), db_config: database_config, is_dev: true }
}


#[cfg(feature = "gcp")]
async fn create_gcs_client() -> Arc<dyn ObjectStore> {
    let gcs_bucket_name = get_environment_variable("GCS_BUCKET_NAME");
    let gcs_service_account = get_environment_variable("GCS_SERVICE_ACCOUNT");

    let gcs = GoogleCloudStorageBuilder::new()
            .with_bucket_name(gcs_bucket_name)
            .with_service_account_path(gcs_service_account)
            .build()
            .expect("Failed to create gcs object store");
    
    Arc::new(gcs)
}

#[cfg(feature = "aws")]
async fn create_s3_client() -> Arc<dyn ObjectStore> {    
    // let shared_config = aws_config::load_from_env().await;

    // let localstack_endpoint = get_environment_variable("LOCALSTACK_ENDPOINT");

    // let store = AmazonS3Builder::new()
    //     .with_config(&shared_config)
    //     .with_endpoint(localstack_endpoint)
    //     .with_path_style()
    //     .build()
    //     .expect("Failed to create S3 object store");
    let aws_region = get_environment_variable("AWS_REGION");
    let aws_access_key_id = get_environment_variable("AWS_ACCESS_KEY_ID");
    let aws_secret_access_key = get_environment_variable("AWS_SECRET_ACCESS_KEY");
    let aws_bucket_name = get_environment_variable("INDEXER_SERVICE_BUCKET");
    let localstack_endpoint = get_environment_variable("LOCALSTACK_ENDPOINT");
    
    let s3 = AmazonS3Builder::new()
        .with_region(aws_region)
        .with_access_key_id(aws_access_key_id)
        .with_secret_access_key(aws_secret_access_key)
        .with_bucket_name(aws_bucket_name)
        // localstack
        .with_endpoint(localstack_endpoint)
        .with_allow_http(true)
        //
        .build()
        .expect("Failed to create S3 object store");

    Arc::new(s3)
}

pub async fn config() -> Guard<Arc<Config>> {
    let cfg = CONFIG.get_or_init(|| async { ArcSwap::from_pointee(init_config().await) }).await;
    cfg.load()
}

/// OnceCell only allows us to initialize the config once and that's how it should be on production.
/// However, when running tests, we often want to reinitialize because we want to clear the DB and
/// set it up again for reuse in new tests. By calling `config_force_init` we replace the already
/// stored config inside `ArcSwap` with the new configuration and pool settings.
#[cfg(test)]
pub async fn config_force_init() {
    match CONFIG.get() {
        Some(arc) => arc.store(Arc::new(init_config().await)),
        None => {
            config().await;
        }
    };
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
