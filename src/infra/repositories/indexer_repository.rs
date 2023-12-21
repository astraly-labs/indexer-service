use std::str::FromStr;

use axum::async_trait;
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, Selectable, SelectableHelper};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use strum::ParseError;
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::infra::db::schema::indexers;
use crate::infra::errors::InfraError;

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = indexers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IndexerDb {
    pub id: Uuid,
    pub status: String,
    pub type_: String,
    pub process_id: Option<i64>,
    pub target_url: Option<String>,
    pub table_name: Option<String>,
    pub status_server_port: Option<i32>,
}

#[derive(Deserialize)]
pub struct IndexerFilter {
    pub status: Option<String>,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = indexers)]
pub struct NewIndexerDb {
    pub id: Uuid,
    pub status: String,
    pub type_: String,
    pub target_url: Option<String>,
    pub table_name: Option<String>,
    pub status_server_port: Option<i32>,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = indexers)]
pub struct UpdateIndexerStatusDb {
    pub id: Uuid,
    pub status: String,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = indexers)]
pub struct UpdateIndexerStatusAndProcessIdDb {
    pub id: Uuid,
    pub status: String,
    pub process_id: i64,
}

#[async_trait]
pub trait Repository {
    async fn insert(&mut self, new_indexer: NewIndexerDb) -> Result<IndexerModel, InfraError>;
    async fn get(&self, id: Uuid) -> Result<IndexerModel, InfraError>;
    async fn get_all(&self, filter: IndexerFilter) -> Result<Vec<IndexerModel>, InfraError>;
    async fn update_status(&mut self, indexer: UpdateIndexerStatusDb) -> Result<IndexerModel, InfraError>;
    async fn update_status_and_process_id(
        &mut self,
        indexer: UpdateIndexerStatusAndProcessIdDb,
    ) -> Result<IndexerModel, InfraError>;
}

pub struct IndexerRepository<'a> {
    pool: &'a Pool<AsyncPgConnection>,
}

impl IndexerRepository<'_> {
    pub fn new(pool: &Pool<AsyncPgConnection>) -> IndexerRepository {
        IndexerRepository { pool }
    }
}

#[async_trait]
impl Repository for IndexerRepository<'_> {
    async fn insert(&mut self, new_indexer: NewIndexerDb) -> Result<IndexerModel, InfraError> {
        _insert(self.pool, new_indexer).await
    }

    async fn get(&self, id: Uuid) -> Result<IndexerModel, InfraError> {
        get(self.pool, id).await
    }

    async fn get_all(&self, filter: IndexerFilter) -> Result<Vec<IndexerModel>, InfraError> {
        get_all(self.pool, filter).await
    }

    async fn update_status(&mut self, indexer: UpdateIndexerStatusDb) -> Result<IndexerModel, InfraError> {
        update_status(self.pool, indexer).await
    }

    async fn update_status_and_process_id(
        &mut self,
        indexer: UpdateIndexerStatusAndProcessIdDb,
    ) -> Result<IndexerModel, InfraError> {
        update_status_and_process_id(self.pool, indexer).await
    }
}

async fn _insert(pool: &Pool<AsyncPgConnection>, new_indexer: NewIndexerDb) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await?;
    let res = diesel::insert_into(indexers::table)
        .values(new_indexer)
        .returning(IndexerDb::as_returning())
        .get_result(&mut conn)
        .await?
        .try_into()
        .map_err(InfraError::ParseError)?;

    Ok(res)
}

async fn get(pool: &Pool<AsyncPgConnection>, id: Uuid) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await?;
    let res = indexers::table
        .filter(indexers::id.eq(id))
        .select(IndexerDb::as_select())
        .get_result::<IndexerDb>(&mut conn)
        .await?
        .try_into()
        .map_err(InfraError::ParseError)?;

    Ok(res)
}

async fn get_all(pool: &Pool<AsyncPgConnection>, filter: IndexerFilter) -> Result<Vec<IndexerModel>, InfraError> {
    let mut conn = pool.get().await?;
    let mut query = indexers::table.into_boxed::<diesel::pg::Pg>();
    if let Some(status) = filter.status {
        query = query.filter(indexers::status.eq(status));
    }
    let res: Vec<IndexerDb> = query.select(IndexerDb::as_select()).load::<IndexerDb>(&mut conn).await?;

    let posts: Vec<IndexerModel> = res
        .into_iter()
        .map(|indexer_db| indexer_db.try_into())
        .collect::<Result<Vec<IndexerModel>, ParseError>>()
        .map_err(InfraError::ParseError)?;

    Ok(posts)
}

