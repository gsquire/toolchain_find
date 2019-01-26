use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::process::Command;

use semver::Version;
use walkdir::WalkDir;

// A `Component` keeps track of the rustc version associated with the component in question.
#[derive(Debug)]
struct Component {
    date_vers: DateVersion,
    path: PathBuf,
}

// A `DateVersion` allows you to sort first by the semantic version and date second if the versions
// are equal.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
struct DateVersion {
    rustc_vers: Version,
    date: String,
}

impl Ord for DateVersion {
    fn cmp(&self, other: &DateVersion) -> Ordering {
        let vers_cmp = self.rustc_vers.cmp(&other.rustc_vers);
        if vers_cmp == Ordering::Equal {
            return self.date.cmp(&other.date);
        }
        vers_cmp
    }
}

impl DateVersion {
    fn new(rustc_vers: Version, date: String) -> DateVersion {
        DateVersion { rustc_vers, date }
    }
}

impl Component {
    fn new(date_vers: DateVersion, path: PathBuf) -> Component {
        Component { date_vers, path }
    }
}

// Return either the user's preferred $RUSTUP_HOME or the default location.
fn rustup_home() -> Option<PathBuf> {
    let mut p = PathBuf::new();
    if let Some(custom_path) = option_env!("RUSTUP_HOME") {
        p.push(custom_path);
        return Some(p);
    }

    let home = dirs::home_dir()?;
    p.push(home);
    p.push(".rustup");

    Some(p)
}

// Try and parse the version from the Rust compiler. If we can not do this, just make it version 0.
fn rustc_version(bin_path: &Path) -> DateVersion {
    let version_zero = Version::new(0, 0, 0);
    let date_zero = String::default();

    match Command::new(bin_path).arg("-V").output() {
        Ok(o) => {
            // This may not be the most ideal way to get the version.
            // It assumes that the output looks like:
            // rustc 1.32.0 (9fda7c223 2019-01-16)
            let output = String::from_utf8(o.stdout).unwrap_or_default();
            let parts = output.split(' ').collect::<Vec<&str>>();
            if parts.len() > 3 {
                let vers = Version::parse(parts[1]).unwrap_or(version_zero);
                let mut date = parts[3].trim_end();
                date = date.trim_end_matches(')');
                let parsed_date = String::from(date);
                return DateVersion::new(vers, parsed_date);
            }
            DateVersion::new(version_zero, date_zero)
        }
        Err(_) => DateVersion::new(version_zero, date_zero),
    }
}

/// Given a Rust component name, search through all of the available toolchains
/// on the system to see if it is installed. It will return the path of the component that has
/// the latest version.
pub fn find_installed_component(name: &str) -> Option<PathBuf> {
    let mut components = Vec::new();
    let mut root = rustup_home()?;
    root.push("toolchains");

    for entry in WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let parent = entry.path().parent()?;
        if parent.ends_with("bin") {
            let bin_name = entry.path().file_name()?;

            if bin_name == name {
                // This assumes that we will always have a rustc in this same toolchain location.
                // I suppose a user could have a very custom build but I am not sure how much we
                // need to support.
                let mut rustc_path = PathBuf::from(parent);
                rustc_path.push("rustc");
                components.push(Component::new(
                    rustc_version(&rustc_path),
                    PathBuf::from(&entry.path()),
                ));
            }
        }
    }

    // Sort by the rustc version leaving the maximal one at the end.
    components.sort_by(|a, b| a.date_vers.cmp(&b.date_vers));

    if let Some(c) = components.pop() {
        return Some(c.path);
    }

    None
}
