{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };

        x11Inputs = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        waylandInputs = with pkgs; [
          libxkbcommon
          wayland
        ];
        buildInputs = with pkgs; [
          udev
          alsa-lib
          vulkan-loader
        ];
        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.just
            rust
          ];
          buildInputs = buildInputs ++ x11Inputs ++ waylandInputs;
          nativeBuildInputs = nativeBuildInputs;

          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath (buildInputs ++ waylandInputs);
          BEVY_ASSET_ROOT = "./";
        };

      }
    );
}
