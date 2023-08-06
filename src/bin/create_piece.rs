use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::{NewPiece, Piece};
use practice_app::schema::pieces;
use std::env;

fn main() {
    let mut args = env::args();

    args.next(); // skip first

    let new_piece = NewPiece {
        title: args.next().unwrap(),
        composer: args.next().unwrap(),
    };

    let mut conn = establish_connection().unwrap();

    let inserted_piece: Piece = diesel::insert_into(pieces::table)
        .values(new_piece)
        .get_result(&mut conn)
        .unwrap();

    println!("Inserted: {inserted_piece}");
}
