// @generated automatically by Diesel CLI.

diesel::table! {
    indexers (id) {
        id -> Uuid,
        status -> Varchar,
        #[sql_name = "type"]
        indexer_type -> Varchar,
        process_id -> Nullable<Int4>,
    }
}

diesel::table! {
    posts (id) {
        id -> Uuid,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    indexers,
    posts,
);
