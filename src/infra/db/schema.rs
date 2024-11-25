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
    }
}

diesel::table! {
    spot_entry (data_id) {
        #[max_length = 255]
        network -> Nullable<Varchar>,
        #[max_length = 255]
        pair_id -> Nullable<Varchar>,
        #[max_length = 255]
        data_id -> Varchar,
        #[max_length = 255]
        block_hash -> Nullable<Varchar>,
        block_number -> Nullable<Int8>,
        block_timestamp -> Nullable<Timestamp>,
        #[max_length = 255]
        transaction_hash -> Nullable<Varchar>,
        price -> Nullable<Numeric>,
        timestamp -> Nullable<Timestamp>,
        #[max_length = 255]
        publisher -> Nullable<Varchar>,
        #[max_length = 255]
        source -> Nullable<Varchar>,
        volume -> Nullable<Numeric>,
        _cursor -> Nullable<Int8>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(indexers, spot_entry,);
