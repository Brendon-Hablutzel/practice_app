use std::fmt::Display;

use crate::schema::{pieces, pieces_practiced, practice_sessions, users};
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

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) TITLE: {} COMPOSER: {}",
            self.piece_id, self.title, self.composer
        )
    }
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = pieces)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
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
    pub user_id: i32,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = pieces_practiced)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewPiecePracticed {
    pub practice_session_id: i32,
    pub piece_id: i32,
    pub user_id: i32,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = practice_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct PracticeSession {
    pub practice_session_id: i32,
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: String,
    pub user_id: i32,
}

impl Display for PracticeSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) DATETIME: {} DURATION: {} INSTRUMENT: {} USER: {}",
            self.practice_session_id,
            self.start_datetime,
            self.duration_mins,
            self.instrument,
            self.user_id
        )
    }
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = practice_sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewPracticeSession {
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: String,
    pub user_id: i32,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub user_id: i32,
    pub user_name: String,
    pub password_hash: String,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) NAME: {} PASS: {}",
            self.user_id, self.user_name, self.password_hash
        )
    }
}
