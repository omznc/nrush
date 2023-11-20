use std::path::PathBuf;

#[derive(Debug)]
pub enum Semver {
	Major,
	Minor,
	Patch,
}

#[derive(PartialEq, Debug)]
pub enum Include {
	Dev,
	Peer,
}

#[derive(PartialEq, Debug)]
pub enum Command {
	Help,
	About,
}

#[derive(Debug)]
pub struct Arguments {
	pub command: Option<Command>,
	pub update: bool,
	pub interactive: bool,
	pub path: PathBuf,
	pub semver: Option<Semver>,
	pub include: Option<Vec<Include>>,
	pub skip_ranges: bool,
	pub update_any: bool,
}

impl Arguments {
	pub fn new() -> Arguments {
		let mut args = Arguments {
			command: None,
			update: false,
			interactive: false,
			path: PathBuf::new(),
			semver: None,
			include: None,
			skip_ranges: false,
			update_any: false,
		};

		let mut args_iter = std::env::args().skip(1);

		while let Some(arg) = args_iter.next() {
			match arg.to_lowercase().as_str() {
				"-u" | "--update" => args.update = true,
				"-i" | "--interactive" => args.interactive = true,
				"-p" | "--path" => {
					if let Some(path) = args_iter.next() {
						args.path = PathBuf::from(path);
						if !args.path.exists() {
							panic!("Path does not exist");
						}
					}
				}
				"-s" | "--semver" => {
					if let Some(semver) = args_iter.next() {
						args.semver = match semver.to_lowercase().as_str() {
							"major" => Some(Semver::Major),
							"minor" => Some(Semver::Minor),
							"patch" => Some(Semver::Patch),
							_ => panic!("Invalid semver type. Must be major, minor, or patch"),
						}
					}
				}
				"--include" => {
					if let Some(include) = args_iter.next() {
						args.include = Some(include.split(",").map(|s| match s {
							"dev" => Include::Dev,
							"peer" => Include::Peer,
							_ => panic!("Invalid include type"),
						}).collect());
					}
				}
				"--skip-ranges" => args.skip_ranges = true,
				"--update-any" => args.update_any = true,
				_ => {
					args.command = match arg.to_lowercase().as_str() {
						"help" => Some(Command::Help),
						"about" => Some(Command::About),
						_ => panic!("Invalid command"),
					}
				}
			}
		}

		args
	}
}