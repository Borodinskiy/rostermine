use std::collections::HashMap;

use tokio::fs::File;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use checksums::hash_file;
use checksums::Algorithm::SHA1 as ALGSHA1;

use super::manifest::{VersionManifest, VersionPackage};

pub struct Version {
	// Required for certain checks
	package: VersionPackage,
	// Optional for offline
	manifest: Option<VersionManifest>,
}

impl Version {
	pub async fn new(version_id: &String) -> Option<Version> {
		let manifest = VersionManifest::new(&version_id).await;

		let package = VersionPackage::new(&version_id, &manifest).await.unwrap();

		Some(Self {
			package,
			manifest,
		})
	}

	//#[tokio::main]
	pub async fn update(&self) {
		let download_pool = self.package.get_data_objects().await.unwrap();

		for object in download_pool {
			println!("PATH: {}\nURL: {}\n",
				object.path,
				object.url
			);
		}
		
	}
}

fn check_existance(path: &Path, hash: &String) -> bool {
	if ! Path::exists(path) { return false }
	*hash == hash_file(path, ALGSHA1).to_lowercase()
}