use crate::schema::{pieces, pieces_practiced, practice_sessions};
use chrono;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = pieces)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Piece {
    pub piece_id: i32,
    pub title: String,
    pub composer: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = pieces)]
pub struct NewPiece {
    pub title: String,
    pub composer: String,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = pieces_practiced)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PiecePracticed {
    pub practice_session_id: i32,
    pub piece_id: i32,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = pieces_practiced)]
pub struct NewPiecePracticed {
    pub practice_session_id: i32,
    pub piece_id: i32,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = practice_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PracticeSession {
    pub practice_session_id: i32,
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = practice_sessions)]
pub struct NewPracticeSession {
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: String,
}
