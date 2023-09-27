use std::str::FromStr;

use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, Selectable, SelectableHelper};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::indexer::{IndexerModel, IndexerStatus, IndexerType};
use crate::infra::db::schema::indexers;
use crate::infra::errors::{adapt_infra_error, InfraError};

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = indexers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IndexerDb {
    pub id: Uuid,
    pub status: String,
    pub indexer_type: String,
    pub process_id: Option<i64>,
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
    pub indexer_type: String,
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

pub async fn _insert(pool: &Pool<AsyncPgConnection>, new_indexer: NewIndexerDb) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await.map_err(adapt_infra_error)?;
    let res = diesel::insert_into(indexers::table)
        .values(new_indexer)
        .returning(IndexerDb::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(adapt_infra_error)?;

    Ok(res.into())
}

pub async fn get(pool: &Pool<AsyncPgConnection>, id: Uuid) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await.map_err(adapt_infra_error)?;
    let res = indexers::table
        .filter(indexers::id.eq(id))
        .select(IndexerDb::as_select())
        .get_result(&mut conn)
        .await
        .map_err(adapt_infra_error)?;

    Ok(res.into())
}

pub async fn get_all(pool: &Pool<AsyncPgConnection>, filter: IndexerFilter) -> Result<Vec<IndexerModel>, InfraError> {
    let mut conn = pool.get().await.map_err(adapt_infra_error)?;
    let mut query = indexers::table.into_boxed::<diesel::pg::Pg>();
    if let Some(status) = filter.status {
        query = query.filter(indexers::status.eq(status));
    }
    let res: Vec<IndexerDb> =
        query.select(IndexerDb::as_select()).load::<IndexerDb>(&mut conn).await.map_err(adapt_infra_error)?;

    let posts: Vec<IndexerModel> = res.into_iter().map(|indexer_db| indexer_db.into()).collect();

    Ok(posts)
}

pub async fn update_status(
    pool: &Pool<AsyncPgConnection>,
    indexer: UpdateIndexerStatusDb,
) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await.map_err(adapt_infra_error)?;
    let res: IndexerDb = diesel::update(indexers::table)
        .filter(indexers::id.eq(indexer.id))
        .set(indexers::status.eq(indexer.status))
        .get_result(&mut conn)
        .await
        .map_err(adapt_infra_error)?;

    Ok(res.into())
}

pub async fn update_status_and_process_id(
    pool: &Pool<AsyncPgConnection>,
    indexer: UpdateIndexerStatusAndProcessIdDb,
) -> Result<IndexerModel, InfraError> {
    let mut conn = pool.get().await.map_err(adapt_infra_error)?;
    let res: IndexerDb = diesel::update(indexers::table)
        .filter(indexers::id.eq(indexer.id))
        .set((indexers::status.eq(indexer.status), indexers::process_id.eq(indexer.process_id)))
        .get_result(&mut conn)
        .await
        .map_err(adapt_infra_error)?;

    Ok(res.into())
}

impl From<IndexerDb> for IndexerModel {
    fn from(value: IndexerDb) -> Self {
        IndexerModel {
            id: value.id,
            status: IndexerStatus::from_str(value.status.as_str()).unwrap(),
            process_id: value.process_id,
            indexer_type: IndexerType::from_str(value.indexer_type.as_str()).unwrap(),
        }
    }
}
