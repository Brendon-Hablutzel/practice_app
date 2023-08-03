use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
pub mod models;
pub mod schema;
use crate::models::{NewPiece, NewPiecePracticed, NewPracticeSession};
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> Result<MysqlConnection, ConnectionError> {
    dotenv().expect(".env should load");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var should be set");

    MysqlConnection::establish(&database_url)
}

pub fn create_piece(
    conn: &mut MysqlConnection,
    title: &str,
    composer: &str,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::pieces;

    let new_piece = NewPiece { title, composer };

    diesel::insert_into(pieces::table)
        .values(&new_piece)
        .execute(conn)
}

pub fn create_practice_session(
    conn: &mut MysqlConnection,
    start_datetime: chrono::NaiveDateTime,
    duration_mins: u32,
    instrument: &str,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::practice_sessions;

    let new_practice_session = NewPracticeSession {
        start_datetime,
        duration_mins,
        instrument,
    };

    diesel::insert_into(practice_sessions::table)
        .values(&new_practice_session)
        .execute(conn)
}

pub fn create_piece_practiced(
    conn: &mut MysqlConnection,
    practice_session_id: i32,
    piece_id: i32,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::pieces_practiced;

    let new_piece_practiced = NewPiecePracticed {
        practice_session_id,
        piece_id,
    };

    diesel::insert_into(pieces_practiced::table)
        .values(&new_piece_practiced)
        .execute(conn)
}
