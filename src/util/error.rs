use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("network request: {0}")]
	Download(#[from] reqwest::Error),

	#[error("filesystem operation: {0}")]
	IO(#[from] std::io::Error),

	#[error("{0}")]
	Default(String),

	#[error("json parsing: {0}")]
	JSONParse(#[from] serde_json::Error),
}
