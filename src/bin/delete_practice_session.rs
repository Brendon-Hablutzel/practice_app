use diesel::{delete, prelude::*};
use practice_app::establish_connection;
use practice_app::schema::practice_sessions;
use std::env;

fn main() {
    let mut args = env::args();

    // skip first
    args.next();

    let practice_session_id: i32 = args.next().unwrap().parse().unwrap();

    let mut conn = establish_connection().unwrap();

    let num_deleted = delete(
        practice_sessions::table
            .filter(practice_sessions::practice_session_id.eq(practice_session_id)),
    )
    .execute(&mut conn)
    .unwrap();

    println!("Practice sessions deleted: {num_deleted}");
}
