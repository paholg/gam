{ pkgs ? import <nixpkgs> {} }:
with pkgs; mkShell {
  nativeBuildInputs = [
    pkgconfig
    clang lld # To use lld linker
  ];
  buildInputs = [
    udev alsaLib vulkan-loader
    xlibsWrapper xorg.libXcursor xorg.libXrandr xorg.libXi # To use x11 feature
    libtorch-bin openssl # for tch-rs
  ];
  shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
    udev alsaLib vulkan-loader 
    libtorch-bin # for tch-rs
  ]}"'';
}