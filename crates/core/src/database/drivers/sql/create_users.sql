CREATE TABLE IF NOT EXISTS users (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    salt TEXT NOT NULL,
    settings TEXT NOT NULL,
    tokens TEXT NOT NULL,
    permissions INT NOT NULL,
    verified INT NOT NULL,
    notification_count INT NOT NULL,
    follower_count INT NOT NULL,
    following_count INT NOT NULL,
    last_seen BIGINT NOT NULL,
    totp TEXT NOT NULL,
    recovery_codes TEXT NOT NULL,
    post_count INT NOT NULL,
    request_count INT NOT NULL
)
