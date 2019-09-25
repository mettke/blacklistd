-- Your SQL goes here
CREATE TABLE blacklist (
    ip bytea NOT NULL,
    ip_type SMALLINT NOT NULL,
    backend_type SMALLINT NOT NULL,
    last_update timestamp NOT NULL DEFAULT current_timestamp,
    PRIMARY KEY(ip, ip_type)
);

CREATE INDEX blacklist_backend_type ON blacklist (backend_type);
CREATE INDEX blacklist_last_update ON blacklist (last_update);
