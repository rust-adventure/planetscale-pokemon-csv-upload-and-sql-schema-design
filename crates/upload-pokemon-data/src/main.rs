mod db;
mod pokemon_csv;
use color_eyre::{eyre, eyre::WrapErr, Section};
use db::*;
use indicatif::ProgressIterator;
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
        .max_connections(5)
        .connect(&database_url)
        .await
        .suggestion("database urls must be in the form `mysql://username:password@host:port/database`")?;

    let mut rdr = csv::Reader::from_path(
        "./crates/upload-pokemon-data/pokemon.csv",
    )?;
    let pokemon = rdr
        .deserialize()
        .collect::<Result<Vec<PokemonCsv>, csv::Error>>()?;

    for record in pokemon.into_iter().progress() {
        let pokemon_row: PokemonTableRow = record.into();
        insert_pokemon(&pool, &pokemon_row).await?;
    }
    Ok(())
}
