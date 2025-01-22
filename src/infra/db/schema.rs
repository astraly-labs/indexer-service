// @generated automatically by Diesel CLI.

diesel::table! {
    indexers (id) {
        id -> Uuid,
        status -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        process_id -> Nullable<Int8>,
        target_url -> Nullable<Varchar>,
        table_name -> Nullable<Varchar>,
        status_server_port -> Nullable<Int4>,
        custom_connection_string -> Nullable<Varchar>,
        starting_block -> Nullable<Int8>,
        indexer_id -> Nullable<Varchar>,
    }
}
