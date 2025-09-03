//! # Bug Reporting Library
//!
//! A Rust library for generating GitHub issue URLs with customizable templates.
//! Supports both `std` and `no_std` environments, making it suitable for embedded systems,
//! WebAssembly, and other constrained environments.
//!
//! ## Features
//!
//! - **Template-based bug reporting**: Define reusable issue templates with placeholders
//! - **GitHub integration**: Generate direct links to GitHub's new issue page
//! - **no_std support**: Works in embedded and constrained environments
//! - **Terminal hyperlinks**: Smart hyperlink detection for modern terminals
//! - **Flexible output**: Customizable output destinations
//!
//! ## Quick Start
//!
//! ### Global Configuration (std only)
//!
//! ```rust
//! use bug::{init, bug, IssueTemplate};
//! 
//! // Initialize with your GitHub repository
//! init("username", "repository")
//!     .add_template("crash", IssueTemplate::new(
//!         "Application Crash: {error_type}",
//!         "The application crashed with error: {error_message}\n\nSteps to reproduce:\n{steps}"
//!     ))
//!     .build()
//!     .expect("Failed to initialize bug reporting");
//! 
//! // Use the bug! macro to report issues
//! let url = bug!("crash", {
//!     error_type = "NullPointerException",
//!     error_message = "Attempted to access null pointer",
//!     steps = "1. Start app\n2. Click button\n3. Crash occurs"
//! });
//! ```
//!
//! ### Handle-based API (std and no_std)
//!
//! ```rust
//! use bug::{init_handle, bug_with_handle, IssueTemplate};
//! 
//! let handle = init_handle("username", "repository")
//!     .add_template("bug", IssueTemplate::new("Bug Report", "Found a bug: {description}"));
//! 
//! let url = bug_with_handle!(handle, "bug", {
//!     description = "Button doesn't work"
//! });
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod url_encode;

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

/// A fast HashMap implementation using FxHasher.
///
/// This type alias provides a HashMap optimized for performance with string keys,
/// which is commonly used throughout this library for template parameters and configuration.
/// 
/// # Examples
/// 
/// ```
/// use bug::FxHashMap;
/// 
/// let mut params: FxHashMap<String, String> = FxHashMap::default();
/// params.insert("key".to_string(), "value".to_string());
/// assert_eq!(params.get("key"), Some(&"value".to_string()));
/// ```
pub type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[cfg(feature = "std")]
use once_cell::sync::OnceCell;

#[cfg(feature = "std")]
static CONFIG: OnceCell<BugReportConfig> = OnceCell::new();

#[cfg(not(feature = "std"))]
static mut CONFIG: Option<BugReportConfig> = None;

/// Trait for outputting bug report information in no_std environments.
///
/// This trait abstracts over different output destinations, allowing bug reports
/// to be written to various targets like stderr, custom loggers, or even no-op
/// implementations for embedded systems.
/// 
/// # Examples
/// 
/// ```
/// use bug::Output;
/// 
/// struct CustomLogger;
/// 
/// impl Output for CustomLogger {
///     fn write_str(&mut self, s: &str) {
///         // Custom logging implementation
///         println!("LOG: {}", s);
///     }
///     
///     fn write_fmt(&mut self, args: core::fmt::Arguments) {
///         // Custom formatted logging
///         println!("LOG: {}", args);
///     }
/// }
/// ```
pub trait Output {
    /// Write a string to the output destination.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to write
    fn write_str(&mut self, s: &str);
    
    /// Write formatted arguments to the output destination.
    ///
    /// # Arguments
    ///
    /// * `args` - The formatted arguments to write
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

/// A no-op output implementation that discards all output.
///
/// This is useful for embedded systems or situations where you want to
/// generate bug report URLs without any console output.
/// 
/// # Examples
/// 
/// ```
/// use bug::{NoOutput, Output};
/// 
/// let mut output = NoOutput;
/// output.write_str("This will be discarded");
/// output.write_fmt(format_args!("This {} will also be discarded", "text"));
/// ```
pub struct NoOutput;

impl Output for NoOutput {
    fn write_str(&mut self, _s: &str) {
        // No-op: discard the output
    }
    
