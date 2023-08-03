use diesel::r2d2::ConnectionManager;
use diesel::{mysql::MysqlConnection, r2d2::Pool};
use diesel::{prelude::*, sql_function};
pub mod models;
pub mod schema;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> Result<MysqlConnection, ConnectionError> {
    dotenv().expect(".env should load");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var should be set");

    MysqlConnection::establish(&database_url)
}

pub fn get_connection_pool() -> Pool<ConnectionManager<MysqlConnection>> {
    dotenv().expect(".env should load");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var should be set");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    Pool::builder()
        .build(manager)
        .expect("Should be able to build connection pool")
}

// danger?
sql_function!(fn last_insert_id() -> Integer);
