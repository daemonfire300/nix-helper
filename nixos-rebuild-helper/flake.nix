{
  description = "Nix-managed Rust bootstrap for nixos-rebuild-helper";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rustfmt"
            "clippy"
          ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        src = ./.;
        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        nixosRebuildHelper = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
        testCheck = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
      in
      {
        packages.default = nixosRebuildHelper;
        packages.nixos-rebuild-helper = nixosRebuildHelper;

        checks.build = nixosRebuildHelper;
        checks.test = testCheck;

        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.rust-analyzer
          ];
        };
      }
    );
}
