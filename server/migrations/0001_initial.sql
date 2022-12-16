CREATE TABLE users (
    username TEXT PRIMARY KEY NOT NULL COLLATE NOCASE,
    stream_key TEXT NOT NULL
) STRICT;

CREATE TABLE users_notification_settings (
    username TEXT PRIMARY KEY NOT NULL COLLATE NOCASE,
    subscribed_to TEXT COLLATE NOCASE,

    FOREIGN KEY(username) REFERENCES users(username) ON DELETE CASCADE
) STRICT;

CREATE TABLE notification_subscriptions (
    username TEXT PRIMARY KEY NOT NULL COLLATE NOCASE,

    endpoint TEXT NOT NULL,
    auth TEXT NOT NULL,
    p256dh TEXT NOT NULL,

    FOREIGN KEY(username) REFERENCES users(username) ON DELETE CASCADE
) STRICT;
