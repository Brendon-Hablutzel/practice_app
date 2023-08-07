use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::PiecePracticedMapping;
use practice_app::schema::pieces_practiced;

fn main() {
    let mut conn = establish_connection().unwrap();

    let all_pieces_practiced = pieces_practiced::table
        .load::<PiecePracticedMapping>(&mut conn)
        .unwrap();

    for mapping in all_pieces_practiced {
        println!("{mapping}");
    }
}
