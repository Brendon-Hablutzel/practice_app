use argon2;
use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::User;
use practice_app::schema::users;
use rand::Rng;
use std::env;

fn main() {
    let mut args = env::args();

    args.next();

    let user_name = args.next().unwrap();
    let password = args.next().unwrap();

    let mut conn = establish_connection().unwrap();

    let salt = rand::thread_rng().gen::<[u8; 16]>();

    let hashed_password =
        argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default()).unwrap();

    let inserted_user: User = diesel::insert_into(users::table)
        .values((
            users::user_name.eq(user_name),
            users::password_hash.eq(hashed_password),
        ))
        .get_result(&mut conn)
        .unwrap();

    println!("Inserted user: {inserted_user}");
}
