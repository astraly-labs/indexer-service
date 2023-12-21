-- Your SQL goes here
ALTER TABLE indexers
ADD COLUMN status_server_port INTEGER DEFAULT 1234;
