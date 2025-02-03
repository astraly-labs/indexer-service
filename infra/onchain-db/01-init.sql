CREATE TABLE mainnet_spot_entry (
    network character varying(255),
    pair_id character varying(255),
    data_id character varying(255) NOT NULL,
    block_hash character varying(255),
    block_number bigint,
    block_timestamp timestamp without time zone,
    transaction_hash character varying(255),
    price numeric,
    timestamp timestamp without time zone,
    publisher character varying(255),
    source character varying(255),
    volume numeric,
    _cursor bigint
);

CREATE TABLE spot_entry (
    network character varying(255),
    pair_id character varying(255),
    data_id character varying(255) NOT NULL,
    block_hash character varying(255),
    block_number bigint,
    block_timestamp timestamp without time zone,
    transaction_hash character varying(255),
    price numeric,
    timestamp timestamp without time zone,
    publisher character varying(255),
    source character varying(255),
    volume numeric,
    _cursor bigint
);

CREATE TABLE mainnet_future_entry (
    network character varying(255),
    pair_id character varying(255),
    data_id character varying(255),
    block_hash character varying(255),
    block_number bigint,
    block_timestamp timestamp without time zone,
    transaction_hash character varying(255),
    price numeric,
    timestamp timestamp without time zone,
    publisher character varying(255),
    source character varying(255),
    volume numeric,
    _cursor bigint,
    expiration_timestamp timestamp without time zone
);

CREATE TABLE future_entry (
    network character varying(255),
    pair_id character varying(255),
    data_id character varying(255),
    block_hash character varying(255),
    block_number bigint,
    block_timestamp timestamp without time zone,
    transaction_hash character varying(255),
    price numeric,
    timestamp timestamp without time zone,
    publisher character varying(255),
    source character varying(255),
    volume numeric,
    _cursor bigint,
    expiration_timestamp timestamp without time zone
);

CREATE TABLE mainnet_spot_checkpoints (
    network character varying(255),
    pair_id character varying(255),
    data_id character varying(255) NOT NULL,
    block_hash character varying(255),
    block_number bigint,
    block_timestamp timestamp without time zone,
    transaction_hash character varying(255),
    price numeric,
    sender_address character varying(255),
    aggregation_mode numeric,
    _cursor bigint,
    timestamp timestamp without time zone,
    nb_sources_aggregated numeric
);

CREATE TABLE spot_checkpoints (
    network character varying(255),
    pair_id character varying(255),
    data_id character varying(255) NOT NULL,
    block_hash character varying(255),
    block_number bigint,
    block_timestamp timestamp without time zone,
    transaction_hash character varying(255),
    price numeric,
    sender_address character varying(255),
    aggregation_mode numeric,
    _cursor bigint,
    timestamp timestamp without time zone,
    nb_sources_aggregated numeric
);

CREATE TABLE vrf_requests (
    network character varying(255),
    request_id numeric,
    seed numeric,
    created_at timestamp without time zone,
    created_at_tx character varying(255),
    callback_address character varying(255),
    callback_fee_limit numeric,
    num_words numeric,
    requestor_address character varying(255),
    updated_at timestamp without time zone,
    updated_at_tx character varying(255),
    status numeric,
    minimum_block_number numeric,
    _cursor int8range,
    data_id character varying(255)
);

CREATE TABLE publishers (
    name VARCHAR NOT NULL,
    website_url VARCHAR NOT NULL,
    mainnet_address VARCHAR,
    publisher_type INTEGER NOT NULL CHECK (publisher_type IN (0, 1)) -- 0 = first party, 1 = 3rd party
);

CREATE TABLE oo_requests (
    network character varying(255),
    data_id VARCHAR,
    assertion_id VARCHAR,
    domain_id VARCHAR,
    claim TEXT,
    asserter character varying(255),
    disputer character varying(255),
    disputed BOOLEAN,
    dispute_id character varying(255),
    callback_recipient character varying(255),
    escalation_manager character varying(255),
    caller character varying(255),
    expiration_timestamp timestamp without time zone,
    settled BOOLEAN,
    settlement_resolution BOOLEAN,
    settle_caller character varying(255),
    currency character varying(255),
    bond NUMERIC,
    _cursor int8range,
    identifier VARCHAR,
    updated_at timestamp without time zone,
    updated_at_tx character varying(255)
);
