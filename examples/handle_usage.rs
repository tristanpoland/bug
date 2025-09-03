#![allow(unused)]

use bug::{bug_with_handle, init_handle, IssueTemplate, BugReportHandle, FxHashMap};

fn main() {
    // Create a bug handle that can be shared across crates
    // This doesn't use global state, so it's perfect for libraries
    let bug_handle = init_handle("myorg", "shared-project")
        .add_template("crash", IssueTemplate::new(
            "Application Crash: {error_type}",
            "## Description\nThe application crashed with error: {error_type}\n\n## Context\n- Function: {function}\n- Line: {line}\n- OS: {os}"
        ).with_labels(vec!["bug".to_string(), "crash".to_string()]))
        .add_template("performance", IssueTemplate::new(
            "Performance Issue: {operation} is too slow",
            "## Performance Problem\n\nOperation: {operation}\nExpected time: {expected}ms\nActual time: {actual}ms\n\n## Environment\nOS: {os}\nVersion: {version}"
        ).with_labels(vec!["performance".to_string()]));

    // Use the handle to report bugs - this works without global initialization
    let crash_url = bug_with_handle!(bug_handle, "crash", {
        error_type = "NullPointerException",
        function = "calculate_sum", 
        line = "42",
        os = std::env::consts::OS
    });

    let perf_url = bug_with_handle!(bug_handle, "performance", {
        operation = "database_query",
        expected = 100,
        actual = 1500,
        os = std::env::consts::OS,
        version = env!("CARGO_PKG_VERSION")
    });

    // You can also call methods directly on the handle
    let mut params = FxHashMap::default();
    params.insert("error_type".to_string(), "MemoryLeak".to_string());
    params.insert("function".to_string(), "allocate_buffer".to_string());
    params.insert("line".to_string(), "123".to_string());
    params.insert("os".to_string(), std::env::consts::OS.to_string());
    
    let direct_url = bug_handle.generate_url("crash", &params).unwrap();
    println!("Direct URL generation: {}", direct_url);

    // The handle can be cloned and passed around
    let cloned_handle = bug_handle.clone();
    demonstrate_in_another_function(cloned_handle);
}

fn demonstrate_in_another_function(handle: BugReportHandle) {
    // This function receives a handle and can use it independently
    bug_with_handle!(handle, "performance", {
        operation = "file_read",
        expected = 50,
        actual = 300,
        os = std::env::consts::OS,
        version = env!("CARGO_PKG_VERSION")
    });
}