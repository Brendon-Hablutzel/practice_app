use crate::schema::{pieces, pieces_practiced, practice_sessions, users};
use chrono;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(primary_key(user_id))]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

#[derive(Queryable, Selectable, Serialize, Identifiable)]
#[diesel(primary_key(practice_session_id))]
#[diesel(table_name = practice_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PracticeSession {
    pub practice_session_id: i32,
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: i32,
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
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertablePracticeSession {
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: i32,
    pub instrument: String,
    pub user_id: i32,
}

#[derive(Queryable, Selectable, Serialize, Identifiable, Deserialize)]
#[diesel(table_name = pieces)]
#[diesel(primary_key(piece_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

#[derive(Insertable, Deserialize, PartialEq)]
#[diesel(table_name = pieces)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertablePiece {
    pub title: String,
    pub composer: String,
}

#[derive(Queryable, Selectable, Serialize, Associations, Identifiable, Insertable, Deserialize)]
#[diesel(primary_key(practice_session_id, piece_id))]
#[diesel(belongs_to(PracticeSession))]
#[diesel(belongs_to(Piece))]
#[diesel(table_name = pieces_practiced)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PiecePracticedMapping {
    pub practice_session_id: i32,
    pub piece_id: i32,
}

impl Display for PiecePracticedMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PRACTICE SESSION: {} PIECE: {}",
            self.practice_session_id, self.piece_id
        )
    }
}
