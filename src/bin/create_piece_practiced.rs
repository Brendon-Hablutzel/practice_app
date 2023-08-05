use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::NewPiecePracticed;
use practice_app::schema::pieces_practiced;
use std::env;

fn main() {
    let mut args = env::args();

    args.next(); // skip first

    let new_piece_practiced = NewPiecePracticed {
        practice_session_id: args.next().unwrap().parse::<i32>().unwrap(),
        piece_id: args.next().unwrap().parse::<i32>().unwrap(),
        user_id: args.next().unwrap().parse::<i32>().unwrap(),
    };

    let mut conn = establish_connection().unwrap();

    let affected_rows = diesel::insert_into(pieces_practiced::table)
        .values(new_piece_practiced)
        .execute(&mut conn)
        .unwrap();

    println!("Affected rows: {affected_rows}");
}
