// src/problem_bank_populator.rs

use crate::problem_bank::*;
use crate::problems::*;
use anyhow::Context;
use sqlx::SqlitePool;

pub async fn populate_problem_bank(pool: &SqlitePool, bank_name: &str) -> anyhow::Result<()> {
    println!("Attempting to load problem bank: '{}'...", bank_name);

    // Step 1: Load the raw problem data from the JSON file.
    let problems_from_json = load_problems(bank_name)
        .with_context(|| format!("Could not load data for bank '{}'", bank_name))?;

    println!(
        "Successfully loaded {} problems from JSON. Syncing with database...",
        problems_from_json.len()
    );

    // Step 2: Iterate through the loaded problems and insert them.
    for pbp in &problems_from_json {
        let problem_to_insert = Problem {
            id: pbp.id,
            order: pbp.order,
            name: pbp.name.clone(),
            difficulty: pbp.difficulty,
            week: pbp.week,
        };

        // Step 3: Call the insert method on the newly created `Problem` instance.
        problem_to_insert.insert(pool).await?;
    }

    println!("Database sync complete for bank '{}'.", bank_name);
    Ok(())
}
