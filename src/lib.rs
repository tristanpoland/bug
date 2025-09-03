#![cfg_attr(not(feature = "std"), no_std)]

//! # bug_rs
//!
//! A simple Rust library for printing a tracing-style error in the event of a bug and allowing users to easily file a bug report via GitHub issues using bug templates.
//!
//! ## Features
//!
//! - Define reusable issue templates with named parameters
//! - Load templates from markdown files using `include_str!` macro
//! - Print formatted bug reports to stderr (similar to tracing output)
//! - Generate clean GitHub issue URLs with pre-filled templates
//! - Support for multiple issue templates per project
//! - URL encoding for special characters
//! - Optional labels for GitHub issues
//! - Parameter validation ensures all placeholders are filled
//!
//! ## Quick Start
//!
//! ```rust
//! use bug_rs::{bug, init, IssueTemplate};
//!
//! # fn main() -> Result<(), &'static str> {
//! // Initialize with your GitHub repository
//! init("username", "repository")
//!     .add_template("crash", IssueTemplate::new(
//!         "Application Crash: {error_type}",
//!         "## Description\nThe application crashed with error: {error_type}\n\n## Context\n- Function: {function}\n- Line: {line}"
//!     ).with_labels(vec!["bug".to_string(), "crash".to_string()]))
//!     .build()?;
//!
//! // Later in your code, when a bug occurs:
//! let url = bug!("crash", {
//!     error_type = "NullPointerException",
//!     function = "calculate_sum",
//!     line = "42"
//! });
//! 
//! println!("Bug report URL: {}", url);
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Usage
//!
//! ### Multi-Crate Projects with Shared Bug Definitions
//!
//! For multi-crate projects, you can create a shared bug reporting handle that doesn't rely on global state:
//!
//! ```rust
//! use bug_rs::{bug_with_handle, init_handle, IssueTemplate, BugReportHandle};
//!
//! # fn main() {
//! // Create a handle that can be shared across crates
//! let bug_handle = init_handle("myorg", "myproject")
//!     .add_template("crash", IssueTemplate::new(
//!         "Application Crash: {error_type}",
//!         "## Description\nThe application crashed with error: {error_type}"
//!     ))
//!     .add_template("performance", IssueTemplate::new(
//!         "Performance Issue: {operation} is too slow",
//!         "Operation: {operation}\nExpected: {expected}ms\nActual: {actual}ms"
//!     ));
//!
//! // Use the handle to report bugs
//! let url = bug_with_handle!(bug_handle, "crash", {
//!     error_type = "NullPointerException"
//! });
//! # }
//! ```
//!
//! ### Using Template Files
//!
//! Create a markdown template file with placeholders:
//!
//! **templates/crash_report.md:**
//! ```markdown
//! Application Crash: {error_type}
//!
//! ## Description
//! The application crashed with error: {error_type}
//!
//! ## Context
//! - Function: {function}
//! - Line: {line}
//! - OS: {os}
//! ```
//!
//! Then load it in your Rust code:
//!
//! ```rust
//! use bug_rs::{init, template_file};
//!
//! # fn main() -> Result<(), &'static str> {
//! init("myorg", "myproject")
//!     .add_template_file("crash", template_file!("templates/crash_report.md", labels: ["bug", "crash"]))
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Multiple Templates
//!
//! ```rust
//! use bug_rs::{init, IssueTemplate, template_file};
//!
//! # fn main() -> Result<(), &'static str> {
//! init("myorg", "myproject")
//!     .add_template("simple", IssueTemplate::new(
//!         "Simple Issue: {title}",
//!         "Description: {description}"
//!     ))
//!     .add_template_file("detailed", template_file!("templates/detailed_report.md"))
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Using the bug! Macro
//!
//! The `bug!` macro provides a clean API similar to tracing macros:
//!
//! ```rust
//! # use bug_rs::{bug, init, IssueTemplate};
//! # init("test", "test").add_template("error", IssueTemplate::new("Error", "Body")).build().unwrap();
//!
//! // Simple usage without parameters
//! bug!("error");
//!
//! // With named parameters
//! bug!("performance", {
//!     operation = "database_query",
//!     expected = 100,
//!     actual = 1500,
//!     os = std::env::consts::OS,
//!     version = env!("CARGO_PKG_VERSION")
//! });
//! ```
//!
//! ## Output Format
//!
//! When `bug!()` is called, it prints to stderr in a format similar to tracing:
//!
//! ```text
//! 🐛 BUG ENCOUNTERED in src/main.rs:45
//!    Template: crash
//!    Parameters:
//!      error_type: NullPointerException
//!      function: calculate_sum
//!      line: 42
//!    File a bug report: https://github.com/username/repository/issues/new?title=Application%20Crash%3A%20NullPointerException&body=...
//! ```

