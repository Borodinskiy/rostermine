use serde::{Deserialize, Serialize};
use checksums::Algorithm::SHA1 as ALGSHA1;
use checksums::hash_file;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use super::manifest::get_version_manifest_url;

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
	#[serde(rename = "assetIndex")]
	asset_index: AssetIndex,
	assets: String,
	#[serde(rename = "complianceLevel")]
	compliance_level: i32,
	downloads: Downloads,
	id: String,
	#[serde(rename = "javaVersion")]
	java_version: JavaVersion,
	libraries: Libraries,
	logging: Logging,
	#[serde(rename = "mainClass")]
	main_class: String,
	#[serde(rename = "minecraftArguments")]
	minecraft_arguments: String,
	#[serde(rename = "minimumLauncherVersion")]
	minimum_launcher_version: i32,
	#[serde(rename = "releaseTime")]
	release_time: String,
	time: String,
	r#type: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct AssetIndex {
	id: String,
	sha1: String,
	size: i32,
	#[serde(rename = "totalSize")]
	total_size: i32,
	url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Downloads {
	client: Download,
	server: Download,
}
#[derive(Debug, Serialize, Deserialize)]
struct Download {
	sha1: String,
	size: i32,
	url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct JavaVersion {
	component: String,
	#[serde(rename = "majorVersion")]
	major_version: i32,
}
#[derive(Debug, Serialize, Deserialize)]
struct Library {
	downloads: LibraryDownloads,
	extract: Option<LibraryDownloadExtractRules>,
	name: String,
	natives: Option<LibraryNatives>,
	rules: Option<Vec<LibraryRule>>,
}
#[derive(Debug, Serialize, Deserialize)]
struct LibraryDownloads {
	artifact: Option<LibraryArtifact>,
	classifiers: Option<HashMap<String, LibraryDownloadClassifier>>,
}
#[derive(Debug, Serialize, Deserialize)]
struct LibraryArtifact {
	path: String,
	sha1: String,
	size: i32,
	url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct LibraryDownloadClassifier {
	path: String,
	sha1: String,
	size: i32,
	url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct LibraryDownloadExtractRules {
	exclude: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct LibraryNatives {
	linux: Option<String>,
	osx: Option<String>,
	windows: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct LibraryRule {
	action: String,
	os: Option<LibraryRuleOS>,
}
#[derive(Debug, Serialize, Deserialize)]
struct LibraryRuleOS {
	name: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Logging {
	client: LoggingRule,
}
#[derive(Debug, Serialize, Deserialize)]
struct LoggingRule {
	argument: String,
	file: LoggingRuleFile,
	r#type: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct LoggingRuleFile {
	id: String,
	sha1: String,
	size: i32,
	url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Objects {
	objects: Assets,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
	hash: String,
	size: u32,
}

pub type Assets = HashMap<String, Asset>;
pub type Libraries = Vec<Library>;

impl Version {
	pub async fn new(version: String) -> Result<Version, reqwest::Error> {
		Ok(reqwest::Client::new()
			.get(get_version_manifest_url(version).await?)
			.send()
			.await?
			.json::<Version>()
			.await?
		)
	}

	pub fn launch(&self) -> Result<(), Box<dyn std::error::Error>>{
		let main_class = format!("data/libraries/net/minecraft/client/{}/client-{}-official.jar",
			self.id, self.id
		);
		let mut class_path = self.libraries
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

		class_path = format!("\"{}:{}\"", class_path, main_class);


		let mut args = vec![
			"/usr/bin/env",
			"java",
			"-Djava.library.path=data/natives",
            "-Xmx4G",
            "-Xms1G",
			"-cp", &class_path,
			&self.main_class,
		];
		for arg in self.minecraft_arguments.split(' ') {
			args.push(match arg {
				"${auth_player_name}" => "Player", // Replace with real auth
				"${version_name}" => &self.id,
				"${game_directory}" => "data/instance",
				"${assets_root}" => "data/assets",
				"${assets_index_name}" => &self.assets,
				"${auth_uuid}" => "0",
				"${auth_access_token}" => "0",
				"${user_type}" => "offline",
				"${version_type}" => &self.r#type,
				_ => arg,
			});
		};

	let command = args.join(" ");

	Command::new("sh")
			.arg("-c")
			.arg(dbg!(command))
			.spawn()?
			.wait()?;

		Ok(())
	}


	pub async fn get_assets_index(&self) -> Result<Assets, reqwest::Error> {
		let response = reqwest::Client::new()
			.get(self.asset_index.url.as_str())
			.send()
			.await?
			.json::<Objects>()
			.await?;

		Ok(response.objects)
	}

	pub async fn download_client(&self) -> Result<(), Box<dyn std::error::Error>> {
		let client = reqwest::Client::new();

		let path_str = format!(
			"data/libraries/net/minecraft/client/{}/client-{}-official.jar",
			self.id, self.id
		);
		let path = Path::new(&path_str);

		let hash = &self.downloads.client.sha1;

		if check_existance(path, hash) {
			println!("SATISFIED: {}", path_str);
			return Ok(());
		}

		let url = &self.downloads.client.url;

		fs::create_dir_all(path.parent().unwrap())?;

		println!("GET: {}", url);
		let bytes = client
			.get(url)
			.send()
			.await?
			.bytes()
			.await?;
		fs::write(path, bytes)?;

		Ok(())
	}

	pub async fn download_assets(&self) -> Result<(), Box<dyn std::error::Error>> {
		let assets = self.get_assets_index().await?;
		let client = reqwest::Client::new();

		for (_, asset) in assets {
			let path_str = format!(
				"data/assets/objects/{}/{}",
				&asset.hash[0..2], asset.hash
			);
			let path = Path::new(&path_str);

			if check_existance(path, &asset.hash) {
				println!("SATISFIED: {}", path_str);
				continue;
			}

			let url = format!(
				"https://resources.download.minecraft.net/{}/{}",
				&asset.hash[0..2], asset.hash
			);

			fs::create_dir_all(path.parent().unwrap())?;

			println!("GET: {}", url);
			let bytes = client
				.get(url)
				.send()
				.await?
				.bytes()
				.await?;
			fs::write(path, bytes)?;
		}

		Ok(())
	}

	pub async fn download_libraries(&self) -> Result<(), Box<dyn std::error::Error>> {
		let client = reqwest::Client::new();

		for library in &self.libraries {
			if library.downloads.artifact.is_none() { continue; }
			let artifact = library.downloads.artifact.as_ref().unwrap();

			let path_str = format!("data/libraries/{}", artifact.path);
			let hash = &artifact.sha1;
			let path = Path::new(&path_str);

			if check_existance(path, hash) {
				println!("SATISFIED: {}", path_str);
				continue;
			}

			let url = &artifact.url;

			fs::create_dir_all(path.parent().unwrap())?;

			println!("GET: {}", url);
			let bytes = client.get(url).send().await?.bytes().await?;
			fs::write(path, bytes)?;
		}

		Ok(())
	}
	
	pub async fn download_natives(&self) -> Result<(), Box<dyn std::error::Error>> {
		let client = reqwest::Client::new();
		let natives = self.libraries
			.iter()
			.filter(|lib| lib.downloads.classifiers.is_some());

		for library in natives {
			for (name, native) in library.downloads.classifiers.as_ref().unwrap() {
				if name != "natives-linux" { continue; }

				let path_str = format!("data/natives/{}", native.path);
				let hash = &native.sha1;
				let path = Path::new(&path_str);
				
				if check_existance(path, hash) {
					println!("SATISFIED: {}", path_str);
					continue;
				}
				
				let url = &native.url;
				
				fs::create_dir_all(path.parent().unwrap())?;
				
				println!("GET: {}", url);
				let bytes = client.get(url).send().await?.bytes().await?;
				fs::write(path, bytes)?;
			}
		}

		Ok(())
	}
}

fn check_existance(path: &Path, hash: &String) -> bool {
	if ! Path::exists(path) { return false }
	*hash == hash_file(path, ALGSHA1).to_lowercase()
}
