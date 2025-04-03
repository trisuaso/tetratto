CREATE TABLE IF NOT EXISTS notifications (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    owner BIGINT NOT NULL,
    read INT NOT NULL
)
