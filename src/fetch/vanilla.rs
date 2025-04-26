use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;

use reqwest::blocking::Client;

use serde::{Deserialize, Serialize};

use checksums::hash_file;
use checksums::Algorithm::SHA1 as ALGSHA1;

use crate::util::error::Error;

/* MANIFEST
* Minecraft version manifest - file that tells us information about minecraft download data
* It contains 2 root structures:
* - "latest": contains id for latest release and snapshot
* - "versions": array of all downloadable minecraft versions manifests
*   Contains version id, url to it's manifest
*/

const URL_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Manifest {
	// [Release/Snapshot] [correspond version]
	pub latest: HashMap<String, String>,
	pub versions: Vec<VersionPackageManifest>,
}

// Here goes manifests for version package
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

/* VERSION PACKAGE
* Contains:
* - "assetIndex": url to json file, which enumerates heavy game assets
* - "assets": name of json file, that should be placed in data/assets/indexes/[name].json
* - "downloads": urls to download main game client or server jar
* - "id": minecraft version
* - "javaVersion": small struct, which informs us about java major version, used for game
* - "libraries": huge array with list of java libraries and some natives
* - "logging": some logging configuration
* - "mainClass": path to java main class (ex.: net.minecraft.client.main.Main)
* - "arguments", "minecraftArguments": list/array of arguments, that should be passed to correspond minecraft version and jvm at launch
* - "releaseTime": game release date
* - "type": version type (snapshot/release)
*/

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
	#[serde(rename = "arguments")]
	pub arguments: Option<ExecArgumentsArray>,
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
pub struct ExecArgumentsArray {
	pub game: Vec<ExecArgument>,
	pub jvm: Vec<ExecArgument>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecArgument {
	String(String),
	Object(ExecArgumentRuled),
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ExecArgumentRuled {
	#[serde(flatten)]
	pub extra: HashMap<String, serde_json::Value>,
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
	pub size: usize,
	pub url: String,
	#[serde(alias = "sha1", alias = "hash")]
	pub hash: Box<str>,
}

trait RetrieveManifest {
	fn retrieve_manifest_text(savepath: &String, url: Option<&String>, hash: Option<&String>) -> Result<String, Error> {
		let path = Path::new(savepath);
		// If we have url and manifest's new hash different from hash of older
		if url.is_some() && ! check_existance(path, hash.unwrap()) {
			let url = url.unwrap();
			let client = Client::new();

			loop {
				match client.get(url).send() {
					Ok(response) => {
						let text = response.text()?;
						// Saving new manifest for future & offline work
						fs::create_dir_all(path.parent().unwrap())?;
						fs::write(path, &text)?;
						return Ok(text);
					},
					Err(e) => println!("GET ERROR: {}: {}\nRetrying. . .",
						url, e
					),
				}
			}
		}

		Ok(fs::read_to_string(path)?)
	}
}

impl Manifest {
	// Manifest is important thing for retrieving up to date game resources
	// If we can't get it, then hash checking of saved versions won't fix errors
	pub fn new() -> Result<Manifest, Error> {
		let path_str = String::from("data/version_manifest_v2.json");
		let path = Path::new(&path_str);

		let url = URL_MANIFEST.to_string();

		// TODO: Change hashing library to normal
		let hash: String;
		if Path::exists(path) {
			hash = hash_file(path, ALGSHA1);
		} else {
			hash = String::from("");
		}

		let text = Self::retrieve_manifest_text(
			&path_str,
			Some(&url),
			Some(&hash)
		)?;

		Ok(serde_json::from_str(text.as_str())?)
	}

	pub fn get_for_version(&self, version_id: &String) -> VersionPackageManifest {
		match version_id.as_str() {
			"release" | "snapshot" => self.get_for_version(&self.latest.get(version_id).unwrap()),
			_ => if let Some(manifest) = self
					.versions
					.iter()
					.find(|&element| element.id == *version_id)
				{
					return manifest.clone();
				} else {
					return self.get_for_version(&"release".to_string());
				}
		}
	}
}
impl RetrieveManifest for Manifest {}

impl VersionPackage {
	pub fn new(manifest: &VersionPackageManifest) -> Result<VersionPackage, Error> {
		let path_str = format!("data/versions/{}/{}.json", manifest.id, manifest.id);
		let text: String;

		text = Self::retrieve_manifest_text(
			&path_str,
			Some(&manifest.url),
			Some(&manifest.hash),
		)?;

		Ok(serde_json::from_str(text.as_str())?)
	}

	pub fn get_data_objects(&self) -> Result<Vec<DataObject>, Error> {
		let mut objects: Vec<DataObject> = Default::default();

		/*
		ASSETS
		*/
		
		let assets_response: AssetsObjects;
		{
			let path = format!("data/assets/indexes/{}.json", self.assets);
			let text = Self::retrieve_manifest_text(
				&path,
				Some(&self.asset_index.url),
				Some(&self.asset_index.hash.clone().into_string())
			)?;
			assets_response = serde_json::from_str(text.as_str())?;
		}

		// Magic number (because some libraries have additional download (native version)
		// that is impossible to count at this stage. Usually it's 1-5 libs
		let poolsize = 13 + assets_response.objects.len() + self.downloads.len() + self.libraries.len();
		objects.try_reserve(poolsize)
			.expect("failed to reserve memory for assets pool");

		// Because "objects" is a HashMap, but with useless info as hash ¯\_(ツ)_/¯
		for (_, asset) in &assets_response.objects {
			let relpath = format!("{}/{}", &asset.hash[0..2], asset.hash);
			objects.push(DataObject {
				path: format!("data/assets/objects/{}", relpath,),
				url: format!("https://resources.download.minecraft.net/{}", relpath,),
				hash: asset.hash.clone(),
				size: asset.size,
			});
		}

		/*
			LIBRARIES
		*/

		for library in &self.libraries {
			// Jar library
			if library.downloads.artifact.is_some() {
				let artifact = library.downloads.artifact.as_ref().unwrap();

				let path = format!("data/libraries/{}", artifact.path);

				objects.push(DataObject {
					path,
					..artifact.clone()
				});
			}
			// Native dll/so library
			if library.downloads.classifiers.is_some() {
				for (name, native) in library.downloads.classifiers.as_ref().unwrap() {
					if name != "natives-linux" {
						continue;
					}

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
				..self.downloads
					.get("client")
					.expect("failed to get minecraft client object")
					.clone()
			});
		}

		if objects.len() > poolsize {
			println!("!!! WARNING !!!");
			println!("Objects pool size is {}, but {} was reserved (magic number: 13)",
				objects.len(), poolsize
			);
		}

		Ok(objects)
	}

	pub fn extract_natives(&self) -> Result<(), Error> {
		let host = format!("natives-{}", std::env::consts::OS);
		println!("Extracting {}. . .", host);
		for native in self
			.libraries
			.iter()
			.filter(|&lib| lib.downloads.classifiers.is_some())
		{
			for (os, object) in native.downloads.classifiers.as_ref().unwrap() {
				if *os == host {
					let path = PathBuf::from(format!("data/versions/{}/natives/{}",
						self.id, object.path
					));

					let target = PathBuf::from(format!("data/versions/{}/natives/extracted",
						self.id
					));

					let bytes = fs::read(path)?;

					zip_extract::extract(io::Cursor::new(bytes), &target, true)
						.expect("zip extraction error");
				}
			}
		}

		Ok(())
	}
}
impl RetrieveManifest for VersionPackage {}

fn check_existance(path: &Path, hash: &String) -> bool {
	if Path::exists(path) {
		if *hash.to_uppercase() == hash_file(path, ALGSHA1) {
			println!("Satisfied: {}", path.to_str().unwrap());
			return true;
		}
		println!("Old version: {}", path.to_str().unwrap());
		return false;
	}
	
	println!("Not exists: {}", path.to_str().unwrap());
	return false;
}

impl DataObject {
	pub fn is_cached(&self) -> bool {
		let path = Path::new(&self.path);

		Path::exists(path) && self.hash.to_uppercase() == hash_file(path, ALGSHA1)
	}
}
