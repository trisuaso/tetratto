CREATE TABLE IF NOT EXISTS reports (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    owner BIGINT NOT NULL,
    content TEXT NOT NULL,
    asset BIGINT NOT NULL,
    asset_type TEXT NOT NULL
)
