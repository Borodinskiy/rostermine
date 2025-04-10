mod auth;
mod fetch;
mod parse;
mod util;

use parse::parse_arguments;
use fetch::version::Version;
use util::error::Error;

fn main() -> Result<(), Error> {
	
	let _ = parse_arguments();
	
	let version_1_12 = Version::new(&String::from("b1.7.3"))?;
	
	version_1_12.update()?;
	//version_1_12.launch()?;
	
	Ok(())
}
