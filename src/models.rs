use crate::schema::{pieces, pieces_practiced, practice_sessions};
use chrono;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = pieces)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Piece {
    pub piece_id: i32,
    pub title: String,
    pub composer: String,
}

#[derive(Insertable)]
#[diesel(table_name = pieces)]
pub struct NewPiece<'a> {
    pub title: &'a str,
    pub composer: &'a str,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = pieces_practiced)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PiecePracticed {
    pub practice_session_id: i32,
    pub piece_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = pieces_practiced)]
pub struct NewPiecePracticed {
    pub practice_session_id: i32,
    pub piece_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = practice_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PracticeSession {
    pub practice_session_id: i32,
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: String,
}

#[derive(Insertable)]
#[diesel(table_name = practice_sessions)]
pub struct NewPracticeSession<'a> {
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: &'a str,
}
