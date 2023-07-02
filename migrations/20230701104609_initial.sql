CREATE TABLE IF NOT EXISTS Items
(
    id          INTEGER PRIMARY KEY NOT NULL,
    name        TEXT                NOT NULL,
    description TEXT                NOT NULL,
    location    TEXT                NOT NULL DEFAULT 'office',
    quantity    TEXT                NOT NULL DEFAULT 0,
    done        BOOLEAN             NOT NULL DEFAULT 0
);
