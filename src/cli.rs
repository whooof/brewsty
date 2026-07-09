//! CLI Companion Tool - Command-line interface for Brewsty operations

use clap::{Parser, Subcommand};

/// Brewsty CLI - Command-line interface for Homebrew package management
#[derive(Parser, Debug)]
#[command(name = "brewsty")]
#[command(author = "Brewsty Team")]
#[command(version = "0.1.0")]
#[command(about = "CLI companion for Brewsty Homebrew manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List installed packages
    List {
        /// Filter by package type (formula, cask, all)
        #[arg(short, long, default_value = "all")]
        package_type: String,

        /// Search query
        #[arg(short, long)]
        search: Option<String>,
    },

    /// Search for packages
    Search {
        /// Search query
        query: String,

        /// Filter by package type (formula, cask, all)
        #[arg(short, long, default_value = "all")]
        package_type: String,
    },

    /// Install a package
    Install {
        /// Package name to install
        name: String,

        /// Package version (if available)
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Uninstall a package
    Uninstall {
        /// Package name to uninstall
        name: String,

        /// Force uninstall
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// Update packages
    Update {
        /// Package name to update (all if not specified)
        name: Option<String>,
    },

    /// Show package information
    Info {
        /// Package name
        name: String,
    },

    /// Detect orphan packages
    Orphans {
        /// Remove orphans instead of just listing
        #[arg(short, long, default_value_t = false)]
        remove: bool,
    },

    /// Show statistics
    Stats,

    /// Export package list
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Export format (json, yaml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Import package list
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,

        /// Actually install packages (dry-run if not specified)
        #[arg(short, long, default_value_t = false)]
        install: bool,
    },
}

/// Output formatter for CLI
pub struct OutputFormatter {
    format: String,
    verbose: bool,
}

impl OutputFormatter {
    pub fn new(format: &str, verbose: bool) -> Self {
        Self {
            format: format.to_string(),
            verbose,
        }
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn format_message(&self, message: &str) -> String {
        match self.format.as_str() {
            "json" => format!("{{\"message\": \"{}\"}}\n", message.replace('"', "\\\"")),
            _ => format!("{}\n", message),
        }
    }

    pub fn format_error(&self, error: &str) -> String {
        match self.format.as_str() {
            "json" => format!("{{\"error\": \"{}\"}}\n", error.replace('"', "\\\"")),
            _ => format!("❌ Error: {}\n", error),
        }
    }

    /// Format verbose output with additional details
    pub fn format_verbose(&self, header: &str, details: &str) -> String {
        if !self.verbose {
            return String::new();
        }
        match self.format.as_str() {
            "json" => format!(
                "{{\"verbose\": {{\"header\": \"{}\", \"details\": \"{}\"}}}}\n",
                header.replace('"', "\\\""),
                details.replace('"', "\\\"")
            ),
            _ => format!("\n📝 {}:\n   {}\n", header, details.replace('\n', "\n   ")),
        }
    }

    /// Format package details in verbose mode
    pub fn format_package_details(&self, pkg: &crate::domain::entities::Package) -> String {
        if !self.verbose {
            return String::new();
        }
        match self.format.as_str() {
            "json" => {
                format!(
                    "{{\"package\": {{\"name\": \"{}\", \"version\": \"{}\", \"type\": \"{:?}\", \"outdated\": {}, \"pinned\": {}}}}}\n",
                    pkg.name.replace('"', "\\\""),
                    pkg.version
                        .as_deref()
                        .unwrap_or("unknown")
                        .replace('"', "\\\""),
                    pkg.package_type,
                    pkg.outdated,
                    pkg.pinned
                )
            }
            _ => {
                let mut output = String::new();
                output.push_str(&format!("   Name: {}\n", pkg.name));
                output.push_str(&format!(
                    "   Version: {}\n",
                    pkg.version.as_deref().unwrap_or("unknown")
                ));
                output.push_str(&format!("   Type: {:?}\n", pkg.package_type));
                output.push_str(&format!(
                    "   Outdated: {}\n",
                    if pkg.outdated { "Yes" } else { "No" }
                ));
                output.push_str(&format!(
                    "   Pinned: {}\n",
                    if pkg.pinned { "Yes" } else { "No" }
                ));
                if let Some(desc) = &pkg.description {
                    output.push_str(&format!("   Description: {}\n", desc));
                }
                output
            }
        }
    }

    /// Format timing information in verbose mode
    pub fn format_timing(&self, operation: &str, duration_ms: u128) -> String {
        if !self.verbose {
            return String::new();
        }
        match self.format.as_str() {
            "json" => format!(
                "{{\"timing\": {{\"operation\": \"{}\", \"duration_ms\": {}}}}}\n",
                operation.replace('"', "\\\""),
                duration_ms
            ),
            _ => format!("⏱️  {} took {}ms\n", operation, duration_ms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_list() {
        let cli = Cli::parse_from(["brewsty", "list"]);
        match cli.command {
            Commands::List {
                package_type,
                search,
            } => {
                assert_eq!(package_type, "all");
                assert!(search.is_none());
            }
            _ => panic!("Expected List command"),
        }
    }

    #[test]
    fn test_cli_parse_install() {
        let cli = Cli::parse_from(["brewsty", "install", "git"]);
        match cli.command {
            Commands::Install { name, version } => {
                assert_eq!(name, "git");
                assert!(version.is_none());
            }
            _ => panic!("Expected Install command"),
        }
    }

    #[test]
    fn test_output_formatter_text() {
        let formatter = OutputFormatter::new("text", false);
        let output = formatter.format_message("Test message");
        assert!(output.contains("Test message"));
    }

    #[test]
    fn test_output_formatter_json() {
        let formatter = OutputFormatter::new("json", false);
        let output = formatter.format_message("Test message");
        assert!(output.contains("\"message\": \"Test message\""));
    }

    #[test]
    fn test_verbose_mode_disabled() {
        let formatter = OutputFormatter::new("text", false);
        assert!(!formatter.is_verbose());
        assert!(formatter.format_verbose("Header", "Details").is_empty());
        assert!(formatter.format_timing("test", 100).is_empty());
    }

    #[test]
    fn test_verbose_mode_enabled() {
        let formatter = OutputFormatter::new("text", true);
        assert!(formatter.is_verbose());
        let verbose = formatter.format_verbose("Header", "Details");
        assert!(verbose.contains("Header"));
        assert!(verbose.contains("Details"));
    }

    #[test]
    fn test_verbose_timing() {
        let formatter = OutputFormatter::new("text", true);
        let timing = formatter.format_timing("test_op", 250);
        assert!(timing.contains("test_op"));
        assert!(timing.contains("250ms"));
    }

    #[test]
    fn test_verbose_json_output() {
        let formatter = OutputFormatter::new("json", true);
        let timing = formatter.format_timing("test_op", 100);
        assert!(timing.contains("\"timing\""));
        assert!(timing.contains("\"test_op\""));
    }
}
