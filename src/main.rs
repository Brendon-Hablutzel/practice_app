use argon2;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use axum_sessions::{
    async_session::MemoryStore,
    extractors::{ReadableSession, WritableSession},
    SessionLayer,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::{DatabaseErrorKind, Error, Error::DatabaseError};
use practice_app::schema::{pieces, pieces_practiced, practice_sessions, users};
use practice_app::{
    get_connection_pool, get_db_conn, get_user_id, into_backend_err, models::*, AppError,
    Credentials, PiecePracticedData, PracticeSessionData,
};
use rand::{Rng, RngCore};
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

struct AppState {
    db: Pool<ConnectionManager<MysqlConnection>>,
}

async fn root() -> &'static str {
    "practice app"
}

async fn get_practice_sessions(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    use practice_app::schema::practice_sessions::dsl::*;

    let all_practice_sessions = into_backend_err!(practice_sessions
        .filter(user_id.eq(current_user_id))
        .load::<PracticeSession>(&mut conn))?;

    Ok(Json(json!({ "practice_sessions": all_practice_sessions })))
}

async fn create_practice_session(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(practice_session): Json<PracticeSessionData>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let insertable_practice_session = practice_session.add_user_id(current_user_id);

    let num_inserted = diesel::insert_into(practice_sessions::table)
        .values(insertable_practice_session)
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict("A practice session at that time already exists".to_owned())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "num_inserted": num_inserted })))
}

async fn delete_practice_session(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Path(pratice_session_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let rows_deleted: usize = into_backend_err!(diesel::delete(
        practice_sessions::table
            .filter(practice_sessions::user_id.eq(current_user_id))
            .filter(practice_sessions::practice_session_id.eq(pratice_session_id))
    )
    .execute(&mut conn))?;

    Ok(Json(json!({ "num_deleted": rows_deleted })))
}

async fn get_pieces(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let all_pieces = into_backend_err!(pieces::table.load::<Piece>(&mut conn))?;

    Ok(Json(json!({ "pieces": all_pieces })))
}

async fn create_piece(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(new_piece): Json<NewPiece>,
) -> Result<Json<Value>, AppError> {
    // don't need user id to insert a piece into db, but this checks to make sure user is logged in
    // (don't want users to be able to create pieces without being logged in)
    let _current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    use practice_app::schema::pieces::dsl::*;

    let num_inserted = diesel::insert_into(pieces)
        .values(new_piece)
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => AppError::Conflict(
                "An entry for a piece with that title and composer already exists".to_owned(),
            ),
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "num_inserted": num_inserted })))
}

async fn delete_piece(
    State(state): State<Arc<AppState>>,
    Path(piece_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let rows_deleted: usize = into_backend_err!(diesel::delete(
        pieces::table.filter(pieces::piece_id.eq(piece_id))
    )
    .execute(&mut conn))?;

    Ok(Json(json!({ "num_deleted": rows_deleted })))
}

async fn get_pieces_practiced(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let all_pieces_practiced = into_backend_err!(pieces_practiced::table
        .filter(pieces_practiced::user_id.eq(current_user_id))
        .load::<PiecePracticed>(&mut conn))?;

    Ok(Json(json!({ "pieces_practiced": all_pieces_practiced })))
}

async fn create_piece_practiced(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(piece_practiced): Json<PiecePracticedData>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    // verify that the specified practice session exists and belongs to the current user:
    practice_sessions::table
        .select(practice_sessions::user_id)
        .filter(practice_sessions::user_id.eq(current_user_id))
        .filter(practice_sessions::practice_session_id.eq(piece_practiced.practice_session_id))
        .first::<i32>(&mut conn)
        .map_err(|e| match e {
            Error::NotFound => AppError::NotFound("Practice session not found".to_owned()),
            _ => AppError::BackendError(e.to_string()),
        })?;

    let insertable_piece_practiced = piece_practiced.add_user_id(current_user_id);

    let num_inserted = diesel::insert_into(pieces_practiced::table)
        .values(insertable_piece_practiced)
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict(
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
    session: ReadableSession,
    Path((practice_session_id, piece_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let rows_deleted = into_backend_err!(diesel::delete(
        pieces_practiced::table
            .filter(pieces_practiced::user_id.eq(current_user_id))
            .filter(pieces_practiced::piece_id.eq(piece_id))
            .filter(pieces_practiced::practice_session_id.eq(practice_session_id)),
    )
    .execute(&mut conn))?;

    Ok(Json(json!({ "num_deleted": rows_deleted })))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(credentials): Json<Credentials>,
) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let salt = rand::thread_rng().gen::<[u8; 16]>();

    let hashed_password = into_backend_err!(argon2::hash_encoded(
        credentials.password.as_bytes(),
        &salt,
        &argon2::Config::default()
    ))?;

    let num_inserted = diesel::insert_into(users::table)
        .values((
            users::user_name.eq(credentials.user_name),
            users::password_hash.eq(hashed_password),
        ))
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict("That username already exists".to_owned())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "num_inserted": num_inserted })))
}

async fn login(
    State(state): State<Arc<AppState>>,
    mut session: WritableSession,
    Json(credentials): Json<Credentials>,
) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let user: User = users::table
        .filter(users::user_name.eq(credentials.user_name))
        .first::<User>(&mut conn)
        .map_err(|e| match e {
            Error::NotFound => AppError::LoginError,
            _ => AppError::BackendError(e.to_string()),
        })?;

    let login_success = into_backend_err!(argon2::verify_encoded(
        &user.password_hash,
        credentials.password.as_bytes()
    ))?;

    if login_success {
        session.regenerate(); // this is supposed to make it more secure or something
        into_backend_err!(session.insert("user_id", user.user_id))?;

        Ok(Json(json!({ "login_success": login_success })))
    } else {
        Err(AppError::LoginError)
    }
}

async fn logout(mut session: WritableSession) -> Json<Value> {
    session.destroy();

    Json(json!({"success": true}))
}

#[tokio::main]
async fn main() {
    let store = MemoryStore::new();

    let mut secret = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut secret);

    let session_layer = SessionLayer::new(store, &secret);

    let shared_state = Arc::new(AppState {
        db: get_connection_pool(),
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/get_practice_sessions", get(get_practice_sessions))
        .route("/get_pieces", get(get_pieces))
        .route("/get_pieces_practiced", get(get_pieces_practiced))
        .route("/create_practice_session", post(create_practice_session))
        .route("/create_piece", post(create_piece))
        .route("/create_piece_practiced", post(create_piece_practiced))
        .route(
            "/delete_practice_session/:practice_session_id",
            delete(delete_practice_session),
        )
        .route("/delete_piece/:piece_id", delete(delete_piece))
        .route(
            "/delete_piece_practiced/:practice_session_id_to_delete/:piece_id_to_delete",
            delete(delete_piece_practiced),
        )
        .route("/create_user", post(create_user))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .layer(session_layer)
        .with_state(shared_state);

    let ip = Ipv4Addr::new(0, 0, 0, 0);
    let addr = SocketAddr::new(IpAddr::V4(ip), 3000);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Should be able to initialize server");
}
