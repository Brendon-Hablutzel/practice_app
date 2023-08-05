CREATE TABLE users (
    user_id INT NOT NULL AUTO_INCREMENT,
    user_name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(100) NOT NULL,
    UNIQUE(user_name),
    PRIMARY KEY(user_id)
);

CREATE TABLE practice_sessions (
    practice_session_id INT NOT NULL AUTO_INCREMENT,
    start_datetime DATETIME NOT NULL,
    duration_mins INT UNSIGNED NOT NULL,
    instrument VARCHAR(20) NOT NULL,
    user_id INT NOT NULL,
    UNIQUE(start_datetime),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    PRIMARY KEY (practice_session_id)
);

CREATE TABLE pieces (
    piece_id INT NOT NULL AUTO_INCREMENT,
    title VARCHAR(255) NOT NULL,
    composer VARCHAR(40) NOT NULL,
    UNIQUE(title, composer),
    PRIMARY KEY (piece_id)
);

CREATE TABLE pieces_practiced (
    practice_session_id INT NOT NULL,
    piece_id INT NOT NULL,
    user_id INT NOT NULL,
    FOREIGN KEY (practice_session_id) REFERENCES practice_sessions(practice_session_id),
    FOREIGN KEY (piece_id) REFERENCES pieces(piece_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    PRIMARY KEY (practice_session_id, piece_id)
);
