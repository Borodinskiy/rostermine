mod fetch;
mod util;

use fetch::{vanilla::Manifest, minecraft::Minecraft};
use util::error::Error;

fn main() -> Result<(), Error> {
	let version_id = String::from("snapshot");
	let manifest = Manifest::new()?;
	let version = manifest.get_for_version(&version_id);

	let data_dir = format!("{}/data", std::env::current_dir()?.display());
	let instance_dir = format!("{}/instances/Default", std::env::current_dir()?.display());

	println!("\nUpdating version {}. . .", &version.id);
	let version = Minecraft::new(data_dir, instance_dir, &version.id)?;
	version.update()?;
	version.launch()?;

	Ok(())
}
