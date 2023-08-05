use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::PracticeSession;
use practice_app::schema::practice_sessions;

fn main() {
    let mut conn = establish_connection().unwrap();

    let all_practice_sessions = practice_sessions::table
        .load::<PracticeSession>(&mut conn)
        .unwrap();

    for practice_session in all_practice_sessions {
        println!("{practice_session}");
    }
}
