-- Your SQL goes here
-- Add custom connection string column
ALTER TABLE indexers
ADD COLUMN custom_connection_string VARCHAR;