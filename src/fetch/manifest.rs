use serde::{Serialize, Deserialize};
use checksums::Algorithm::SHA1 as ALGSHA1;
use checksums::hash_file;

use std::collections::HashMap;
use std::{fs, io};
use std::path::Path;
use std::process::Command;

const URL_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
	pub latest: ManifestLatestVersion,
	pub versions: Vec<VersionManifest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestLatestVersion {
	pub release: String,
	pub snapshot: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionManifest {
	pub id: String,
	pub r#type: String,
	pub url: String,
	pub time: String,
	#[serde(rename = "releaseTime")]
	pub release_time: String,
	pub sha1: String,
	#[serde(rename = "complianceLevel")]
	pub complicance_level: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionPackage {
	#[serde(rename = "assetIndex")]
	pub asset_index: AssetIndex,
	pub assets: String,
	#[serde(rename = "complianceLevel")]
	pub compliance_level: i32,
	pub downloads: Downloads,
	pub id: String,
	#[serde(rename = "javaVersion")]
	pub java_version: JavaVersion,
	pub libraries: Vec<Library>,
	pub logging: Logging,
	#[serde(rename = "mainClass")]
	pub main_class: String,
	// New versions style
	#[serde(skip)]
	pub arguments: Option<String>,
	// Old versions style
	#[serde(rename = "minecraftArguments")]
	pub minecraft_arguments: Option<String>,
	#[serde(rename = "minimumLauncherVersion")]
	pub minimum_launcher_version: i32,
	#[serde(rename = "releaseTime")]
	pub release_time: String,
	pub time: String,
	pub r#type: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetIndex {
	pub id: String,
	pub sha1: String,
	pub size: i32,
	#[serde(rename = "totalSize")]
	pub total_size: i32,
	pub url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Downloads {
	pub client: Download,
	pub server: Download,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Download {
	pub sha1: String,
	pub size: i32,
	pub url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct JavaVersion {
	pub component: String,
	#[serde(rename = "majorVersion")]
	pub major_version: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
	pub downloads: LibraryDownloads,
	pub extract: Option<LibraryDownloadExtractRules>,
	pub name: String,
	pub natives: Option<LibraryNatives>,
	pub rules: Option<Vec<LibraryRule>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloads {
	pub artifact: Option<LibraryArtifact>,
	pub classifiers: Option<HashMap<String, LibraryDownloadClassifier>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryArtifact {
	pub path: String,
	pub sha1: String,
	pub size: i32,
	pub url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloadClassifier {
	pub path: String,
	pub sha1: String,
	pub size: i32,
	pub url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryDownloadExtractRules {
	pub exclude: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryNatives {
	pub linux: Option<String>,
	pub osx: Option<String>,
	pub windows: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryRule {
	pub action: String,
	pub os: Option<LibraryRuleOS>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryRuleOS {
	pub name: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Logging {
	pub client: LoggingRule,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingRule {
	pub argument: String,
	pub file: LoggingRuleFile,
	pub r#type: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingRuleFile {
	pub id: String,
	pub sha1: String,
	pub size: i32,
	pub url: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MinecraftArguments {
	pub game: Option<Vec<MinecraftArgument>>,
	pub jvm: Option<Vec<MinecraftArgument>>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinecraftArgument {
	Str(String),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetsObjects {
	pub objects: HashMap<String, Asset>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
	pub hash: String,
	pub size: u32,
}


pub struct DataObject {
	pub path: String,
	pub url: String,
	pub hash: String,
}

impl Manifest {
	// Manifest is important thing for getting up to date assets
	// If we can't get it, then only hash checking of saved versions will work
	pub async fn new() -> Option<Manifest> {
		let url = String::from(URL_MANIFEST);
		let response = reqwest::Client::new()
			.get(url)
			.send()
			.await;
		if let Ok(text) = response {
			return Some(text.json().await.unwrap());
		} else {
			return None;
		}
	}

	pub fn get_for_version(&self, version_id: &String) -> Option<VersionManifest> {
		match version_id.as_str() {
			"release" => self.get_for_version(&self.latest.release),
			"snapshot" => self.get_for_version(&self.latest.snapshot),
			_ => {
				self.versions.iter()
					.find(|element| element.id == *version_id)
					.cloned()
			},
		}
	}
}

impl VersionManifest {
	pub async fn new(version_id: &String) -> Option<VersionManifest> {
		let manifest = Manifest::new().await;

		if let Some(response) = manifest {
			return response.get_for_version(&version_id);
		} else {
			return None;
		}
	}
}

impl VersionPackage {
	pub async fn new(version: &String, manifest: &Option<VersionManifest>) -> Result<VersionPackage, Box<dyn std::error::Error>> {
		// If we can't get manifest, final try is read cached version json
		if manifest.is_none() {
			return Ok(Self::read_from_file(version)?);
		}

		let url = &manifest.as_ref().unwrap().url;
		let hash = &manifest.as_ref().unwrap().sha1;
		let version = &manifest.as_ref().unwrap().id;

		let path_str = format!(
			"data/versions/{}/{}.json",
			version, version,
		);

		let path = Path::new(&path_str);

		// If manifest's file similliar to cached
		if check_existance(path, &hash) {
			return Ok(Self::read_from_file(version)?);
		}

		let response = reqwest::Client::new()
			.get(url)
			.send()
			.await;

		if let Ok(response) = response {
			return Ok(response.json().await?);
		} else {
			return Ok(Self::read_from_file(version)?);
		};
	}

	fn read_from_file(version: &String) -> Result<VersionPackage, io::Error> {
		Ok(
			serde_json::from_str(
				fs::read_to_string(
					format!(
						"data/versions/{}/{}.json",
						version, version,
					)
				)?
				.as_str()
			)?
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

		let minecraft_arguments = &self.minecraft_arguments.as_ref().unwrap();

		let mut args = vec![
			"/usr/bin/env",
			"java",
			"-Djava.library.path=data/natives",
            "-Xmx4G",
            "-Xms1G",
			"-cp", &class_path,
			&self.main_class,
		];
		for arg in minecraft_arguments.split(' ') {
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

	pub async fn get_data_objects(&self) -> Result<Vec<DataObject>, Box<dyn std::error::Error>> {
		let client = reqwest::Client::new();

		let assets_response = client
			.get(self.asset_index.url.as_str())
			.send()
			.await?
			.json::<AssetsObjects>()
			.await?;

		let mut objects: Vec<DataObject> = Default::default();

		/*
			ASSETS
		*/

		for (_, asset) in &assets_response.objects {
			let asset_2 = &asset.hash[0..2];
			objects.push(DataObject {
				path: format!(
					"data/assets/objects/{}/{}",
					asset_2, asset.hash
				),
				url: format!(
					"https://resources.download.minecraft.net/{}/{}",
					asset_2, asset.hash
				),
				hash: asset.hash.clone(),
			});
		}

		/*
			LIBRARIES
		*/

		for library in &self.libraries {
			if library.downloads.artifact.is_none() { continue; }

			let artifact = library.downloads.artifact.as_ref().unwrap();

			let path = format!("data/libraries/{}", artifact.path);

			objects.push(DataObject {
				path,
				url: artifact.url.clone(),
				hash: artifact.sha1.clone(),
			});
		}

		/*
			NATIVES
		*/

		let natives = self.libraries
			.iter()
			.filter(|lib| lib.downloads.classifiers.is_some());

		for library in natives {
			for (name, native) in library.downloads.classifiers.as_ref().unwrap() {
				if name != "natives-linux" { continue; }

				let path = format!("data/natives/{}", native.path);
				objects.push(DataObject {
					path,
					url: native.url.clone(),
					hash: native.sha1.clone(),
				});
			}
		}

		/*
			CLIENT
		*/

		{
			let path= format!(
				"data/libraries/net/minecraft/client/{}/client-{}-official.jar",
				self.id, self.id
			);
			
			objects.push(DataObject {
				path,
				url: self.downloads.client.url.clone(),
				hash: self.downloads.client.sha1.clone(),
			});
		}

		Ok(objects)
	}

}

fn check_existance(path: &Path, hash: &String) -> bool {
	if ! Path::exists(path) { return false }
	*hash == hash_file(path, ALGSHA1).to_lowercase()
}
