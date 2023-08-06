use chrono::NaiveDateTime;
use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::{NewPracticeSession, PracticeSession};
use practice_app::schema::practice_sessions;
use std::env;

fn main() {
    let mut args = env::args();

    args.next(); // skip first

    let new_practice_session = NewPracticeSession {
        start_datetime: NaiveDateTime::parse_from_str(&args.next().unwrap(), "%Y-%m-%dT%H:%M:%S")
            .unwrap(),
        duration_mins: args.next().unwrap().parse::<i32>().unwrap(),
        instrument: args.next().unwrap(),
        user_id: args.next().unwrap().parse::<i32>().unwrap(),
    };

    let mut conn = establish_connection().unwrap();

    let inserted_practice_session: PracticeSession = diesel::insert_into(practice_sessions::table)
        .values(new_practice_session)
        .get_result(&mut conn)
        .unwrap();

    println!("Inserted practice session: {inserted_practice_session}");
}
