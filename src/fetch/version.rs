use std::path::Path;
use std::fs;
use std::process::Command;

use reqwest::blocking::Client;

use indicatif::{ProgressBar, ProgressStyle};

use crate::util::error::Error;

use super::vanilla::{DataObject, Manifest, VersionPackage};

pub struct Version {
	// Required for certain checks
	package: VersionPackage,
}

impl Version {
	pub fn new(version_id: &String) -> Result<Version, Error> {
		let manifest = Manifest::new().unwrap();
		let version_manifest = manifest
			.get_for_version(version_id);
//			.expect("Failed to find sufficient minecraft version");

		Ok(Self {
			package: VersionPackage::new(&version_manifest)?,
		})
	}

	pub fn update(&self) -> Result<(), Error> {
		let client = Client::new();
		
		let objects = self.package.get_data_objects()?;

		let mut size = 0;
		for object in &objects {
			size += object.size;
		}
		println!("Size in storage: {} MB", size as f32 / 1048576f32);

		let bar = ProgressBar::new(objects.len() as u64).with_style(
			ProgressStyle::with_template(&"[{elapsed_precise}] {msg}\n{bar:20} {pos:>5}/{len}")
				.expect("error in... Progress bar styling :/")
		);

		println!("Checking objects in storage. . .");
		for object in &objects {
			Self::update_task(&client, &bar, &object);
			bar.inc(1);
		};

		bar.set_message("DONE!");
		bar.finish();

		self.package.extract_natives()?;

		Ok(())
	}

	fn update_task(client: &Client, bar: &ProgressBar, object: &DataObject) {
		if object.is_cached() {
			if bar.message() != "OK" {
				bar.set_message("OK");
			}
			return;
		}
		
		let path = Path::new(&object.path);
		fs::create_dir_all(path.parent().unwrap())
			.expect("failed to create dir for data object");

		bar.set_message(format!("GET {}", object.url));
		loop {
			match client.get(&object.url).send() {
				Ok(response) => {
					if let Ok(bytes) = response.bytes() {
						fs::write(path, bytes).unwrap();
						break;
					}
				},
				Err(e) => bar.set_message(format!("ERROR: {e}\nRetrying. . .")),
			}
		}
	}

	pub fn launch(&self) -> Result<(), Error>{
		let main_class = format!("data/libraries/net/minecraft/client/{}/client-{}-official.jar",
			self.package.id, self.package.id
		);

		let mut class_path = self.package.libraries
			.iter()
			.filter(|lib| lib.downloads.artifact.is_some())
			.map(|lib|
				format!("data/libraries/{}",
					lib.downloads.artifact.as_ref()
						.unwrap()
						.path
				)
			)
			.collect::<Vec<_>>()
			.join(":");

		class_path = format!("{}:{}", class_path, main_class);

		let natives_arg = format!("-Djava.library.path='data/versions/{}/natives/extracted'", self.package.id);

		let minecraft_arguments = "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type}"
			.split(' ');

		let mut args = vec![
			&natives_arg,
			"-Xms1G",
			"-Xmx4G",
			"-cp", &class_path,
			&self.package.main_class,
		];
		for arg in minecraft_arguments {
			args.push(match arg {
				"${auth_player_name}" => "Player", // Replace with real auth
				"${version_name}" => &self.package.id,
				"${game_directory}" => "data/instances/Default",
				"${assets_root}" => "data/assets",
				"${assets_index_name}" => &self.package.assets,
				"${auth_uuid}" => "0",
				"${auth_access_token}" => "0",
				"${user_type}" => "offline",
				"${version_type}" => &self.package.r#type,
				_ => arg,
			});
		};

		Command::new("java")
			.args(dbg!(args))
			.spawn()?
			.wait()?;

		Ok(())
	}
}