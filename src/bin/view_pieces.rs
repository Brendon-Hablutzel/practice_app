use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::Piece;
use practice_app::schema::pieces;

fn main() {
    let mut conn = establish_connection().unwrap();

    let all_pieces = pieces::table.load::<Piece>(&mut conn).unwrap();

    for piece in all_pieces {
        println!("{piece}");
    }
}
