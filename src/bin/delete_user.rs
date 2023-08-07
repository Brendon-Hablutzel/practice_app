use diesel::{delete, prelude::*};
use practice_app::establish_connection;
use practice_app::schema::users;
use std::env;

fn main() {
    let mut args = env::args();

    // skip first
    args.next();

    let user_id: i32 = args.next().unwrap().parse().unwrap();

    let mut conn = establish_connection().unwrap();

    let num_deleted = delete(users::table.filter(users::user_id.eq(user_id)))
        .execute(&mut conn)
        .unwrap();

    println!("Users: {num_deleted}");
}
