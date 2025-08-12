/// A CLI to track your LeetCode progress.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    command: Option<Commands>,

    /// Populates the database from a problem bank JSON file in the ./static/ directory.
    #[arg(long)] // This creates the `--build` flag
    build: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Shows the next unattempted problem to practice.
    #[command(name = "next", alias = "n")]
    Next,

    /// Logs an attempt for a specific problem.
    Attempt {
        /// The LeetCode ID of the problem.
        id: i64,

        /// Your rating of the attempt (1=ShortFail, 2=LongFail, 3=Messy, 4=Hard, 5=Easy).
        #[arg(value_parser = clap::value_parser!(u8).range(1..=5))]
        rating: u8,

        /// The date of the attempt in YYYY-MM-DD format (optional, defaults to today).
        date: Option<String>,
    },
}

/// Converts the 1-5 integer rating from the CLI to the AttemptRating enum.
fn map_rating(rating_num: u8) -> AttemptRating {
    match rating_num {
        1 => AttemptRating::ShortFail,
        2 => AttemptRating::LongFail,
        3 => AttemptRating::Messy,
        4 => AttemptRating::Hard,
        5 => AttemptRating::Easy,
        _ => unreachable!(), // Clap's range validation prevents this
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Database Setup ---
    let db_url = "sqlite:lc_tracking.db";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            db_url
                .parse::<sqlx::sqlite::SqliteConnectOptions>()?
                .create_if_missing(true),
        )
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    // --- Parse CLI commands ---
    let cli = Cli::parse();

    // --- Handle the --build flag first ---
    // If the user provides `--build <filename>`, we run the populator and exit.
    if let Some(bank_name) = cli.build {
        println!("\n--- Starting Problem Bank Population ---");
        if let Err(e) = populate_problem_bank(&pool, &bank_name).await {
            eprintln!("Error during population: {:?}", e);
        } else {
            println!("--- Population Task Finished ---");
        }
        return Ok(());
    }

    // --- Handle Subcommands if --build was not used ---
    if let Some(command) = cli.command {
        match command {
            Commands::Next => {
                println!("\n--- Finding next problem to attempt ---");
                match fetch_next_unattempted_problem(&pool).await {
                    Ok(Some(problem)) => {
                        println!("Next up is: #{} - {}", problem.order, problem.name);
                        println!("LeetCode ID: {}", problem.id);
                        if let Some(diff) = problem.difficulty {
                            println!("Difficulty: {:?}", diff);
                        }
                    }
                    Ok(None) => {
                        println!("🎉 Congratulations! You have attempted all problems!");
                    }
                    Err(e) => {
                        eprintln!("Error fetching next problem: {:?}", e);
                    }
                }
            }
            Commands::Attempt { id, rating, date } => {
                println!("\n--- Logging attempt for problem {} ---", id);
                let attempt_rating = map_rating(rating);
                let attempt_date = date
                    .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                    .transpose()
                    .context("Failed to parse date. Please use YYYY-MM-DD format.")?;

                if fetch_progress(&pool, id).await?.is_some() {
                    println!("Updating existing progress...");
                    update_progress(&pool, id, attempt_rating, attempt_date).await?;
                } else {
                    println!("Logging first attempt...");
                    add_or_replace_progress(&pool, id, attempt_rating, attempt_date).await?;
                }
                println!(
                    "Successfully logged attempt for problem {} with rating: {:?}",
                    id, attempt_rating
                );
            }
        }
    } else {
        // If no subcommand and no --build flag was given, print help.
        println!("No command given. Use --help to see available commands.");
    }

    Ok(())
}

pub mod db;
pub mod problem_attempts;
pub mod problem_bank;
pub mod problem_bank_populator;
pub mod problems;

use crate::problem_bank_populator::populate_problem_bank;
use anyhow::Context;
use clap::Parser;
use clap::Subcommand;
use db::*;
use problem_attempts::AttemptRating;
use problem_attempts::ProblemAttempt;
use problems::Problem;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::types::chrono::NaiveDate;
