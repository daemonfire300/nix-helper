# Config Contract

This document freezes the offline contract between `opentofu/` and `nix-cache-scaleway/` for the first tooling milestone.

## Manifest

- Tracked example manifest: `examples/infrastructure-manifest.example.json`
- Reserved runtime manifest path: `runtime/infrastructure-manifest.json`
- Intended generator: `tofu output -json manifest > ../runtime/infrastructure-manifest.json`

Required manifest fields:

- `cache_name`: logical Nix cache name used for signing-key generation and consumer trust strings
- `project_id`: dedicated Scaleway Project identifier
- `bucket_name`: private Object Storage bucket name
- `region`: Scaleway region for the bucket
- `endpoint`: S3-compatible endpoint hostname
- `author_application_id`: IAM application ID that future publish credentials must match
- `consumer_application_id`: IAM application ID that future consumer credentials must match

The Rust config layer treats all of these fields as required and non-empty.

## Secret Layout

Reserved runtime secret paths:

- `runtime/secrets/operator.sops.yaml`
- `runtime/secrets/author.sops.yaml`
- `runtime/secrets/consumer.sops.yaml`
- `runtime/secrets/signing.sops.yaml`

Tracked encrypted examples:

- `examples/secrets/operator.example.sops.yaml`
- `examples/secrets/author.example.sops.yaml`
- `examples/secrets/consumer.example.sops.yaml`
- `examples/secrets/signing.example.sops.yaml`

Each secret file is role-scoped and must stay encrypted with `sops` for the local recipient policy in [`.sops.yaml`](/home/julius/oss/me/nix-helper/cache/scaleway/.sops.yaml).

Expected keys by file:

- `operator.sops.yaml`
  `schema_version`
  `operator.organization_id`
  `operator.access_key`
  `operator.secret_key`
- `author.sops.yaml`
  `schema_version`
  `author.application_id`
  `author.access_key`
  `author.secret_key`
- `consumer.sops.yaml`
  `schema_version`
  `consumer.application_id`
  `consumer.access_key`
  `consumer.secret_key`
- `signing.sops.yaml`
  `schema_version`
  `signing.cache_name`
  `signing.private_key`

The Rust validation layer parses these YAML documents directly and rejects missing keys or empty values. It also enforces the cross-file matches that later workflows depend on:

- `author.application_id == manifest.author_application_id`
- `consumer.application_id == manifest.consumer_application_id`
- `signing.cache_name == manifest.cache_name`

## Non-Secret Outputs

- Reserved public key output path: `runtime/public/cache.pub`
- Tracked example public key path: `examples/public/cache.pub`

This file stays outside the encrypted secret set so later consumer configuration can reference it directly.

## Consumer Configuration Example

- Tracked example consumer configuration path: `examples/consumer-config.example.nix`

Consumer-facing Nix configuration must include both:

- a substituter entry using the derived Scaleway S3 URL
- the matching trusted public key from `cache.pub`

Expected shape:

```nix
{
  nix.settings = {
    extra-substituters = [
      "s3://<bucket>?endpoint=s3.<region>.scw.cloud&region=<region>&scheme=https"
    ];
    extra-trusted-public-keys = [
      "<cache-name>:<base64-public-key>"
    ];
  };
}
```

The Rust `verify` workflow validates the consumer config against the manifest-derived store URL and the exact public key contents it is given.

## Publish Contract

The `publish` workflow remains dry-run-only in this milestone, but its preflight and rendered command are fixed now so later work does not change the interface silently.

Derived Scaleway store URL:

- `s3://<bucket>?endpoint=s3.<region>.scw.cloud&region=<region>&scheme=https`

The Rust command validates that `manifest.endpoint` matches the region-derived hostname before rendering the publish plan.

Expected publish-time inputs:

- `runtime/infrastructure-manifest.json`
- `runtime/secrets/author.sops.yaml`
- `runtime/secrets/signing.sops.yaml`
- one or more `/nix/store/...` paths to publish

Expected future live publish environment:

- `AWS_ACCESS_KEY_ID` sourced from `author.access_key`
- `AWS_SECRET_ACCESS_KEY` sourced from `author.secret_key`
- `AWS_REGION` sourced from `manifest.region`
- a temporary decrypted signing key file derived from `signing.private_key`

For future live execution, `publish` is expected to materialize the signing key into a temporary path shaped like `<tmp>/nix-cache-scaleway/publish/<cache_name>.sec` and pass it to Nix through `secret-key-files`.

The dry-run output renders the exact `nix copy` command shape that later live execution will use:

```sh
nix copy --option secret-key-files <tmp>/nix-cache-scaleway/publish/<cache-name>.sec --to \
  's3://<bucket>?endpoint=s3.<region>.scw.cloud&region=<region>&scheme=https' \
  /nix/store/<path>...
```
