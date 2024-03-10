{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-overlay.follows = "rust-overlay";
      inputs.flake-utils.follows = "flake-utils";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = inputs:
    with inputs; let
      system = "x86_64-linux";
      overlays = [
        cargo2nix.overlays.default
        rust-overlay.overlays.default
      ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      runtimeInputs = with pkgs; [vulkan-loader udev alsa-lib];
      x11Inputs = with pkgs; [
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
      ];
      nativeBuildInputs = with pkgs; [pkg-config rust-analyzer];
      rustPkgs = pkgs.rustBuilder.makePackageSet {
        packageFun = import ./Cargo.nix;
        rustToolchain = toolchain;

        packageOverrides = pkgs:
          pkgs.rustBuilder.overrides.all
          ++ [
            (pkgs.rustBuilder.rustLib.makeOverride {
              name = "alsa-sys";
              overrideAttrs = drv: {
                propagatedBuildInputs = drv.propagatedBuildInputs or [] ++ [pkgs.alsa-lib];
              };
            })
          ];
      };
    in {
      packages.${system} = {
        default = (rustPkgs.workspace.gam {}).bin;
      };
      devShell.${system} = pkgs.mkShell {
        packages = [cargo2nix.packages.${system}.cargo2nix];
        buildInputs = buildInputs ++ runtimeInputs ++ x11Inputs ++ waylandInputs;
        nativeBuildInputs = nativeBuildInputs;

        shellHook = ''
          export LD_LIBRARY_PATH="${nixpkgs.lib.makeLibraryPath runtimeInputs}"
        '';
      };
    };
}