    fn write_fmt(&mut self, _args: core::fmt::Arguments) {
        // No-op: discard the formatted output
    }
}

/// Configuration for the bug reporting system.
///
/// This struct holds all the configuration needed to generate bug reports,
/// including GitHub repository information, issue templates, and hyperlink preferences.
/// 
/// # Examples
/// 
/// ```
/// use bug::{BugReportConfig, HyperlinkMode, FxHashMap};
/// 
/// let config = BugReportConfig {
///     github_owner: "octocat".to_string(),
///     github_repo: "Hello-World".to_string(),
///     templates: FxHashMap::default(),
///     template_files: FxHashMap::default(),
///     use_hyperlinks: HyperlinkMode::Auto,
/// };
/// 
/// assert_eq!(config.github_owner, "octocat");
/// assert_eq!(config.github_repo, "Hello-World");
/// ```
#[derive(Debug, Clone)]
pub struct BugReportConfig {
    /// The GitHub username or organization name
    pub github_owner: String,
    /// The GitHub repository name
    pub github_repo: String,
    /// Map of template names to issue templates
    pub templates: FxHashMap<String, IssueTemplate>,
    /// Map of template file names to template files
    pub template_files: FxHashMap<String, TemplateFile>,
    /// How to handle hyperlinks in terminal output
    pub use_hyperlinks: HyperlinkMode,
}

/// Controls how hyperlinks are displayed in terminal output.
///
/// Modern terminals support clickable hyperlinks using ANSI escape sequences.
/// This enum allows you to control when to use them.
/// 
/// # Examples
/// 
/// ```
/// use bug::HyperlinkMode;
/// 
/// // Automatically detect terminal support
/// let auto_mode = HyperlinkMode::Auto;
/// 
/// // Always show hyperlinks (good for known compatible terminals)
/// let always_mode = HyperlinkMode::Always;
/// 
/// // Never show hyperlinks (good for logs or unknown terminals)
/// let never_mode = HyperlinkMode::Never;
/// ```
#[derive(Debug, Clone)]
pub enum HyperlinkMode {
    /// Automatically detect terminal hyperlink support based on environment variables
    Auto,
    /// Always use hyperlinks regardless of terminal detection
    Always,
    /// Never use hyperlinks, always show full URLs
    Never,
}

/// A GitHub issue template with title, body, and labels.
///
/// Issue templates define the structure of bug reports that will be submitted to GitHub.
/// They support placeholder substitution using `{placeholder}` syntax.
/// 
/// # Examples
/// 
/// ```
/// use bug::IssueTemplate;
/// 
/// let template = IssueTemplate::new(
///     "Bug: {component} not working",
///     "## Problem\n{description}\n\n## Expected Behavior\n{expected}\n\n## Actual Behavior\n{actual}"
/// ).with_labels(vec!["bug".to_string(), "needs-investigation".to_string()]);
/// 
/// assert_eq!(template.title, "Bug: {component} not working");
/// assert_eq!(template.labels.len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct IssueTemplate {
    /// The title template for the GitHub issue
    pub title: String,
    /// The body template for the GitHub issue
    pub body: String,
    /// Labels to apply to the GitHub issue
    pub labels: Vec<String>,
}

/// A template loaded from a static string (typically from `include_str!`).
///
/// Template files allow you to store issue templates in separate files and embed
/// them at compile time. The first line becomes the title, and the rest becomes the body.
/// 
/// # File Format
/// 
/// ```text
/// Issue Title Here
/// Issue body content starts here.
/// It can span multiple lines.
/// 
/// Placeholders like {param} are supported.
/// ```
/// 
/// # Examples
/// 
/// ```
/// use bug::TemplateFile;
/// 
/// let template_file = TemplateFile::new("Bug Report\nFound a bug: {description}")
///     .with_labels(vec!["bug".to_string()]);
/// 
/// let parsed = template_file.parse().unwrap();
/// assert_eq!(parsed.title, "Bug Report");
/// assert_eq!(parsed.body, "Found a bug: {description}");
/// ```
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// The raw template content (first line is title, rest is body)
    pub content: &'static str,
    /// Labels to apply to issues created from this template
    pub labels: Vec<String>,
}

impl TemplateFile {
    /// Create a new template file with the given content.
    /// 
    /// # Arguments
    /// 
    /// * `content` - The template content where first line is the title
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::TemplateFile;
    /// 
    /// let template = TemplateFile::new("Bug Title\nBug description with {param}");
    /// assert_eq!(template.content, "Bug Title\nBug description with {param}");
    /// assert!(template.labels.is_empty());
    /// ```
    pub fn new(content: &'static str) -> Self {
        Self {
            content,
            labels: Vec::new(),
        }
    }

    /// Add labels to this template file.
    /// 
    /// # Arguments
    /// 
    /// * `labels` - Vector of label strings to apply to issues
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::TemplateFile;
    /// 
    /// let template = TemplateFile::new("Title\nBody")
    ///     .with_labels(vec!["bug".to_string(), "urgent".to_string()]);
    /// assert_eq!(template.labels.len(), 2);
    /// ```
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    /// Parse the template file content into an IssueTemplate.
    /// 
    /// The first line of the content becomes the title, and the remaining
    /// lines become the body. Empty templates are rejected.
    /// 
    /// # Returns
    /// 
    /// * `Ok(IssueTemplate)` - Successfully parsed template
    /// * `Err(String)` - Error message if parsing fails
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::TemplateFile;
    /// 
    /// let template_file = TemplateFile::new("Bug Report\nSomething is broken: {issue}");
    /// let parsed = template_file.parse().unwrap();
    /// assert_eq!(parsed.title, "Bug Report");
    /// assert_eq!(parsed.body, "Something is broken: {issue}");
    /// 
    /// // Empty templates fail
    /// let empty_template = TemplateFile::new("");
    /// assert!(empty_template.parse().is_err());
    /// ```
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

