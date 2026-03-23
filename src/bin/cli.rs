//! Brewsty CLI - Command-line interface for Homebrew package management

use brewsty::cli::{Cli, Commands, OutputFormatter};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let formatter = OutputFormatter::new(&cli.format, cli.verbose);

    match cli.command {
        Commands::List {
            package_type,
            search,
        } => {
            println!("{}", formatter.format_message("Listing packages..."));
            // TODO: Implement package listing
            println!(
                "{}",
                formatter.format_message("Package listing not yet implemented")
            );
        }
        Commands::Search {
            query,
            package_type,
        } => {
            println!(
                "{}",
                formatter.format_message(&format!("Searching for: {}", query))
            );
            // TODO: Implement search
            println!("{}", formatter.format_message("Search not yet implemented"));
        }
        Commands::Install { name, version } => {
            println!(
                "{}",
                formatter.format_message(&format!(
                    "Installing: {} {}",
                    name,
                    version.unwrap_or_default()
                ))
            );
            // TODO: Implement install
            println!(
                "{}",
                formatter.format_message("Install not yet implemented")
            );
        }
        Commands::Uninstall { name, force } => {
            println!(
                "{}",
                formatter.format_message(&format!("Uninstalling: {} (force: {})", name, force))
            );
            // TODO: Implement uninstall
            println!(
                "{}",
                formatter.format_message("Uninstall not yet implemented")
            );
        }
        Commands::Update { name } => {
            let msg = match name {
                Some(pkg) => format!("Updating: {}", pkg),
                None => "Updating all packages".to_string(),
            };
            println!("{}", formatter.format_message(&msg));
            // TODO: Implement update
            println!("{}", formatter.format_message("Update not yet implemented"));
        }
        Commands::Info { name } => {
            println!(
                "{}",
                formatter.format_message(&format!("Package info: {}", name))
            );
            // TODO: Implement info
            println!("{}", formatter.format_message("Info not yet implemented"));
        }
        Commands::Orphans { remove } => {
            println!(
                "{}",
                formatter.format_message(if remove {
                    "Removing orphans..."
                } else {
                    "Detecting orphans..."
                })
            );
            // TODO: Implement orphan detection
            println!(
                "{}",
                formatter.format_message("Orphan detection not yet implemented")
            );
        }
        Commands::Stats => {
            println!("{}", formatter.format_message("Statistics"));
            // TODO: Implement stats
            println!("{}", formatter.format_message("Stats not yet implemented"));
        }
        Commands::Export { output, format } => {
            println!(
                "{}",
                formatter.format_message(&format!("Exporting to: {:?}", output))
            );
            // TODO: Implement export
            println!("{}", formatter.format_message("Export not yet implemented"));
        }
        Commands::Import { input, install } => {
            println!(
                "{}",
                formatter
                    .format_message(&format!("Importing from: {} (install: {})", input, install))
            );
            // TODO: Implement import
            println!("{}", formatter.format_message("Import not yet implemented"));
        }
    }
}
