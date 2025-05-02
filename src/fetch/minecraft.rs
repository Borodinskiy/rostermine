use std::fs;
use std::path::Path;
use std::process::Command;

use std::collections::HashMap;

use reqwest::blocking::Client;

use indicatif::{ProgressBar, ProgressStyle};

use crate::util::error::Error;

use super::vanilla::{DataObject, LaunchArgumentsType, Manifest, Vanilla};

pub struct Minecraft {
	// Required for certain checks
	package: Vanilla,

	instance_dir: String,

	assets_dir: String,
	libraries_dir: String,
	versions_dir: String,
}

impl Minecraft {
	pub fn new(data_dir: String, instance_dir: String, version_id: &String) -> Result<Self, Error> {
		let manifest = Manifest::new().unwrap();
		let version_manifest = manifest.get_for_version(version_id);
		//			.expect("Failed to find sufficient minecraft version");

		Ok(Self {
			package: Vanilla::new(&version_manifest)?,
			instance_dir,
			assets_dir: format!("{data_dir}/assets"),
			libraries_dir: format!("{data_dir}/libraries"),
			versions_dir: format!("{data_dir}/versions"),
		})
	}

	pub fn update(&self) -> Result<(), Error> {
		let client = Client::new();

		let objects = self.package.get_data_objects()?;

		let mut size = 0;
		for object in &objects {
			size += object.size;
		}
		println!("Size inside storage: {} MB", size as f32 / 1048576f32);

		let bar = ProgressBar::new(objects.len() as u64).with_style(
			ProgressStyle::with_template(&"[{elapsed_precise}] {bar:20} {pos:>5}/{len} {msg}")
				.expect("error in... Progress bar styling :/"),
		);

		println!("Checking storage. . .");
		for object in &objects {
			Self::update_task(&client, &bar, &object);
			bar.inc(1);
		}

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
		fs::create_dir_all(path.parent().unwrap()).expect("failed to create dir for data object");

		bar.set_message(format!("GET {}", object.url));
		loop {
			match client.get(&object.url).send() {
				Ok(response) => {
					if let Ok(bytes) = response.bytes() {
						fs::write(path, bytes).unwrap();
						break;
					}
				}
				Err(e) => bar.set_message(format!("ERROR: {e}\nRetrying. . .")),
			}
		}
	}

	pub fn launch(&self) -> Result<(), Error> {
		let class_separator = match std::env::consts::OS {
			"linux" | "macos" => ":",
			_ => ";",
		};

		let class_path = self
			.package
			.libraries
			.iter()
			.filter(|lib| lib.downloads.artifact.is_some())
			.map(|lib| {
				format!(
					"{}/{}",
					self.libraries_dir,
					lib.downloads.artifact.as_ref().unwrap().path
				)
			})
			.chain(vec![format!(
				"{}/net/minecraft/client/{}/client-{}-official.jar",
				self.libraries_dir, self.package.id, self.package.id
			)])
			.collect::<Vec<_>>()
			.join(class_separator);

		let main_class = &self.package.main_class;

		let natives_directory = format!("{}/{}/natives", self.versions_dir, self.package.id);

		let natives_override = [
			format!("-Djava.library.path={natives_directory}"),
			format!("-Djna.tmpdir={natives_directory}"),
			format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={natives_directory}"),
			format!("-Dio.netty.native.workdir={natives_directory}"),
		];

		let mut jvm_arguments = vec!["-Xms1G", "-Xmx4G"];

		// let logging_argument = self.package.get_logging_argument();

		// if logging_argument.is_some() {
		// 	jvm_arguments.push(logging_argument.as_ref().unwrap().as_str());
		// }

		let minecraft_jvm_arguments: Vec<&str> = self
			.package
			.get_launch_arguments(LaunchArgumentsType::Jvm)
			.unwrap_or(vec![natives_override[0].as_str(), "-cp", &class_path])
			.iter()
			.map(|&argument| match argument {
				"-Djava.library.path=${natives_directory}" => natives_override[0].as_str(),
				"-Djna.tmpdir=${natives_directory}" => natives_override[1].as_str(),
				"-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}" => {
					natives_override[2].as_str()
				}
				"-Dio.netty.native.workdir=${natives_directory}" => natives_override[3].as_str(),
				"-Dminecraft.launcher.brand=${launcher_name}" => {
					"-Dminecraft.launcher.brand=rostermine"
				}
				"-Dminecraft.launcher.version=${launcher_version}" => {
					"-Dminecraft.launcher.version=0.1.0"
				}

				"${classpath}" => &class_path,

				_ => argument,
			})
			.collect();

		let minecraft_arguments: Vec<&str> = self
			.package
			.get_launch_arguments(LaunchArgumentsType::Game)
			.expect("could not launch minecraft. No launch arguments in version manifest")
			.iter()
			.map(|&argument| match argument {
				"${auth_player_name}" => "Player", // Replace with real auth
				"${version_name}" => &self.package.id,
				"${game_directory}" => &self.instance_dir,
				"${assets_root}" => &self.assets_dir,
				"${game_assets}" => &self.assets_dir,
				"${assets_index_name}" => &self.package.assets,
				"${auth_uuid}" => "0",
				"${auth_access_token}" => "0",
				"${user_type}" => "offline",
				"${user_properties}" => "{}",
				"${version_type}" => &self.package.r#type,

				_ => argument,
			})
			.collect();

		let mut envs: HashMap<String, String> = Default::default();

		match &std::env::consts::OS {
			&"linux" => {
				envs.insert(
					String::from("LD_LIBRARY_PATH"),
					match std::env::var("LD_LIBRARY_PATH") {
						Ok(oldvar) => format!("{natives_directory}:{oldvar}"),
						Err(_) => natives_directory,
					},
				);
			}
			_ => {}
		}

		Command::new("java")
			.current_dir(&self.instance_dir)
			.envs(envs)
			.args(jvm_arguments)
			.args(minecraft_jvm_arguments)
			.arg(main_class)
			.args(minecraft_arguments)
			.spawn()?
			.wait()?;

		Ok(())
	}
}
