use crate::util::error::Error;

pub enum Argument {
	SetVersion(String),
	SetInstanceDir(String),
	SetDataDir(String),
	GetHelp,
	GetProgramVersion,
}

impl Argument {
	fn parse(previous: &String, current: Option<String>) -> Result<Argument, Error> {
		if let Some(current) = current {
			match previous.as_str() {
				"-l" | "--launch" => return Ok(Self::SetVersion(current)),
				"-i" | "--instance-dir" => return Ok(Self::SetInstanceDir(current)),
				"-d" | "--data-dir" => return Ok(Self::SetDataDir(current)),

				_ => return Err(Error::Default(format!("wrong argument: {previous}"))),
			}
		}

		match previous.as_str() {
			"-h" | "--help" => Ok(Self::GetHelp),
			"-v" | "--version" => Ok(Self::GetProgramVersion),

			_ => Ok(Self::GetHelp),
		}
	}

	pub fn get_parsed() -> Result<Vec<Self>, Error> {
		let mut result: Vec<Self> = Default::default();
		let arguments = std::env::args();
		result.reserve(arguments.len());

		let mut previous = String::from("");

		for argument in arguments {
			if argument.starts_with("-") {
				if previous.len() > 0 {
					return Err(Error::Default(format!(
						"value not provided for argument {previous}"
					)));
				}

				previous = argument;
			} else if previous.len() > 0 {
				result.push(Self::parse(&previous, Some(argument))?);
				previous.clear();
			}
		}

		if previous.len() > 0 {
			result.push(Self::parse(&previous, None)?);
		}

		Ok(result)
	}

	pub fn print_help_and_exit() -> Result<(), Error> {
		let path = std::env::current_exe();
		let current_exe = match path.as_ref() {
			Ok(path) => match path.iter().last() {
				Some(exe) => exe.to_str().unwrap(),
				None => "rostermine",
			},
			Err(_) => "rostermine",
		};

		println!("USAGE: {current_exe} -l [version id]",);
		println!("-l\t--launch [version id] - Launch minecraft");
		println!("-i\t--instance-dir [path] - Directory for game saves, mods, etc.");
		println!("-d\t--data-dir [path]     - TODO");
		println!("-h\t--help                - Help ;/");

		std::process::exit(0);
	}

	pub fn print_version_and_exit() -> Result<(), Error> {
		println!("RosterMine 0.1.0");
		std::process::exit(0);
	}
}
