use std::path::Path;
use std::fs;
use reqwest::blocking::Client;
use std::process::Command;

use super::manifest::{DataObject, VersionManifest, VersionPackage};

pub struct Version {
	// Required for certain checks
	package: VersionPackage,
	// Optional for offline
	manifest: Option<VersionManifest>,
}

impl Version {
	pub fn new(version_id: &String) -> Option<Version> {
		let manifest = VersionManifest::new(&version_id);

		let package = VersionPackage::new(&version_id, &manifest).unwrap();

		Some(Self {
			package,
			manifest,
		})
	}

	pub fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
		let download_pool = self.package.get_data_objects().unwrap();

		let client = Client::new();

		let mut i = 1;
		let max = download_pool.len();
		for object in download_pool {
			println!("Task: {}/{}", i, max);
			Version::update_task(&client, object);
			i += 1;
		};
		
		Ok(())
	}

	fn update_task(client: &Client, object: DataObject) {
		println!("PATH: {}\nURL: {}",
			object.path,
			object.url
		);
		
		if object.is_cached() {
			println!("SATISFIED\n");
			return;
		}
		
		let path = Path::new(&object.path);
		fs::create_dir_all(path.parent().unwrap()).unwrap();

		println!("GET\n");
		let bytes = client.get(&object.url).send().unwrap().bytes().unwrap();
		fs::write(path, bytes).unwrap();
	}

	pub fn launch(&self) -> Result<(), Box<dyn std::error::Error>>{
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
			.join(";");

		class_path = format!("{};{}", class_path, main_class);

		let minecraft_arguments = &self.package.minecraft_arguments.as_ref().unwrap();

		let mut args = vec![
			"-Djava.library.path=data/natives",
			"-Xms1G",
			"-Xmx4G",
			"-cp", &class_path,
			&self.package.main_class,
		];
		for arg in minecraft_arguments.split(' ') {
			args.push(match arg {
				"${auth_player_name}" => "Player", // Replace with real auth
				"${version_name}" => &self.package.id,
				"${game_directory}" => "data/instance",
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