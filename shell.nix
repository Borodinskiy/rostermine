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
	LD_LIBRARY_PATH = with pkgs // pkgs.xorg; "${libXxf86vm}/lib:${glfw}/lib:${libGL}/lib:${libXi}:${libXrandr}/lib:${libX11}/lib:${libXext}/lib:${libXcursor}/lib:${libXrandr}/pkgs:${glfw3-minecraft}/lib:${wayland}/lib";
}
