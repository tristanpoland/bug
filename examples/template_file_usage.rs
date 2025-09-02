#![allow(unused)]

use bug_rs::{bug, init, template_file};

fn main() -> Result<(), &'static str> {
    init("tristanpoland", "GLUE")
        .add_template_file("crash", template_file!("../templates/crash_report.md", labels: ["bug", "crash"]))
        .add_template_file("performance", template_file!("../templates/performance_issue.md", labels: ["performance", "optimization"]))
        .build()?;

    let crash_url = bug!("crash", {
        error_type = "NullPointerException",
        function = "calculate_sum",
        line = "42",
        os = std::env::consts::OS,
        version = env!("CARGO_PKG_VERSION"),
        step1 = "Open the application",
        step2 = "Click on calculate button",
        step3 = "Application crashes",
        expected_behavior = "Should calculate the sum correctly",
        additional_info = "This happens only on Windows 11"
    });

    let perf_url = bug!("performance", {
        operation = "database_query",
        expected = "100",
        actual = "1500",
        ratio = "15",
        os = std::env::consts::OS,
        version = env!("CARGO_PKG_VERSION"),
        hardware = "Intel i7-12700K, 32GB RAM",
        profiling_data = "CPU: 45%, Memory: 2.1GB, Disk I/O: 150MB/s",
        impact_description = "User experience is significantly degraded"
    });

    Ok(())
}