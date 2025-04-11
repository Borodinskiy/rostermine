use std::path::Path;
use std::fs;
use reqwest::blocking::Client;
use std::process::Command;
use crate::util::error::Error;

use super::json::{DataObject, Manifest, VersionPackage, VersionPackageManifest};

pub struct Version {
	// Required for certain checks
	package: VersionPackage,
}

impl Version {
	pub fn new(version_id: &String) -> Result<Version, Error> {
		let manifest = Manifest::new().unwrap();
		let version_manifest = VersionPackageManifest::new(&version_id, &manifest);

		Ok(Self {
			package: VersionPackage::new(&version_id, &version_manifest)?,
		})
	}

	pub fn update(&self) -> Result<(), Error> {
		let objects = self.package.get_data_objects()?;

		let client = Client::new();

		let mut i = 1usize;
		let max = objects.len();
		for object in objects {
			println!("Check: {}/{}", i, max);
			Version::update_task(&client, &object);
			i += 1;
		};
		
		Ok(())
	}

	fn update_task(client: &Client, object: &DataObject) {
		if object.is_cached() { return; }
		
		let path = Path::new(&object.path);
		fs::create_dir_all(path.parent().unwrap()).unwrap();

		println!("GET: {}\n", object.url);
		loop {
			match client.get(&object.url).send() {
				Ok(response) => {
					let bytes = response.bytes().unwrap();
					fs::write(path, bytes).unwrap();
					break;
				},
				Err(e) => println!("GET ERROR: {e}\nRetrying. . ."),
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
			.join(";");

		class_path = format!("{};{}", class_path, main_class);

		let minecraft_arguments = "--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${assets_root} --assetIndex ${assets_index_name} --uuid ${auth_uuid} --accessToken ${auth_access_token} --userType ${user_type} --versionType ${version_type}"
			.split(' ');

		let mut args = vec![
			"-Djava.library.path=data/natives",
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