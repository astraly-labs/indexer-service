-- Your SQL goes here
ALTER TABLE indexers
ADD COLUMN status_server_port SMALLINT DEFAULT 1234;
