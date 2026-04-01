# OpenTofu Scaffold

This directory holds the static OpenTofu scaffold for the Scaleway-backed Nix cache layout.

The current milestone is intentionally scaffold-only:

- no live `tofu apply`
- no live API key creation
- no live Scaleway verification

The root module models:

- one dedicated Scaleway Project
- one private Object Storage bucket per environment
- separate `admin`, `author`, and `consumer` IAM applications
- project-scoped IAM policies
- a private bucket policy with no public principals
- bucket-side default server-side encryption

The output contract is exposed both as individual outputs and as a combined `manifest` object so later Rust commands can consume the same JSON shape expected by `nix-cache-scaleway`.

The combined manifest includes the logical `cache_name` used for signing-key naming in addition to the bucket and IAM identifiers. The intended offline export target is `../runtime/infrastructure-manifest.json` relative to this directory.

Because the `nixos-25.11` packaged Scaleway provider does not yet expose the dedicated bucket encryption resource, the scaffold models that one surface through the AWS S3-compatible encryption resource while keeping the rest of the infrastructure in the Scaleway provider.
