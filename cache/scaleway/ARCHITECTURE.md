# Architecture

The `cache/scaleway` subproject is intended to support declarative setup of a Nix binary cache backed by Scaleway Object Storage.

At the TASK-1 stage, this workspace only bootstraps developer tooling and secret-handling defaults. Actual cache provisioning, bucket configuration, and consumer integration are future work.

## Flake

The subproject now ships a standalone `flake.nix` that provides a reproducible development shell for:

- local iteration on the Scaleway cache workspace
- a pinned Nix toolchain baseline via `nixpkgs` `nixos-25.11`

The default devShell includes:

- `s3cmd`
- `typst`
- `rage`
- `sops`
- `ripgrep`
- `git`
- `curl`
- `jq`

## Secrets

Secrets in this subproject are expected to be handled with `sops`.

- Recipient defaults are defined in `.sops.yaml`.
- The current bootstrap recipient set contains Julius' public age key from `julius.pub.rage`.
- New or updated secret material under `cache/scaleway` should be encrypted for the configured recipients.