async fn update_status(
    pool: &Pool<AsyncPgConnection>,
    indexer: UpdateIndexerStatusDb,
) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await?;
    let res = diesel::update(indexers::table)
        .filter(indexers::id.eq(indexer.id))
        .set(indexers::status.eq(indexer.status))
        .get_result::<IndexerDb>(&mut conn)
        .await?
        .try_into()
        .map_err(InfraError::ParseError)?;

    Ok(res)
}

async fn update_status_and_process_id(
    pool: &Pool<AsyncPgConnection>,
    indexer: UpdateIndexerStatusAndProcessIdDb,
) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await?;
    let res = diesel::update(indexers::table)
        .filter(indexers::id.eq(indexer.id))
        .set((indexers::status.eq(indexer.status), indexers::process_id.eq(indexer.process_id)))
        .get_result::<IndexerDb>(&mut conn)
        .await?
        .try_into()
        .map_err(InfraError::ParseError)?;

    Ok(res)
}

impl TryFrom<NewIndexerDb> for IndexerModel {
    type Error = ParseError;
    fn try_from(value: NewIndexerDb) -> Result<Self, Self::Error> {
        let model = IndexerDb {
            id: value.id,
            status: value.status,
            type_: value.type_,
            target_url: value.target_url,
            process_id: None,
            table_name: value.table_name,
            status_server_port: value.status_server_port,
        }
        .try_into()?;
        Ok(model)
    }
}

impl TryFrom<IndexerDb> for IndexerModel {
    type Error = ParseError;
    fn try_from(value: IndexerDb) -> Result<Self, Self::Error> {
        let model = IndexerModel {
            id: value.id,
            status: IndexerStatus::from_str(value.status.as_str())?,
            process_id: value.process_id,
            indexer_type: IndexerType::from_str(value.type_.as_str())?,
            target_url: value.target_url,
            table_name: value.table_name,
            status_server_port: value.status_server_port,
        };
        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("Created", Ok(IndexerStatus::Created))]
    #[case("Running", Ok(IndexerStatus::Running))]
    #[case("FailedRunning", Ok(IndexerStatus::FailedRunning))]
    #[case("Stopped", Ok(IndexerStatus::Stopped))]
    #[case("FailedStopping", Ok(IndexerStatus::FailedStopping))]
    #[case("InvalidStatus", Err(ParseError::VariantNotFound))]
    fn test_from_indexer_db_to_indexer_model_status(
        #[case] status: &'static str,
        #[case] expected_status: Result<IndexerStatus, ParseError>,
    ) {
        let id = Uuid::new_v4();
        let process_id = Some(1234);
        let target_url = "http://example.com";
        let indexer_type = "Webhook";
        let table_name = "test_table";

        let indexer_db = IndexerDb {
            id,
            status: status.to_string(),
            type_: indexer_type.to_string(),
            process_id,
            target_url: Some(target_url.to_string()),
            table_name: Some(table_name.into()),
            status_server_port: Some(1234),
        };

        let indexer_model: Result<IndexerModel, ParseError> = indexer_db.try_into();

        match indexer_model {
            Ok(model) => {
                assert_eq!(model.id, id);
                assert_eq!(model.status, expected_status.unwrap());
                assert_eq!(model.indexer_type, IndexerType::from_str(indexer_type).unwrap());
                assert_eq!(model.process_id, process_id);
                assert_eq!(model.target_url, Some(target_url.to_string()));
                assert_eq!(model.table_name, Some(table_name.into()));
            }
            Err(e) => {
                assert_eq!(e, expected_status.unwrap_err());
            }
        }
    }

    #[rstest]
    #[case("Webhook", Ok(IndexerType::Webhook))]
    #[case("InvalidType", Err(ParseError::VariantNotFound))]
    fn test_from_indexer_db_to_indexer_model_type(
        #[case] indexer_type: &'static str,
        #[case] expected_type: Result<IndexerType, ParseError>,
    ) {
        let id = Uuid::new_v4();
        let process_id = Some(1234);
        let target_url = "http://example.com";
        let status = "Created";
        let table_name = "test_table";

        let indexer_db = IndexerDb {
            id,
            status: status.to_string(),
            type_: indexer_type.to_string(),
            process_id,
            target_url: Some(target_url.to_string()),
            table_name: Some(table_name.into()),
            status_server_port: Some(1234),
        };

        let indexer_model: Result<IndexerModel, ParseError> = indexer_db.try_into();

        match indexer_model {
            Ok(model) => {
                assert_eq!(model.id, id);
                assert_eq!(model.status, IndexerStatus::from_str(status).unwrap());
                assert_eq!(model.indexer_type, expected_type.unwrap());
                assert_eq!(model.process_id, process_id);
                assert_eq!(model.target_url, Some(target_url.to_string()));
                assert_eq!(model.table_name, Some(table_name.into()));
            }
            Err(e) => {
                assert_eq!(e, expected_type.unwrap_err());
            }
        }
    }
}
