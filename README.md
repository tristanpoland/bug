# bug

[![Crates.io](https://img.shields.io/crates/v/bug.svg)](https://crates.io/crates/bug)
[![Documentation](https://docs.rs/bug/badge.svg)](https://docs.rs/bug)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library for streamlined bug reporting that generates GitHub issue URLs with pre-filled templates. When bugs occur, `bug` prints tracing-style error messages to stderr and provides clean GitHub URLs for easy bug reporting.

## ✨ Features

- 🎯 **Template-based bug reporting** - Define reusable issue templates with named parameters
- 📄 **File-based templates** - Load templates from markdown files using `include_str!` macro  
- 🏷️ **Label support** - Automatically apply labels to GitHub issues
- ✅ **Parameter validation** - Ensures all template placeholders are properly filled at compile time
- 🌐 **URL encoding** - Handles special characters in URLs automatically
- 📁 **Multiple templates** - Support for different issue types per project

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bug = "0.1.0"
```

## 🚀 Quick Start

```rust
use bug::{bug, init, IssueTemplate};

fn main() -> Result<(), &'static str> {
    // Initialize with your GitHub repository
    init("username", "repository")
        .add_template("crash", IssueTemplate::new(
            "Application Crash: {error_type}",
            "## Description\nThe application crashed with error: {error_type}\n\n## Context\n- Function: {function}\n- Line: {line}"
        ).with_labels(vec!["bug".to_string(), "crash".to_string()]))
        .build()?;

    // Later in your code, when a bug occurs:
    let url = bug!("crash", {
        error_type = "NullPointerException",
        function = "calculate_sum",
        line = "42"
    });
    
    println!("Bug report URL: {}", url);
    Ok(())
}
```

This outputs:
```
🐛 BUG ENCOUNTERED in src/main.rs:15
   Template: crash
   Parameters:
     error_type: NullPointerException
     function: calculate_sum
     line: 42
   File a bug report: https://github.com/username/repository/issues/new?title=Application%20Crash%3A%20NullPointerException&body=...
```

## 📋 Template Files

Create structured markdown templates for consistent bug reports:

**templates/crash_report.md:**
```markdown
Application Crash: {error_type}

## Description
The application crashed with error: {error_type}

## Context
- Function: {function}
- Line: {line}
- OS: {os}
- Version: {version}

## Steps to Reproduce
1. {step1}
2. {step2}
3. {step3}

## Expected Behavior
{expected_behavior}

## Additional Information
{additional_info}
```

**Load in Rust:**
```rust
use bug::{init, template_file};

fn main() -> Result<(), &'static str> {
    init("myorg", "myproject")
        .add_template_file("crash", template_file!("templates/crash_report.md", labels: ["bug", "crash"]))
        .build()?;
    Ok(())
}
```

## 🎯 Usage Examples

### Multiple Template Types

```rust
use bug::{init, IssueTemplate, template_file};

fn main() -> Result<(), &'static str> {
    init("myorg", "myproject")
        // Simple inline template
        .add_template("simple", IssueTemplate::new(
            "Simple Issue: {title}",
            "Description: {description}"
        ))
        // File-based templates with labels
        .add_template_file("crash", template_file!("templates/crash_report.md", labels: ["bug", "crash"]))
        .add_template_file("performance", template_file!("templates/performance_issue.md", labels: ["performance", "optimization"]))
        .build()?;
    Ok(())
}
```

### Comprehensive Bug Reports

```rust
// Performance issue reporting
let url = bug!("performance", {
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

// Simple usage without parameters  
bug!("error");
```

## 📊 Output Format

The `bug!()` macro prints structured information to stderr:

```
🐛 BUG ENCOUNTERED in src/database.rs:127
   Template: performance
   Parameters:
     operation: database_query
     expected: 100
     actual: 1500
     ratio: 15
     os: windows
     version: 0.1.0
   File a bug report: https://github.com/myorg/myproject/issues/new?title=Performance%20Issue...
```

## 🔗 Terminal Hyperlinks & Clean URLs

Generated GitHub URLs can be quite long (800+ characters) due to URL-encoded template content, making log messages verbose and cluttering terminal output. Terminal hyperlink support using ANSI escape sequences to display clean, clickable text while hiding the long URL underneath.

### 🖱️ Hyperlink Modes

Configure how links are displayed:

```rust
use bug::{init, HyperlinkMode};

// Auto-detect terminal hyperlink support (default)
init("user", "repo")
    .hyperlinks(HyperlinkMode::Auto)
    .build()?;

// Always use hyperlinks  
init("user", "repo")
    .hyperlinks(HyperlinkMode::Always)
    .build()?;

// Never use hyperlinks (show full URLs)
init("user", "repo")
    .hyperlinks(HyperlinkMode::Never)
    .build()?;
```

### 📺 Supported Terminals

Automatic detection works with:
- **Windows Terminal**
- **iTerm2** (macOS)
- **WezTerm**
- **Alacritty** 
- **VS Code Integrated Terminal**
- Most **xterm-compatible** terminals

### 📄 Output Comparison

**With Hyperlinks** (clean):
```
🐛 BUG ENCOUNTERED in src/main.rs:45
   Template: crash
   Parameters:
     error_type: NullPointerException
     function: calculate_sum
   File a bug report
```

**Without Hyperlinks** (traditional):
```
🐛 BUG ENCOUNTERED in src/main.rs:45
   Template: crash  
   Parameters:
     error_type: NullPointerException
     function: calculate_sum
   File a bug report: https://github.com/user/repo/issues/new?title=Application%20Crash...
```

The hyperlink text "File a bug report" becomes clickable and opens the full GitHub issue URL when clicked, keeping your logs clean while maintaining full functionality.

## 📚 API Reference

### Core Functions

- `init(owner, repo)` - Initialize bug reporting configuration
- `bug!(template, {params})` - Report a bug with given template and parameters
- `create_terminal_hyperlink(url, text)` - Create ANSI hyperlink escape sequence
- `supports_hyperlinks()` - Detect terminal hyperlink support

### Structs

- `IssueTemplate` - Represents a GitHub issue template
- `TemplateFile` - File-based template with validation
- `BugReportConfigBuilder` - Fluent API for configuration
- `HyperlinkMode` - Configure hyperlink display behavior

### Enums

- `HyperlinkMode::Auto` - Auto-detect terminal support (default)
- `HyperlinkMode::Always` - Always use hyperlinks
- `HyperlinkMode::Never` - Always show full URLs

### Macros

- `template_file!(path, labels: [...])` - Load template from file
- `bug!(template, {key = value, ...})` - Report bug with parameters

## 🧪 Examples

See the [`examples/`](examples/) directory for complete working examples:
- [`template_file_usage.rs`](examples/template_file_usage.rs) - File-based templates
- Template files in [`templates/`](templates/) directory

Run examples:
```bash
cargo run --example template_file_usage
```

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.