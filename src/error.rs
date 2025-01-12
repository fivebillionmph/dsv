use std::{error::Error, fmt};

#[derive(Debug)]
pub struct AppError {
	message: String,
}

impl AppError {
	pub fn new(msg: &str) -> Self {
		Self {
			message: msg.into(),
		}
	}
}

impl fmt::Display for AppError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.message)
	}
}

impl Error for AppError {}