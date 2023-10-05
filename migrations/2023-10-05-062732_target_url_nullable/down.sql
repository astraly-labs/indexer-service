-- This file should undo anything in `up.sql`
ALTER TABLE indexers
ALTER COLUMN target_url SET NOT NULL;