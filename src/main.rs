use std::fs;

mod auth;
mod fetch;
mod parse;

use parse::parse_arguments;
use fetch::version::Version;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let match_result = parse_arguments();

	let version_id = String::from("1.14");

	let version_1_12 = Version::new(&version_id).await;

	version_1_12.unwrap().update().await;

	Ok(())
}
