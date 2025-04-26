{ pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
	nativeBuildInputs = with pkgs; [
		rustc
		cargo

		pkg-config
		gcc
		cmake
		openssl
	];

	packages = with pkgs // pkgs.xorg;[
		stdenv.cc.cc
		jdk
		rustfmt
		## native versions
		glfw3-minecraft
		openal

		## openal
		alsa-lib
		libjack2
		libpulseaudio
		pipewire

		## glfw
		libGL
		libX11
		libXcursor
		libXext
		libXrandr
		libXxf86vm

		udev

		vulkan-loader
	];

	RUST_SRC_PATH = with pkgs; "${rustPlatform.rustLibSrc}";
	LD_LIBRARY_PATH = with pkgs // pkgs.xorg; lib.makeLibraryPath [
		libXxf86vm
		glfw
		libGL
		libXi
		libXrandr
		libX11
		libXext
		libXcursor
		libXrandr
		glfw3-minecraft
		wayland
	];
}
