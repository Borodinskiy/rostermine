mod auth;
mod fetch;
mod util;

use fetch::{vanilla::Manifest, version::Version};
use util::error::Error;

fn main() -> Result<(), Error> {
	let version_id = String::from("CHANGEME (snapshot/release/1.8.9/etc)");
	let manifest = Manifest::new()?;
	let version = manifest.get_for_version(&version_id);
	println!("\nUpdating version {}. . .", &version.id);
	let version = Version::new(&version.id)?;
	version.update()?;
	version.launch()?;

	Ok(())
}
