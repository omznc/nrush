use std::path::PathBuf;
use std::time::Instant;
use std::{fs, io};

use dialoguer::theme::ColorfulTheme;
use dialoguer::MultiSelect;
use semver::Version;
use serde_json::Value;
use tokio::main;

use arguments::Include;
use constants::{ABOUT, GRAY, HELP};

use crate::arguments::{Arguments, Command};
use crate::constants::RESET;
use crate::helpers::prompt_confirm;
use crate::packages::{get_current_package_version, package_type, set_new_package_version};
use crate::progress::create_progress_bar;

mod arguments;
mod constants;
mod helpers;
mod packages;
mod progress;

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

    let package_names: Vec<String> = match json_data["dependencies"].as_object() {
        Some(obj) => obj.keys().map(|x| x.to_string()).collect(),
        None => {
            Vec::new() // Return empty vector if no dependencies are found
        }
    };

    let mut dev_package_names: Vec<String> = Vec::new();
    let mut peer_package_names: Vec<String> = Vec::new();

    let mut fetch_version_tasks = package_names
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

            fetch_version_tasks.append(
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

            fetch_version_tasks.append(
                &mut peer_package_names
                    .iter()
                    .cloned() // Clone each package name
                    .map(|package| packages::fetch_package_version(package))
                    .collect::<Vec<_>>(),
            );
        }
    }
    let time_elapsed = Instant::now();

    let progress_bar = create_progress_bar(
        fetch_version_tasks.len() as u64,
        "Fetching package versions...",
    );

    let fetch_version_results =
        futures::future::join_all(fetch_version_tasks.into_iter().map(|task| {
            let results = progress_bar.clone();
            async move {
                let result = task.await;
                results.inc(1);
                result
            }
        }))
        .await;
    progress_bar.finish_and_clear();

    fn get_include_message(include: Option<&Vec<Include>>) -> &str {
        match include {
            Some(v) if v.contains(&Include::Dev) && v.contains(&Include::Peer) => {
                " including dev and peer dependencies"
            }
            Some(v) if v.contains(&Include::Dev) => " including dev dependencies",
            Some(v) if v.contains(&Include::Peer) => " including peer dependencies",
            _ => "",
        }
    }

    print!("\x1B[2J\x1B[1;1H");
    let include_message = get_include_message(args.include.as_ref());

    println!(
        "{}Checked {} packages in {}ms{}.{}",
        GRAY,
        fetch_version_results.len(),
        time_elapsed.elapsed().as_millis(),
        include_message,
        RESET,
    );

    let mut to_update = vec![];
    for result in fetch_version_results {
        match result {
            Ok((package, version)) => {
                let is_dev = dev_package_names.contains(&package);
                let is_peer = peer_package_names.contains(&package);
                let current_version = get_current_package_version(
                    &package,
                    &json_data,
                    &dev_package_names,
                    &peer_package_names,
                );
                let semver_current_version =
                    Version::parse(&packages::normalize_version(&current_version));
                let semver_latest_version = Version::parse(&packages::normalize_version(&version));
                if let (Ok(curr_ver), Ok(latest_ver)) =
                    (semver_current_version, semver_latest_version)
                {
                    if latest_ver > curr_ver {
                        to_update.push((package.clone(), version, is_dev, is_peer));
                    }
                } else if current_version == "*" && args.update_any {
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

    let generate_items =
        |to_update: &Vec<(String, String, bool, bool)>, json_data: &Value| -> Vec<String> {
            let mut items = vec![];
            for &(ref package, ref version, ref is_dev, ref is_peer) in to_update {
                let current_version = get_current_package_version(
                    &package,
                    json_data,
                    &dev_package_names,
                    &peer_package_names,
                );
                let type_str = package_type(is_dev, is_peer);
                items.push(format!(
                    "{}: {} -> {} ({})",
                    package,
                    current_version,
                    packages::colorize_version(&current_version, &version),
                    type_str
                ));
            }
            items
        };

    if !args.interactive && !args.update {
        let items = generate_items(&to_update, &mut json_data);
        for item in items {
            println!("{}", item);
        }

        let user_input = prompt_confirm(
            "\nDo you want to update all of these packages? (y/N)",
            false,
        );

        if !user_input {
            println!("No packages were updated.");
            return;
        }

        for &(ref package, ref version, ref is_dev, ref is_peer) in &to_update {
            set_new_package_version(&package, &version, *is_dev, *is_peer, &mut json_data);
        }
        let new_json = serde_json::to_string_pretty(&json_data).unwrap();
        fs::write(&path, new_json).expect("Unable to write file");

        println!("Updated {} packages.", to_update.len());
    }

    if args.interactive && args.update {
        println!(
            "{}You're using both interactive and update flags. Continuing with interactive mode.{}",
            GRAY, RESET
        );
    }

    if args.interactive {
        let mut selected = vec![];
        let items = generate_items(&to_update, &json_data);

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
