use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use once_cell::sync::OnceCell;
use regex::Regex;
use semver::Version;
use walkdir::WalkDir;

// A `Component` keeps track of the rustc version associated with the component in question.
#[derive(Debug)]
struct Component {
    date_vers: Option<DateVersion>,
    path: PathBuf,
}

// A `DateVersion` allows you to sort first by the semantic version and date second if the versions
// are equal.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
struct DateVersion {
    rustc_vers: Option<Version>,
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
    fn new(rustc_vers: Option<Version>, date: String) -> DateVersion {
        DateVersion { rustc_vers, date }
    }
}

impl Component {
    fn new(date_vers: Option<DateVersion>, path: PathBuf) -> Component {
        Component { date_vers, path }
    }
}

// Given the version string from rustc, attempt to parse the date.
fn parse_rustc_date(rustc_v: &[u8]) -> Option<DateVersion> {
    static PATTERN: OnceCell<Regex> = OnceCell::new();

    // This may not be the most ideal way to get the version.
    // It assumes that the output looks like:
    // rustc 1.32.0 (9fda7c223 2019-01-16)
    let pattern = PATTERN.get_or_init(|| {
        Regex::new(
            r"rustc (\d+.\d+.\d+(?:-[\.0-9a-z]+)?)(?: \([[:alnum:]]+ (\d{4}-\d{2}-\d{2})\))?",
        )
        .unwrap()
    });

    let version = str::from_utf8(rustc_v).unwrap_or_default();
    let captures = pattern.captures(version)?;
    let vers = Version::parse(captures.get(1).map_or("", |v| v.as_str())).ok();
    let date = String::from(captures.get(2).map_or("", |v| v.as_str()));

    Some(DateVersion::new(vers, date))
}

// Try and parse the version from the Rust compiler.
fn rustc_version(bin_path: &Path) -> Option<DateVersion> {
    match Command::new(bin_path).arg("-V").output() {
        Ok(o) => parse_rustc_date(&o.stdout),
        Err(_) => None,
    }
}

/// Given a Rust component name, search through all of the available toolchains
/// on the system to see if it is installed. It will return the path of the component that has
/// the latest version.
pub fn find_installed_component(name: &str) -> Option<PathBuf> {
    let mut components = Vec::new();
    let mut root = home::rustup_home().ok()?;
    root.push("toolchains");

    // For Windows, we need to add an exe extension.
    let mut n = String::from(name);
    let name = if cfg!(windows) {
        n.push_str(".exe");
        n
    } else {
        n
    };

    for entry in WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let parent = entry.path().parent()?;
        if parent.ends_with("bin") {
            let bin_name = entry.path().file_name()?;

            if bin_name == name.as_str() {
                // This assumes that we will always have a rustc in this same toolchain location.
                // I suppose a user could have a very custom build but I am not sure how much we
                // need to support.
                let mut rustc_path = PathBuf::from(parent);
                if cfg!(windows) {
                    rustc_path.push("rustc.exe");
                } else {
                    rustc_path.push("rustc");
                }
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

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use semver::Version;

    use super::{parse_rustc_date, DateVersion};

    #[test]
    fn test_parse_rustc_date() {
        let cases = vec![
            "".as_bytes(),
            "rustc not found".as_bytes(),
            "rustc 1.34.0-nightly (097c04cf4 2019-02-24)".as_bytes(),
            "rustc 1.34.0-beta.1 (744b374ab 2019-02-26)".as_bytes(),
            "rustc 1.35.0-dev".as_bytes(),
            "rustc 1.32.0 (9fda7c223 2019-01-16)".as_bytes(),
        ];
        let expected = vec![
            None,
            None,
            Some(DateVersion::new(
                Some(Version::parse("1.34.0-nightly").unwrap()),
                String::from("2019-02-24"),
            )),
            Some(DateVersion::new(
                Some(Version::parse("1.34.0-beta.1").unwrap()),
                String::from("2019-02-26"),
            )),
            Some(DateVersion::new(
                Some(Version::parse("1.35.0-dev").unwrap()),
                String::from(""),
            )),
            Some(DateVersion::new(
                Some(Version::parse("1.32.0").unwrap()),
                String::from("2019-01-16"),
            )),
        ];

        for (i, case) in cases.iter().enumerate() {
            assert_eq!(parse_rustc_date(case), expected[i]);
        }
    }

    #[test]
    fn test_version_parse_fail() {
        let v2 = Version::parse("1.1.0").unwrap();
        let d1 = DateVersion::new(None, String::from("2019-01-01"));
        let d2 = DateVersion::new(Some(v2), String::from("2019-01-01"));

        assert!(d2 > d1);
    }

    #[test]
    fn test_different_versions() {
        let v1 = Version::parse("1.2.3").unwrap();
        let v2 = Version::parse("1.1.0").unwrap();
        let d1 = DateVersion::new(Some(v1), String::from("2019-01-01"));
        let d2 = DateVersion::new(Some(v2), String::from("2019-01-01"));

        assert!(d2 < d1);
    }

    #[test]
    fn test_many_nightly_strings() {
        let v = Version::parse("1.0.0-nightly").unwrap();
        let mut versions = vec![
            DateVersion::new(Some(v.clone()), String::from("2019-02-20")),
            DateVersion::new(Some(v.clone()), String::from("2019-02-24")),
            DateVersion::new(Some(v.clone()), String::from("2019-01-10")),
        ];
        versions.sort();

        assert_eq!(
            versions.pop().unwrap(),
            DateVersion::new(Some(v.clone()), String::from("2019-02-24"))
        );
    }

    #[test]
    fn test_date_version_compare() {
        let d1 = DateVersion::new(Some(Version::parse("1.34.0").unwrap()), String::from(""));
        let d2 = DateVersion::new(
            Some(Version::parse("1.33.0").unwrap()),
            String::from("2019-04-20"),
        );
        let d3 = DateVersion::new(
            Some(Version::parse("1.33.0").unwrap()),
            String::from("2019-04-17"),
        );

        assert_eq!(d1.cmp(&d2), Ordering::Greater);
        assert_eq!(d2.cmp(&d3), Ordering::Greater);
    }
}
