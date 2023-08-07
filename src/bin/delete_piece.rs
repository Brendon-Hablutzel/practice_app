use diesel::{delete, prelude::*};
use practice_app::establish_connection;
use practice_app::schema::pieces;
use std::env;

fn main() {
    let mut args = env::args();

    // skip first
    args.next();

    let piece_id: i32 = args.next().unwrap().parse().unwrap();

    let mut conn = establish_connection().unwrap();

    let num_deleted = delete(pieces::table.filter(pieces::piece_id.eq(piece_id)))
        .execute(&mut conn)
        .unwrap();

    println!("Pieces deleted: {num_deleted}");
}
