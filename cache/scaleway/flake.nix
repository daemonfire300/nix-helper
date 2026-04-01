{
  description = "Bootstrap flake for the Scaleway binary cache workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      crane,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      mkOutputs =
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
          workspaceDir = ./.;
          crateDir = workspaceDir + "/nix-cache-scaleway";
          crateManifest = crateDir + "/Cargo.toml";
          crateLock = crateDir + "/Cargo.lock";
          crateExists = builtins.pathExists crateManifest;
          opentofuDir = ./. + "/opentofu";
          opentofuScaffoldExists = builtins.pathExists (opentofuDir + "/versions.tf");
          awsTofuProvider = pkgs.terraform-providers.hashicorp_aws;
          scalewayTofuProvider = pkgs.terraform-providers.scaleway_scaleway;
          tofuProviderOverrides = pkgs.runCommand "nix-cache-scaleway-opentofu-provider-overrides" { } ''
            mkdir -p "$out/aws" "$out/scaleway"
            ln -s ${awsTofuProvider}/libexec/terraform-providers/registry.terraform.io/hashicorp/aws/${awsTofuProvider.version}/linux_amd64/* "$out/aws/"
            ln -s ${scalewayTofuProvider}/libexec/terraform-providers/registry.terraform.io/scaleway/scaleway/${scalewayTofuProvider.version}/linux_amd64/* "$out/scaleway/"
          '';
          tofuCliConfig = pkgs.writeText "nix-cache-scaleway-opentofu.tfrc" ''
            provider_installation {
              dev_overrides {
                "hashicorp/aws"    = "${tofuProviderOverrides}/aws"
                "scaleway/scaleway" = "${tofuProviderOverrides}/scaleway"
              }

              direct {}
            }
          '';

          nixCacheScaleway =
            if crateExists then
              craneLib.buildPackage {
                pname = "nix-cache-scaleway";
                version = "0.1.0";
                src = workspaceDir;
                cargoExtraArgs = "--manifest-path nix-cache-scaleway/Cargo.toml";
                strictDeps = true;
                cargoLock = crateLock;
              }
            else
              pkgs.writeShellApplication {
                name = "nix-cache-scaleway";
                text = ''
                  echo "TASK-6 not implemented yet: nix-cache-scaleway crate is not present." >&2
                  exit 1
                '';
              };

          opentofuStaticCheck =
            if opentofuScaffoldExists then
              pkgs.runCommand "nix-cache-scaleway-opentofu-static-check" {
                nativeBuildInputs = [
                  pkgs.opentofu
                  awsTofuProvider
                  scalewayTofuProvider
                ];
              } ''
                export HOME="$TMPDIR/home"
                export TF_CLI_CONFIG_FILE="${tofuCliConfig}"
                mkdir -p "$HOME"
                cp -r ${opentofuDir} "$TMPDIR/opentofu"
                chmod -R u+w "$TMPDIR/opentofu"
                cd "$TMPDIR/opentofu"

                tofu fmt -check -recursive
                tofu validate

                touch "$out"
              ''
            else
              pkgs.runCommand "nix-cache-scaleway-opentofu-static-check-skipped" { } ''
                touch "$out"
              '';

          devToolsCheck = pkgs.runCommand "nix-cache-scaleway-dev-tools-check" {
            nativeBuildInputs = [
              pkgs.opentofu
              awsTofuProvider
              scalewayTofuProvider
              rustToolchain
              pkgs.rust-analyzer
              pkgs.cargo-nextest
              pkgs.s3cmd
              pkgs.sops
              pkgs.rage
              pkgs.jq
              pkgs.ripgrep
            ];
          } ''
            tofu version >/dev/null
            rustc --version >/dev/null
            cargo --version >/dev/null
            cargo fmt --version >/dev/null
            cargo clippy --version >/dev/null
            cargo nextest --version >/dev/null
            rust-analyzer --version >/dev/null
            s3cmd --version >/dev/null
            sops --version >/dev/null
            rage --version >/dev/null
            jq --version >/dev/null
            rg --version >/dev/null

            touch "$out"
          '';
        in
        {
          packages = {
            default = nixCacheScaleway;
            nix-cache-scaleway = nixCacheScaleway;
          };

          checks = {
            nix-cache-scaleway = nixCacheScaleway;
            dev-tools = devToolsCheck;
            opentofu-static = opentofuStaticCheck;
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              s3cmd
              typst
              rage
              sops
              ripgrep
              git
              curl
              jq
              opentofu
              awsTofuProvider
              scalewayTofuProvider
              rustToolchain
              rust-analyzer
              cargo-nextest
            ];

            shellHook = ''
              export TF_CLI_CONFIG_FILE="${tofuCliConfig}"
            '';
          };
        };
    in
    {
      packages = nixpkgs.lib.genAttrs systems (system: (mkOutputs system).packages);
      checks = nixpkgs.lib.genAttrs systems (system: (mkOutputs system).checks);
      devShells = nixpkgs.lib.genAttrs systems (system: (mkOutputs system).devShells);
    };
}
