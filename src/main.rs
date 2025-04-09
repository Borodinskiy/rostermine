mod auth;
mod fetch;
mod parse;

use parse::parse_arguments;
use fetch::version::Version;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let _ = parse_arguments();

	let version_id = String::from("1.12.2");

	let version_1_12 = Version::new(&version_id).unwrap();

	version_1_12.update();
	version_1_12.launch();

	Ok(())
}
