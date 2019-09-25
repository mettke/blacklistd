-- This file should undo anything in `up.sql`
DROP INDEX blacklist_backend_type;
DROP INDEX blacklist_last_update;

DROP TABLE blacklist;
