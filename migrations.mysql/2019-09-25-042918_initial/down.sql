-- This file should undo anything in `up.sql`
DROP INDEX blacklist_backend_type ON blacklist;
DROP INDEX blacklist_last_update ON blacklist;

DROP TABLE blacklist;
