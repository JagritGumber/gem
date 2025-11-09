use std::fmt;

#[derive(Debug)]
pub struct LexError {
	pub message: String,
	pub line: usize,
	pub column: usize,
}

impl fmt::Display for LexError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"LexError, {}, Line: {}, Column: {}",
			self.message, self.line, self.column
		)
	}
}

impl std::error::Error for LexError {}

