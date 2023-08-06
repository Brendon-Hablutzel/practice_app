use argon2;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
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
    get_connection_pool, get_db_conn, get_user_id, map_backend_err, models::*,
    verify_practice_session_ownership, AppError, Credentials, IncompleteNewPracticeSession,
    PracticeSessionWithPieces,
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
        map_backend_err!(practice_sessions::table
            .filter(practice_sessions::user_id.eq(current_user_id))
            .filter(practice_sessions::practice_session_id.eq(path_params.0))
            .load::<PracticeSession>(&mut conn))?
    } else {
        map_backend_err!(practice_sessions::table
            .filter(practice_sessions::user_id.eq(current_user_id))
            .load::<PracticeSession>(&mut conn))?
    };

    // get the pieces practiced in the above practice sessions
    let pieces_practiced: Vec<Vec<(PiecePracticedMapping, Piece)>> =
        map_backend_err!(PiecePracticedMapping::belonging_to(&practice_sessions)
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
        json!({"success": true, "practice_sessions": practice_sessions_with_pieces}),
    ))
}

async fn create_practice_session(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(practice_session_data): Json<IncompleteNewPracticeSession>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let inserted_practice_session: PracticeSession = diesel::insert_into(practice_sessions::table)
        .values(practice_session_data.make_insertable(current_user_id)?)
        .get_result(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict("A practice session at that time already exists".to_string())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(
        json!({ "success": true, "practice_session": inserted_practice_session }),
    ))
}

async fn delete_practice_session(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Path(pratice_session_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let rows_deleted: usize = map_backend_err!(diesel::delete(
        practice_sessions::table
            .filter(practice_sessions::user_id.eq(current_user_id))
            .filter(practice_sessions::practice_session_id.eq(pratice_session_id))
    )
    .execute(&mut conn))?;

    Ok(Json(
        json!({ "success": rows_deleted > 0, "num_deleted": rows_deleted }),
    ))
}

async fn get_pieces(
    State(state): State<Arc<AppState>>,
    path_params: Option<Path<i32>>,
) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let pieces = if let Some(path_params) = path_params {
        map_backend_err!(pieces::table
            .filter(pieces::piece_id.eq(path_params.0))
            .load::<Piece>(&mut conn))?
    } else {
        map_backend_err!(pieces::table.load::<Piece>(&mut conn))?
    };

    Ok(Json(json!({ "success": true, "pieces": pieces })))
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
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict("That piece is already registered in the database".to_string())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(json!({ "success": true, "piece": inserted_piece })))
}

async fn delete_piece(
    State(state): State<Arc<AppState>>,
    Path(piece_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let rows_deleted: usize = map_backend_err!(diesel::delete(
        pieces::table.filter(pieces::piece_id.eq(piece_id))
    )
    .execute(&mut conn))?;

    Ok(Json(
        json!({ "success": rows_deleted > 0, "num_deleted": rows_deleted }),
    ))
}

async fn create_piece_practiced(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(piece_practiced_mapping): Json<NewPiecePracticedMapping>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    // verify that the practice session in the mapping belongs to the current user
    // (this macro uses ? to return an error if it does not)
    verify_practice_session_ownership!(
        &mut conn,
        piece_practiced_mapping.practice_session_id,
        current_user_id
    );

    let inserted_mapping = diesel::insert_into(pieces_practiced::table)
        .values(piece_practiced_mapping)
        .execute(&mut conn)
        .map_err(|e| match e {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                AppError::Conflict("Entry already exists".to_string())
            }
            DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                // "piece not found" must be the case because:
                // 1: practice session existence is verified above using verify_practice_session_ownership!()
                // 2: piece_id is the only foreign key remaining
                AppError::ClientError("Piece not found".to_string())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(
        json!({ "success": true, "piece_practiced": inserted_mapping }),
    ))
}

async fn delete_piece_practiced(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Path((practice_session_id, piece_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    // verify that the practice session in the mapping belongs to the current user
    // (this macro uses ? to return an error if it does not)
    verify_practice_session_ownership!(&mut conn, practice_session_id, current_user_id);

    let rows_deleted = map_backend_err!(diesel::delete(
        pieces_practiced::table
            .filter(pieces_practiced::piece_id.eq(piece_id))
            .filter(pieces_practiced::practice_session_id.eq(practice_session_id)),
    )
    .execute(&mut conn))?;

    Ok(Json(
        json!({ "success": rows_deleted > 0, "num_deleted": rows_deleted }),
    ))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    Json(credentials): Json<Credentials>,
) -> Result<Response, AppError> {
    let current_user_id = get_user_id!(session);
    if current_user_id.is_ok() {
        return Err(AppError::Forbidden(
            "Cannot create a new user while logged in".to_owned(),
        ));
    }

    let mut conn = get_db_conn!(state)?;

    let salt = rand::thread_rng().gen::<[u8; 16]>();

    let hashed_password = map_backend_err!(argon2::hash_encoded(
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
                AppError::Conflict("A user with that name already exists".to_string())
            }
            _ => AppError::BackendError(e.to_string()),
        })?;

    Ok(Json(
        json!({ "success": true, "user": {"user_id": inserted_user.user_id, "user_name": inserted_user.user_name} }),
    )
    .into_response())
}

async fn login(
    State(state): State<Arc<AppState>>,
    mut session: WritableSession,
    Json(credentials): Json<Credentials>,
) -> Result<Response, AppError> {
    let current_user_id = get_user_id!(session);
    if current_user_id.is_ok() {
        return Ok(Redirect::to("/").into_response());
    }

    let mut conn = get_db_conn!(state)?;

    let user: User = users::table
        .filter(users::user_name.eq(credentials.user_name))
        .first::<User>(&mut conn)
        .map_err(|e| match e {
            Error::NotFound => AppError::LoginError,
            _ => AppError::BackendError(e.to_string()),
        })?;

    let login_success = map_backend_err!(argon2::verify_encoded(
        &user.password_hash,
        credentials.password.as_bytes()
    ))?;

    if login_success {
        session.regenerate(); // this is supposed to make it more secure or something
        map_backend_err!(session.insert("user_id", user.user_id))?;

        Ok(Json(json!({ "success": login_success })).into_response())
    } else {
        Err(AppError::LoginError)
    }
}

async fn logout(mut session: WritableSession) -> Result<Response, AppError> {
    let _current_user_id = get_user_id!(session)?;

    session.destroy();

    Ok(Json(json!({"success": true})).into_response())
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
