use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::{DatabaseErrorKind, Error::DatabaseError};
use practice_app::get_connection_pool;
use practice_app::models::*;
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

struct AppState {
    db: Pool<ConnectionManager<MysqlConnection>>,
}

enum AppError {
    BackendError(String),
    ClientError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BackendError(info) => (StatusCode::INTERNAL_SERVER_ERROR, info),
            AppError::ClientError(info) => (StatusCode::BAD_REQUEST, info),
        }
        .into_response()
    }
}

macro_rules! into_backend_err {
    ($fallible:expr) => {
        $fallible.map_err(|e| AppError::BackendError(e.to_string()))
    };
}

async fn root() -> &'static str {
    "practice app"
}

async fn get_practice_sessions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::practice_sessions::dsl::*;

    let all_practice_sessions =
        into_backend_err!(practice_sessions.load::<PracticeSession>(&mut conn))?;

    Ok(Json(json!({ "practice_sessions": all_practice_sessions })))
}

async fn add_practice_session(
    State(state): State<Arc<AppState>>,
    Json(new_practice_session): Json<NewPracticeSession>,
) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::practice_sessions::dsl::*;

    let num_inserted = diesel::insert_into(practice_sessions)
        .values(new_practice_session)
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::ClientError("A practice session at that time already exists".to_owned())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "num_inserted": num_inserted })))
}

async fn delete_practice_session(
    State(state): State<Arc<AppState>>,
    Path(pratice_session_id_to_delete): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::practice_sessions::dsl::*;

    let rows_deleted: usize = into_backend_err!(diesel::delete(
        practice_sessions.filter(practice_session_id.eq(pratice_session_id_to_delete))
    )
    .execute(&mut conn))?;

    Ok(Json(json!({ "num_deleted": rows_deleted })))
}

async fn get_pieces(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::pieces::dsl::*;

    let all_pieces = into_backend_err!(pieces.load::<Piece>(&mut conn))?;

    Ok(Json(json!({ "pieces": all_pieces })))
}

async fn add_piece(
    State(state): State<Arc<AppState>>,
    Json(new_piece): Json<NewPiece>,
) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::pieces::dsl::*;

    let num_inserted = diesel::insert_into(pieces)
        .values(new_piece)
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => AppError::ClientError(
                "An entry for a piece with that title and composer already exists".to_owned(),
            ),
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "num_inserted": num_inserted })))
}

async fn delete_piece(
    State(state): State<Arc<AppState>>,
    Path(piece_id_to_delete): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::pieces::dsl::*;

    let rows_deleted = into_backend_err!(diesel::delete(
        pieces.filter(piece_id.eq(piece_id_to_delete))
    )
    .execute(&mut conn))?;

    Ok(Json(json!({ "num_deleted": rows_deleted })))
}

async fn get_pieces_practiced(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::pieces_practiced::dsl::*;

    let all_pieces_practiced =
        into_backend_err!(pieces_practiced.load::<PiecePracticed>(&mut conn))?;

    Ok(Json(json!({ "pieces_practiced": all_pieces_practiced })))
}

async fn add_piece_practiced(
    State(state): State<Arc<AppState>>,
    Json(new_piece_practiced): Json<NewPiecePracticed>,
) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::pieces_practiced::dsl::*;

    let num_inserted = diesel::insert_into(pieces_practiced)
        .values(new_piece_practiced)
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::ClientError(
                    "A piece practiced mapping with those piece and practice session ids already exists".to_owned()
                )
            }
            DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                AppError::ClientError(e.to_string())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "num_inserted": num_inserted })))
}

async fn delete_piece_practiced(
    State(state): State<Arc<AppState>>,
    Path((practice_session_id_to_delete, piece_id_to_delete)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let mut conn = into_backend_err!(state.db.clone().get())?;

    use practice_app::schema::pieces_practiced::dsl::*;

    let rows_deleted = into_backend_err!(diesel::delete(
        pieces_practiced
            .filter(piece_id.eq(piece_id_to_delete))
            .filter(practice_session_id.eq(practice_session_id_to_delete)),
    )
    .execute(&mut conn))?;

    Ok(Json(json!({ "num_deleted": rows_deleted })))
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        db: get_connection_pool(),
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/get_practice_sessions", get(get_practice_sessions))
        .route("/get_pieces", get(get_pieces))
        .route("/get_pieces_practiced", get(get_pieces_practiced))
        .route("/add_practice_session", post(add_practice_session))
        .route("/add_piece", post(add_piece))
        .route("/add_piece_practiced", post(add_piece_practiced))
        .route(
            "/delete_practice_session/:practice_session_id_to_delete",
            delete(delete_practice_session),
        )
        .route("/delete_piece/:piece_id_to_delete", delete(delete_piece))
        .route(
            "/delete_piece_practiced/:practice_session_id_to_delete/:piece_id_to_delete",
            delete(delete_piece_practiced),
        )
        .with_state(shared_state);

    let ip = Ipv4Addr::new(0, 0, 0, 0);
    let addr = SocketAddr::new(IpAddr::V4(ip), 3000);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Should be able to initialize server");
}
