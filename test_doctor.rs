fn main() {
    let output = "Please note that these warnings are just used to help the Homebrew maintainers
with debugging if you file an issue. If everything you use Homebrew for is
working fine: please don't worry or file an issue; just ignore this. Thanks!

Warning: Some installed casks are deprecated or disabled.
You should find replacements for the following casks:
  amazon-luna

Warning: Some installed kegs have no formulae!
This means they were either deleted or installed manually.
You should find replacements for the following formulae:
  codex";

    let mut warnings = Vec::new();
    let mut current_title = String::new();
    let mut current_body = String::new();
    let mut in_warning = false;

    for line in output.lines() {
        if line.starts_with("Warning: ") {
            if in_warning {
                warnings.push((current_title.clone(), current_body.trim().to_string()));
                current_body.clear();
            }
            current_title = line.trim_start_matches("Warning: ").to_string();
            in_warning = true;
        } else if in_warning {
            if line.is_empty() && !current_body.is_empty() {
                current_body.push('\n');
            } else {
                current_body.push_str(line);
                current_body.push('\n');
            }
        }
    }

    if in_warning {
        warnings.push((current_title, current_body.trim().to_string()));
    }
    
    println!("{:#?}", warnings);
}
