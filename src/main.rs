mod auth;
mod fetch;
mod util;

use fetch::version::Version;
use util::error::Error;

fn main() -> Result<(), Error> {
	let version = Version::new(&String::from("1.21"))?;

	version.update()?;
	version.launch()?;
	
	Ok(())
}
