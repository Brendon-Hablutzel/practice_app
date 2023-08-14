use argon2;
use axum::extract::{Path, Query, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method, Request};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use axum_sessions::{
    async_session::MemoryStore,
    extractors::{ReadableSession, WritableSession},
    SessionLayer,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::{DatabaseErrorKind, Error, Error::DatabaseError};
use dotenvy::dotenv;
use log::info;
use practice_app::schema::{pieces, pieces_practiced, practice_sessions, users};
use practice_app::{
    get_connection_pool, get_db_conn, get_user_id, map_backend_err, models::*,
    verify_practice_session_ownership, AppError, Credentials, IncompleteNewPracticeSession,
    PracticeSessionWithPieces,
};
use rand::{Rng, RngCore};
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

struct AppState {
    db: Pool<ConnectionManager<PgConnection>>,
}

#[derive(Deserialize)]
struct PracticeSessionsQueryParams {
    practice_session_id: Option<i32>,
    min_datetime: Option<NaiveDateTime>,
    max_datetime: Option<NaiveDateTime>,
    min_duration_mins: Option<u32>,
    max_duration_mins: Option<u32>,
    instrument: Option<String>,
}

async fn get_practice_sessions(
    State(state): State<Arc<AppState>>,
    session: ReadableSession,
    query_params: Query<PracticeSessionsQueryParams>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    let mut query = practice_sessions::table
        .into_boxed()
        .filter(practice_sessions::user_id.eq(current_user_id));

    if let Some(practice_session_id) = query_params.practice_session_id {
        query = query.filter(practice_sessions::practice_session_id.eq(practice_session_id));
    }

    if let Some(min_datetime) = query_params.min_datetime {
        query = query.filter(practice_sessions::start_datetime.ge(min_datetime));
    }

    if let Some(max_datetime) = query_params.max_datetime {
        query = query.filter(practice_sessions::start_datetime.le(max_datetime));
    }

    if let Some(min_duration_mins) = query_params.min_duration_mins {
        query = query.filter(practice_sessions::duration_mins.ge(
            i32::try_from(min_duration_mins).map_err(|_| {
                AppError::ClientError("Invalid value for min_duration_mins".to_owned())
            })?,
        ));
    }

    if let Some(max_duration_mins) = query_params.max_duration_mins {
        query = query.filter(practice_sessions::duration_mins.le(
            i32::try_from(max_duration_mins).map_err(|_| {
                AppError::ClientError("Invalid value for max_duration_mins".to_owned())
            })?,
        ));
    }

    if let Some(instrument) = &query_params.instrument {
        query = query.filter(practice_sessions::instrument.eq(instrument));
    }

    let practice_sessions: Vec<PracticeSession> =
        map_backend_err!(query.load::<PracticeSession>(&mut conn))?;

    // get the pieces practiced in the above practice sessions
    let pieces_practiced: Vec<Vec<(PiecePracticedMapping, Piece)>> =
        map_backend_err!(PiecePracticedMapping::belonging_to(&practice_sessions)
            .inner_join(pieces::table)
            .load(&mut conn))?
        .grouped_by(&practice_sessions);

    // join together the practice sessions with the pieces practiced in each
    let practice_sessions: Vec<PracticeSessionWithPieces> = practice_sessions
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
        json!({"success": true, "practice_sessions": practice_sessions}),
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
    Path(practice_session_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let current_user_id = get_user_id!(session)?;

    let mut conn = get_db_conn!(state)?;

    // delete the pieces practiced mappings that link to this practice session first...
    let _practice_session_id: i32 =
        verify_practice_session_ownership(&mut conn, practice_session_id, current_user_id)?;

    let pieces_practiced_deleted: usize = map_backend_err!(diesel::delete(
        pieces_practiced::table
            .filter(pieces_practiced::practice_session_id.eq(practice_session_id))
    )
    .execute(&mut conn))?;

    // ...then delete practice session itself
    let rows_deleted: usize = map_backend_err!(diesel::delete(
        practice_sessions::table
            .filter(practice_sessions::user_id.eq(current_user_id))
            .filter(practice_sessions::practice_session_id.eq(practice_session_id))
    )
    .execute(&mut conn))?;

    Ok(Json(
        json!({ "success": rows_deleted > 0, "num_deleted": rows_deleted, "pieces_practiced_mappings_deleted": pieces_practiced_deleted }),
    ))
}

#[derive(Deserialize)]
struct GetPiecesQueryParams {
    piece_id: Option<i32>,
    title: Option<String>,    // match containing
    composer: Option<String>, // match containing
}

async fn get_pieces(
    State(state): State<Arc<AppState>>,
    query_params: Query<GetPiecesQueryParams>,
) -> Result<Json<Value>, AppError> {
    let mut conn = get_db_conn!(state)?;

    let mut query = pieces::table.into_boxed();

    if let Some(piece_id) = query_params.piece_id {
        query = query.filter(pieces::piece_id.eq(piece_id));
    }

    if let Some(title) = &query_params.title {
        query = query.filter(pieces::title.ilike(format!("%{}%", title)));
    }

    if let Some(composer) = &query_params.composer {
        query = query.filter(pieces::composer.ilike(format!("%{}%", composer)));
    }

    let pieces: Vec<Piece> = map_backend_err!(query.load::<Piece>(&mut conn))?;

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
    let _practice_session_id = verify_practice_session_ownership(
        &mut conn,
        piece_practiced_mapping.practice_session_id,
        current_user_id,
    )?;

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
    let _practice_session_id =
        verify_practice_session_ownership(&mut conn, practice_session_id, current_user_id)?;

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

    if credentials.user_name.len() > 100 {
        return Err(AppError::ClientError("User name too long".to_owned()));
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
        return Err(AppError::Forbidden("Already logged in".to_owned()));
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

        Ok(Json(json!({
            "success": login_success,
            "user_id": user.user_id,
            "user_name": user.user_name
        }))
        .into_response())
    } else {
        Err(AppError::LoginError)
    }
}

async fn logout(mut session: WritableSession) -> Result<Response, AppError> {
    let _current_user_id = get_user_id!(session)?;

    session.destroy();

    Ok(Json(json!({"success": true})).into_response())
}

async fn logger_middleware<B>(request: Request<B>, next: Next<B>) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    info!("Received request: {} {}", method, uri);

    let response = next.run(request).await;

    info!(
        "Processed request {} {}, sending response: {}",
        method,
        uri,
        response.status()
    );

    response
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let store = MemoryStore::new();
    info!("Initialized memory store for sessions");

    let mut secret = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut secret);

    let session_layer = SessionLayer::new(store, &secret);

    let shared_state = Arc::new(AppState {
        db: get_connection_pool(),
    });
    info!("Initialized database connection");

    dotenv().expect(".env should load");
    let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL env var should be set");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true)
        .allow_origin(frontend_url.parse::<HeaderValue>().unwrap());

    let app = Router::new()
        .route("/api/get_practice_sessions", get(get_practice_sessions))
        .route("/api/get_pieces", get(get_pieces))
        .route(
            "/api/create_practice_session",
            post(create_practice_session),
        )
        .route("/api/create_piece", post(create_piece))
        .route("/api/create_piece_practiced", post(create_piece_practiced))
        .route(
            "/api/delete_practice_session/:practice_session_id",
            delete(delete_practice_session),
        )
        .route("/api/delete_piece/:piece_id", delete(delete_piece))
        .route(
            "/api/delete_piece_practiced/:practice_session_id_to_delete/:piece_id_to_delete",
            delete(delete_piece_practiced),
        )
        .route("/api/create_user", post(create_user))
        .route("/api/login", post(login))
        .route("/api/logout", get(logout))
        .layer(session_layer)
        .layer(cors)
        .layer(middleware::from_fn(logger_middleware))
        .with_state(shared_state);

    let ip = Ipv4Addr::new(0, 0, 0, 0);
    let addr = SocketAddr::new(IpAddr::V4(ip), 5000);
    info!("Starting server...");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Should be able to initialize server");
}
