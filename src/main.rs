use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::*;

fn view_practice_sessions(connection: &mut MysqlConnection) {
    use practice_app::schema::practice_sessions::dsl::*;

    let results = practice_sessions
        .load::<PracticeSession>(connection)
        .unwrap();
    for session in results {
        println!(
            "{} practice session started at {} and lasted {} mins (id: {})",
            session.instrument,
            session.start_datetime,
            session.duration_mins,
            session.practice_session_id
        );
    }
}

fn view_pieces(connection: &mut MysqlConnection) {
    use practice_app::schema::pieces::dsl::*;

    let results = pieces.load::<Piece>(connection).unwrap();
    for piece in results {
        println!(
            "piece: {} by {} (id: {})",
            piece.title, piece.composer, piece.piece_id
        );
    }
}

fn view_pieces_practiced(connection: &mut MysqlConnection) {
    use practice_app::schema::pieces_practiced::dsl::*;

    let results = pieces_practiced.load::<PiecePracticed>(connection).unwrap();
    for entry in results {
        println!(
            "session: {}, piece: {}",
            entry.practice_session_id, entry.piece_id
        );
    }
}

fn main() {
    let mut connection = establish_connection().unwrap();

    println!("practice sessions:");
    view_practice_sessions(&mut connection);

    println!("\npieces:");
    view_pieces(&mut connection);

    println!("\npieces_practiced:");
    view_pieces_practiced(&mut connection);
}
