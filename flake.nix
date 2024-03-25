{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # rust-overlay = {
    #   url = "github:oxalica/rust-overlay";
    #   inputs.nixpkgs.follows = "nixpkgs";
    #   inputs.flake-utils.follows = "flake-utils";
    # };
  };

  outputs = inputs:
    with inputs; let
      system = "x86_64-linux";
      # overlays = [
      #   rust-overlay.overlays.default
      # ];
      pkgs = import nixpkgs {
        inherit system;
      };

      # rust_toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

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
      nativeBuildInputs = with pkgs; [pkg-config];
    in {
      packages.${system} = {
        # default = (rustPkgs.workspace.gam {}).bin;
      };

      devShell.${system} = pkgs.mkShell {
        packages = with pkgs; [just];
        buildInputs = buildInputs ++ x11Inputs ++ waylandInputs;
        nativeBuildInputs = nativeBuildInputs;

        LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath (buildInputs ++ waylandInputs);
        BEVY_ASSET_ROOT = "./";
      };
    };
}
