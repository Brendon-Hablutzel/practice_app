CREATE TABLE users (
    user_id SERIAL NOT NULL,
    user_name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(100) NOT NULL,
    UNIQUE(user_name),
    PRIMARY KEY(user_id)
);

CREATE TABLE practice_sessions (
    practice_session_id SERIAL NOT NULL,
    start_datetime TIMESTAMP NOT NULL,
    duration_mins INT NOT NULL, -- change this to INTERVAL type?
    instrument VARCHAR(20) NOT NULL,
    user_id INT NOT NULL,
    UNIQUE(start_datetime, user_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    PRIMARY KEY (practice_session_id)
);

CREATE TABLE pieces (
    piece_id SERIAL NOT NULL,
    title VARCHAR(255) NOT NULL,
    composer VARCHAR(40) NOT NULL,
    UNIQUE(title, composer),
    PRIMARY KEY (piece_id)
);

CREATE TABLE pieces_practiced (
    practice_session_id INT NOT NULL,
    piece_id INT NOT NULL,
    FOREIGN KEY (practice_session_id) REFERENCES practice_sessions(practice_session_id),
    FOREIGN KEY (piece_id) REFERENCES pieces(piece_id),
    PRIMARY KEY (practice_session_id, piece_id)
);
