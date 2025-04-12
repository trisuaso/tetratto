CREATE TABLE IF NOT EXISTS user_warnings (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    receiver BIGINT NOT NULL,
    moderator BIGINT NOT NULL,
    content TEXT NOT NULL
)
