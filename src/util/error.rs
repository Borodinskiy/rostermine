use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("failure in network request: {0}")]
	Download(#[from] reqwest::Error),

	#[error("unable to do a filesystem operation: {0}")]
	IO(#[from] std::io::Error),

	#[error("general error: {0}")]
	Default(#[from] Box<dyn std::error::Error>),

	#[error("failure in json parsing: {0}")]
	JSONParse(#[from] serde_json::Error),
}
