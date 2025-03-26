CREATE TABLE IF NOT EXISTS posts (
    id INTEGER NOT NULL PRIMARY KEY,
    created INTEGER NOT NULL,
    content TEXT NOT NULL,
    owner INTEGER NOT NULL,
    journal INTEGER NOT NULL,
    context TEXT NOT NULL,
    replying_to INTEGER, -- the ID of the post this is a comment on... NULL means it isn't a reply
    -- likes
    likes INTEGER NOT NULL,
    dislikes INTEGER NOT NULL,
    -- other counts
    comment_count INTEGER NOT NULL
)
