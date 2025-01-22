-- Your SQL goes here

ALTER TABLE indexers ADD COLUMN starting_block BIGINT;
ALTER TABLE indexers ADD COLUMN indexer_id VARCHAR;
