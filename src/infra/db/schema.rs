// @generated automatically by Diesel CLI.

diesel::table! {
    indexers (id) {
        id -> Uuid,
        status -> Varchar,
        #[sql_name = "type"]
        indexer_type -> Varchar,
        process_id -> Nullable<Int8>,
        target_url -> Varchar,
    }
}
