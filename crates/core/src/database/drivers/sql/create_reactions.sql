CREATE TABLE IF NOT EXISTS reactions (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    owner BIGINT NOT NULL,
    asset BIGINT NOT NULL,
    asset_type TEXT NOT NULL,
    is_like BIGINT NOT NULL
)
