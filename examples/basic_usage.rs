#![allow(unused)]

use bug_rs::{bug, init, IssueTemplate};

fn main() -> Result<(), &'static str> {
    // Initialize the bug reporter with your GitHub repo
    init("tristanpoland", "GLUE")
        .add_template("crash", IssueTemplate::new(
            "Application Crash: {error_type}",
            "## Description\nThe application crashed with error: {error_type}\n\n## Context\n- Function: {function}\n- Line: {line}\n- OS: {os}"
        ).with_labels(vec!["bug".to_string(), "crash".to_string()]))
        .add_template("performance", IssueTemplate::new(
            "Performance Issue: {operation} is too slow",
            "## Performance Problem\n\nOperation: {operation}\nExpected time: {expected}ms\nActual time: {actual}ms\n\n## Environment\nOS: {os}\nVersion: {version}"
        ).with_labels(vec!["performance".to_string()]))
        .build()?;

    // Simulate different types of bugs
    let crash_url = bug!("crash", {
        error_type = "NullPointerException",
        function = "calculate_sum",
        line = "42",
        os = std::env::consts::OS
    });
    
    let perf_url = bug!("performance", {
        operation = "database_query",
        expected = 100,
        actual = 1500,
        os = std::env::consts::OS,
        version = env!("CARGO_PKG_VERSION")
    });

    bug!("crash");

    Ok(())
}