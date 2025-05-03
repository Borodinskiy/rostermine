use std::fs;
use std::path::Path;

use reqwest::blocking::Client;

use checksums::{hash_file, Algorithm};

use crate::util::error::Error;

pub trait RetrievePlainText {
	fn retrieve_text(
		savepath: &String,
		url: &String,
		hash: Option<&String>,
	) -> Result<String, Error> {
		let path = Path::new(savepath);
		// If saved file on disk are different
		if hash.is_none() || !check_existance(path, hash.unwrap()) {
			let client = Client::new();
			let mut tries = 5usize;

			loop {
				match client.get(url).send() {
					Ok(response) => {
						let text = response.text()?;
						// Saving new manifest for future & offline work
						fs::create_dir_all(path.parent().unwrap())?;
						fs::write(path, &text)?;
						return Ok(text);
					}
					Err(e) => {
						println!("GET ERROR FOR \"{url}\":\t{e}");
						if tries > 0 {
							println!("\tRetrying ({tries}). . .");
							tries -= 1;
						} else {
								// Going to last hope - read from file
								break;
						}
					}
				}
			}
		}

		Ok(fs::read_to_string(path)?)
	}
}

fn check_existance(path: &Path, hash: &String) -> bool {
	Path::exists(path) && *hash.to_uppercase() == hash_file(path, Algorithm::SHA1)
}