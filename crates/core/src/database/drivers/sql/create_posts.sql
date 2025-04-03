CREATE TABLE IF NOT EXISTS posts (
    id BIGINT NOT NULL PRIMARY KEY,
    created BIGINT NOT NULL,
    content TEXT NOT NULL,
    owner BIGINT NOT NULL,
    community BIGINT NOT NULL,
    context TEXT NOT NULL,
    replying_to BIGINT, -- the ID of the post this is a comment on... NULL means it isn't a reply
    -- likes
    likes BIGINT NOT NULL,
    dislikes BIGINT NOT NULL,
    -- other counts
    comment_count BIGINT NOT NULL
)
