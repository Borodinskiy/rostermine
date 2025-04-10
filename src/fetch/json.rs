use serde::{Serialize, Deserialize};
use checksums::Algorithm::SHA1 as ALGSHA1;
use checksums::hash_file;

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use reqwest::blocking::Client;
use crate::util::error::Error;

const URL_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
	// [Release/Snapshot] [correspond version]
	pub latest: HashMap<String, String>,
	pub versions: Vec<VersionPackageManifest>,
}
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct VersionPackageManifest {
	pub id: String,
	pub r#type: String,
	pub url: String,
	pub time: String,
	#[serde(rename = "releaseTime")]
	pub release_time: String,
	#[serde(rename = "sha1")]
	pub hash: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct VersionPackage {
	#[serde(rename = "assetIndex")]
	pub asset_index: DataObject,
	pub assets: String,
	// Client/Server
	pub downloads: HashMap<String, DataObject>,
	pub id: String,
	#[serde(rename = "javaVersion")]
	pub java_version: JavaVersion,
	pub libraries: Vec<Library>,
	// Client/Server
	pub logging: HashMap<String, Logging>,
	#[serde(rename = "mainClass")]
	pub main_class: String,
	#[serde(rename = "minecraftArguments")]
	pub minecraft_arguments: Option<String>,
	#[serde(rename = "releaseTime")]
	pub release_time: String,
	pub r#type: String,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct JavaVersion {
	pub component: String,
	#[serde(rename = "majorVersion")]
	pub major_version: i32,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Library {
	pub downloads: LibraryDownloads,
	pub name: String,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LibraryDownloads {
	pub artifact: Option<DataObject>,
	pub classifiers: Option<HashMap<String, DataObject>>,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Logging {
	pub argument: String,
	pub file: LoggingRuleFile,
	pub r#type: String,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingRuleFile {
	pub id: String,
	pub sha1: String,
	pub size: i32,
	pub url: String,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AssetsObjects {
	pub objects: HashMap<String, DataObject>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DataObject {
	pub path: String,
	pub url: String,
	#[serde(alias = "sha1", alias = "hash")]
	pub hash: String,
}

impl Manifest {
	// Manifest is important thing for getting up to date assets
	// If we can't get it, then only hash checking of saved versions will work
	pub fn new() -> Option<Self> {
		let url = String::from(URL_MANIFEST);
		let response = Client::new()
			.get(url)
			.send();
		if let Ok(text) = response {
			return Some(text.json().unwrap());
		} else {
			return None;
		}
	}

	pub fn get_for_version(&self, version_id: &String) -> Option<VersionPackageManifest> {
		match version_id.as_str() {
			"release" | "snapshot" => self.get_for_version(&self.latest.get(version_id).unwrap()),
			_ => {
				self.versions.iter()
					.find(|element| element.id == *version_id)
					.cloned()
			},
		}
	}
}

impl VersionPackageManifest {
	pub fn new(version_id: &String, manifest: &Manifest) -> Option<Self> {
		manifest.get_for_version(&version_id)
	}
}

impl VersionPackage {
	pub fn new(version_id: &String, manifest: &Option<VersionPackageManifest>) -> Result<VersionPackage, Error> {
		// If we can't get manifest, final try is read cached version json
		if manifest.is_none() {
			return Self::read_from_file(version_id);
		}
		let manifest = &manifest.as_ref().unwrap();

		let path_str = format!(
			"data/versions/{}/{}.json",
			manifest.id, manifest.id,
		);

		let path = Path::new(&path_str);

		// If manifest's file similliar to cached
		if check_existance(path, &manifest.hash) {
			return Ok(Self::read_from_file(&manifest.id)?);
		}

		let response = Client::new()
			.get(&manifest.url)
			.send();

		if let Ok(response) = response {
			return Ok(response.json()?);
		} else {
			return Self::read_from_file(&manifest.id);
		};
	}

	fn read_from_file(version: &String) -> Result<VersionPackage, Error> {
		Ok(
			serde_json::from_str(
				fs::read_to_string(
					format!(
						"data/versions/{}/{}.json",
						version, version,
					)
				)?
				.as_str()
			).unwrap()
		)
	}

	pub fn get_data_objects(&self) -> Result<Vec<DataObject>, Error> {
		let client = Client::new();

		let assets_response = client
			.get(self.asset_index.url.as_str())
			.send()?
			.json::<AssetsObjects>()?;

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
			if library.downloads.artifact.is_some() {
				let artifact = library.downloads.artifact.as_ref().unwrap();
				
				let path = format!("data/libraries/{}", artifact.path);
				
				objects.push(DataObject {
					path,
					..artifact.clone()
				});
			}
			if library.downloads.classifiers.is_some() {
				for (name, native) in library.downloads.classifiers.as_ref().unwrap() {
					if name != "natives-linux" { continue; }
					
					let path = format!("data/versions/{}/natives/{}", self.id, native.path);
					objects.push(DataObject {
						path,
						..native.clone()
					});
				}
			}
		}

		/*
			CLIENT
		*/

		{
			let path = format!(
				"data/libraries/net/minecraft/client/{}/client-{}-official.jar",
				self.id, self.id
			);
			
			objects.push(DataObject {
				path,
				..self.downloads.get("client").unwrap().clone()
			});
		}

		Ok(objects)
	}

}

fn check_existance(path: &Path, hash: &String) -> bool {
	Path::exists(path) && *hash == hash_file(path, ALGSHA1).to_lowercase()
}

impl DataObject {
	pub fn is_cached(&self) -> bool {
		let path = Path::new(&self.path);

		Path::exists(path) && self.hash == hash_file(path, ALGSHA1).to_lowercase()
	}
}
