use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use checksums::{hash_file, Algorithm};

use crate::util::error::Error;

use super::textfile::RetrievePlainText;

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
	pub versions: Vec<VanillaManifest>,
}

// Here goes manifests for version package
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct VanillaManifest {
	pub id: String,
	pub r#type: String,
	pub url: String,
	pub time: String,
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
#[serde(default, rename_all = "camelCase")]
pub struct Vanilla {
	pub asset_index: DataObject,
	pub assets: String,
	// Client/Server
	pub downloads: HashMap<String, DataObject>,
	pub id: String,
	pub java_version: JavaVersion,
	pub libraries: Vec<Library>,
	// Client/Server
	pub logging: HashMap<String, Logging>,
	pub main_class: String,
	pub minecraft_arguments: Option<String>,
	pub arguments: Option<ExecArgumentsArray>,
	pub release_time: String,
	pub r#type: String,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct JavaVersion {
	pub component: String,
	pub major_version: i32,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Library {
	pub downloads: LibraryDownloads,
	pub name: String,
	pub rules: Option<Vec<Rule>>,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LibraryDownloads {
	pub artifact: Option<DataObject>,
	pub classifiers: Option<HashMap<String, DataObject>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
	action: Action,
	os: Option<OS>,
}
#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
	Allow,
	Disallow,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct OS {
	name: Option<OSName>,
	arch: Option<OSArch>,
}
#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OSName {
	Windows,
	Linux,
	OSX,
	Undefined,
}
#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OSArch {
	X86,
	X86_64,
	Undefined,
}
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Logging {
	pub argument: String,
	pub file: DataObject,
	pub r#type: String,
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

#[derive(PartialEq)]
pub enum LaunchArgumentsType {
	Game,
	Jvm,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DataObject {
	#[serde(alias = "id", alias = "path")]
	pub path: String,
	pub size: usize,
	pub url: String,
	#[serde(alias = "sha1", alias = "hash")]
	pub hash: Box<str>,
}

impl Rule {
	pub fn check(&self, host: &OS) -> bool {
		if let Some(os) = self.os.as_ref() {
			if let Some(name) = os.name.as_ref() {
				let hostname = host.name.as_ref().unwrap_or(&OSName::Undefined);
				return (self.action == Action::Allow && *hostname == *name)
					|| (self.action == Action::Disallow && *hostname != *name);
			}
			if let Some(arch) = os.arch.as_ref() {
				let hostarch = host.arch.as_ref().unwrap_or(&OSArch::Undefined);
				return (self.action == Action::Allow && *hostarch == *arch)
					|| (self.action == Action::Disallow && *hostarch != *arch);
			}
		}

		self.action == Action::Allow
	}

	pub fn check_complex(rules: &Vec<Self>, host: &OS) -> bool {
		let mut result = false;
		for rule in rules {
			result = rule.check(&host);
		}

		result
	}
	pub fn check_some_complex(rules: Option<&Vec<Self>>, host: &OS) -> bool {
		if let Some(rules) = rules {
			return Self::check_complex(rules, host);
		}

		// Ignoring disallowing because of no rules was given
		true
	}
}

impl OS {
	pub fn current() -> Self {
		Self {
			name: Some(OSName::current()),
			arch: Some(OSArch::current()),
		}
	}
}

impl OSName {
	pub fn current() -> Self {
		match &std::env::consts::OS {
			&"linux" => OSName::Linux,
			&"windows" => OSName::Windows,
			&"macos" => OSName::OSX,
			_ => OSName::Undefined,
		}
	}
}

impl OSArch {
	pub fn current() -> Self {
		match &std::env::consts::ARCH {
			&"x86" => OSArch::X86,
			&"x86_64" => OSArch::X86_64,
			_ => OSArch::Undefined,
		}
	}
}

impl Manifest {
	// Manifest is important thing for retrieving up to date game resources
	// If we can't get it, then hash checking of saved versions won't fix errors
	pub fn new() -> Result<Self, Error> {
		let path = String::from("data/version_manifest_v2.json");

		Ok(serde_json::from_str(
			Self::retrieve_text(&path, &URL_MANIFEST.to_string(), None)?.as_str(),
		)?)
	}

	pub fn get_for_version(&self, version_id: &String) -> VanillaManifest {
		match version_id.as_str() {
			"release" | "snapshot" => self.get_for_version(&self.latest.get(version_id).unwrap()),
			_ => {
				if let Some(manifest) = self
					.versions
					.iter()
					.find(|&element| element.id == *version_id)
				{
					manifest.clone()
				} else {
					print!(
						"FAILED \"{}\": no such version. Falling back to latest release",
						version_id
					);

					self.get_for_version(&"release".to_string())
				}
			}
		}
	}
}
impl RetrievePlainText for Manifest {}

impl Vanilla {
	pub fn new(manifest: &VanillaManifest) -> Result<Self, Error> {
		let path = format!("data/versions/{}/{}.json", manifest.id, manifest.id);

		Ok(serde_json::from_str(
			Self::retrieve_text(&path, &manifest.url, Some(&manifest.hash))?.as_str(),
		)?)
	}

	pub fn get_data_objects(&self) -> Result<Vec<DataObject>, Error> {
		let mut objects: Vec<DataObject> = Default::default();

		/*
		ASSETS
		*/

		let assets_response: AssetsObjects;
		{
			let path = format!("data/assets/indexes/{}.json", self.assets);
			let text = Self::retrieve_text(
				&path,
				&self.asset_index.url,
				Some(&self.asset_index.hash.clone().into_string()),
			)?;
			assets_response = serde_json::from_str(text.as_str())?;
		}

		// Magic number (because some libraries have additional download (native version)
		// that is impossible to count at this stage. Usually it's 1-5 libs
		let poolsize =
			13 + assets_response.objects.len() + self.downloads.len() + self.libraries.len();
		objects
			.try_reserve(poolsize)
			.expect("failed to reserve memory for assets pool");

		// Because "objects" is a HashMap, but with useless info as hash ¯\_(ツ)_/¯
		for (_, asset) in &assets_response.objects {
			let relpath = format!("{}/{}", &asset.hash[0..2], asset.hash);
			objects.push(DataObject {
				path: format!("data/assets/objects/{}", relpath),
				url: format!("https://resources.download.minecraft.net/{}", relpath),
				hash: asset.hash.clone(),
				size: asset.size,
			});
		}

		/*
			LIBRARIES
		*/

		let host = OS::current();

		for library in &self.libraries {
			// Filtering out foreign natives
			if !Rule::check_some_complex(library.rules.as_ref(), &host) {
				continue;
			}
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
				let host = format!("natives-{}", std::env::consts::OS);
				for (name, native) in library.downloads.classifiers.as_ref().unwrap() {
					if *name != host {
						continue;
					}

					let path = format!("data/libraries/{}", native.path);
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
				..self
					.downloads
					.get("client")
					.expect("failed to get minecraft client object")
					.clone()
			});
		}

		/*
			LOGGING
		*/

		if let Some(logging) = self.logging.get("client") {
			let path = format!(
				"data/assets/objects/{}/{}/{}",
				&logging.file.hash[0..2],
				logging.file.hash,
				logging.file.path
			);

			objects.push(DataObject {
				path,
				..logging.file.clone()
			});
		}

		if objects.len() > poolsize {
			println!("!!! WARNING !!!");
			println!(
				"Objects pool size is {}, but {} was reserved (magic number: 13)",
				objects.len(),
				poolsize
			);
		}

		Ok(objects)
	}

	pub fn extract_natives(&self) -> Result<(), Error> {
		let host = OS::current();
		let host_str = format!("natives-{}", std::env::consts::OS);

		let root = String::from("data/libraries");
		let target = PathBuf::from(format!("data/versions/{}/natives", self.id));
		fs::create_dir_all(&target)?;

		for native in self
			.libraries
			.iter()
			.filter(|&lib| lib.downloads.classifiers.is_some() || lib.rules.is_some())
		{
			if !Rule::check_some_complex(native.rules.as_ref(), &host) {
				continue;
			}
			if let Some(classifiers) = native.downloads.classifiers.as_ref() {
				for (os, object) in classifiers {
					if *os == host_str {
						object.extract_to(&root, &target)?;
						break;
					}
				}
			}
		}

		Ok(())
	}

	pub fn get_class_path(&self, libraries_dir: &String, versions_dir: &String) -> String {
		let class_separator = match std::env::consts::OS {
			"windows" => ";",
			_ => ":",
		};

		self.libraries
			.iter()
			.filter(|lib| lib.downloads.artifact.is_some())
			.map(|lib| {
				format!(
					"{}/{}",
					libraries_dir,
					lib.downloads.artifact.as_ref().unwrap().path
				)
			})
			.chain(vec![
				format!(
					"{}/net/minecraft/client/{}/client-{}-official.jar",
					libraries_dir, self.id, self.id
				),
				format!(
					"{}/{}/natives",
					versions_dir, self.id
				),
			])
			.collect::<Vec<_>>()
			.join(class_separator)
	}

	pub fn get_launch_arguments(&self, r#type: LaunchArgumentsType) -> Option<Vec<&str>> {
		// If we have classic string with arguments
		if let Some(arguments) = self.minecraft_arguments.as_ref() {
			match r#type {
				LaunchArgumentsType::Game => return Some(arguments.split(" ").collect()),
				LaunchArgumentsType::Jvm => return None,
			}
		}

		// If we have complex arguments array
		if let Some(arguments) = self.arguments.as_ref() {
			let iter = match r#type {
				LaunchArgumentsType::Game => arguments.game.iter(),
				LaunchArgumentsType::Jvm => arguments.jvm.iter(),
			};
			let mut str: Vec<&str> = Default::default();

			str.try_reserve(iter.clone().count())
				.expect("failed to reserve memory for launch arguments generation");

			for argument in iter {
				//TODO: object parsing
				// str.push(match argument {
				// 	ExecArgument::String(string) => string.as_str(),
				// 	ExecArgument::Object(_object) => {
				// 		""
				// 	}
				// });
				if let ExecArgument::String(argument) = argument {
					str.push(argument);
				}
			}

			return Some(str);
		}

		None
	}
}
impl RetrievePlainText for Vanilla {}

impl DataObject {
	pub fn is_cached(&self) -> bool {
		let path = Path::new(&self.path);

		Path::exists(path) && self.hash.to_uppercase() == hash_file(path, Algorithm::SHA1)
	}

	pub fn extract_to(&self, root: &String, target: &PathBuf) -> Result<(), Error> {
		let bytes = fs::read(format!("{}/{}", root, self.path))?;

		zip_extract::extract(io::Cursor::new(bytes), &target, true)
			.expect("native jar extraction error");

		Ok(())
	}
}