    /// Validate that all required parameters are provided and no extra parameters exist.
    /// 
    /// This method extracts all placeholders from the template content and ensures
    /// that the provided parameters match exactly.
    /// 
    /// # Arguments
    /// 
    /// * `params` - Map of parameter names to values
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - All parameters are valid
    /// * `Err(String)` - Error describing missing or unused parameters
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{TemplateFile, FxHashMap};
    /// 
    /// let template = TemplateFile::new("Bug: {component}\nError: {message}");
    /// let mut params = FxHashMap::default();
    /// params.insert("component".to_string(), "UI".to_string());
    /// params.insert("message".to_string(), "Button broken".to_string());
    /// 
    /// assert!(template.validate_params(&params).is_ok());
    /// 
    /// // Missing parameter
    /// let mut incomplete_params = FxHashMap::default();
    /// incomplete_params.insert("component".to_string(), "UI".to_string());
    /// assert!(template.validate_params(&incomplete_params).is_err());
    /// ```
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
    /// Create a new issue template with title and body.
    /// 
    /// # Arguments
    /// 
    /// * `title` - The issue title template (supports placeholders)
    /// * `body` - The issue body template (supports placeholders)
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::IssueTemplate;
    /// 
    /// let template = IssueTemplate::new("Bug in {module}", "Error: {error_msg}");
    /// assert_eq!(template.title, "Bug in {module}");
    /// assert_eq!(template.body, "Error: {error_msg}");
    /// assert!(template.labels.is_empty());
    /// ```
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            labels: Vec::new(),
        }
    }

    /// Create an issue template from a template file with parameter substitution.
    /// 
    /// # Arguments
    /// 
    /// * `template_file` - The template file to parse
    /// * `params` - Parameters to substitute in the template
    /// 
    /// # Returns
    /// 
    /// * `Ok(IssueTemplate)` - Successfully created template with parameters filled
    /// * `Err(String)` - Error from validation or parsing
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{TemplateFile, IssueTemplate, FxHashMap};
    /// 
    /// let template_file = TemplateFile::new("Bug: {type}\nDescription: {desc}");
    /// let mut params = FxHashMap::default();
    /// params.insert("type".to_string(), "Crash".to_string());
    /// params.insert("desc".to_string(), "App crashes on startup".to_string());
    /// 
    /// let issue = IssueTemplate::from_template_file(&template_file, &params).unwrap();
    /// assert_eq!(issue.title, "Bug: Crash");
    /// assert_eq!(issue.body, "Description: App crashes on startup");
    /// ```
    pub fn from_template_file(template_file: &TemplateFile, params: &FxHashMap<String, String>) -> Result<Self, String> {
        template_file.validate_params(params)?;
        let parsed_template = template_file.parse()?;
        Ok(parsed_template.fill_params(params))
    }

    /// Add labels to this issue template.
    /// 
    /// # Arguments
    /// 
    /// * `labels` - Vector of label strings to apply to the GitHub issue
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::IssueTemplate;
    /// 
    /// let template = IssueTemplate::new("Bug Title", "Bug description")
    ///     .with_labels(vec!["bug".to_string(), "high-priority".to_string()]);
    /// assert_eq!(template.labels.len(), 2);
    /// assert!(template.labels.contains(&"bug".to_string()));
    /// ```
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    /// Fill template placeholders with provided parameters.
    /// 
    /// This method replaces all `{placeholder}` patterns in the title and body
    /// with the corresponding values from the params map.
    /// 
    /// # Arguments
    /// 
    /// * `params` - Map of parameter names to replacement values
    /// 
    /// # Returns
    /// 
    /// A new `IssueTemplate` with placeholders replaced by parameter values.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{IssueTemplate, FxHashMap};
    /// 
    /// let template = IssueTemplate::new("Error in {component}", "Details: {message}");
    /// let mut params = FxHashMap::default();
    /// params.insert("component".to_string(), "parser".to_string());
    /// params.insert("message".to_string(), "Invalid syntax".to_string());
    /// 
    /// let filled = template.fill_params(&params);
    /// assert_eq!(filled.title, "Error in parser");
    /// assert_eq!(filled.body, "Details: Invalid syntax");
    /// ```
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

