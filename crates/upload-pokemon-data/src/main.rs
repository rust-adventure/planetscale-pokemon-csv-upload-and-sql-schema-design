mod db;
mod pokemon_csv;
use color_eyre::{eyre, eyre::WrapErr, Section};
use db::*;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use pokemon_csv::*;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let database_url = env::var("DATABASE_URL")
        .wrap_err("Must have a DATABASE_URL set")
        .suggestion("Run `pscale connect <database> <branch>` to get a connection")?;

    let pool = MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .suggestion("database urls must be in the form `mysql://username:password@host:port/database`")?;

    let mut rdr = csv::Reader::from_path(
        "./crates/upload-pokemon-data/pokemon.csv",
    )?;
    let mut work: FuturesUnordered<_> = rdr
        .deserialize()
        .map(|result| {
            let record: PokemonCsv = result.unwrap();
            let pokemon_row: PokemonTableRow =
                record.into();
            println!(
                "{} {:?} {}",
                pokemon_row.pokedex_id,
                pokemon_row.id,
                pokemon_row.name
            );
            tokio::spawn(insert_pokemon(
                pool.clone(),
                pokemon_row,
            ))
        })
        .collect();
    let mut i = 0;
    while let Some(item) = work.next().await {
        item??;
        i += 1;
        println!("{}", i);
    }
    Ok(())
}
