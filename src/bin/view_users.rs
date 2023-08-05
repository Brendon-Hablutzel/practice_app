use diesel::prelude::*;
use practice_app::establish_connection;
use practice_app::models::User;
use practice_app::schema::users;

fn main() {
    let mut conn = establish_connection().unwrap();

    let all_users = users::table.load::<User>(&mut conn).unwrap();

    for user in all_users {
        println!("{user}");
    }
}
