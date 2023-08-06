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
    get_connection_pool, get_db_conn, get_user_id, handle_db_insert_err, into_backend_err,
    models::*, AppError, Credentials, IncompleteNewPracticeSession, PracticeSessionWithPieces,
};
use rand::{Rng, RngCore};
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

struct AppState {
    db: Pool<ConnectionManager<PgConnection>>,
}

async fn root() -> &'static str {
    "practice app"
}

async fn get_practice_sessions(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    path_params: Option<Path<i32>>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    // fetch the desired practice sessions
    let practice_sessions: Vec<PracticeSession> = if let Some(path_params) = path_params {
        into_backend_err!(practice_sessions::table
            .filter(practice_sessions::user_id.eq(current_user_id))
            .filter(practice_sessions::practice_session_id.eq(path_params.0))
            .load::<PracticeSession>(&mut conn))?
    } else {
        into_backend_err!(practice_sessions::table
            .filter(practice_sessions::user_id.eq(current_user_id))
            .load::<PracticeSession>(&mut conn))?
    };

    // get the pieces practiced in the above practice sessions
    let pieces_practiced: Vec<Vec<(PiecePracticedMapping, Piece)>> =
        into_backend_err!(PiecePracticedMapping::belonging_to(&practice_sessions)
            .inner_join(pieces::table)
            .load(&mut conn))?
        .grouped_by(&practice_sessions);

    // join together the practice sessions with the pieces practiced in each
    let practice_sessions_with_pieces: Vec<PracticeSessionWithPieces> = practice_sessions
        .into_iter()
        .zip(pieces_practiced)
        .map(|(practice_session, pieces_practiced)| {
            PracticeSessionWithPieces::new(
                practice_session,
                pieces_practiced
                    .into_iter()
                    .map(|(_, piece)| piece)
                    .collect(),
            )
        })
        .collect();

    Ok(Json(
        json!({"practice_sessions": practice_sessions_with_pieces}),
    ))
}

async fn create_practice_session(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(practice_session_data): Json<IncompleteNewPracticeSession>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let inserted_practice_session: PracticeSession =
        handle_db_insert_err!(diesel::insert_into(practice_sessions::table)
            .values(practice_session_data.make_insertable(current_user_id)?)
            .get_result(&mut conn))?;

    // NOTE: even if creating a piece practiced mapping fails,
    // the practice session will have already been inserted above
    let pieces_practiced: Vec<PiecePracticedMapping> =
        handle_db_insert_err!(diesel::insert_into(pieces_practiced::table)
            .values(
                practice_session_data
                    .get_pieces_practiced_mappings(inserted_practice_session.practice_session_id),
            )
            .get_results(&mut conn))?;

    Ok(Json(json!({
        "practice_session": inserted_practice_session,
        "pieces_practiced": pieces_practiced
    })))
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

async fn get_pieces(
    State(state): State<Arc<AppState>>,
    path_params: Option<Path<i32>>,
) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let pieces = if let Some(path_params) = path_params {
        into_backend_err!(pieces::table
            .filter(pieces::piece_id.eq(path_params.0))
            .load::<Piece>(&mut conn))?
    } else {
        into_backend_err!(pieces::table.load::<Piece>(&mut conn))?
    };

    Ok(Json(json!({ "pieces": pieces })))
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

    let inserted_piece: Piece = diesel::insert_into(pieces::table)
        .values(new_piece)
        .get_result(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => AppError::Conflict(
                "An entry for a piece with that title and composer already exists".to_owned(),
            ),
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "piece": inserted_piece })))
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

async fn create_piece_practiced(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(piece_practiced_mapping): Json<NewPiecePracticedMapping>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    // verify that the specified practice session exists and belongs to the current user:
    practice_sessions::table
        .select(practice_sessions::user_id)
        .filter(practice_sessions::user_id.eq(current_user_id))
        .filter(
            practice_sessions::practice_session_id.eq(piece_practiced_mapping.practice_session_id),
        )
        .first::<i32>(&mut conn)
        .map_err(|e| match e {
            Error::NotFound => AppError::NotFound("Practice session not found".to_owned()),
            _ => AppError::BackendError(e.to_string()),
        })?;

    let inserted_mapping = diesel::insert_into(pieces_practiced::table)
        .values(piece_practiced_mapping)
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

    Ok(Json(json!({ "piece_practiced": inserted_mapping })))
}

async fn delete_piece_practiced(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Path((practice_session_id, piece_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    // verify that the practice session in the mapping belongs to the current user
    practice_sessions::table
        .select(practice_sessions::user_id)
        .filter(practice_sessions::user_id.eq(current_user_id))
        .filter(practice_sessions::practice_session_id.eq(practice_session_id))
        .first::<i32>(&mut conn)
        .map_err(|e| match e {
            Error::NotFound => AppError::NotFound("Practice session not found".to_owned()),
            _ => AppError::BackendError(e.to_string()),
        })?;

    let rows_deleted = into_backend_err!(diesel::delete(
        pieces_practiced::table
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

    let inserted_user: User = diesel::insert_into(users::table)
        .values((
            users::user_name.eq(credentials.user_name),
            users::password_hash.eq(hashed_password),
        ))
        .get_result(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict("That username already exists".to_owned())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(
        json!({ "user": {"user_id": inserted_user.user_id, "user_name": inserted_user.user_name} }),
    ))
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
        .route(
            "/get_practice_sessions/:practice_session_id",
            get(get_practice_sessions),
        )
        .route("/get_pieces", get(get_pieces))
        .route("/get_pieces/:piece_id", get(get_pieces))
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
