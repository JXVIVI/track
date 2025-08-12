#[derive(Debug, FromRow)]
pub struct Problem {
    pub id: i64, // LeetCode ID
    pub order: i64,
    pub name: String,
    pub difficulty: Option<LeetCodeDifficulty>,
    pub week: Option<i64>,
}

impl Problem {
    pub async fn insert(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO problems (id, "order", name, difficulty, week)
            VALUES (?, ?, ?, ?, ?)
            "#,
            self.id,
            self.order,
            self.name,
            self.difficulty,
            self.week
        )
        .execute(pool)
        .await
        .with_context(|| format!("Failed to insert problem: {}", self.name))?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Deserialize)]
#[sqlx(type_name = "TEXT")]
pub enum LeetCodeDifficulty {
    Easy,
    Medium,
    Hard,
}

use anyhow::Context;
use sqlx::FromRow;
use sqlx::SqlitePool;
