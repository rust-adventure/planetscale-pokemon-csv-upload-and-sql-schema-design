mod pokemon_csv;
use miette::{miette, IntoDiagnostic, WrapErr};
use pokemon_csv::*;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
mod db;
use db::*;

#[tokio::main]
async fn main() -> miette::Result<()> {
    let database_url = env::var("DATABASE_URL").map_err(|e| {
        miette!(
            help="Run `pscale connect <database> <branch>` to get a connection",
            "{e}"
        )
    })
    .wrap_err("Must have a DATABASE_URL set")?;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| {
            miette!(
                help="database urls must be in the form `mysql://username:password@host:port/database`",
                "{e}"
            )
        })?;

    let mut rdr = csv::Reader::from_path(
        "./crates/upload-pokemon-data/pokemon.csv",
    )
    .into_diagnostic()?;
    for result in rdr.deserialize() {
        let record: PokemonCsv =
            result.into_diagnostic()?;
        let pokemon_row: PokemonTableRow = record.into();
        println!(
            "{} {:?} {}",
            pokemon_row.pokedex_id,
            pokemon_row.id,
            pokemon_row.name
        );
        insert_pokemon(&pool, &pokemon_row)
            .await
            .into_diagnostic()?;
    }

    dbg!(PokemonId::new());

    Ok(())
}