mod url_encode;

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
    format,
};

use hashbrown::HashMap;
use rustc_hash::FxHasher;
use core::hash::BuildHasherDefault;

pub type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[cfg(feature = "std")]
use once_cell::sync::OnceCell;

#[cfg(feature = "std")]
static CONFIG: OnceCell<BugReportConfig> = OnceCell::new();

#[cfg(not(feature = "std"))]
static mut CONFIG: Option<BugReportConfig> = None;

/// Output trait for no_std compatibility
pub trait Output {
    fn write_str(&mut self, s: &str);
    fn write_fmt(&mut self, args: core::fmt::Arguments);
}

#[cfg(feature = "std")]
impl Output for std::io::Stderr {
    fn write_str(&mut self, s: &str) {
        eprint!("{}", s);
    }
    
    fn write_fmt(&mut self, args: core::fmt::Arguments) {
        eprint!("{}", args);
    }
}

/// Default no-op output for no_std
pub struct NoOutput;

impl Output for NoOutput {
    fn write_str(&mut self, _s: &str) {}
    fn write_fmt(&mut self, _args: core::fmt::Arguments) {}
}

#[derive(Debug, Clone)]
pub struct BugReportConfig {
    pub github_owner: String,
    pub github_repo: String,
    pub templates: FxHashMap<String, IssueTemplate>,
    pub template_files: FxHashMap<String, TemplateFile>,
    pub use_hyperlinks: HyperlinkMode,
}

#[derive(Debug, Clone)]
pub enum HyperlinkMode {
    /// Automatically detect terminal hyperlink support
    Auto,
    /// Always use hyperlinks
    Always,
    /// Never use hyperlinks (show full URLs)
    Never,
}

#[derive(Debug, Clone)]
pub struct IssueTemplate {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateFile {
    pub content: &'static str,
    pub labels: Vec<String>,
}

impl TemplateFile {
    pub fn new(content: &'static str) -> Self {
        Self {
            content,
            labels: Vec::new(),
        }
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn parse(&self) -> Result<IssueTemplate, String> {
        let lines: Vec<&str> = self.content.lines().collect();
        
        if lines.is_empty() {
            return Err("Template file is empty".to_string());
        }

        let title = lines[0].trim();
        if title.is_empty() {
            return Err("Template must have a title on the first line".to_string());
        }

        let body = if lines.len() > 1 {
            lines[1..].join("\n").trim().to_string()
        } else {
            String::new()
        };

        Ok(IssueTemplate {
            title: title.to_string(),
            body,
            labels: self.labels.clone(),
        })
    }

    pub fn validate_params(&self, params: &FxHashMap<String, String>) -> Result<(), String> {
        let placeholders = extract_placeholders(self.content);
        
        for placeholder in &placeholders {
            if !params.contains_key(placeholder) {
                return Err(format!("Missing required parameter: {}", placeholder));
            }
        }

        for param_key in params.keys() {
            if !placeholders.contains(param_key) {
                return Err(format!("Unused parameter: {}", param_key));
            }
        }

        Ok(())
    }
}

impl IssueTemplate {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            labels: Vec::new(),
        }
    }

    pub fn from_template_file(template_file: &TemplateFile, params: &FxHashMap<String, String>) -> Result<Self, String> {
        template_file.validate_params(params)?;
        let parsed_template = template_file.parse()?;
        Ok(parsed_template.fill_params(params))
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn fill_params(&self, params: &FxHashMap<String, String>) -> IssueTemplate {
        let mut filled_title = self.title.clone();
        let mut filled_body = self.body.clone();

        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            filled_title = filled_title.replace(&placeholder, value);
            filled_body = filled_body.replace(&placeholder, value);
        }

        IssueTemplate {
            title: filled_title,
            body: filled_body,
            labels: self.labels.clone(),
        }
    }
}

