use indicatif::ProgressBar;
use crate::constants::GRAY;

pub fn create_progress_bar(fetch_version_tasks_len: u64, message: &str) -> ProgressBar {
	ProgressBar::new(fetch_version_tasks_len)
		.with_message(
			format!("{}{}", GRAY, message)
		)
		.with_style(
			indicatif::ProgressStyle::default_bar()
				.template("{spinner:.green} [{elapsed}] [{bar:40.cyan/blue}] {pos:>1}/{len:1} {msg}")
				.unwrap()
				.progress_chars("#>-")
		)
}