mod auth;
mod fetch;
mod util;

use fetch::{vanilla::Manifest, version::Version};
use util::error::Error;

fn main() -> Result<(), Error> {
	let manifest = Manifest::new()?;
	for version in manifest.versions {
		println!("\nUpdating version {}. . .", &version.id);
		let version = Version::new(&version.id)?;
		version.update()?;
	}

	Ok(())
}
