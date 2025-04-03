CREATE TABLE IF NOT EXISTS users (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    salt TEXT NOT NULL,
    settings TEXT NOT NULL,
    tokens TEXT NOT NULL,
    permissions BIGINT NOT NULL,
    verified BIGINT NOT NULL,
    notification_count BIGINT NOT NULL,
    follower_count BIGINT NOT NULL,
    following_count BIGINT NOT NULL,
    last_seen BIGINT NOT NULL
)
