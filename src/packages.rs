use crate::constants::{MAJOR, MINOR, PATCH, RESET};
use reqwest::{get, Error};
use semver::Version;
use serde_json::Value;

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
        format!("{}{}.{}.{} ({}){}", MAJOR, latest_version.major, latest_version.minor, latest_version.patch, "major", RESET)
    } else if current_version.minor < latest_version.minor {
        format!("{}.{}{}.{} ({}){}", current_version.major, MINOR, latest_version.minor, latest_version.patch, "minor", RESET)
    } else if current_version.patch < latest_version.patch {
        format!("{}.{}.{}{} ({}){}", current_version.major, current_version.minor, PATCH, latest_version.patch, "patch", RESET)
    } else {
        latest_version.to_string()
    }
}

pub fn package_type(is_dev: &bool, is_peer: &bool) -> &'static str {
    if *is_dev {
        "dev"
    } else if *is_peer {
        "peer"
    } else {
        "prod"
    }
}

pub fn package_ranking(package: &str) -> i32 {
    if package.contains("(major)") { 1 }
    else if package.contains("(minor)") { 2 }
    else if package.contains("(patch)") { 3 }
    else { 4 }
}


pub fn get_current_package_version(package: &str, json_data: &Value, dev_package_names: &Vec<String>, peer_package_names: &Vec<String>) -> String {
    let get_version_from_dependency_type = |dependency_type: &str| {
        json_data[dependency_type][package]
            .as_str()
            .unwrap_or("Version not found")
            .to_string()
    };

    let get_dependency_type = |package: &str| -> &str {
        if dev_package_names.contains(&package.to_string()) {
            "devDependencies"
        } else if peer_package_names.contains(&package.to_string()) {
            "peerDependencies"
        } else {
            "dependencies"
        }
    };

    get_version_from_dependency_type(get_dependency_type(package))
}

pub fn set_new_package_version(package: &str, version: &str, is_dev: bool, is_peer: bool, json_data: &mut Value) {
    let get_dependency_type = |is_dev: bool, is_peer: bool| {
        if is_dev {
            "devDependencies"
        } else if is_peer {
            "peerDependencies"
        } else {
            "dependencies"
        }
    };

    let dependency_type = get_dependency_type(is_dev, is_peer);

    let current_version = get_current_package_version(&package, &json_data, &vec![], &vec![]);
    let current_version_range = get_version_range(&current_version);
    let new_version = format!("{}{}", current_version_range, version);
    json_data[dependency_type][package] = Value::String(new_version);
}

