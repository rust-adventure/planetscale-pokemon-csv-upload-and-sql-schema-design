mod pokemon_csv;
use miette::{miette, IntoDiagnostic, WrapErr};
use pokemon_csv::*;
use sqlx::mysql::MySqlPoolOptions;
use std::{collections::HashMap, env};
mod db;
use db::*;
use indicatif::ProgressIterator;

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

    let pokemon = rdr
        .deserialize()
        .collect::<Result<Vec<PokemonCsv>, csv::Error>>()
        .into_diagnostic()?;

    let mut pokemon_map: HashMap<String, PokemonId> =
        HashMap::new();

    for record in pokemon.clone().into_iter().progress() {
        let pokemon_row: PokemonTableRow =
            record.clone().into();
        insert_pokemon(&pool, &pokemon_row)
            .await
            .into_diagnostic()?;
        for ability in record.abilities.iter() {
            sqlx::query!(
                r#"
            INSERT INTO abilities (
                id, pokemon_id, ability
            ) VALUES (?, ?, ?)"#,
                PokemonId::new(),
                pokemon_row.id,
                ability,
            )
            .execute(&pool)
            .await
            .into_diagnostic()?;
        }
        for egg_group in record.egg_groups.iter() {
            sqlx::query!(
                r#"
            INSERT INTO egg_groups (
                id, pokemon_id, egg_group
            ) VALUES (?, ?, ?)"#,
                PokemonId::new(),
                pokemon_row.id,
                egg_group,
            )
            .execute(&pool)
            .await
            .into_diagnostic()?;
        }
        for typing in record.typing.iter() {
            sqlx::query!(
                r#"
            INSERT INTO typing (
                id, pokemon_id, typing
            ) VALUES (?, ?, ?)"#,
                PokemonId::new(),
                pokemon_row.id,
                typing,
            )
            .execute(&pool)
            .await
            .into_diagnostic()?;
        }
        pokemon_map.insert(record.name, pokemon_row.id);
    }

    for pokemon in pokemon
        .into_iter()
        .progress()
        .filter(|pokemon| pokemon.evolves_from.is_some())
    {
        let name = pokemon.evolves_from.expect(
            "Expected a value here since we just checked",
        );
        let pokemon_id = pokemon_map.get(&pokemon.name);
        let evolves_from_id = pokemon_map.get(&name);

        sqlx::query!(
            r#"
        INSERT INTO evolutions (
            id, pokemon_id, evolves_from
        ) VALUES (?, ?, ?)"#,
            PokemonId::new(),
            pokemon_id,
            evolves_from_id,
        )
        .execute(&pool)
        .await
        .into_diagnostic()?;
    }

    Ok(())
}
