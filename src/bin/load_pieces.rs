use diesel::*;
use practice_app::{establish_connection, models::InsertablePiece, schema::pieces};
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
struct WorkJson {
    title: String,
}

#[derive(Deserialize)]
struct ComposerJson {
    complete_name: String,
    popular: String,
    works: Vec<WorkJson>,
}

#[derive(Deserialize)]
struct ResponseJson {
    composers: Vec<ComposerJson>,
}

#[tokio::main]
async fn main() {
    let mut conn = establish_connection().unwrap();

    let url = "https://api.openopus.org/work/dump.json";

    let body = reqwest::get(url)
        .await
        .unwrap()
        .json::<ResponseJson>()
        .await
        .unwrap();

    let pieces: Vec<InsertablePiece> = body
        .composers
        .into_iter()
        .filter(|composer| composer.popular == "1")
        .map(|composer| {
            composer
                .works
                .into_iter()
                .map(|work| InsertablePiece {
                    composer: composer.complete_name.clone(),
                    title: work.title,
                })
                .collect::<Vec<InsertablePiece>>()
        })
        .flatten()
        .collect();

    let non_duplicates = pieces
        .iter()
        .enumerate()
        .filter(|(index, piece)| *index == pieces.iter().position(|x| x == *piece).unwrap())
        .map(|(_, piece)| piece)
        .collect::<Vec<&InsertablePiece>>();

    let res = diesel::insert_into(pieces::table)
        .values(non_duplicates)
        .execute(&mut conn)
        .unwrap();

    println!("{res} rows affected");
}
