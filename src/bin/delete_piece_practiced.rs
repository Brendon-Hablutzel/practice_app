use diesel::{delete, prelude::*};
use practice_app::establish_connection;
use practice_app::schema::pieces_practiced;
use std::env;

fn main() {
    let mut args = env::args();

    // skip first
    args.next();

    let practice_session_id: i32 = args.next().unwrap().parse().unwrap();
    let piece_id: i32 = args.next().unwrap().parse().unwrap();

    let mut conn = establish_connection().unwrap();

    let num_deleted = delete(
        pieces_practiced::table
            .filter(pieces_practiced::practice_session_id.eq(practice_session_id))
            .filter(pieces_practiced::piece_id.eq(piece_id)),
    )
    .execute(&mut conn)
    .unwrap();

    println!("Piece practiced mappings deleted: {num_deleted}");
}
