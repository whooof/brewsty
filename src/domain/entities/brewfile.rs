use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrewfileSyncPreview {
    pub brewfile_path: String,
    pub missing_dependencies: Vec<String>,
    pub extra_dependencies: Vec<String>,
}

impl BrewfileSyncPreview {
    pub fn new(brewfile_path: String, missing: Vec<String>, extra: Vec<String>) -> Self {
        Self {
            brewfile_path,
            missing_dependencies: missing,
            extra_dependencies: extra,
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.missing_dependencies.is_empty() || !self.extra_dependencies.is_empty()
    }

    pub fn parse_check_and_cleanup(path: &str, check_output: &str, cleanup_output: &str) -> Self {
        let mut missing = Vec::new();
        for line in check_output.lines() {
            #[allow(clippy::collapsible_if)]
            if let Some(dep) = line.strip_prefix("→ ") {
                if let Some(idx) = dep.find(" needs to be installed or updated.") {
                    missing.push(dep[..idx].to_string());
                }
            }
        }

        let mut extra = Vec::new();
        let mut in_extra = false;
        for line in cleanup_output.lines() {
            if line.starts_with("Would uninstall") || line.starts_with("Would remove") {
                in_extra = true;
                continue;
            }
            if in_extra {
                if line.is_empty() || line.starts_with("Run `brew bundle") {
                    in_extra = false;
                    continue;
                }
                extra.push(line.trim().to_string());
            }
        }

        Self::new(path.to_string(), missing, extra)
    }
}
