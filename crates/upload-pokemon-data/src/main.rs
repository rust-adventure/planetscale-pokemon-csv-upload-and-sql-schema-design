mod pokemon_csv;
use miette::{miette, IntoDiagnostic, WrapErr};
use pokemon_csv::*;
use std::env;
mod db;
use db::*;

fn main() -> miette::Result<()> {
    let database_url = env::var("DATABASE_URL").map_err(|e| {
        miette!(
            help="Run `pscale connect <database> <branch>` to get a connection",
            "{e}"
        )
    })
    .wrap_err("Must have a DATABASE_URL set")?;

    let mut rdr = csv::Reader::from_path(
        "./crates/upload-pokemon-data/pokemon.csv",
    )
    .into_diagnostic()?;
    for result in rdr.deserialize() {
        let record: PokemonCsv =
            result.into_diagnostic()?;
        let pokemon_row: PokemonTableRow = record.into();
        dbg!(pokemon_row);
    }

    dbg!(PokemonId::new());

    Ok(())
}
