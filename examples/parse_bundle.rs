fn main() {
    let check_output = "brew bundle can't satisfy your Brewfile's dependencies.
→ Cask android-commandlinetools needs to be installed or updated.
→ Cask atuin-desktop needs to be installed or updated.
→ Formula caddy needs to be installed or updated.
→ Formula dnsmasq needs to be installed or updated.
Satisfy missing dependencies with `brew bundle install`.";

    let cleanup_output = "Would uninstall formulae:
libimagequant
libraqm
pillow
Run `brew bundle cleanup --force` to make these changes.";

    let mut missing = Vec::new();
    for line in check_output.lines() {
        if line.starts_with("→ ")
            && let Some(dep) = line.strip_prefix("→ ")
            && let Some(idx) = dep.find(" needs to be installed or updated.")
        {
            missing.push(dep[..idx].to_string());
        }
    }

    let mut extra = Vec::new();
    let mut in_extra = false;
    for line in cleanup_output.lines() {
        if line.starts_with("Would uninstall") {
            in_extra = true;
            continue;
        }
        if in_extra {
            if line.is_empty()
                || line.starts_with("Run `brew bundle")
                || line.starts_with("Would uninstall")
            {
                if line.starts_with("Would uninstall") {
                    continue;
                }
                if line.starts_with("Run `brew bundle") || line.is_empty() {
                    in_extra = false;
                    continue;
                }
            }
            extra.push(line.trim().to_string());
        }
    }

    println!("Missing: {:#?}", missing);
    println!("Extra: {:#?}", extra);
}
