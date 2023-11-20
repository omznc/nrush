use crossterm::event::{read, KeyCode, KeyEvent};
use dialoguer;
use indicatif;
use reqwest;
use semver::Version;
use serde_json::Value;
use std::io;
use std::path::PathBuf;
use tokio;

// Function to fetch package version asynchronously
async fn fetch_package_version(package: String) -> Result<(String, String), reqwest::Error> {
	let npm_url = format!("https://registry.npmjs.org/{}/latest", &package);
	let response = reqwest::get(&npm_url).await?;
	let json = response.json::<Value>().await?;
	let latest_version = json["version"]
		.as_str()
		.unwrap_or("Version not found")
		.to_string();
	Ok((package, latest_version))
}

// Function to normalize version strings
fn normalize_version(version: &str) -> String {
	version
		.chars()
		.filter(|&c| c.is_ascii_digit() || c == '.')
		.collect()
}

// Prompt user
fn prompt_confirm(message: &str, default: bool) -> bool {
	println!("{}", message);

	loop {
		if let Ok(crossterm::event::Event::Key(KeyEvent { code, .. })) = read() {
			match code {
				KeyCode::Char('y') | KeyCode::Char('Y') => return true,
				KeyCode::Char('n') | KeyCode::Char('N') => return false,
				KeyCode::Enter => return default,
				_ => (),
			}
		}
	}
}

// Version colors
const MAJOR: &str = "\x1b[31m";
const MINOR: &str = "\x1b[33m";
const PATCH: &str = "\x1b[32m";

// Other colors
const GRAY: &str = "\x1b[90m";

// Function to colorize version strings
fn colorize_version(current_version: &str, latest_version: &str) -> String {
	let current_version = normalize_version(current_version);
	let latest_version = normalize_version(latest_version);

	let current_version = Version::parse(&current_version).unwrap();
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
			io::stdin()
				.read_line(&mut user_input)
				.expect("Failed to read user input");
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

	let dev_package_names: Vec<String> = match json_data["devDependencies"].as_object() {
		Some(obj) => obj.keys().map(|x| x.to_string()).collect(),
		None => {
			// Handle the case when "devDependencies" is not an object or is None
			Vec::new() // Or any default value or error handling logic
		}
	};

	let mut tasks = package_names
		.iter()
		.cloned() // Clone each package name
		.map(|package| fetch_package_version(package))
		.collect::<Vec<_>>();

	if include_dev {
		tasks.append(
			&mut dev_package_names
				.iter()
				.cloned() // Clone each package name
				.map(|package| fetch_package_version(package))
				.collect::<Vec<_>>(),
		);
	}

	let results = indicatif::ProgressBar::new(tasks.len() as u64);
	println!("Checking {} packages for updates...", tasks.len());

	let time_elapsed = std::time::Instant::now();
	let results = futures::future::join_all(tasks.into_iter().map(|task| {
		let results = results.clone();
		async move {
			let result = task.await;
			results.inc(1);
			result
		}
	}))
		.await;

	print!("\x1B[2J\x1B[1;1H");
	println!(
		"{}Checked {} packages in {}ms.\x1b[0m",
		GRAY.to_string(),
		results.len(),
		time_elapsed.elapsed().as_millis()
	);

	let mut to_update = vec![];

	let get_current_package_version = |package: &str, json_data: &Value| -> String {
		// Use cloned json_data to avoid borrowing conflicts
		let is_dev = dev_package_names.contains(&package.to_string());
		if is_dev {
			json_data["devDependencies"][package]
				.as_str()
				.unwrap_or("Version not found")
				.to_string()
		} else {
			json_data["dependencies"][package]
				.as_str()
				.unwrap_or("Version not found")
				.to_string()
		}
	};

	let set_new_package_version =
		|package: &str, version: &str, is_dev: bool, json_data: &mut Value| {
			// Use cloned json_data to avoid borrowing conflicts
			if is_dev {
				json_data["devDependencies"][package] = Value::String(version.to_string());
			} else {
				json_data["dependencies"][package] = Value::String(version.to_string());
			}
		};

	for result in results {
		match result {
			Ok((package, version)) => {
				let is_dev = dev_package_names.contains(&package);
				let semver_current_version = Version::parse(&normalize_version(
					&get_current_package_version(&package, &json_data),
				));
				let semver_latest_version = Version::parse(&normalize_version(&version));

				if let (Ok(curr_ver), Ok(latest_ver)) =
					(semver_current_version, semver_latest_version)
				{
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

	let is_interactive =
		args.contains(&"-i".to_string()) || args.contains(&"--interactive".to_string());
	let mut is_update = args.contains(&"-u".to_string()) || args.contains(&"--update".to_string());

	if !is_interactive && !is_update {
		for &(ref package, ref version, ref _is_dev) in &to_update {
			let current_version = get_current_package_version(&package, &json_data);
			println!(
				"{}: {} -> {}",
				package,
				current_version,
				colorize_version(&current_version, &version)
			);
		}

		let user_input =
			prompt_confirm("Do you want to update all of these packages? (y/N)", false);
		if user_input {
			for &(ref package, ref version, ref is_dev) in &to_update {
				set_new_package_version(&package, &version, *is_dev, &mut json_data);
			}
			let new_json = serde_json::to_string_pretty(&json_data).unwrap();
			std::fs::write(&path, new_json).expect("Unable to write file");
			println!("Updated {} packages.", to_update.len());
		} else {
			println!("No packages were updated.");
		}
		return;
	}

	if is_interactive && is_update {
		println!(
			"{}You're using both interactive and update flags. Continuing with interactive mode.\x1b[0m",
			GRAY.to_string()
		);
		is_update = false;
	}

	if is_interactive {
		let mut selected = vec![];
		let mut items = vec![];
		for &(ref package, ref version, ref _is_dev) in &to_update {
			let current_version = get_current_package_version(&package, &json_data);
			items.push(format!(
				"{}: {} -> {}",
				package,
				current_version,
				colorize_version(&current_version, &version)
			));
		}

		let selections = dialoguer::MultiSelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
			.with_prompt(format!("Select packages to update {}(space to select, enter to confirm, arrow keys to navigate, a to toggle all)\x1b[0m", GRAY.to_string()))
			.items(&items)
			.defaults(
				(0..items.len())
					.map(|_| true)
					.collect::<Vec<_>>()
					.as_slice(),
			)
			.interact()
			.expect("Failed to read user input");

		for selection in selections {
			selected.push(selection);
		}

        if selected.is_empty() {
            println!("Nothing was selected so no packages were updated.");
            return;
        }

		for (i, &(ref package, ref version, ref is_dev)) in to_update.iter().enumerate() {
			if selected.contains(&i) {
				set_new_package_version(&package, &version, *is_dev, &mut json_data);
			}
		}

		let new_json = serde_json::to_string_pretty(&json_data).unwrap();
		std::fs::write(&path, new_json).expect("Unable to write file");
		println!("Updated {} package(s)", selected.len());
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
		println!(
			"Updated {} package(s) in {}ms.",
			to_update.len(),
			current_time.elapsed().as_millis()
		);
	}
}