/// Extract placeholder names from template content.
/// 
/// This function scans the content for `{placeholder}` patterns and returns
/// a vector of unique placeholder names. Only valid identifiers (alphanumeric
/// characters and underscores) are recognized as placeholders.
/// 
/// # Arguments
/// 
/// * `content` - The template content to scan
/// 
/// # Returns
/// 
/// A vector of unique placeholder names found in the content.
/// 
/// # Examples
/// 
/// ```
/// use bug::extract_placeholders;
/// 
/// let content = "Error in {module}: {message}. See {module} docs.";
/// let placeholders = extract_placeholders(content);
/// assert_eq!(placeholders.len(), 2);
/// assert!(placeholders.contains(&"module".to_string()));
/// assert!(placeholders.contains(&"message".to_string()));
/// 
/// // Invalid placeholders with spaces are ignored
/// let invalid_content = "Invalid: {123} {with space} {valid_name}";
/// let valid_placeholders = extract_placeholders(invalid_content);
/// assert_eq!(valid_placeholders, vec!["123".to_string(), "valid_name".to_string()]);
/// ```
pub fn extract_placeholders(content: &str) -> Vec<String> {
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

/// Macro to create a `TemplateFile` from a file path at compile time.
/// 
/// This macro uses `include_str!` to embed the template content directly into
/// the binary at compile time. It supports an optional `labels` parameter to
/// add GitHub issue labels.
/// 
/// # Syntax
/// 
/// - `template_file!("path/to/template.txt")` - Basic usage
/// - `template_file!("path/to/template.txt", labels: ["bug", "urgent"])` - With labels
/// 
/// # Examples
/// 
/// ```ignore
/// use bug::template_file;
/// 
/// // Basic usage (assumes you have a template.txt file)
/// let template = template_file!("templates/bug_report.txt");
/// 
/// // With labels
/// let labeled_template = template_file!(
///     "templates/crash_report.txt", 
///     labels: ["bug", "crash", "high-priority"]
/// );
/// ```
/// 
/// # Template File Format
/// 
/// Template files should have the title on the first line and the body on subsequent lines:
/// 
/// ```text
/// Bug Report: {component}
/// ## Description
/// {description}
/// 
/// ## Steps to Reproduce
/// {steps}
/// ```
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

/// Initialize a bug report configuration builder (std only).
/// 
/// This function creates a new configuration builder that allows you to set up
/// templates and options before building the global configuration. This is only
/// available with the "std" feature.
/// 
/// # Arguments
/// 
/// * `github_owner` - GitHub username or organization name
/// * `github_repo` - GitHub repository name
/// 
/// # Returns
/// 
/// A `BugReportConfigBuilder` that can be used to configure templates and options.
/// 
/// # Examples
/// 
/// ```
/// use bug::{init, IssueTemplate};
/// 
/// # #[cfg(feature = "std")] {
/// let result = init("octocat", "Hello-World")
///     .add_template("bug", IssueTemplate::new("Bug Report", "Something is broken"))
///     .hyperlinks(bug::HyperlinkMode::Always)
///     .build();
/// # }
/// ```
pub fn init(github_owner: impl Into<String>, github_repo: impl Into<String>) -> BugReportConfigBuilder {
    BugReportConfigBuilder::new(github_owner.into(), github_repo.into())
}

/// Initialize a bug report handle (works in both std and no_std).
/// 
/// This function creates a handle-based configuration that doesn't rely on
/// global state. It can be used in both std and no_std environments.
/// 
/// # Arguments
/// 
/// * `github_owner` - GitHub username or organization name
/// * `github_repo` - GitHub repository name
/// 
/// # Returns
/// 
/// A `BugReportHandle` that can be used to generate bug reports.
/// 
/// # Examples
/// 
/// ```
/// use bug::{init_handle, IssueTemplate};
/// 
/// let handle = init_handle("octocat", "Hello-World")
///     .add_template("crash", IssueTemplate::new("Crash Report", "App crashed: {reason}"))
///     .hyperlinks(bug::HyperlinkMode::Never);
/// 
/// // Use with bug_with_handle! macro
/// ```
pub fn init_handle(github_owner: impl Into<String>, github_repo: impl Into<String>) -> BugReportHandle {
    BugReportHandle::new(github_owner.into(), github_repo.into())
}

/// Builder for configuring the global bug reporting system (std only).
/// 
/// This builder allows you to add templates, configure hyperlink behavior,
/// and build the global configuration. Once built, the configuration is
/// stored globally and used by the `bug!` macro.
/// 
/// # Examples
/// 
/// ```
/// use bug::{init, IssueTemplate, HyperlinkMode};
/// 
/// # #[cfg(feature = "std")] {
/// let builder = init("owner", "repo")
///     .add_template("error", IssueTemplate::new("Error Report", "An error occurred"))
///     .hyperlinks(HyperlinkMode::Auto);
/// # }
/// ```
pub struct BugReportConfigBuilder {
    config: BugReportConfig,
}

impl BugReportConfigBuilder {
    /// Create a new configuration builder.
    /// 
    /// # Arguments
    /// 
    /// * `github_owner` - GitHub username or organization
    /// * `github_repo` - GitHub repository name
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

    /// Add an issue template to the configuration.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Name to identify the template
    /// * `template` - The issue template to add
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init, IssueTemplate};
    /// 
    /// # #[cfg(feature = "std")] {
    /// let builder = init("owner", "repo")
    ///     .add_template("bug", IssueTemplate::new("Bug Report", "Found a bug"));
    /// # }
    /// ```
    pub fn add_template(mut self, name: impl Into<String>, template: IssueTemplate) -> Self {
        self.config.templates.insert(name.into(), template);
        self
    }

    /// Add a template file to the configuration.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Name to identify the template file
    /// * `template_file` - The template file to add
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init, TemplateFile};
    /// 
    /// # #[cfg(feature = "std")] {
    /// let builder = init("owner", "repo")
    ///     .add_template_file("crash", TemplateFile::new("Crash Report\nApp crashed"));
    /// # }
    /// ```
    pub fn add_template_file(mut self, name: impl Into<String>, template_file: TemplateFile) -> Self {
        self.config.template_files.insert(name.into(), template_file);
        self
    }

    /// Configure hyperlink behavior for terminal output.
    /// 
    /// # Arguments
    /// 
    /// * `mode` - How to handle hyperlinks in output
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init, HyperlinkMode};
    /// 
    /// # #[cfg(feature = "std")] {
    /// let builder = init("owner", "repo")
    ///     .hyperlinks(HyperlinkMode::Always);
    /// # }
    /// ```
    pub fn hyperlinks(mut self, mode: HyperlinkMode) -> Self {
        self.config.use_hyperlinks = mode;
        self
    }

    /// Build and install the global configuration (std only).
    /// 
    /// This method finalizes the configuration and stores it globally.
    /// After calling this, the `bug!` macro can be used throughout the application.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Configuration was successfully installed
    /// * `Err(&'static str)` - Configuration was already initialized
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init, IssueTemplate};
    /// 
    /// # #[cfg(feature = "std")] {
    /// let result = init("owner", "repo")
    ///     .add_template("bug", IssueTemplate::new("Bug", "Description"))
    ///     .build();
    /// assert!(result.is_ok() || result == Err("Bug reporting already initialized"));
    /// # }
    /// ```
    #[cfg(feature = "std")]
    pub fn build(self) -> Result<(), &'static str> {
        CONFIG.set(self.config).map_err(|_| "Bug reporting already initialized")
    }
    
    /// Build and install the global configuration (no_std only).
    /// 
    /// This method finalizes the configuration and stores it globally.
    /// In no_std environments, this uses unsafe code to manage static state.
    /// 
    /// # Safety
    /// 
    /// This function is unsafe because it modifies global mutable static state.
    /// It should only be called once during application initialization.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Configuration was successfully installed
    /// * `Err(&'static str)` - Configuration was already initialized
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init, IssueTemplate};
    /// 
    /// # #[cfg(not(feature = "std"))] {
    /// unsafe {
    ///     let result = init("owner", "repo")
    ///         .add_template("bug", IssueTemplate::new("Bug", "Description"))
    ///         .build();
    ///     assert!(result.is_ok() || result == Err("Bug reporting already initialized"));
    /// }
    /// # }
    /// ```
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

