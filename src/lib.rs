use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{pg::PgConnection, r2d2::Pool};
use models::{NewPiecePracticed, NewPracticeSession};
use serde::Deserialize;
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
pub struct PracticeSessionData {
    pub start_datetime: chrono::NaiveDateTime,
    pub duration_mins: i32,
    pub instrument: String,
}

impl PracticeSessionData {
    pub fn add_user_id(self, user_id: i32) -> NewPracticeSession {
        NewPracticeSession {
            start_datetime: self.start_datetime,
            duration_mins: self.duration_mins,
            instrument: self.instrument,
            user_id,
        }
    }
}

#[derive(Deserialize)]
pub struct PiecePracticedData {
    pub practice_session_id: i32,
    pub piece_id: i32,
}

impl PiecePracticedData {
    pub fn add_user_id(self, user_id: i32) -> NewPiecePracticed {
        NewPiecePracticed {
            practice_session_id: self.practice_session_id,
            piece_id: self.piece_id,
            user_id,
        }
    }
}

#[derive(Deserialize)]
pub struct Credentials {
    pub user_name: String,
    pub password: String,
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
