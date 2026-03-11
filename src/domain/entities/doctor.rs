use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorOutput {
    pub is_ready: bool,
    pub warnings: Vec<DoctorWarning>,
    pub raw_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorWarning {
    pub title: String,
    pub body: String,
}

impl DoctorOutput {
    pub fn parse(output: &str) -> Self {
        let is_ready = output.contains("Your system is ready to brew.");
        let mut warnings = Vec::new();

        let mut current_title = String::new();
        let mut current_body = String::new();
        let mut in_warning = false;

        for line in output.lines() {
            if line.starts_with("Warning: ") {
                if in_warning {
                    warnings.push(DoctorWarning {
                        title: current_title.clone(),
                        body: current_body.trim().to_string(),
                    });
                    current_body.clear();
                }
                current_title = line.trim_start_matches("Warning: ").to_string();
                in_warning = true;
            } else if in_warning {
                if line.is_empty() && !current_body.is_empty() {
                    // Empty lines separate warnings, but sometimes they are inside warnings.
                    // Usually, a double empty line or next warning indicates end.
                    // Let's just accumulate until next Warning or end.
                    current_body.push('\n');
                } else {
                    current_body.push_str(line);
                    current_body.push('\n');
                }
            }
        }

        if in_warning {
            warnings.push(DoctorWarning {
                title: current_title,
                body: current_body.trim().to_string(),
            });
        }

        Self {
            is_ready,
            warnings,
            raw_output: output.to_string(),
        }
    }
}
