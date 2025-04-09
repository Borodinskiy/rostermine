
use tokio::fs::File;
use tokio::fs;
use std::{hash::DefaultHasher, path::Path, thread::JoinHandle};
use tokio::io::AsyncWriteExt;
use reqwest::Client;
use checksums::hash_file;
use checksums::Algorithm::SHA1 as ALGSHA1;

use super::manifest::{DataObject, VersionManifest, VersionPackage};

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

	pub async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
		let download_pool = self.package.get_data_objects().await.unwrap();

		let client = Client::new();

		let mut i = 1;
		let max = download_pool.len();
		for object in download_pool {
			println!("Task: {}/{}", i, max);
			Version::update_task(&client, object).await;
			i += 1;
		};
		
		Ok(())
	}

	async fn update_task(client: &Client, object: DataObject) {
		println!("PATH: {}\nURL: {}",
			object.path,
			object.url
		);
		
		
		if object.is_cached() {
			println!("SATISFIED\n");
			return;
		}
		
		let path = Path::new(&object.path);
		fs::create_dir_all(path.parent().unwrap()).await.unwrap();

		println!("GET\n");
		let bytes = client.get(&object.url).send().await.unwrap().bytes().await.unwrap();
		fs::write(path, bytes).await.unwrap();
	}
}