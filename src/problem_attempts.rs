#[derive(Debug, FromRow)]
pub struct ProblemAttempt {
    pub problem_id: i64,
    pub last_attempted: NaiveDate,
    pub attempt_rating: AttemptRating,
    pub next_attempt_date: Option<NaiveDate>,
    pub number_of_attempts: i64,
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum AttemptRating {
    Easy,
    Hard,
    Messy,
    LongFail,
    ShortFail,
}

impl ProblemAttempt {
    pub fn new_attempt(
        problem_id: i64,
        attempt_rating: AttemptRating,
        attempt_date: Option<NaiveDate>,
    ) -> Self {
        let last_attempted = match attempt_date {
            Some(date) => date,
            None => Local::now().date_naive(),
        };

        ProblemAttempt {
            problem_id,
            last_attempted,
            attempt_rating,
            next_attempt_date: next_interval(attempt_rating, 0).map(|days| last_attempted + days),
            number_of_attempts: 1,
        }
    }

    pub fn update_attempt(
        &mut self,
        latest_rating: AttemptRating,
        attempt_date: Option<NaiveDate>,
    ) {
        self.attempt_rating = latest_rating;
        self.number_of_attempts += 1;

        self.last_attempted = match attempt_date {
            Some(date) => date,
            None => Local::now().date_naive(),
        };

        self.next_attempt_date = next_interval(latest_rating, self.number_of_attempts)
            .map(|days| self.last_attempted + days);
    }
}

fn next_interval(
    most_recent_attempt_rating: AttemptRating,
    total_number_of_attempts: i64,
) -> Option<Duration> {
    let very_clever_calculation_for_days = 1;
    Some(Duration::days(very_clever_calculation_for_days))
}

use chrono::{Duration, Local, NaiveDate};
use sqlx::FromRow;
