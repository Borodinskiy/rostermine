{ pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
	nativeBuildInputs = with pkgs; [
		gcc
		pkg-config
		cmake

		rustc
		cargo

		openssl
	];

	packages = with pkgs // pkgs.xorg;[
		jdk
		rustfmt
	];

	RUST_SRC_PATH = with pkgs; "${rustPlatform.rustLibSrc}";
	LD_LIBRARY_PATH = with pkgs // pkgs.xorg; lib.makeLibraryPath [
		libX11
		libXext
		libXcursor
		libXrandr
		libXxf86vm
		libpulseaudio
		alsa-lib
		libGL
		glfw
		openal
		(lib.getLib stdenv.cc.cc)

		udev

		flite # Text-to-speech
	];
}
