-- migrations/TIMESTAMP_create_progress_tables.sql

-- Table for the static problem definitions.
-- This will be populated from your JSON file.
CREATE TABLE problems (
    id INTEGER PRIMARY KEY,      -- The LeetCode problem ID.
    "order" INTEGER NOT NULL,    -- The order of the problem in the bank.
    name TEXT NOT NULL UNIQUE,   -- The name of the problem.
    difficulty TEXT,             -- 'Easy', 'Medium', 'Hard'. Nullable.
    week INTEGER                 -- The week number. Nullable.
);

-- Table for the dynamic progress summary.
-- This table will be updated as you attempt problems.
CREATE TABLE progress (
    -- This is both the Primary Key for this table and a
    -- Foreign Key pointing to the 'problems' table.
    problem_id INTEGER PRIMARY KEY,

    -- Fields from your ProblemAttempt struct
    last_attempted TEXT NOT NULL,
    attempt_rating TEXT NOT NULL,
    next_attempt_date TEXT,      -- Nullable
    number_of_attempts INTEGER NOT NULL,

    FOREIGN KEY (problem_id) REFERENCES problems(id) ON DELETE CASCADE
);
