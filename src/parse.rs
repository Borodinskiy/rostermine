use clap::{command, Arg, ArgGroup, ArgMatches};

pub fn parse_arguments() -> ArgMatches {
	command!()
		.about("A simple minecraft launcher")
		.group(ArgGroup::new("launch")
			.arg("version-id")
		)
		.group(
			ArgGroup::new("auth")
		)
		.arg(
			Arg::new("version-id")
				.short('l')
				.long("launch")
				.aliases(["start"])
				.help("Download and launch specified version")
				.conflicts_with("get-manifest")
		)
		.arg(
			Arg::new("get-manifest")
				.short('M')
				.long("version-manifest")
				.help("Print minecraft versions manifest to stdout")
		)
		.get_matches()
}
