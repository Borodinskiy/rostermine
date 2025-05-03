mod arguments;
mod fetch;
mod util;

use fetch::{vanilla::Manifest, minecraft::Minecraft};
use util::error::Error;
use arguments::Argument;

fn main() -> Result<(), Error> {
	let working_dir = std::env::current_dir()?;
	let working_dir = working_dir.display();

	let mut version_id = String::from("release");

	let mut data_dir = format!("{working_dir}/data");
	let mut instance_dir = format!("{working_dir}/instances/Default");

	for arg in Argument::get_parsed()? {
		match arg {
			Argument::SetVersion(id) => version_id = id,
			Argument::SetInstanceDir(dir) => instance_dir = dir,
			Argument::SetDataDir(dir) => data_dir = dir,
			Argument::GetHelp => Argument::print_help_and_exit()?,
			Argument::GetProgramVersion => Argument::print_version_and_exit()?,
		}
	}

	let manifest = Manifest::new()?;
	let version = manifest.get_for_version(&version_id);

	println!("\nUpdating version {}. . .", &version.id);
	let version = Minecraft::new(data_dir, instance_dir, &version.id)?;
	version.update()?;
	version.launch()?;

	Ok(())
}
