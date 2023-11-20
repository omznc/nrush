use reqwest::{Error, get};
use serde_json::Value;
use semver::Version;
use crate::constants::{MAJOR, MINOR, PATCH};

// Function to fetch package version asynchronously
pub async fn fetch_package_version(package: String) -> Result<(String, String), Error> {
	let npm_url = format!("https://registry.npmjs.org/{}/latest", &package);
	let response = get(&npm_url).await?;
	let json = response.json::<Value>().await?;
	let latest_version = json["version"]
		.as_str()
		.unwrap_or("Version not found")
		.to_string();
	Ok((package, latest_version))
}

// Function to normalize version strings
pub fn normalize_version(version: &str) -> String {
	version
		.chars()
		.filter(|&c| c.is_ascii_digit() || c == '.')
		.collect()
}

// Function to return ranges of the version
pub fn get_version_range(version: &str) -> String {
	if version == "*" {
		return "".to_string();
	}
	version
		.chars()
		.filter(|&c| !c.is_ascii_digit() && c != '.')
		.collect()
}

// Function to colorize version strings
pub fn colorize_version(current_version: &str, latest_version: &str) -> String {
	let current_version = normalize_version(current_version);
	let latest_version = normalize_version(latest_version);

	let current_version = if current_version == "" {
		Version::parse("0.0.0").unwrap()
	} else {
		Version::parse(&current_version).unwrap()
	};

	let latest_version = Version::parse(&latest_version).unwrap();

	if current_version.major < latest_version.major {
		format!("{}{} ({})\x1b[0m", MAJOR, latest_version, "major")
	} else if current_version.minor < latest_version.minor {
		format!("{}{} ({})\x1b[0m", MINOR, latest_version, "minor")
	} else if current_version.patch < latest_version.patch {
		format!("{}{} ({})\x1b[0m", PATCH, latest_version, "patch")
	} else {
		latest_version.to_string()
	}
}
