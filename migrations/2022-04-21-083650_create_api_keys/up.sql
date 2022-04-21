CREATE TABLE IF NOT EXISTS api_keys (
                                    id BIGINT NOT NULL PRIMARY KEY AUTOINCREMENT,
                                    key TEXT NOT NULL UNIQUE,
                                    description TEXT
);

CREATE UNIQUE INDEX idx_api_keys_key
    ON api_keys (key);