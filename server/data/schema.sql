DROP TABLE IF EXISTS stars;

CREATE TABLE stars (
    id INTEGER UNIQUE NOT NULL,
    message_id INTEGER NOT NULL,
    room_id INTEGER NOT NULL,
    time_stamp INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    user_name TEXT NOT NULL
);

CREATE UNIQUE INDEX stars_message_user ON stars(message_id, user_id);