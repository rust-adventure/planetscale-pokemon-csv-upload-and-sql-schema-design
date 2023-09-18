mod pokemon_csv;
use pokemon_csv::*;
mod db;
use db::*;

fn main() -> Result<(), csv::Error> {
    let mut rdr = csv::Reader::from_path(
        "./crates/upload-pokemon-data/pokemon.csv",
    )?;
    for result in rdr.deserialize() {
        let record: PokemonCsv = result?;
        let pokemon_row: PokemonTableRow = record.into();
        dbg!(pokemon_row);
    }

    dbg!(PokemonId::new());

    Ok(())
}
