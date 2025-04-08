use serde::{Serialize, Deserialize};

const URL_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Serialize, Deserialize)]
struct Manifest {
	pub latest: VersionLatest,
	pub versions: Vec<Version>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionLatest {
	release: String,
	snapshot: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Version {
	id: String,
	r#type: String,
	url: String,
	time: String,
	#[serde(rename = "releaseTime")]
	release_time: String,
	sha1: String,
	#[serde(rename = "complianceLevel")]
	complicance_level: i32,
}

async fn get_manifest() -> Result<Manifest, reqwest::Error> {
	let url = String::from(URL_MANIFEST);
	let response = reqwest::Client::new()
		.get(url)
		.send()
		.await?
		.json::<Manifest>()
		.await?;

	Ok(response)
}

pub async fn get_version_manifest_url(version: String) -> Result<String, reqwest::Error> {
	Ok(get_manifest()
		.await?
		.versions
		.iter()
		.find(|element| element.id == version)
		.unwrap()
		.url
		.clone())
}
