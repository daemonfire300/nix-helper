{
  description = "Streaming Typst to Bionic Typst converter";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    crane = {
      url = "github:ipetkov/crane";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      packages = nixpkgs.lib.genAttrs systems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "cargo"
              "clippy"
              "rust-src"
            ];
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
          src = craneLib.cleanCargoSource ./.;
          commonArgs = {
            inherit src;
            strictDeps = true;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          package = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
        in
        {
          default = package;
        }
      );

      apps = nixpkgs.lib.genAttrs systems (
        system:
        let
          package = self.packages.${system}.default;
        in
        {
          default = {
            type = "app";
            program = "${package}/bin/bionic-typst";
          };
        }
      );

      devShells = nixpkgs.lib.genAttrs systems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "cargo"
              "clippy"
              "rust-analyzer"
              "rust-src"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = [
              toolchain
              pkgs.typos
            ];
          };
        }
      );
    };
}
