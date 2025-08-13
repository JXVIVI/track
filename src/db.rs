use crate::problem_attempts::{AttemptRating, ProblemAttempt};
use crate::problems::LeetCodeDifficulty;
use crate::Problem;
use anyhow::Context;
use chrono::NaiveDate;
use sqlx::FromRow;
use sqlx::SqlitePool;

#[derive(Debug, FromRow)]
pub struct ProgressView {
    pub problem_id: i64,
    pub name: String,
    pub difficulty: Option<LeetCodeDifficulty>,
    pub last_attempted: NaiveDate,
    pub attempt_rating: AttemptRating,
    pub number_of_attempts: i64,
}

/// Fetches the current progress for a single problem from the database.
///
/// Returns `Ok(None)` if no progress has been logged for this problem yet.
pub async fn fetch_progress(
    pool: &SqlitePool,
    problem_id: i64,
) -> anyhow::Result<Option<ProblemAttempt>> {
    // THE FIX: Use the `query_as()` function instead of the `query_as!` macro.
    // This correctly leverages the `FromRow` trait on your `ProblemAttempt` struct
    // and the `Type` trait on your enums and NaiveDate.
    let progress =
        sqlx::query_as::<_, ProblemAttempt>("SELECT * FROM progress WHERE problem_id = ?")
            .bind(problem_id) // Use .bind() to pass arguments to a query_as function
            .fetch_optional(pool)
            .await
            .with_context(|| format!("Failed to fetch progress for problem_id: {}", problem_id))?;

    Ok(progress)
}

/// Adds a new progress entry or replaces an existing one for a given problem.
///
/// This function mirrors the logic of `ProblemAttempt::new_attempt`. It uses
/// `INSERT OR REPLACE` to ensure that there is always only one progress row
/// per problem, effectively overwriting any previous attempt history.
///
/// # Arguments
/// * `pool` - A reference to the `sqlx` connection pool.
/// * `problem_id` - The ID of the problem being attempted.
/// * `rating` - The `AttemptRating` for this new attempt.
/// * `attempt_date` - An optional date for the attempt. If `None`, today's date is used.
pub async fn add_or_replace_progress(
    pool: &SqlitePool,
    problem_id: i64,
    rating: AttemptRating,
    attempt_date: Option<NaiveDate>,
) -> anyhow::Result<()> {
    // Use your existing logic to construct the new progress state.
    let new_progress = ProblemAttempt::new_attempt(problem_id, rating, attempt_date);

    // Execute the query to insert or replace the row in the `progress` table.
    sqlx::query!(
        r#"
        INSERT OR REPLACE INTO progress (problem_id, last_attempted, attempt_rating, next_attempt_date, number_of_attempts)
        VALUES (?, ?, ?, ?, ?)
        "#,
        new_progress.problem_id,
        new_progress.last_attempted,
        new_progress.attempt_rating,
        new_progress.next_attempt_date,
        new_progress.number_of_attempts
    )
    .execute(pool)
    .await
    .with_context(|| format!("Failed to add/replace progress for problem_id: {}", problem_id))?;

    Ok(())
}

/// Updates the progress for a problem that has already been attempted.
///
/// This function mirrors the logic of `ProblemAttempt::update_attempt`. It will
/// first fetch the existing progress, update it in memory, and then write the
/// new state back to the database.
///
/// # Errors
/// Returns an error if no progress has been logged for the problem yet.
pub async fn update_progress(
    pool: &SqlitePool,
    problem_id: i64,
    latest_rating: AttemptRating,
    attempt_date: Option<NaiveDate>,
) -> anyhow::Result<()> {
    // 1. Fetch the current progress from the database.
    let mut current_progress = fetch_progress(pool, problem_id)
        .await?
        .context("Cannot update progress for a problem that has no attempts yet. Use `add_or_replace_progress` for the first attempt.")?;

    // 2. Use your existing logic to update the struct in memory.
    current_progress.update_attempt(latest_rating, attempt_date);

    // 3. Write the updated struct back to the database.
    sqlx::query!(
        r#"
        UPDATE progress
        SET last_attempted = ?, attempt_rating = ?, next_attempt_date = ?, number_of_attempts = ?
        WHERE problem_id = ?
        "#,
        current_progress.last_attempted,
        current_progress.attempt_rating,
        current_progress.next_attempt_date,
        current_progress.number_of_attempts,
        current_progress.problem_id
    )
    .execute(pool)
    .await
    .with_context(|| format!("Failed to update progress for problem_id: {}", problem_id))?;

    Ok(())
}

pub async fn fetch_next_unattempted_problem(pool: &SqlitePool) -> anyhow::Result<Option<Problem>> {
    // THE FIX: Use the `query_as()` function instead of the `query_as!` macro.
    // This correctly leverages the `FromRow` trait on your `Problem` struct.
    let next_problem = sqlx::query_as::<_, Problem>(
        r#"
        SELECT
            p.id, p."order", p.name, p.difficulty, p.week
        FROM
            problems p
        LEFT JOIN
            progress pr ON p.id = pr.problem_id
        WHERE
            pr.problem_id IS NULL
        ORDER BY
            p."order" ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch the next unattempted problem.")?;

    Ok(next_problem)
}

pub async fn fetch_all_progress(pool: &SqlitePool) -> anyhow::Result<Vec<ProgressView>> {
    let progress_list = sqlx::query_as::<_, ProgressView>(
        r#"
        SELECT
            p.id as problem_id,
            p.name,
            p.difficulty,
            pr.last_attempted,
            pr.attempt_rating,
            pr.number_of_attempts
        FROM
            progress pr
        JOIN
            problems p ON pr.problem_id = p.id
        ORDER BY
            pr.last_attempted DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch progress list from database.")?;

    Ok(progress_list)
}

pub async fn fetch_all_problems(pool: &SqlitePool) -> anyhow::Result<Vec<Problem>> {
    let all_problems = sqlx::query_as::<_, Problem>(
        r#"
        SELECT id, "order", name, difficulty, week
        FROM problems
        ORDER BY week ASC, "order" ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch all problems from the database.")?;

    Ok(all_problems)
}