/// A handle for bug reporting that doesn't rely on global state.
/// 
/// This struct provides the same functionality as the global configuration
/// but can be used in no_std environments and allows multiple independent
/// configurations within the same application.
/// 
/// # Examples
/// 
/// ```
/// use bug::{init_handle, IssueTemplate, FxHashMap};
/// 
/// let handle = init_handle("octocat", "Hello-World")
///     .add_template("bug", IssueTemplate::new("Bug Report", "Issue: {description}"));
/// 
/// let mut params = FxHashMap::default();
/// params.insert("description".to_string(), "Button not working".to_string());
/// 
/// let url = handle.generate_url("bug", &params).unwrap();
/// assert!(url.contains("github.com/octocat/Hello-World/issues/new"));
/// ```
#[derive(Debug, Clone)]
pub struct BugReportHandle {
    config: BugReportConfig,
}

impl BugReportHandle {
    /// Create a new bug report handle.
    /// 
    /// # Arguments
    /// 
    /// * `github_owner` - GitHub username or organization
    /// * `github_repo` - GitHub repository name
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

    /// Add an issue template to this handle.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Name to identify the template
    /// * `template` - The issue template to add
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, IssueTemplate};
    /// 
    /// let handle = init_handle("owner", "repo")
    ///     .add_template("bug", IssueTemplate::new("Bug Report", "Found a bug"));
    /// ```
    pub fn add_template(mut self, name: impl Into<String>, template: IssueTemplate) -> Self {
        self.config.templates.insert(name.into(), template);
        self
    }

    /// Add a template file to this handle.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Name to identify the template file
    /// * `template_file` - The template file to add
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, TemplateFile};
    /// 
    /// let handle = init_handle("owner", "repo")
    ///     .add_template_file("crash", TemplateFile::new("Crash Report\nApp crashed"));
    /// ```
    pub fn add_template_file(mut self, name: impl Into<String>, template_file: TemplateFile) -> Self {
        self.config.template_files.insert(name.into(), template_file);
        self
    }

    /// Configure hyperlink behavior for this handle.
    /// 
    /// # Arguments
    /// 
    /// * `mode` - How to handle hyperlinks in output
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, HyperlinkMode};
    /// 
    /// let handle = init_handle("owner", "repo")
    ///     .hyperlinks(HyperlinkMode::Always);
    /// ```
    pub fn hyperlinks(mut self, mode: HyperlinkMode) -> Self {
        self.config.use_hyperlinks = mode;
        self
    }

    /// Generate a GitHub issue URL from a template and parameters.
    /// 
    /// This method fills the specified template with the provided parameters
    /// and generates a complete GitHub issue URL with query parameters for
    /// title, body, and labels.
    /// 
    /// # Arguments
    /// 
    /// * `template_name` - Name of the template to use
    /// * `params` - Parameters to substitute in the template
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` - The generated GitHub issue URL
    /// * `Err(String)` - Error message if template not found or validation fails
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, IssueTemplate, FxHashMap};
    /// 
    /// let handle = init_handle("octocat", "Hello-World")
    ///     .add_template("bug", IssueTemplate::new("Bug: {component}", "Error: {message}"));
    /// 
    /// let mut params = FxHashMap::default();
    /// params.insert("component".to_string(), "UI".to_string());
    /// params.insert("message".to_string(), "Button not working".to_string());
    /// 
    /// let url = handle.generate_url("bug", &params).unwrap();
    /// assert!(url.contains("github.com/octocat/Hello-World/issues/new"));
    /// assert!(url.contains("title=Bug%3A+UI"));
    /// ```
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

    /// Report a bug with no output (silent mode).
    /// 
    /// This method generates a bug report URL but doesn't produce any output.
    /// Useful when you only need the URL without console output.
    /// 
    /// # Arguments
    /// 
    /// * `template_name` - Name of the template to use
    /// * `params` - Parameters to substitute in the template
    /// * `file` - Source file name where the bug occurred
    /// * `line` - Line number where the bug occurred
    /// 
    /// # Returns
    /// 
    /// The generated GitHub issue URL, or empty string on error.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, IssueTemplate, FxHashMap};
    /// 
    /// let handle = init_handle("owner", "repo")
    ///     .add_template("error", IssueTemplate::new("Error", "Something broke"));
    /// 
    /// let params = FxHashMap::default();
    /// let url = handle.report_bug("error", &params, "main.rs", 42);
    /// assert!(url.contains("github.com"));
    /// ```
    pub fn report_bug(&self, template_name: &str, params: &FxHashMap<String, String>, file: &str, line: u32) -> String {
        self.report_bug_with_output(template_name, params, file, line, &mut NoOutput)
    }
    
    /// Report a bug with output to stderr (std only).
    /// 
    /// This method generates a bug report URL and prints formatted bug
    /// information to stderr, including file location and parameters.
    /// 
    /// # Arguments
    /// 
    /// * `template_name` - Name of the template to use
    /// * `params` - Parameters to substitute in the template
    /// * `file` - Source file name where the bug occurred
    /// * `line` - Line number where the bug occurred
    /// 
    /// # Returns
    /// 
    /// The generated GitHub issue URL, or empty string on error.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, IssueTemplate, FxHashMap};
    /// 
    /// # #[cfg(feature = "std")] {
    /// let handle = init_handle("owner", "repo")
    ///     .add_template("crash", IssueTemplate::new("Crash", "App crashed"));
    /// 
    /// let params = FxHashMap::default();
    /// let url = handle.report_bug_stderr("crash", &params, "main.rs", 42);
    /// // This will print to stderr and return the URL
    /// # }
    /// ```
    #[cfg(feature = "std")]
    pub fn report_bug_stderr(&self, template_name: &str, params: &FxHashMap<String, String>, file: &str, line: u32) -> String {
        self.report_bug_with_output(template_name, params, file, line, &mut std::io::stderr())
    }
    
    /// Report a bug with custom output destination.
    /// 
    /// This is the most flexible bug reporting method, allowing you to specify
    /// a custom output destination. It generates the URL and writes formatted
    /// bug information to the provided output.
    /// 
    /// # Arguments
    /// 
    /// * `template_name` - Name of the template to use
    /// * `params` - Parameters to substitute in the template
    /// * `file` - Source file name where the bug occurred
    /// * `line` - Line number where the bug occurred
    /// * `output` - Custom output destination implementing `Output` trait
    /// 
    /// # Returns
    /// 
    /// The generated GitHub issue URL, or empty string on error.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, IssueTemplate, FxHashMap, Output};
    /// 
    /// struct MockOutput(String);
    /// 
    /// impl Output for MockOutput {
    ///     fn write_str(&mut self, s: &str) {
    ///         self.0.push_str(s);
    ///     }
    ///     fn write_fmt(&mut self, args: core::fmt::Arguments) {
    ///         self.0.push_str(&format!("{}", args));
    ///     }
    /// }
    /// 
    /// let handle = init_handle("owner", "repo")
    ///     .add_template("test", IssueTemplate::new("Test", "Test bug"));
    /// 
    /// let params = FxHashMap::default();
    /// let mut output = MockOutput(String::new());
    /// let url = handle.report_bug_with_output("test", &params, "test.rs", 10, &mut output);
    /// 
    /// assert!(url.contains("github.com"));
    /// assert!(output.0.contains("BUG ENCOUNTERED"));
    /// ```
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

    /// Get a reference to the internal configuration.
    /// 
    /// This method provides read-only access to the handle's configuration,
    /// allowing you to inspect templates, repository information, and settings.
    /// 
    /// # Returns
    /// 
    /// A reference to the `BugReportConfig` used by this handle.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bug::{init_handle, IssueTemplate};
    /// 
    /// let handle = init_handle("octocat", "Hello-World")
    ///     .add_template("test", IssueTemplate::new("Test", "Test issue"));
    /// 
    /// let config = handle.config();
    /// assert_eq!(config.github_owner, "octocat");
    /// assert_eq!(config.github_repo, "Hello-World");
    /// assert_eq!(config.templates.len(), 1);
    /// ```
    pub fn config(&self) -> &BugReportConfig {
        &self.config
    }
}

