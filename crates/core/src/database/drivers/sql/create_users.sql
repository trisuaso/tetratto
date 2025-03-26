CREATE TABLE IF NOT EXISTS users (
    id INTEGER NOT NULL PRIMARY KEY,
    created INTEGER NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    salt TEXT NOT NULL,
    settings TEXT NOT NULL,
    tokens TEXT NOT NULL,
    permissions INTEGER NOT NULL,
    -- counts
    notification_count INTEGER NOT NULL,
    follower_count INTEGER NOT NULL,
    following_count INTEGER NOT NULL
)
