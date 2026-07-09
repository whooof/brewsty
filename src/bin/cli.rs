//! Brewsty CLI - Command-line interface for Homebrew package management

use brewsty::application::use_case_container::UseCaseContainer;
use brewsty::application::use_cases::export_packages::export_packages_to_json;
use brewsty::application::use_cases::import_packages::{import_packages, parse_package_type};
use brewsty::cli::{Cli, Commands, OutputFormatter};
use brewsty::domain::entities::{PackageCategory, PackageType};
use brewsty::infrastructure::brew::{
    BrewPackageListRepository, BrewPackageRepository, BrewServiceRepository,
};
use brewsty::infrastructure::history_repository::FileHistoryRepository;
use clap::Parser;
use std::path::Path;
use std::sync::Arc;

/// Create the use case container with all repositories
fn create_container() -> UseCaseContainer {
    let package_repo = Arc::new(BrewPackageRepository::new());
    let service_repo = Arc::new(BrewServiceRepository::new());
    let package_list_repo = Arc::new(BrewPackageListRepository::new());
    let history_repo = Arc::new(FileHistoryRepository::new());

    UseCaseContainer::new(package_repo, service_repo, package_list_repo, history_repo)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let formatter = OutputFormatter::new(&cli.format, cli.verbose);
    let container = create_container();

    match cli.command {
        Commands::List {
            package_type,
            search,
        } => {
            let pt = parse_package_type_arg(&package_type);
            match container.list_installed.execute(pt).await {
                Ok(packages) => {
                    let filtered = if let Some(query) = search {
                        packages
                            .into_iter()
                            .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()))
                            .collect::<Vec<_>>()
                    } else {
                        packages
                    };

                    println!(
                        "{}",
                        formatter.format_message(&format!("Found {} packages:", filtered.len()))
                    );
                    for pkg in filtered {
                        let version = pkg.version.unwrap_or_else(|| "unknown".to_string());
                        println!("  • {} ({}) - {:?}", pkg.name, version, pkg.package_type);
                    }
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Failed to list packages: {}", e))
                    );
                }
            }
        }
        Commands::Search {
            query,
            package_type,
        } => {
            let pt = parse_package_type_arg(&package_type);
            match container.search.execute(&query, pt).await {
                Ok(packages) => {
                    println!(
                        "{}",
                        formatter.format_message(&format!("Search results for '{}':", query))
                    );
                    if packages.is_empty() {
                        println!("  No packages found.");
                    } else {
                        for pkg in packages {
                            let version = pkg.version.unwrap_or_else(|| "unknown".to_string());
                            let desc = pkg.description.as_deref().unwrap_or("No description");
                            println!("  • {} ({}) - {}", pkg.name, version, desc);
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Search failed: {}", e))
                    );
                }
            }
        }
        Commands::Install { name, version } => {
            let pkg = brewsty::domain::entities::Package {
                name,
                version,
                available_version: None,
                description: None,
                package_type: PackageType::Formula,
                installed: false,
                outdated: false,
                version_load_failed: false,
                pinned: false,
                installed_size: None,
                category: PackageCategory::Other,
            };

            match container.install.execute(pkg).await {
                Ok(_) => {
                    println!(
                        "{}",
                        formatter.format_message("Package installed successfully!")
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Installation failed: {}", e))
                    );
                }
            }
        }
        Commands::Uninstall { name, force } => {
            let pkg = brewsty::domain::entities::Package {
                name,
                version: None,
                available_version: None,
                description: None,
                package_type: PackageType::Formula,
                installed: true,
                outdated: false,
                version_load_failed: false,
                pinned: false,
                installed_size: None,
                category: PackageCategory::Other,
            };

            let _ = force; // Force flag acknowledged
            match container.uninstall.execute(pkg).await {
                Ok(_) => {
                    println!(
                        "{}",
                        formatter.format_message("Package uninstalled successfully!")
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Uninstallation failed: {}", e))
                    );
                }
            }
        }
        Commands::Update { name } => match name {
            Some(pkg_name) => {
                let pkg = brewsty::domain::entities::Package {
                    name: pkg_name,
                    version: None,
                    available_version: None,
                    description: None,
                    package_type: PackageType::Formula,
                    installed: true,
                    outdated: true,
                    version_load_failed: false,
                    pinned: false,
                    installed_size: None,
                    category: PackageCategory::Other,
                };
                match container.update.execute(&pkg).await {
                    Ok(_) => {
                        println!(
                            "{}",
                            formatter.format_message("Package updated successfully!")
                        );
                    }
                    Err(e) => {
                        println!(
                            "{}",
                            formatter.format_error(&format!("Update failed: {}", e))
                        );
                    }
                }
            }
            None => match container.update_all.execute().await {
                Ok(_) => {
                    println!(
                        "{}",
                        formatter.format_message("All packages updated successfully!")
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Update all failed: {}", e))
                    );
                }
            },
        },
        Commands::Info { name } => {
            match container
                .get_package_info
                .execute(&name, PackageType::Formula)
                .await
            {
                Ok(pkg) => {
                    println!(
                        "{}",
                        formatter.format_message(&format!("Package: {}", pkg.name))
                    );
                    println!(
                        "  Version: {}",
                        pkg.version.unwrap_or_else(|| "unknown".to_string())
                    );
                    if let Some(desc) = pkg.description {
                        println!("  Description: {}", desc);
                    }
                    println!("  Type: {:?}", pkg.package_type);
                    println!("  Installed: {}", if pkg.installed { "Yes" } else { "No" });
                    println!("  Outdated: {}", if pkg.outdated { "Yes" } else { "No" });
                    println!("  Pinned: {}", if pkg.pinned { "Yes" } else { "No" });
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Package info not found: {}", e))
                    );
                }
            }
        }
        Commands::Orphans { remove } => match container.clean_orphans.preview().await {
            Ok(preview) => {
                println!(
                    "{}",
                    formatter
                        .format_message(&format!("Found {} orphan packages:", preview.items.len()))
                );
                for item in &preview.items {
                    println!("  • {} ({})", item.path, format_bytes(item.size));
                }
                println!(
                    "\nTotal space to reclaim: {}",
                    format_bytes(preview.total_size)
                );

                if remove {
                    match container.clean_orphans.execute().await {
                        Ok(_) => {
                            println!(
                                "{}",
                                formatter.format_message("Orphans removed successfully!")
                            );
                        }
                        Err(e) => {
                            println!(
                                "{}",
                                formatter.format_error(&format!("Failed to remove orphans: {}", e))
                            );
                        }
                    }
                } else {
                    println!("\nRun with --remove to delete these packages.");
                }
            }
            Err(e) => {
                println!(
                    "{}",
                    formatter.format_error(&format!("Failed to detect orphans: {}", e))
                );
            }
        },
        Commands::Stats => {
            // Get installed packages count
            let formulae = container
                .list_installed
                .execute(PackageType::Formula)
                .await
                .unwrap_or_default();
            let casks = container
                .list_installed
                .execute(PackageType::Cask)
                .await
                .unwrap_or_default();

            println!("{}", formatter.format_message("Brewsty Statistics"));
            println!("  ━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Formulae installed: {}", formulae.len());
            println!("  Casks installed: {}", casks.len());
            println!("  Total packages: {}", formulae.len() + casks.len());

            // Count outdated
            let outdated_formulae = container
                .list_outdated
                .execute(PackageType::Formula)
                .await
                .unwrap_or_default();
            let outdated_casks = container
                .list_outdated
                .execute(PackageType::Cask)
                .await
                .unwrap_or_default();
            println!(
                "  Outdated: {}",
                outdated_formulae.len() + outdated_casks.len()
            );
        }
        Commands::Export { output, format } => {
            let packages = container
                .list_installed
                .execute(PackageType::Formula)
                .await
                .unwrap_or_default();
            let casks = container
                .list_installed
                .execute(PackageType::Cask)
                .await
                .unwrap_or_default();
            let all_packages = [packages, casks].concat();

            let output_path = output.unwrap_or_else(|| "brewsty-export.json".to_string());
            let path = Path::new(&output_path);

            match export_packages_to_json(&all_packages, path) {
                Ok(_) => {
                    println!(
                        "{}",
                        formatter.format_message(&format!(
                            "Exported {} packages to {}",
                            all_packages.len(),
                            output_path
                        ))
                    );
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Export failed: {}", e))
                    );
                }
            }

            let _ = format; // Format arg acknowledged (currently only JSON supported)
        }
        Commands::Import { input, install } => {
            let path = Path::new(&input);
            match import_packages(path) {
                Ok(packages) => {
                    println!(
                        "{}",
                        formatter.format_message(&format!(
                            "Imported {} packages from {}",
                            packages.len(),
                            input
                        ))
                    );

                    if install {
                        let mut success = 0;
                        let mut failed = 0;

                        for pkg_data in packages {
                            if let Some(pt) = parse_package_type(&pkg_data.package_type) {
                                let pkg = brewsty::domain::entities::Package {
                                    name: pkg_data.name,
                                    version: Some(pkg_data.version),
                                    available_version: None,
                                    description: None,
                                    package_type: pt,
                                    installed: false,
                                    outdated: false,
                                    version_load_failed: false,
                                    pinned: false,
                                    installed_size: None,
                                    category: PackageCategory::Other,
                                };

                                match container.install.execute(pkg).await {
                                    Ok(_) => success += 1,
                                    Err(_) => failed += 1,
                                }
                            }
                        }

                        println!(
                            "{}",
                            formatter.format_message(&format!(
                                "Installed: {}, Failed: {}",
                                success, failed
                            ))
                        );
                    } else {
                        println!(
                            "{}",
                            formatter.format_message(
                                "Dry run complete. Use --install to actually install packages."
                            )
                        );
                    }
                }
                Err(e) => {
                    println!(
                        "{}",
                        formatter.format_error(&format!("Import failed: {}", e))
                    );
                }
            }
        }
    }
}

/// Parse package type argument from string
fn parse_package_type_arg(s: &str) -> PackageType {
    match s.to_lowercase().as_str() {
        "formula" | "formulae" => PackageType::Formula,
        "cask" => PackageType::Cask,
        _ => PackageType::Formula, // Default to formula
    }
}

/// Format bytes to human readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
