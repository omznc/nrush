use std::{fs, io};
use std::path::PathBuf;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, read};
use dialoguer;
use dialoguer::MultiSelect;
use dialoguer::theme::ColorfulTheme;
use indicatif::ProgressBar;
use semver::Version;
use serde_json::Value;
use tokio::main;

use arguments::Include;
use constants::{ABOUT, GRAY, HELP};

use crate::arguments::{Arguments, Command};

mod arguments;
mod packages;
mod constants;

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

#[main]
async fn main() {
	let path = PathBuf::from("package.json");
	let args = Arguments::new();
	let current_time = Instant::now(); // Used if -u or --update is passed
	if !args.command.is_none() {
		match args.command.unwrap() {
			Command::Help => {
				println!("{}", HELP);
			}
			Command::About => {
				println!("{}", ABOUT);
			}
		}
		return;
	}

	let file_content = match fs::read_to_string(&path) {
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

	let mut dev_package_names: Vec<String> = Vec::new();
	let mut peer_package_names: Vec<String> = Vec::new();

	let mut tasks = package_names
		.iter()
		.cloned()
		.map(|package| packages::fetch_package_version(package))
		.collect::<Vec<_>>();

	if let Some(includes) = &args.include {
		if includes.contains(&Include::Dev) {
			dev_package_names = match json_data["devDependencies"].as_object() {
				Some(obj) => obj.keys().map(|x| x.to_string()).collect(),
				None => {
					Vec::new() // Return empty vector if no dev dependencies are found
				}
			};

			tasks.append(
				&mut dev_package_names
					.iter()
					.cloned() // Clone each package name
					.map(|package| packages::fetch_package_version(package))
					.collect::<Vec<_>>(),
			);
		}

		if includes.contains(&Include::Peer) {
			peer_package_names = match json_data["peerDependencies"].as_object() {
				Some(obj) => obj.keys().map(|x| x.to_string()).collect(),
				None => {
					Vec::new() // Return empty vector if no peer dependencies are found
				}
			};

			tasks.append(
				&mut peer_package_names
					.iter()
					.cloned() // Clone each package name
					.map(|package| packages::fetch_package_version(package))
					.collect::<Vec<_>>(),
			);
		}
	}

	let results = ProgressBar::new(tasks.len() as u64);
	println!("Checking {} packages for updates...", tasks.len());

	let time_elapsed = Instant::now();
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
	let include_message = if let Some(include_value) = &args.include {
		if include_value.contains(&Include::Dev) && include_value.contains(&Include::Peer) {
			" including dev and peer dependencies"
		} else if include_value.contains(&Include::Dev) {
			" including dev dependencies"
		} else if include_value.contains(&Include::Peer) {
			" including peer dependencies"
		} else {
			""
		}
	} else {
		""
	};

	println!(
		"{}Checked {} packages in {}ms{}.\x1b[0m",
		GRAY.to_string(),
		results.len(),
		time_elapsed.elapsed().as_millis(),
		include_message
	);

	let mut to_update = vec![];

	let get_current_package_version = |package: &str, json_data: &Value| -> String {
		let get_version_from_dependency = |dependency_type: &str| {
			json_data[dependency_type][package]
				.as_str()
				.unwrap_or("Version not found")
				.to_string()
		};

		if dev_package_names.contains(&package.to_string()) {
			get_version_from_dependency("devDependencies")
		} else if peer_package_names.contains(&package.to_string()) {
			get_version_from_dependency("peerDependencies")
		} else {
			get_version_from_dependency("dependencies")
		}
	};

	let set_new_package_version =
		|package: &str, version: &str, is_dev: bool, is_peer: bool, json_data: &mut Value| {
			let get_dependency_type = |is_dev: bool, is_peer: bool| {
				if is_dev {
					"devDependencies"
				} else if is_peer {
					"peerDependencies"
				} else {
					"dependencies"
				}
			};

			if !args.skip_ranges {
				let current_version = get_current_package_version(&package, &json_data);
				let current_version_range = packages::get_version_range(&current_version);
				let new_version = format!("{}{}", current_version_range, version);
				let dependency_type = get_dependency_type(is_dev, is_peer);
				json_data[dependency_type][package] = Value::String(new_version);
			} else {
				let dependency_type = get_dependency_type(is_dev, is_peer);
				json_data[dependency_type][package] = Value::String(version.to_string());
			}
		};

	for result in results {
		match result {
			Ok((package, version)) => {
				let is_dev = dev_package_names.contains(&package);
				let is_peer = peer_package_names.contains(&package);
				let current_version = get_current_package_version(&package, &json_data);
				let semver_current_version = Version::parse(&packages::normalize_version(
					&current_version,
				));
				let semver_latest_version = Version::parse(&packages::normalize_version(&version));
				if let (Ok(curr_ver), Ok(latest_ver)) = (semver_current_version, semver_latest_version)
				{
					if latest_ver > curr_ver {
						to_update.push((package.clone(), version, is_dev, is_peer));
					}
				} else if current_version == "*" && args.update_any  {
						to_update.push((package.clone(), version, is_dev, is_peer));
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

	if !args.interactive && !args.update {
		for &(ref package, ref version, ref is_dev, ref is_peer) in &to_update {
			let current_version = get_current_package_version(&package, &json_data);
			println!(
                "{}: {} -> {} ({})",
                package,
                current_version,
                packages::colorize_version(&current_version, &version),
                if *is_dev {
					"dev"
				} else if *is_peer {
					"peer"
				} else {
					"prod"
				}
			);
		}

		let user_input =
			prompt_confirm("Do you want to update all of these packages? (y/N)", false);
		if user_input {
			for &(ref package, ref version, ref is_dev, ref is_peer) in &to_update {
				set_new_package_version(&package, &version, *is_dev, *is_peer, &mut json_data);
			}
			let new_json = serde_json::to_string_pretty(&json_data).unwrap();
			fs::write(&path, new_json).expect("Unable to write file");
			println!("Updated {} packages.", to_update.len());
		} else {
			println!("No packages were updated.");
		}
		return;
	}

	if args.interactive && args.update {
		println!(
			"{}You're using both interactive and update flags. Continuing with interactive mode.\x1b[0m",
			GRAY.to_string()
		);
	}

	if args.interactive {
		let mut selected = vec![];
		let mut items = vec![];
		for &(ref package, ref version, ref is_dev, ref is_peer) in &to_update {
			let current_version = get_current_package_version(&package, &json_data);
			items.push(format!(
                "{}: {} -> {} ({})",
                package,
                current_version,
                packages::colorize_version(&current_version, &version),
                if *is_dev {
					"dev"
				} else if *is_peer {
					"peer"
				} else {
					"prod"
				}
			));
		}

		let selections = MultiSelect::with_theme(&ColorfulTheme::default())
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

		for (i, &(ref package, ref version, ref is_dev, ref is_peer)) in
		to_update.iter().enumerate()
		{
			if selected.contains(&i) {
				set_new_package_version(&package, &version, *is_dev, *is_peer, &mut json_data);
			}
		}

		let new_json = serde_json::to_string_pretty(&json_data).unwrap();
		fs::write(&path, new_json).expect("Unable to write file");
		println!("Updated {} package(s)", selected.len());
		return;
	}

	if args.update {
		for &(ref package, ref version, is_dev, is_peer) in &to_update {
			if is_dev {
				json_data["devDependencies"][package] = Value::String(version.clone());
			} else if is_peer {
				json_data["peerDependencies"][package] = Value::String(version.clone());
			} else {
				json_data["dependencies"][package] = Value::String(version.clone());
			}
		}
		let new_json = serde_json::to_string_pretty(&json_data).unwrap();
		fs::write(&path, new_json).expect("Unable to write file");
		println!(
			"Updated {} package(s) in {}ms.",
			to_update.len(),
			current_time.elapsed().as_millis()
		);
	}
}
