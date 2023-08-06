// @generated automatically by Diesel CLI.

diesel::table! {
    pieces (piece_id) {
        piece_id -> Int4,
        title -> Varchar,
        composer -> Varchar,
    }
}

diesel::table! {
    pieces_practiced (practice_session_id, piece_id) {
        practice_session_id -> Int4,
        piece_id -> Int4,
    }
}

diesel::table! {
    practice_sessions (practice_session_id) {
        practice_session_id -> Int4,
        start_datetime -> Timestamp,
        duration_mins -> Int4,
        instrument -> Varchar,
        user_id -> Int4,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Int4,
        user_name -> Varchar,
        password_hash -> Varchar,
    }
}

diesel::joinable!(pieces_practiced -> pieces (piece_id));
diesel::joinable!(pieces_practiced -> practice_sessions (practice_session_id));
diesel::joinable!(practice_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    pieces,
    pieces_practiced,
    practice_sessions,
    users,
);