fn extract_placeholders(content: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut chars = content.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut placeholder = String::new();
            let mut found_end = false;
            
            while let Some(inner_ch) = chars.next() {
                if inner_ch == '}' {
                    found_end = true;
                    break;
                } else if inner_ch.is_alphanumeric() || inner_ch == '_' {
                    placeholder.push(inner_ch);
                } else {
                    placeholder.clear();
                    break;
                }
            }
            
            if found_end && !placeholder.is_empty() && !placeholders.contains(&placeholder) {
                placeholders.push(placeholder);
            }
        }
    }
    
    placeholders
}

#[macro_export]
macro_rules! template_file {
    ($path:expr) => {
        $crate::TemplateFile::new(include_str!($path))
    };
    ($path:expr, labels: [$($label:expr),* $(,)?]) => {
        $crate::TemplateFile::new(include_str!($path))
            .with_labels(vec![$($label.to_string()),*])
    };
}

pub fn init(github_owner: impl Into<String>, github_repo: impl Into<String>) -> BugReportConfigBuilder {
    BugReportConfigBuilder::new(github_owner.into(), github_repo.into())
}

pub fn init_handle(github_owner: impl Into<String>, github_repo: impl Into<String>) -> BugReportHandle {
    BugReportHandle::new(github_owner.into(), github_repo.into())
}

pub struct BugReportConfigBuilder {
    config: BugReportConfig,
}

impl BugReportConfigBuilder {
    fn new(github_owner: String, github_repo: String) -> Self {
        Self {
            config: BugReportConfig {
                github_owner,
                github_repo,
                templates: FxHashMap::default(),
                template_files: FxHashMap::default(),
                use_hyperlinks: HyperlinkMode::Auto,
            },
        }
    }

    pub fn add_template(mut self, name: impl Into<String>, template: IssueTemplate) -> Self {
        self.config.templates.insert(name.into(), template);
        self
    }

    pub fn add_template_file(mut self, name: impl Into<String>, template_file: TemplateFile) -> Self {
        self.config.template_files.insert(name.into(), template_file);
        self
    }

    pub fn hyperlinks(mut self, mode: HyperlinkMode) -> Self {
        self.config.use_hyperlinks = mode;
        self
    }

