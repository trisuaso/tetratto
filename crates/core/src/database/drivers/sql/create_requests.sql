CREATE TABLE IF NOT EXISTS requests (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    owner BIGINT NOT NULL,
    action_type TEXT NOT NULL,
    linked_asset BIGINT NOT NULL
)
