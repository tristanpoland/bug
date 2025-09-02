use bug_rs::{bug, init, IssueTemplate, HyperlinkMode};

fn main() -> Result<(), &'static str> {
    // Initialize with auto-detection (default)
    init("tristanpoland", "GLUE")
        .add_template("demo", IssueTemplate::new(
            "Demo Issue: {title}",
            "This is a demo issue with title: {title}"
        ))
        .hyperlinks(HyperlinkMode::Auto) // This is the default
        .build()?;

    println!("Auto-detect hyperlinks mode:");
    let _url1 = bug!("demo", {
        title = "Hyperlink Demo"
    });

    Ok(())
}