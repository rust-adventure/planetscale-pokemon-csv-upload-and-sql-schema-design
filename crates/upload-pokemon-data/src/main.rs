use eyre::Result;
use indicatif::ProgressIterator;
use sqlx::mysql::MySqlPoolOptions;
use std::{
    collections::HashMap, convert::TryFrom, env, fs,
};

mod db;
mod pokemon_csv;

use db::*;
use pokemon_csv::*;

#[tokio::main]
async fn main() -> Result<()> {
    let database_url =
        env::var("DATABASE_URL").expect("a db url");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let pokemon_csv = fs::read_to_string("./pokemon.csv")?;
    let csv_size = pokemon_csv.lines().count();

    let mut csv_reader =
        csv::Reader::from_reader(pokemon_csv.as_bytes());
    let it = csv_reader.deserialize::<PokemonCsv>();

    let mut pokemon_map: HashMap<
        String,
        (PokemonCsv, PokemonTableRow),
    > = HashMap::new();

    for row in it.progress_count(u64::try_from(csv_size)?) {
        let pokemon = row?;
        let pokemon_db: PokemonTableRow =
            pokemon.clone().into();

        insert_pokemon(&pool, &pokemon_db).await?;

        for ability in pokemon.abilities.iter() {
            sqlx::query!(
                r#"
            INSERT INTO abilities_table (
                id, pokemon_id, ability
            ) VALUES (?, ?, ?)"#,
                PokemonId::new(),
                pokemon_db.id,
                ability,
            )
            .execute(&pool)
            .await?;
        }
        for egg_group in pokemon.egg_groups.iter() {
            sqlx::query!(
                r#"
            INSERT INTO egg_groups_table (
                id, pokemon_id, egg_group
            ) VALUES (?, ?, ?)"#,
                PokemonId::new(),
                pokemon_db.id,
                egg_group,
            )
            .execute(&pool)
            .await?;
        }
        for typing in pokemon.typing.iter() {
            sqlx::query!(
                r#"
            INSERT INTO typing_table (
                id, pokemon_id, typing
            ) VALUES (?, ?, ?)"#,
                PokemonId::new(),
                pokemon_db.id,
                typing,
            )
            .execute(&pool)
            .await?;
        }
        pokemon_map.insert(
            pokemon.name.clone(),
            (pokemon, pokemon_db),
        );
    }

    for (base_pokemon_id, evolves_from_name) in pokemon_map
        .iter()
        .filter_map(|(_, (pokemon, pokemon_db))| {
            pokemon
                .evolves_from
                .clone()
                .map(|name| (&pokemon_db.id, name))
        })
    {
        let evolves_from_pokemon = pokemon_map
            .iter()
            .find_map(|(_, (_, pkm_db))| {
                (pkm_db.name == evolves_from_name)
                    .then(|| &pkm_db.id)
            });

        if let Some(evolves_from_id) = evolves_from_pokemon
        {
            sqlx::query!(
                r#"
                INSERT INTO evolutions_table (
                    id, pokemon_id, evolves_from
                ) VALUES (?, ?, ?)"#,
                PokemonId::new(),
                base_pokemon_id,
                evolves_from_id,
            )
            .execute(&pool)
            .await?;
        };
    }

    Ok(())
}
