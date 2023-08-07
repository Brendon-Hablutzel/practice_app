use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::{NewPiecePracticedMapping, PiecePracticedMapping};
use practice_app::schema::pieces_practiced;
use std::env;

fn main() {
    let mut args = env::args();

    args.next();

    let mapping = NewPiecePracticedMapping {
        practice_session_id: args.next().unwrap().parse().unwrap(),
        piece_id: args.next().unwrap().parse().unwrap(),
    };

    let mut conn = establish_connection().unwrap();

    let inserted_mapping: PiecePracticedMapping = diesel::insert_into(pieces_practiced::table)
        .values(mapping)
        .get_result(&mut conn)
        .unwrap();

    println!("Inserted piece practiced mapping: {inserted_mapping}");
}
