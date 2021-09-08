mod db;
mod pokemon_csv;
use color_eyre::{eyre, eyre::WrapErr, Section};
use db::*;
use pokemon_csv::*;
use std::env;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let database_url = env::var("DATABASE_URL")
        .wrap_err("Must have a DATABASE_URL set")
        .suggestion("Run `pscale connect <database> <branch>` to get a connection")?;

    let mut rdr = csv::Reader::from_path(
        "./crates/upload-pokemon-data/pokemon.csv",
    )?;
    for result in rdr.deserialize() {
        let record: PokemonCsv = result?;
        let pokemon_row: PokemonTableRow = record.into();
        println!("{:?}", pokemon_row);
    }
    Ok(())
}