    #[cfg(feature = "std")]
    pub fn build(self) -> Result<(), &'static str> {
        CONFIG.set(self.config).map_err(|_| "Bug reporting already initialized")
    }
    
    #[cfg(not(feature = "std"))]
    pub unsafe fn build(self) -> Result<(), &'static str> {
        unsafe {
            match CONFIG {
                Some(_) => return Err("Bug reporting already initialized"),
                None => CONFIG = Some(self.config),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BugReportHandle {
    config: BugReportConfig,
}

impl BugReportHandle {
    fn new(github_owner: String, github_repo: String) -> Self {
        Self {
            config: BugReportConfig {
                github_owner,
                github_repo,
                templates: FxHashMap::default(),
                template_files: FxHashMap::default(),
                use_hyperlinks: HyperlinkMode::Auto,
            },
        }
    }

    pub fn add_template(mut self, name: impl Into<String>, template: IssueTemplate) -> Self {
        self.config.templates.insert(name.into(), template);
        self
    }

    pub fn add_template_file(mut self, name: impl Into<String>, template_file: TemplateFile) -> Self {
        self.config.template_files.insert(name.into(), template_file);
        self
    }

    pub fn hyperlinks(mut self, mode: HyperlinkMode) -> Self {
        self.config.use_hyperlinks = mode;
        self
    }

    pub fn generate_url(&self, template_name: &str, params: &FxHashMap<String, String>) -> Result<String, String> {
        let filled_template = if let Some(template) = self.config.templates.get(template_name) {
            template.fill_params(params)
        } else if let Some(template_file) = self.config.template_files.get(template_name) {
            IssueTemplate::from_template_file(template_file, params)?
        } else {
            return Err(format!("Template '{}' not found", template_name));
        };
        
        let mut url = format!(
            "https://github.com/{}/{}/issues/new",
            self.config.github_owner, self.config.github_repo
        );

        let mut query_params = Vec::new();
        
        if !filled_template.title.is_empty() {
            query_params.push(format!("title={}", url_encode::encode(&filled_template.title)));
        }
        
        if !filled_template.body.is_empty() {
            query_params.push(format!("body={}", url_encode::encode(&filled_template.body)));
        }
        
        if !filled_template.labels.is_empty() {
            let labels_str = filled_template.labels.join(",");
            query_params.push(format!("labels={}", url_encode::encode(&labels_str)));
        }

        if !query_params.is_empty() {
            url.push('?');
            url.push_str(&query_params.join("&"));
        }

        Ok(url)
    }

    pub fn report_bug(&self, template_name: &str, params: &FxHashMap<String, String>, file: &str, line: u32) -> String {
        self.report_bug_with_output(template_name, params, file, line, &mut NoOutput)
    }
    
    #[cfg(feature = "std")]
    pub fn report_bug_stderr(&self, template_name: &str, params: &FxHashMap<String, String>, file: &str, line: u32) -> String {
        self.report_bug_with_output(template_name, params, file, line, &mut std::io::stderr())
    }
    
    pub fn report_bug_with_output(&self, template_name: &str, params: &FxHashMap<String, String>, file: &str, line: u32, output: &mut dyn Output) -> String {
        match self.generate_url(template_name, params) {
            Ok(url) => {
                output.write_fmt(format_args!("🐛 BUG ENCOUNTERED in {}:{}\n", file, line));
                output.write_fmt(format_args!("   Template: {}\n", template_name));
                if !params.is_empty() {
                    output.write_str("   Parameters:\n");
                    for (key, value) in params {
                        output.write_fmt(format_args!("     {}: {}\n", key, value));
                    }
                }
                let should_use_hyperlinks = match self.config.use_hyperlinks {
                    HyperlinkMode::Auto => supports_hyperlinks(),
                    HyperlinkMode::Always => true,
                    HyperlinkMode::Never => false,
                };
                
                if should_use_hyperlinks {
                    output.write_fmt(format_args!("   {}\n", create_terminal_hyperlink(&url, "File a bug report")));
                } else {
                    output.write_fmt(format_args!("   File a bug report: {}\n", url));
                }
                output.write_str("\n");
                url
            }
            Err(e) => {
                output.write_fmt(format_args!("🐛 BUG ENCOUNTERED in {}:{}\n", file, line));
                output.write_fmt(format_args!("   Error generating bug report: {}\n", e));
                output.write_str("\n");
                String::new()
            }
        }
    }

    pub fn config(&self) -> &BugReportConfig {
        &self.config
    }
}

#[cfg(feature = "std")]
pub fn generate_github_url(template_name: &str, params: &FxHashMap<String, String>) -> Result<String, String> {
    let config = CONFIG.get().ok_or("Bug reporting not initialized. Call bug_rs::init() first.")?;
    
    let filled_template = if let Some(template) = config.templates.get(template_name) {
        template.fill_params(params)
    } else if let Some(template_file) = config.template_files.get(template_name) {
        IssueTemplate::from_template_file(template_file, params)?
    } else {
        return Err(format!("Template '{}' not found", template_name));
    };
    
    let mut url = format!(
        "https://github.com/{}/{}/issues/new",
        config.github_owner, config.github_repo
    );

    let mut query_params = Vec::new();
    
    if !filled_template.title.is_empty() {
        query_params.push(format!("title={}", url_encode::encode(&filled_template.title)));
    }
    
    if !filled_template.body.is_empty() {
        query_params.push(format!("body={}", url_encode::encode(&filled_template.body)));
    }
    
    if !filled_template.labels.is_empty() {
        let labels_str = filled_template.labels.join(",");
        query_params.push(format!("labels={}", url_encode::encode(&labels_str)));
    }

    if !query_params.is_empty() {
        url.push('?');
        url.push_str(&query_params.join("&"));
    }

    Ok(url)
}

/// Creates a terminal hyperlink using ANSI escape sequences
/// Format: \x1b]8;;URL\x1b\\TEXT\x1b]8;;\x1b\\
pub fn create_terminal_hyperlink(url: &str, text: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
}

/// Gets the hyperlink mode from configuration
#[cfg(feature = "std")]
pub fn get_hyperlink_mode() -> HyperlinkMode {
    CONFIG.get()
        .map(|config| config.use_hyperlinks.clone())
        .unwrap_or(HyperlinkMode::Never)
}

/// Gets the hyperlink mode from configuration (no_std version)
#[cfg(not(feature = "std"))]
pub unsafe fn get_hyperlink_mode() -> HyperlinkMode {
    unsafe {
        match core::ptr::addr_of!(CONFIG).read() {
            Some(config) => config.use_hyperlinks.clone(),
            None => HyperlinkMode::Never,
        }
    }
}

/// Detects if the terminal supports hyperlinks by checking environment variables
/// Only available with std feature
#[cfg(feature = "std")]
pub fn supports_hyperlinks() -> bool {
    // Check for common terminal emulators that support hyperlinks
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("xterm") || term.contains("screen") || term.contains("tmux") {
            return true;
        }
    }
    
    // Check for specific terminal programs
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        match term_program.as_str() {
            "iTerm.app" | "WezTerm" | "Alacritty" | "Windows Terminal" => return true,
            _ => {}
        }
    }
    
    // Check for VS Code integrated terminal
    if std::env::var("VSCODE_INJECTION").is_ok() {
        return true;
    }
    
    // Default to false for unknown terminals
    false
}

