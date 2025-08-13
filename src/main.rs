/// A CLI to track your LeetCode progress.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The command to execute. If no command is given, help is shown.
    #[command(subcommand)]
    command: Option<Commands>,

    /// Populates the database from a problem bank JSON file in the ./static/ directory.
    #[arg(long)]
    build: Option<String>,

    /// Shows current progress and statistics for all attempted problems.
    #[arg(long)]
    progress: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Shows the next unattempted problem to practice.
    #[command(name = "next", alias = "n")]
    Next {
        /// Display the problem details in a long, descriptive format.
        #[arg(long, short)]
        long: bool,
    },

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

    /// Shows all problems in the database, grouped by week.
    All,
}

/// Converts the 1-5 integer rating from the CLI to the AttemptRating enum.
fn map_rating(rating_num: u8) -> AttemptRating {
    match rating_num {
        1 => AttemptRating::ShortFail,
        2 => AttemptRating::LongFail,
        3 => AttemptRating::Messy,
        4 => AttemptRating::Hard,
        5 => AttemptRating::Easy,
        _ => unreachable!(),
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

    // --- Handle top-level flags first ---
    if let Some(bank_name) = cli.build {
        println!("\n--- Starting Problem Bank Population ---");
        if let Err(e) = populate_problem_bank(&pool, &bank_name).await {
            eprintln!("Error during population: {:?}", e);
        } else {
            println!("--- Population Task Finished ---");
        }
        return Ok(());
    }

    if cli.progress {
        println!("\n--- Current Progress ---");
        let progress_list = fetch_all_progress(&pool).await?;
        if progress_list.is_empty() {
            println!("No problems have been attempted yet. Use the 'attempt' command to start!");
        } else {
            for item in &progress_list {
                println!(
                    "  - #{:<5} {:<40} Rating: {:<10} Attempts: {}",
                    item.problem_id,
                    item.name,
                    format!("{:?}", item.attempt_rating),
                    item.number_of_attempts
                );
            }
            let mut stats: HashMap<AttemptRating, u32> = HashMap::new();
            for item in &progress_list {
                *stats.entry(item.attempt_rating).or_insert(0) += 1;
            }
            println!("\n--- Statistics ---");
            println!("Total Problems Attempted: {}", progress_list.len());
            for (rating, count) in stats {
                println!("  - {:<10}: {}", format!("{:?}", rating), count);
            }
        }
        return Ok(());
    }

    // --- Handle Subcommands ---
    if let Some(command) = cli.command {
        match command {
            Commands::Next { long } => match fetch_next_unattempted_problem(&pool).await {
                Ok(Some(problem)) => {
                    if long {
                        println!("\n--- Next Problem to Attempt ---");
                        println!("Order: #{}", problem.order);
                        println!("Name:  {}", problem.name);
                        println!("ID:    {}", problem.id);
                        if let Some(diff) = problem.difficulty {
                            println!("Diff:  {:?}", diff);
                        }
                    } else {
                        println!("{}", problem.id);
                    }
                }
                Ok(None) => {
                    if long {
                        println!("\n🎉 Congratulations! You have attempted all problems!");
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching next problem: {:?}", e);
                }
            },
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
            Commands::All => {
                println!("\n--- All Problems ---");
                let all_problems = fetch_all_problems(&pool).await?;
                if all_problems.is_empty() {
                    println!("No problems found in the database. Use the --build command to populate it.");
                } else {
                    let mut last_printed_week: Option<i64> = None;
                    for problem in &all_problems {
                        if problem.week != last_printed_week {
                            if let Some(week_num) = problem.week {
                                println!("\nWeek: {}", week_num);
                            } else {
                                println!("\nWeek: Unassigned");
                            }
                            last_printed_week = problem.week;
                        }
                        println!("  {}: {} - {}", problem.order, problem.name, problem.id);
                        if let Some(diff) = problem.difficulty {
                            println!("    Difficulty: {:?}", diff);
                        }
                    }
                }
            }
        }
    } else {
        // If no command or flag was given, print help.
        Cli::parse_from(["", "--help"]);
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
use std::collections::HashMap;