/// Generate a GitHub issue URL using the global configuration (std only).
/// 
/// This function generates a bug report URL using the global configuration
/// set up with `init().build()`. It's a convenience function for when you
/// don't want to use the `bug!` macro but still want to use global config.
/// 
/// # Arguments
/// 
/// * `template_name` - Name of the template to use
/// * `params` - Parameters to substitute in the template
/// 
/// # Returns
/// 
/// * `Ok(String)` - The generated GitHub issue URL
/// * `Err(String)` - Error if not initialized or template not found
/// 
/// # Examples
/// 
/// ```
/// use bug::{init, generate_github_url, IssueTemplate, FxHashMap};
/// 
/// # #[cfg(feature = "std")] {
/// // First initialize the global config
/// init("owner", "repo")
///     .add_template("error", IssueTemplate::new("Error", "An error occurred"))
///     .build()
///     .expect("Failed to initialize");
/// 
/// let mut params = FxHashMap::default();
/// let url = generate_github_url("error", &params).unwrap();
/// assert!(url.contains("github.com/owner/repo"));
/// # }
/// ```
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

/// Create a clickable terminal hyperlink using ANSI escape sequences.
/// 
/// This function creates a hyperlink that modern terminals can display as
/// clickable text. The hyperlink uses the OSC 8 escape sequence standard.
/// 
/// # Arguments
/// 
/// * `url` - The target URL for the hyperlink
/// * `text` - The display text for the hyperlink
/// 
/// # Returns
/// 
/// A string containing the ANSI escape sequences for a terminal hyperlink.
/// 
/// # Format
/// 
/// The generated string follows this format:
/// `\x1b]8;;URL\x1b\\TEXT\x1b]8;;\x1b\\`
/// 
/// # Examples
/// 
/// ```
/// use bug::create_terminal_hyperlink;
/// 
/// let link = create_terminal_hyperlink("https://github.com", "GitHub");
/// println!("{}", link); // Will show as clickable "GitHub" in supported terminals
/// 
/// // The actual string contains escape sequences
/// assert!(link.contains("https://github.com"));
/// assert!(link.contains("GitHub"));
/// ```
/// 
/// # Terminal Support
/// 
/// This works in terminals that support OSC 8 hyperlinks, including:
/// - iTerm2
/// - Windows Terminal
/// - VS Code terminal
/// - Some versions of xterm
pub fn create_terminal_hyperlink(url: &str, text: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
}

