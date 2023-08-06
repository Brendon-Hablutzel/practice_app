use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{pg::PgConnection, r2d2::Pool};
use models::{NewPiecePracticedMapping, NewPracticeSession, Piece, PracticeSession};
use serde::{Deserialize, Serialize};
pub mod models;
pub mod schema;
use dotenvy::dotenv;
use serde_json::json;
use std::env;

#[macro_export]
macro_rules! into_backend_err {
    ($fallible:expr) => {
        $fallible.map_err(|e| AppError::BackendError(e.to_string()))
    };
}

#[macro_export]
macro_rules! handle_db_insert_err {
    ($insert_query:expr) => {
        $insert_query.map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict(e.to_string())
            }
            DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                AppError::ClientError(e.to_string())
            }
            _ => AppError::BackendError(e.to_string()),
        })
    };
}

#[macro_export]
macro_rules! get_db_conn {
    ($state:ident) => {
        into_backend_err!($state.db.clone().get())
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
            AppError::BackendError(info) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": info})),
            ),
            AppError::ClientError(info) => (StatusCode::BAD_REQUEST, Json(json!({"error": info}))),
            AppError::LoginError => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid login credentials"})),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Unauthorized"})),
            ),
            AppError::Forbidden(info) => (StatusCode::FORBIDDEN, Json(json!({"error": info}))),
            AppError::Conflict(info) => (StatusCode::CONFLICT, Json(json!({"error": info}))),
            AppError::NotFound(info) => (StatusCode::NOT_FOUND, Json(json!({"error": info}))),
        }
        .into_response()
    }
}

#[derive(Deserialize)]
pub struct IncompleteNewPracticeSession {
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: u32,
    pub instrument: String,
    pub pieces_practiced_ids: Vec<i32>,
}

impl IncompleteNewPracticeSession {
    pub fn make_insertable(&self, user_id: i32) -> Result<NewPracticeSession, AppError> {
        Ok(NewPracticeSession {
            start_datetime: self.start_datetime,
            duration_mins: i32::try_from(self.duration_mins).map_err(|_| {
                AppError::ClientError("Invalid practice session duration".to_owned())
            })?,
            instrument: self.instrument.clone(),
            user_id,
        })
    }

    pub fn get_pieces_practiced_mappings(
        &self,
        practice_session_id: i32,
    ) -> Vec<NewPiecePracticedMapping> {
        self.pieces_practiced_ids
            .iter()
            .map(|&piece_id| NewPiecePracticedMapping {
                practice_session_id,
                piece_id,
            })
            .collect()
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
