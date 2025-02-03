-- This file should undo anything in `up.sql`

ALTER TABLE indexers DROP COLUMN starting_block;
ALTER TABLE indexers DROP COLUMN indexer_id;
