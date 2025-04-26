use std::path::Path;
use std::fs;
use std::process::Command;

use reqwest::blocking::Client;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::util::error::Error;

use super::vanilla::{DataObject, LaunchArgumentsType, Manifest, VersionPackage};

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
			ProgressStyle::with_template(&"[{elapsed_precise}] {bar:20} {pos:>5}/{len} {msg}")
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
		let mut class_path = self.package.libraries
			.iter()
			.filter(|lib| lib.downloads.artifact.is_some())
			.map(|lib| format!("data/libraries/{}",
				lib.downloads.artifact.as_ref()
					.unwrap()
					.path
			))
			.collect::<Vec<_>>();

		class_path.push(format!(
			"data/libraries/net/minecraft/client/{}/client-{}-official.jar",
			self.package.id, self.package.id
		));

		let separator = match std::env::consts::OS {
			"linux" => ":",

			_ => ";",
		};

		let class_path_str = class_path.join(separator);

		let natives_directory = format!("data/versions/{}/natives/extracted", self.package.id);

		let natives_override = [
			format!("-Djava.library.path={natives_directory}"),
			format!("-Djna.tmpdir={natives_directory}"),
			format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={natives_directory}"),
			format!("-Dio.netty.native.workdir{natives_directory}"),
		];

		let mut jvm_arguments = vec![
			"-Xms1G", "-Xmx4G",
			"-cp", class_path_str.as_str(),
		];

		jvm_arguments.push(&self.package.main_class);

		let minecraft_arguments: Vec<&str> = self
			.package
			.get_launch_arguments(LaunchArgumentsType::Game)
			.expect("could not launch minecraft. No launch arguments in version manifest")
			.iter()
			.map(|&argument| match argument {
				// already defined
				"-cp" => "",
				"${classpath}" => "",

				"-Djava.library.path=${natives_directory}" => natives_override[0].as_str(),
				"-Djna.tmpdir=${natives_directory}" => natives_override[1].as_str(),
				"-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}" => natives_override[2].as_str(),
				"-Dio.netty.native.workdir=${natives_directory}" => natives_override[3].as_str(),
				"-Dminecraft.launcher.brand=${launcher_name}"=> "-Dminecraft.launcher.brand=rostermine",
				"-Dminecraft.launcher.version=${launcher_version}" => "-Dminecraft.launcher.version=a0.0.1",

				"${auth_player_name}" => "Player", // Replace with real auth
				"${version_name}" => self.package.id.as_str(),
				"${game_directory}" => "instances/Default",
				"${assets_root}" => "data/assets",
				"${assets_index_name}" => self.package.assets.as_str(),
				"${auth_uuid}" => "0",
				"${auth_access_token}" => "0",
				"${user_type}" => "offline",
				"${version_type}" => self.package.r#type.as_str(),

				_ => argument,
			})
			.collect();

		Command::new("java")
			.args(dbg!(jvm_arguments))
			.args(dbg!(minecraft_arguments))
			.spawn()?
			.wait()?;

		Ok(())
	}
}