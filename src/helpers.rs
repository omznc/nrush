use crossterm::event::{KeyCode, KeyEvent, read};

pub fn prompt_confirm(message: &str, default: bool) -> bool {
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