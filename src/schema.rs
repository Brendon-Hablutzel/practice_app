// @generated automatically by Diesel CLI.

diesel::table! {
    pieces (piece_id) {
        piece_id -> Integer,
        title -> Varchar,
        composer -> Varchar,
    }
}

diesel::table! {
    pieces_practiced (practice_session_id, piece_id) {
        practice_session_id -> Integer,
        piece_id -> Integer,
    }
}

diesel::table! {
    practice_sessions (practice_session_id) {
        practice_session_id -> Integer,
        start_datetime -> Datetime,
        duration_mins -> Unsigned<Integer>,
        instrument -> Varchar,
    }
}

diesel::joinable!(pieces_practiced -> pieces (piece_id));
diesel::joinable!(pieces_practiced -> practice_sessions (practice_session_id));

diesel::allow_tables_to_appear_in_same_query!(
    pieces,
    pieces_practiced,
    practice_sessions,
);