/// No-std version always returns false - user must specify hyperlink mode explicitly
#[cfg(not(feature = "std"))]
pub fn supports_hyperlinks() -> bool {
    false
}

#[macro_export]
macro_rules! bug {
    ($template:expr) => {
        $crate::bug!($template, {})
    };
    ($template:expr, { $($key:ident = $value:expr),* $(,)? }) => {{
        use $crate::FxHashMap;
        
        let mut params = FxHashMap::default();
        $(
            params.insert(stringify!($key).to_string(), $value.to_string());
        )*

        #[cfg(feature = "std")]
        {
            match $crate::generate_github_url($template, &params) {
                Ok(url) => {
                    eprintln!("🐛 BUG ENCOUNTERED in {}:{}", file!(), line!());
                    eprintln!("   Template: {}", $template);
                    if !params.is_empty() {
                        eprintln!("   Parameters:");
                        for (key, value) in &params {
                            eprintln!("     {}: {}", key, value);
                        }
                    }
                    let should_use_hyperlinks = match $crate::get_hyperlink_mode() {
                        $crate::HyperlinkMode::Auto => $crate::supports_hyperlinks(),
                        $crate::HyperlinkMode::Always => true,
                        $crate::HyperlinkMode::Never => false,
                    };
                    
                    if should_use_hyperlinks {
                        eprintln!("   {}", $crate::create_terminal_hyperlink(&url, "File a bug report"));
                    } else {
                        eprintln!("   File a bug report: {}", url);
                    }
                    eprintln!();
                    url
                }
                Err(e) => {
                    eprintln!("🐛 BUG ENCOUNTERED in {}:{}", file!(), line!());
                    eprintln!("   Error generating bug report: {}", e);
                    eprintln!();
                    String::new()
                }
            }
        }
        #[cfg(not(feature = "std"))]
        {
            // In no_std mode, we can't use the global config, so just return empty string
            // User should use bug_with_handle! instead
            String::new()
        }
    }};
}

#[macro_export]
macro_rules! bug_with_handle {
    ($handle:expr, $template:expr) => {
        $crate::bug_with_handle!($handle, $template, {})
    };
    ($handle:expr, $template:expr, { $($key:ident = $value:expr),* $(,)? }) => {{
        use $crate::FxHashMap;
        
        let mut params = FxHashMap::default();
        $(
            params.insert(stringify!($key).to_string(), $value.to_string());
        )*

        $handle.report_bug($template, &params, file!(), line!())
    }};
}

