CREATE TABLE users (
    username TEXT PRIMARY KEY NOT NULL COLLATE NOCASE,
    created_at INTEGER NOT NULL,
    password_hash TEXT NOT NULL,
    stream_key TEXT NOT NULL
) STRICT;

CREATE TABLE user_sessions (
    id INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,

    CONSTRAINT fk_username_assoc
        FOREIGN KEY (username)
        REFERENCES users (username)
        ON DELETE CASCADE
) STRICT;
