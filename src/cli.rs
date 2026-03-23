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
}
