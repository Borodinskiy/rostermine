use std::fs;

mod auth;
mod fetch;
mod parse;

use parse::parse_arguments;

use fetch::version::Version;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	//let match_result = parse_arguments();

	let version = dbg!(Version::new(String::from("1.12.2")).await?);

	version.download_client().await?;
	version.download_assets().await?;
	version.download_libraries().await?;
	version.download_natives().await?;

	version.launch()?;

	Ok(())
}
