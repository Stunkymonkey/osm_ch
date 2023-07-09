{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    # nativeBuildInputs is usually what you want -- tools you need to run
    nativeBuildInputs = with pkgs.buildPackages; [
      brotli
      pkg-config
      # rust
      cargo
      cargo-outdated
      rustfmt
      clippy
    ];
}
