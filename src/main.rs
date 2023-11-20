use serde_json::Value;
use reqwest;
use std::path::{Path, PathBuf};
use std::io;
use semver::Version;
use tokio;

// Function to fetch package version asynchronously
async fn fetch_package_version(package: String) -> Result<(String, String), reqwest::Error> {
	let npm_url = format!("https://registry.npmjs.org/{}/latest", &package);
	let response = reqwest::get(&npm_url).await?;
	let json = response.json::<Value>().await?;
	let latest_version = json["version"].as_str().unwrap_or("Version not found").to_string();
	Ok((package, latest_version))
}

// Function to normalize version strings
fn normalize_version(version: &str) -> String {
	version.chars().filter(|&c| c.is_ascii_digit() || c == '.').collect()
}

// Function to get package manager. Choices: npm, yarn, pnpm, bun
fn get_package_manager() -> String {
	let yarn_lock_exists = Path::new("yarn.lock").exists();
	let pnpm_lock_exists = Path::new("pnpm-lock.yaml").exists();
	let bun_lock_exists = Path::new("bun.lock").exists();

	if yarn_lock_exists {
		return "yarn".to_string();
	} else if pnpm_lock_exists {
		return "pnpm".to_string();
	} else if bun_lock_exists {
		return "bun".to_string();
	} else {
		return "npm".to_string();
	}
}

fn prompt_install() {
	let package_manager = get_package_manager();
	println!("Would you like to run {} install?", package_manager);
	let mut user_input = String::new();
	io::stdin().read_line(&mut user_input).expect("Failed to read user input");
	if ["y", "Y", ""].contains(&user_input.trim()) {
		let mut command = std::process::Command::new(package_manager);
		command.arg("install");
		let output = command.output().expect("Failed to execute command");
		println!("{}", String::from_utf8_lossy(&output.stdout));
	} else {
		println!("Aborted.");
	}

}

#[tokio::main]
async fn main() {
	let path = PathBuf::from("package.json");
	let args: Vec<String> = std::env::args().collect();

	let include_dev = args.contains(&"-d".to_string()) || args.contains(&"--dev".to_string());

	// benchmarking
	let current_time = std::time::Instant::now();

	let file_content = match std::fs::read_to_string(&path) {
		Ok(content) => content,
		Err(_) => {
			println!("No package.json found in the current path. Please specify the path to package.json:");
			let mut user_input = String::new();
			io::stdin().read_line(&mut user_input).expect("Failed to read user input");
			user_input.trim().to_string()
		}
	};

	let mut json_data: Value = serde_json::from_str(&file_content).expect("Unable to parse JSON");

	// Clone keys to ensure data lives long enough for the asynchronous tasks
	let package_names: Vec<String> = json_data["dependencies"]
		.as_object()
		.unwrap()
		.keys()
		.map(|x| x.to_string())
		.collect();

	let dev_package_names: Vec<String> = json_data["devDependencies"]
		.as_object()
		.unwrap()
		.keys()
		.map(|x| x.to_string())
		.collect();

	let mut tasks = package_names
		.iter()
		.cloned() // Clone each package name
		.map(|package| fetch_package_version(package))
		.collect::<Vec<_>>();

	if include_dev {
		tasks.append(&mut dev_package_names
			.iter()
			.cloned() // Clone each package name
			.map(|package| fetch_package_version(package))
			.collect::<Vec<_>>());
	}

	let results = futures::future::join_all(tasks).await;
	let mut to_update = vec![];

	for result in results {
		match result {
			Ok((package, version)) => {
				let is_dev = dev_package_names.contains(&package);
				let current_version = if is_dev {
					json_data["devDependencies"][&package].as_str().unwrap_or("Version not found")
				} else {
					json_data["dependencies"][&package].as_str().unwrap_or("Version not found")
				};

				let semver_current_version = Version::parse(&normalize_version(current_version));
				let semver_latest_version = Version::parse(&normalize_version(&version));

				if let (Ok(curr_ver), Ok(latest_ver)) = (semver_current_version, semver_latest_version) {
					if latest_ver > curr_ver {
						to_update.push((package.clone(), version, is_dev));
					}
				}
			}
			Err(e) => {
				println!("Error fetching package version: {}", e);
			}
		}
	}

	if to_update.is_empty() {
		println!("Everything is up to date!");
		return;
	}

	println!("Packages to update:");
	for &(ref package, ref version, ref is_dev) in &to_update {
		let current_version = if *is_dev {
			json_data["devDependencies"][&package].as_str().unwrap_or("Version not found")
		} else {
			json_data["dependencies"][&package].as_str().unwrap_or("Version not found")
		};
		println!("{}: {} -> {}", package, current_version, version);
	}

	let is_interactive = args.contains(&"-i".to_string()) || args.contains(&"--interactive".to_string());
	let mut is_update = args.contains(&"-u".to_string()) || args.contains(&"--update".to_string());

	if !is_interactive && !is_update {
		println!("Do you want to update these packages? (Y/n)");
		let mut user_input = String::new();
		io::stdin().read_line(&mut user_input).expect("Failed to read user input");
        if ["y", "Y", ""].contains(&user_input.trim()) {
			for &(ref package, ref version, ref is_dev) in &to_update {
				if *is_dev {
					json_data["devDependencies"][package] = Value::String(version.clone());
				} else {
					json_data["dependencies"][package] = Value::String(version.clone());
				}
			}
			let new_json = serde_json::to_string_pretty(&json_data).unwrap();
			std::fs::write(&path, new_json).expect("Unable to write file");
	        println!("Updated {} packages.", to_update.len());
	        // prompt_install();
        } else {
	        println!("Aborted.");
        }
		return;
	}

	if is_interactive && is_update {
		println!("You're using both interactive and update flags. Continuing with interactive mode...");
		is_update = false;
	}

	if is_interactive {
		println!("Interactive mode not implemented yet");
		return;
	}
	if is_update {
		for &(ref package, ref version, is_dev) in &to_update {
			if is_dev {
				json_data["devDependencies"][package] = Value::String(version.clone());
			} else {
				json_data["dependencies"][package] = Value::String(version.clone());
			}
		}
		let new_json = serde_json::to_string_pretty(&json_data).unwrap();
		std::fs::write(&path, new_json).expect("Unable to write file");
		println!("Updated {} package(s) in {}ms.", to_update.len(), current_time.elapsed().as_millis());
		// prompt_install();
		return;
	}

}

