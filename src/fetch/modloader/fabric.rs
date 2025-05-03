use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::fetch::textfile::RetrievePlainText;

use crate::util::error::Error;

const URL_FABRIC: &str = "https://meta.fabricmc.net/v2/versions/loader";

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct VersionManifest {
	pub separator: String,
	pub build: usize,
	pub maven: String,
	pub version: String,
	pub stable: bool,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct Version {
	pub id: String,
	pub inherits_from: String,
	pub release_time: String,
	pub r#type: String,
	pub main_class: String,

	pub arguments: HashMap<String, Vec<String>>,

	pub libraries: Vec<Library>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Library {
	pub name: Box<str>,
	pub url: Box<str>,
	#[serde(rename = "sha1")]
	pub hash: Box<str>,
	pub size: usize,
}

impl VersionManifest {
	pub fn new() -> Result<Self, Error> {
		let path = String::from("data/fabric_manifest.json");

		Ok(serde_json::from_str(
			Self::retrieve_text(
				&path,
				&URL_FABRIC.to_string(),
				None,
			)?
			.as_str(),
		)?)
	}
}
impl RetrievePlainText for VersionManifest {}

impl RetrievePlainText for Version {}
