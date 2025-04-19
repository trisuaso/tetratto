CREATE TABLE IF NOT EXISTS questions (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    owner BIGINT NOT NULL,
    receiver BIGINT NOT NULL,
    content TEXT NOT NULL,
    is_global INT NOT NULL,
    answer_count INT NOT NULL,
    community BIGINT NOT NULL,
    -- likes
    likes INT NOT NULL,
    dislikes INT NOT NULL,
    -- ...
    context TEXT NOT NULL,
    ip TEXT NOT NULL
)