/// Get the hyperlink mode from the global configuration (std only).
/// 
/// This function retrieves the hyperlink mode setting from the global
/// configuration. If no configuration has been set, it returns `Never`.
/// 
/// # Returns
/// 
/// The current `HyperlinkMode` setting, or `Never` if not initialized.
/// 
/// # Examples
/// 
/// ```
/// use bug::{init, get_hyperlink_mode, HyperlinkMode};
/// 
/// # #[cfg(feature = "std")] {
/// // Before initialization, returns Never
/// let mode = get_hyperlink_mode();
/// // mode will be Never since no config is set yet
/// 
/// // After initialization
/// init("owner", "repo")
///     .hyperlinks(HyperlinkMode::Always)
///     .build()
///     .ok(); // Ignore if already initialized
/// 
/// let mode = get_hyperlink_mode();
/// // mode should now be Always (if initialization succeeded)
/// # }
/// ```
#[cfg(feature = "std")]
pub fn get_hyperlink_mode() -> HyperlinkMode {
    CONFIG.get()
        .map(|config| config.use_hyperlinks.clone())
        .unwrap_or(HyperlinkMode::Never)
}

/// Get the hyperlink mode from the global configuration (no_std version).
/// 
/// This function retrieves the hyperlink mode setting from the global
/// configuration in no_std environments. It uses unsafe code to access
/// the static configuration.
/// 
/// # Safety
/// 
/// This function is unsafe because it reads from global mutable static state.
/// It should only be called after the configuration has been properly initialized.
/// 
/// # Returns
/// 
/// The current `HyperlinkMode` setting, or `Never` if not initialized.
/// 
/// # Examples
/// 
/// ```
/// use bug::{init, HyperlinkMode};
/// 
/// # #[cfg(not(feature = "std"))] {
/// unsafe {
///     // After initialization with unsafe build()
///     let mode = bug::get_hyperlink_mode();
///     // Returns the configured hyperlink mode
/// }
/// # }
/// ```
#[cfg(not(feature = "std"))]
pub unsafe fn get_hyperlink_mode() -> HyperlinkMode {
    unsafe {
        match core::ptr::addr_of!(CONFIG).read() {
            Some(config) => config.use_hyperlinks.clone(),
            None => HyperlinkMode::Never,
        }
    }
}

/// Detect if the current terminal supports clickable hyperlinks (std only).
/// 
/// This function attempts to detect hyperlink support by checking various
/// environment variables that indicate terminal capabilities. It checks for
/// known terminal emulators and programs that support OSC 8 hyperlinks.
/// 
/// # Detection Logic
/// 
/// The function checks for:
/// - Common terminal types in `TERM` environment variable
/// - Specific terminal programs in `TERM_PROGRAM` environment variable  
/// - VS Code integrated terminal via `VSCODE_INJECTION`
/// 
/// # Returns
/// 
/// - `true` if hyperlinks are likely supported
/// - `false` if hyperlinks are not supported or detection is uncertain
/// 
/// # Examples
/// 
/// ```
/// use bug::supports_hyperlinks;
/// 
/// # #[cfg(feature = "std")] {
/// if supports_hyperlinks() {
///     println!("Terminal supports hyperlinks!");
/// } else {
///     println!("Terminal may not support hyperlinks");
/// }
/// # }
/// ```
/// 
/// # Supported Terminals
/// 
/// Known to work with:
/// - iTerm2 (macOS)
/// - Windows Terminal
/// - WezTerm
/// - Alacritty
/// - VS Code integrated terminal
/// - xterm (recent versions)
/// - screen/tmux (with proper terminal support)
/// 
/// # Limitations
/// 
/// Terminal detection is heuristic-based and may not be 100% accurate.
/// When in doubt, you can explicitly set the hyperlink mode using
/// `HyperlinkMode::Always` or `HyperlinkMode::Never`.
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

