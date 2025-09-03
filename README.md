# bug

[![Crates.io](https://img.shields.io/crates/v/bug.svg)](https://crates.io/crates/bug)
[![Documentation](https://docs.rs/bug/badge.svg)](https://docs.rs/bug)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library for streamlined bug reporting that generates GitHub issue URLs with pre-filled templates. Works in both `std` and `no_std` environments. When bugs occur, `bug` prints tracing-style error messages to stderr and provides clean GitHub URLs for easy bug reporting.

## ✨ Features

- 🎯 **Template-based bug reporting** - Define reusable issue templates with named parameters
- 📄 **File-based templates** - Load templates from markdown files using `include_str!` macro  
- 🏷️ **Label support** - Automatically apply labels to GitHub issues
- ✅ **Parameter validation** - Ensures all template placeholders are properly filled at compile time
- 🌐 **URL encoding** - Handles special characters in URLs automatically
- 📁 **Multiple templates** - Support for different issue types per project
- 🛠️ **no_std support** - Works in embedded and no_std environments with `--no-default-features`
- 📦 **Handle-based API** - Alternative API that doesn't rely on global state

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bug = "0.2.0"
```

### no_std Support

For embedded or no_std environments:

```toml
[dependencies]
bug = { version = "0.2.0", default-features = false }
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

## 🛠️ no_std and Handle-based API

For `no_std` environments or when you prefer not to use global state, use the handle-based API:

```rust
use bug::{bug_with_handle, init_handle, IssueTemplate, FxHashMap};

fn main() {
    // Create a handle that doesn't use global state
    let bug_handle = init_handle("myorg", "myproject")
        .add_template("crash", IssueTemplate::new(
            "Application Crash: {error_type}",
            "## Description\nThe application crashed with error: {error_type}"
        ))
        .add_template("performance", IssueTemplate::new(
            "Performance Issue: {operation} is too slow",
            "Operation: {operation}\nExpected: {expected}ms\nActual: {actual}ms"
        ));

    // Use the handle to report bugs
    let url = bug_with_handle!(bug_handle, "crash", {
        error_type = "NullPointerException"
    });

    // Or call methods directly with FxHashMap
    let mut params = FxHashMap::default();
    params.insert("operation".to_string(), "database_query".to_string());
    params.insert("expected".to_string(), "100".to_string());
    params.insert("actual".to_string(), "1500".to_string());
    
    let direct_url = bug_handle.generate_url("performance", &params).unwrap();
}
```

### no_std Considerations

In `no_std` mode:
- Use `FxHashMap` instead of `std::collections::HashMap`
- Global `bug!()` macro returns empty string (use `bug_with_handle!()` instead)
- Terminal hyperlink detection is disabled (specify `HyperlinkMode::Always` or `Never` explicitly)
- Custom output via the `Output` trait for logging to different targets

### Custom Output in no_std

```rust
#[cfg(not(feature = "std"))]
use bug::{Output, BugReportHandle};

struct MyOutput;

impl Output for MyOutput {
    fn write_str(&mut self, s: &str) {
        // Send to your logging system, UART, etc.
        my_log_function(s);
    }
    
    fn write_fmt(&mut self, args: core::fmt::Arguments) {
        // Format and send to your logging system
        my_log_function(&format!("{}", args));
    }
}

let mut output = MyOutput;
let url = bug_handle.report_bug_with_output("crash", &params, file!(), line!(), &mut output);
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

- `init(owner, repo)` - Initialize bug reporting configuration (std only)
- `init_handle(owner, repo)` - Create a bug report handle (std and no_std)
- `bug!(template, {params})` - Report a bug with given template and parameters (std only)
- `bug_with_handle!(handle, template, {params})` - Report bug using handle (std and no_std)
- `create_terminal_hyperlink(url, text)` - Create ANSI hyperlink escape sequence
- `supports_hyperlinks()` - Detect terminal hyperlink support (std only, no_std returns false)

### Structs

- `IssueTemplate` - Represents a GitHub issue template
- `TemplateFile` - File-based template with validation  
- `BugReportConfigBuilder` - Fluent API for configuration (std only)
- `BugReportHandle` - Handle-based bug reporting (std and no_std)
- `HyperlinkMode` - Configure hyperlink display behavior

### Types

- `FxHashMap<K, V>` - HashMap type used by the API (re-exported from hashbrown)
- `Output` - Trait for custom output in no_std environments

### Enums

- `HyperlinkMode::Auto` - Auto-detect terminal support (default, std only)
- `HyperlinkMode::Always` - Always use hyperlinks
- `HyperlinkMode::Never` - Always show full URLs

### Macros

- `template_file!(path, labels: [...])` - Load template from file
- `bug!(template, {key = value, ...})` - Report bug with parameters (std only)
- `bug_with_handle!(handle, template, {key = value, ...})` - Report bug with handle (std and no_std)

### Feature Flags

- `std` (default) - Enable std support with global state and environment detection
- When `std` is disabled: no_std mode with handle-based API only

## 🧪 Examples

See the [`examples/`](examples/) directory for complete working examples:
- [`basic_usage.rs`](examples/basic_usage.rs) - Basic global state usage
- [`handle_usage.rs`](examples/handle_usage.rs) - Handle-based API (works in no_std)
- [`template_file_usage.rs`](examples/template_file_usage.rs) - File-based templates
- [`hyperlink_demo.rs`](examples/hyperlink_demo.rs) - Terminal hyperlink examples
- Template files in [`templates/`](templates/) directory

Run examples:
```bash
cargo run --example basic_usage
cargo run --example handle_usage
cargo run --example template_file_usage
```

For no_std examples:
```bash
cargo run --example handle_usage --no-default-features
```

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.