CREATE TABLE practice_sessions (
    practice_session_id INT NOT NULL AUTO_INCREMENT,
    start_datetime DATETIME NOT NULL,
    duration_mins INT UNSIGNED NOT NULL,
    instrument VARCHAR(20) NOT NULL,
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
    PRIMARY KEY (practice_session_id, piece_id),
    FOREIGN KEY (practice_session_id) REFERENCES practice_sessions(practice_session_id),
    FOREIGN KEY (piece_id) REFERENCES pieces(piece_id)
);
