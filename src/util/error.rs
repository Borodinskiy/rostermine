use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("failure in network request")]
	Download(#[from] reqwest::Error),
	#[error("unable to do a filesystem operation: {0}")]
	IO(#[from] std::io::Error),
	#[error("{0}")]
	Default(#[from] Box<dyn std::error::Error>),
}
