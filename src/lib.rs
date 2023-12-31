use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::result::Error;
use diesel::{pg::PgConnection, r2d2::Pool};
use log::error;
use models::{InsertablePracticeSession, Piece, PracticeSession};
use schema::practice_sessions;
use serde::{Deserialize, Serialize};
pub mod models;
pub mod schema;
use dotenvy::dotenv;
use serde_json::json;
use std::env;

// returns the practice session id if the user does own it, otherwise errors
pub fn verify_practice_session_ownership(
    conn: &mut PgConnection,
    practice_session_id: i32,
    current_user_id: i32,
) -> Result<i32, AppError> {
    practice_sessions::table
        .select(practice_sessions::user_id)
        .filter(practice_sessions::user_id.eq(current_user_id))
        .filter(practice_sessions::practice_session_id.eq(practice_session_id))
        .first::<i32>(conn)
        .map_err(|e| match e {
            Error::NotFound => AppError::NotFound("Practice session not found".to_owned()),
            _ => AppError::BackendError(e.to_string()),
        })
}

#[macro_export]
macro_rules! map_backend_err {
    ($fallible:expr) => {
        $fallible.map_err(|e| AppError::BackendError(e.to_string()))
    };
}

#[macro_export]
macro_rules! get_db_conn {
    ($state:ident) => {
        map_backend_err!($state.db.clone().get())
    };
}

#[macro_export]
macro_rules! get_user_id {
    ($session:ident) => {
        $session.get::<i32>("user_id").ok_or(AppError::Unauthorized)
    };
}

pub enum AppError {
    BackendError(String),
    ClientError(String),
    LoginError,
    Unauthorized,
    Forbidden(String),
    Conflict(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BackendError(info) => {
                error!("SERVER ERROR: {info}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"success": false, "error": "Server error"})),
                )
            }
            AppError::ClientError(info) => (
                StatusCode::BAD_REQUEST,
                Json(json!({"success": false, "error": info})),
            ),
            AppError::LoginError => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"success": false, "error": "Invalid login credentials"})),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"success": false, "error": "Unauthorized"})),
            ),
            AppError::Forbidden(info) => (
                StatusCode::FORBIDDEN,
                Json(json!({"success": false, "error": info})),
            ),
            AppError::Conflict(info) => (
                StatusCode::CONFLICT,
                Json(json!({"success": false, "error": info})),
            ),
            AppError::NotFound(info) => (
                StatusCode::NOT_FOUND,
                Json(json!({"success": false, "error": info})),
            ),
        }
        .into_response()
    }
}

#[derive(Deserialize)]
pub struct NewPracticeSessionData {
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: String,
    pub pieces_practiced: Vec<Piece>,
}

impl NewPracticeSessionData {
    pub fn make_insertable(&self, user_id: i32) -> Result<InsertablePracticeSession, AppError> {
        Ok(InsertablePracticeSession {
            start_datetime: self.start_datetime,
            duration_mins: i32::try_from(self.duration_mins).map_err(|_| {
                AppError::ClientError("Invalid practice session duration".to_owned())
            })?,
            instrument: self.instrument.clone(),
            user_id,
        })
    }
}

#[derive(Deserialize)]
pub struct Credentials {
    pub user_name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct PracticeSessionWithPieces {
    start_datetime: NaiveDateTime,
    duration_mins: i32,
    instrument: String,
    practice_session_id: i32,
    user_id: i32,
    pieces_practiced: Vec<Piece>,
}

impl PracticeSessionWithPieces {
    pub fn new(db_practice_session: PracticeSession, pieces_practiced: Vec<Piece>) -> Self {
        let PracticeSession {
            start_datetime,
            duration_mins,
            instrument,
            practice_session_id,
            user_id,
        } = db_practice_session;
        Self {
            start_datetime,
            duration_mins,
            instrument,
            practice_session_id,
            user_id,
            pieces_practiced,
        }
    }
}

pub fn establish_connection() -> Result<PgConnection, ConnectionError> {
    dotenv().expect(".env should load");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var should be set");

    PgConnection::establish(&database_url)
}

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().expect(".env should load");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var should be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .build(manager)
        .expect("Should be able to build connection pool")
}
