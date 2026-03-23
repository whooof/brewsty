//! Package category detection and classification

use crate::domain::entities::PackageType;

/// Package categories for filtering and organization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageCategory {
    Development,
    Languages,
    Databases,
    Web,
    System,
    Utilities,
    Media,
    Games,
    Science,
    Security,
    Networking,
    Other,
}

impl PackageCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackageCategory::Development => "Development",
            PackageCategory::Languages => "Languages",
            PackageCategory::Databases => "Databases",
            PackageCategory::Web => "Web",
            PackageCategory::System => "System",
            PackageCategory::Utilities => "Utilities",
            PackageCategory::Media => "Media",
            PackageCategory::Games => "Games",
            PackageCategory::Science => "Science",
            PackageCategory::Security => "Security",
            PackageCategory::Networking => "Networking",
            PackageCategory::Other => "Other",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            PackageCategory::Development => "🛠️",
            PackageCategory::Languages => "📝",
            PackageCategory::Databases => "🗄️",
            PackageCategory::Web => "🌐",
            PackageCategory::System => "⚙️",
            PackageCategory::Utilities => "🔧",
            PackageCategory::Media => "🎬",
            PackageCategory::Games => "🎮",
            PackageCategory::Science => "🔬",
            PackageCategory::Security => "🔒",
            PackageCategory::Networking => "🌐",
            PackageCategory::Other => "📦",
        }
    }

    /// Detect category from package name and description
    pub fn from_package(name: &str, description: Option<&str>) -> Self {
        let name_lower = name.to_lowercase();
        let desc_lower = description.unwrap_or("").to_lowercase();
        let combined = format!("{} {}", name_lower, desc_lower);

        // Media (check early to avoid miscategorization)
        if combined.contains("ffmpeg")
            || combined.contains("vlc")
            || combined.contains("audio")
            || combined.contains("video")
            || combined.contains("image")
            || combined.contains("photo")
            || combined.contains("gimp")
            || combined.contains("blender")
            || combined.contains("obs")
            || combined.contains("streaming")
        {
            return PackageCategory::Media;
        }

        // Development tools
        if combined.contains("git")
            || combined.contains("compiler")
            || combined.contains("debugger")
            || combined.contains("ide")
            || combined.contains("editor")
            || combined.contains("vim")
            || combined.contains("emacs")
            || combined.contains("vscode")
            || combined.contains("jetbrains")
            || combined.contains("xcode")
            || combined.contains("cmake")
            || combined.contains("make")
            || combined.contains("ninja")
            || combined.contains("build")
        {
            return PackageCategory::Development;
        }

        // Programming languages
        if combined.contains("python")
            || combined.contains("ruby")
            || combined.contains("node")
            || combined.contains("npm")
            || combined.contains("yarn")
            || combined.contains("go ")
            || combined.contains("golang")
            || combined.contains("rust")
            || combined.contains("cargo")
            || combined.contains("java")
            || combined.contains("jdk")
            || combined.contains("scala")
            || combined.contains("kotlin")
            || combined.contains("php")
            || combined.contains("perl")
            || combined.contains("lua")
            || combined.contains("haskell")
            || combined.contains("swift")
            || combined.contains("dart")
        {
            return PackageCategory::Languages;
        }

        // Databases
        if combined.contains("postgres")
            || combined.contains("mysql")
            || combined.contains("mariadb")
            || combined.contains("mongodb")
            || combined.contains("redis")
            || combined.contains("sqlite")
            || combined.contains("database")
            || combined.contains("db ")
            || combined.contains("sql")
            || combined.contains("cassandra")
            || combined.contains("elasticsearch")
        {
            return PackageCategory::Databases;
        }

        // Web
        if combined.contains("nginx")
            || combined.contains("apache")
            || combined.contains("httpd")
            || combined.contains("caddy")
            || combined.contains("webpack")
            || combined.contains("vite")
            || combined.contains("gatsby")
            || combined.contains("next")
            || combined.contains("nuxt")
            || combined.contains("react")
            || combined.contains("vue")
            || combined.contains("angular")
        {
            return PackageCategory::Web;
        }

        // System
        if combined.contains("kernel")
            || combined.contains("firmware")
            || combined.contains("driver")
            || combined.contains("boot")
            || combined.contains("systemd")
            || combined.contains("init")
            || combined.contains("bash")
            || combined.contains("zsh")
            || combined.contains("shell")
        {
            return PackageCategory::System;
        }

        // Games
        if combined.contains("game")
            || combined.contains("steam")
            || combined.contains("emulator")
            || combined.contains("retroarch")
            || combined.contains("dolphin")
            || combined.contains("rpcs3")
        {
            return PackageCategory::Games;
        }

        // Science
        if combined.contains("science")
            || combined.contains("math")
            || combined.contains("physics")
            || combined.contains("chemistry")
            || combined.contains("biology")
            || combined.contains("jupyter")
            || combined.contains("numpy")
            || combined.contains("scipy")
        {
            return PackageCategory::Science;
        }

        // Security
        if combined.contains("security")
            || combined.contains("crypto")
            || combined.contains("encryption")
            || combined.contains("password")
            || combined.contains("vpn")
            || combined.contains("firewall")
            || combined.contains("antivirus")
            || combined.contains("nmap")
            || combined.contains("wireshark")
        {
            return PackageCategory::Security;
        }

        // Networking
        if combined.contains("network")
            || combined.contains("dns")
            || combined.contains("dhcp")
            || combined.contains("proxy")
            || combined.contains("curl")
            || combined.contains("wget")
            || combined.contains("ssh")
            || combined.contains("ftp")
            || combined.contains("sftp")
        {
            return PackageCategory::Networking;
        }

        // Default to Utilities or Other
        if combined.contains("cli")
            || combined.contains("tool")
            || combined.contains("utility")
            || combined.contains("helper")
        {
            PackageCategory::Utilities
        } else {
            PackageCategory::Other
        }
    }

    /// Get all categories as a vector
    pub fn all_categories() -> Vec<Self> {
        vec![
            PackageCategory::Development,
            PackageCategory::Languages,
            PackageCategory::Databases,
            PackageCategory::Web,
            PackageCategory::System,
            PackageCategory::Utilities,
            PackageCategory::Media,
            PackageCategory::Games,
            PackageCategory::Science,
            PackageCategory::Security,
            PackageCategory::Networking,
            PackageCategory::Other,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_package_git() {
        let category = PackageCategory::from_package("git", Some("Distributed version control"));
        assert_eq!(category, PackageCategory::Development);
    }

    #[test]
    fn test_category_from_package_python() {
        let category =
            PackageCategory::from_package("python@3.11", Some("Python programming language"));
        assert_eq!(category, PackageCategory::Languages);
    }

    #[test]
    fn test_category_from_package_postgres() {
        let category = PackageCategory::from_package("postgresql@15", Some("PostgreSQL database"));
        assert_eq!(category, PackageCategory::Databases);
    }

    #[test]
    fn test_category_from_package_nginx() {
        let category = PackageCategory::from_package("nginx", Some("Web server"));
        assert_eq!(category, PackageCategory::Web);
    }

    #[test]
    fn test_category_from_package_ffmpeg() {
        let category = PackageCategory::from_package("ffmpeg", Some("Video/audio processing"));
        assert_eq!(category, PackageCategory::Media);
    }

    #[test]
    fn test_category_icons() {
        assert_eq!(PackageCategory::Development.icon(), "🛠️");
        assert_eq!(PackageCategory::Languages.icon(), "📝");
        assert_eq!(PackageCategory::Databases.icon(), "🗄️");
        assert_eq!(PackageCategory::Other.icon(), "📦");
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(PackageCategory::Development.as_str(), "Development");
        assert_eq!(PackageCategory::Other.as_str(), "Other");
    }

    #[test]
    fn test_all_categories() {
        let categories = PackageCategory::all_categories();
        assert_eq!(categories.len(), 12);
    }
}
