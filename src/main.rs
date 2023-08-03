use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use practice_app::models::*;
use practice_app::{get_connection_pool, last_insert_id};
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

struct AppState {
    db: Pool<ConnectionManager<MysqlConnection>>,
}

async fn root() -> &'static str {
    "practice app"
}

async fn get_practice_sessions(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::practice_sessions::dsl::*;

    let all_practice_sessions = practice_sessions
        .load::<PracticeSession>(&mut conn)
        .unwrap();

    Json(json!({ "practice_sessions": all_practice_sessions }))
}

async fn add_practice_session(
    State(state): State<Arc<AppState>>,
    Json(new_practice_session): Json<NewPracticeSession>,
) -> Json<PracticeSession> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::practice_sessions::dsl::*;

    diesel::insert_into(practice_sessions)
        .values(&new_practice_session)
        .execute(&mut conn)
        .unwrap();

    let last_generated_id = diesel::select(last_insert_id())
        .first::<i32>(&mut conn)
        .unwrap();

    let last_inserted_session = practice_sessions
        .filter(practice_session_id.eq(last_generated_id))
        .first::<PracticeSession>(&mut conn)
        .unwrap();

    Json(last_inserted_session)
}

async fn delete_practice_session(
    State(state): State<Arc<AppState>>,
    Path(pratice_session_id_to_delete): Path<i32>,
) -> Json<Value> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::practice_sessions::dsl::*;

    let rows_deleted = diesel::delete(
        practice_sessions.filter(practice_session_id.eq(pratice_session_id_to_delete)),
    )
    .execute(&mut conn)
    .unwrap();

    Json(json!({ "num_deleted": rows_deleted }))
}

async fn get_pieces(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::pieces::dsl::*;

    let all_pieces = pieces.load::<Piece>(&mut conn).unwrap();

    Json(json!({ "pieces": all_pieces }))
}

async fn add_piece(
    State(state): State<Arc<AppState>>,
    Json(new_piece): Json<NewPiece>,
) -> Json<Piece> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::pieces::dsl::*;

    diesel::insert_into(pieces)
        .values(new_piece)
        .execute(&mut conn)
        .unwrap();

    let last_generated_id = diesel::select(last_insert_id())
        .first::<i32>(&mut conn)
        .unwrap();

    let last_inserted_piece = pieces
        .filter(piece_id.eq(last_generated_id))
        .first::<Piece>(&mut conn)
        .unwrap();

    Json(last_inserted_piece)
}

async fn delete_piece(
    State(state): State<Arc<AppState>>,
    Path(piece_id_to_delete): Path<i32>,
) -> Json<Value> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::pieces::dsl::*;

    let rows_deleted = diesel::delete(pieces.filter(piece_id.eq(piece_id_to_delete)))
        .execute(&mut conn)
        .unwrap();

    Json(json!({ "num_deleted": rows_deleted }))
}

async fn get_pieces_practiced(State(state): State<Arc<AppState>>) -> Json<Value> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::pieces_practiced::dsl::*;

    let all_pieces_practiced = pieces_practiced.load::<PiecePracticed>(&mut conn).unwrap();

    Json(json!({ "pieces_practiced": all_pieces_practiced }))
}

async fn add_piece_practiced(
    State(state): State<Arc<AppState>>,
    Json(new_piece_practiced): Json<NewPiecePracticed>,
) -> Json<Value> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::pieces_practiced::dsl::*;

    let session_id = new_piece_practiced.practice_session_id;

    diesel::insert_into(pieces_practiced)
        .values(new_piece_practiced)
        .execute(&mut conn)
        .unwrap();

    let pieces_from_session = pieces_practiced
        .filter(practice_session_id.eq(session_id))
        .load::<PiecePracticed>(&mut conn)
        .unwrap();

    Json(json!({ "practice_session_id": session_id, "pieces_from_session": pieces_from_session }))
}

async fn delete_piece_practiced(
    State(state): State<Arc<AppState>>,
    Path((practice_session_id_to_delete, piece_id_to_delete)): Path<(i32, i32)>,
) -> Json<Value> {
    let mut conn = state.db.clone().get().unwrap();

    use practice_app::schema::pieces_practiced::dsl::*;

    let rows_deleted = diesel::delete(
        pieces_practiced
            .filter(piece_id.eq(piece_id_to_delete))
            .filter(practice_session_id.eq(practice_session_id_to_delete)),
    )
    .execute(&mut conn)
    .unwrap();

    Json(json!({ "num_deleted": rows_deleted }))
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
        .unwrap();
}
