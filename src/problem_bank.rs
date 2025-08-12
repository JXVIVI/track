#[derive(Debug, serde::Deserialize)]
pub struct ProblemBankProblem {
    pub id: i64,
    pub order: i64,
    pub name: String,
    pub difficulty: Option<LeetCodeDifficulty>,
    pub week: Option<i64>,
    pub url: String,
}

impl ProblemBankProblem {
    pub fn get_id(&self) -> anyhow::Result<i64> {
        let script_path = "./static/scripts/get_lc_id.sh";

        // 1. Set up the command to run the shell script.
        let output = Command::new(script_path)
            .arg(&self.url) // Pass the problem's URL as the first argument
            .output()
            .with_context(|| format!("Failed to execute script at '{}'. Is it executable (`chmod +x`) and in the correct path?", script_path))?;

        // 2. Check if the script itself exited with an error.
        if !output.status.success() {
            // If the script failed, its error messages are usually on stderr.
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Script execution failed with status {}:\n{}",
                output.status,
                stderr
            );
        }

        // 3. Process the successful output (stdout).
        let stdout_str = String::from_utf8(output.stdout)
            .context("Failed to read script output as UTF-8 string.")?;

        // 4. Trim whitespace (like newlines) and parse the string into an i64.
        let parsed_id = stdout_str.trim().parse::<i64>().with_context(|| {
            format!(
                "Failed to parse script output '{}' as a number.",
                stdout_str.trim()
            )
        })?;

        Ok(parsed_id)
    }

    pub fn to_problem(&self) -> anyhow::Result<Problem> {
        Ok(Problem {
            id: self.get_id()?,
            order: self.order,
            name: self.name.clone(),
            difficulty: self.difficulty,
            week: self.week,
        })
    }
}

pub fn load_problems(name: &str) -> anyhow::Result<Vec<ProblemBankProblem>> {
    let mut path = PathBuf::from(".");
    path.push("static");
    path.push(name);

    let file = File::open(path)?;

    let reader = BufReader::new(file);

    let problems = serde_json::from_reader(reader)?;

    Ok(problems)
}

use crate::problems::*;
use anyhow::Context;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