/// Hyperlink support detection for no_std environments.
/// 
/// In no_std environments, environment variables are not available,
/// so this function always returns `false`. Users should explicitly
/// configure hyperlink behavior using `HyperlinkMode`.
/// 
/// # Returns
/// 
/// Always returns `false` in no_std environments.
/// 
/// # Examples
/// 
/// ```
/// use bug::supports_hyperlinks;
/// 
/// # #[cfg(not(feature = "std"))] {
/// // Always returns false in no_std
/// assert_eq!(supports_hyperlinks(), false);
/// # }
/// ```
/// 
/// # Recommendation
/// 
/// In no_std environments, explicitly set the hyperlink mode:
/// 
/// ```ignore
/// use bug::{init_handle, HyperlinkMode};
/// 
/// let handle = init_handle("owner", "repo")
///     .hyperlinks(HyperlinkMode::Always); // or Never
/// ```
#[cfg(not(feature = "std"))]
pub fn supports_hyperlinks() -> bool {
    false
}

/// Report a bug using the global configuration (std only).
/// 
/// This macro generates a GitHub issue URL using a predefined template and
/// parameters, then prints bug report information to stderr. It uses the
/// global configuration set up with `init().build()`.
/// 
/// # Syntax
/// 
/// - `bug!("template_name")` - Use template without parameters
/// - `bug!("template_name", { param1 = value1, param2 = value2 })` - With parameters
/// 
/// # Returns
/// 
/// Returns the generated GitHub issue URL as a `String`, or an empty string if
/// an error occurs or in no_std mode.
/// 
/// # Examples
/// 
/// ```ignore
/// use bug::{init, bug, IssueTemplate};
/// 
/// // First, initialize the global configuration
/// init("octocat", "Hello-World")
///     .add_template("crash", IssueTemplate::new(
///         "Application Crash: {error_type}",
///         "Error: {error_message}\nContext: {context}"
///     ))
///     .build()
///     .expect("Failed to initialize");
/// 
/// // Report a bug with parameters
/// let url = bug!("crash", {
///     error_type = "NullPointerException",
///     error_message = "Attempted to access null pointer",
///     context = "user clicked submit button"
/// });
/// 
/// // Report a bug without parameters
/// let url = bug!("simple_template");
/// ```
/// 
/// # Output Format
/// 
/// The macro prints to stderr in this format:
/// ```text
/// 🐛 BUG ENCOUNTERED in src/main.rs:42
///    Template: crash
///    Parameters:
///      error_type: NullPointerException
///      error_message: Attempted to access null pointer
///    File a bug report: https://github.com/...
/// ```
/// 
/// # Platform Support
/// 
/// - **std**: Full functionality with terminal output
/// - **no_std**: Returns empty string, no output (use `bug_with_handle!` instead)
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

/// Report a bug using a specific handle (works in both std and no_std).
/// 
/// This macro generates a GitHub issue URL using a `BugReportHandle` instance
/// and prints bug report information. Unlike `bug!`, this works in both std
/// and no_std environments since it doesn't rely on global state.
/// 
/// # Syntax
/// 
/// - `bug_with_handle!(handle, "template_name")` - Use template without parameters
/// - `bug_with_handle!(handle, "template_name", { param1 = value1, param2 = value2 })` - With parameters
/// 
/// # Returns
/// 
/// Returns the generated GitHub issue URL as a `String`.
/// 
/// # Examples
/// 
/// ```
/// use bug::{init_handle, bug_with_handle, IssueTemplate};
/// 
/// let handle = init_handle("octocat", "Hello-World")
///     .add_template("error", IssueTemplate::new(
///         "Error: {type}",
///         "An error occurred: {message}"
///     ));
/// 
/// // Report with parameters
/// let url = bug_with_handle!(handle, "error", {
///     type = "ValidationError",
///     message = "Invalid input provided"
/// });
/// 
/// // Report without parameters (if template has no placeholders)
/// let simple_handle = init_handle("owner", "repo")
///     .add_template("simple", IssueTemplate::new("Simple Bug", "Something broke"));
/// let url = bug_with_handle!(simple_handle, "simple");
/// ```
/// 
/// # Output (when using stderr output)
/// 
/// With `report_bug_stderr()`, the macro prints to stderr:
/// ```text
/// 🐛 BUG ENCOUNTERED in src/main.rs:42
///    Template: error
///    Parameters:
///      type: ValidationError
///      message: Invalid input provided
///    File a bug report: https://github.com/...
/// ```
/// 
/// # Platform Support
/// 
/// - **std**: Full functionality with configurable output
/// - **no_std**: Works with custom `Output` implementations
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

