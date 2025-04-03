CREATE TABLE IF NOT EXISTS posts (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    content TEXT NOT NULL,
    owner BIGINT NOT NULL,
    community BIGINT NOT NULL,
    context TEXT NOT NULL,
    replying_to BIGINT, -- the ID of the post this is a comment on... 0 means it isn't a reply
    -- likes
    likes INT NOT NULL,
    dislikes INT NOT NULL,
    -- other counts
    comment_count INT NOT NULL
)
