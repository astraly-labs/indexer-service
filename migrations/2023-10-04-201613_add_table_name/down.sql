-- This file should undo anything in `up.sql`
ALTER TABLE indexers
DROP COLUMN table_name;