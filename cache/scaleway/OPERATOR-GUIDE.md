# Operator Guide

This milestone is offline-safe scaffolding only.

- Do not run live Scaleway provisioning from this workspace yet.
- Do not expect `publish` or `verify` to contact Scaleway in the current phase.
- Use `--dry-run` flows to validate the contract locally.

## Local Validation Now

1. Validate the OpenTofu tree without applying anything.

   ```sh
   cd cache/scaleway/opentofu
   tofu fmt -check -recursive
   tofu validate
   ```

2. Export the manifest into the reserved runtime location.

   ```sh
   cd cache/scaleway/opentofu
   tofu output -json manifest > ../runtime/infrastructure-manifest.json
   ```

3. Bootstrap local signing and secret material.

   ```sh
   cd cache/scaleway
   cargo run --manifest-path nix-cache-scaleway/Cargo.toml -- bootstrap \
     --manifest runtime/infrastructure-manifest.json \
     --operator-secret runtime/secrets/operator.sops.yaml \
     --author-secret runtime/secrets/author.sops.yaml \
     --consumer-secret runtime/secrets/consumer.sops.yaml \
     --signing-secret runtime/secrets/signing.sops.yaml \
     --public-key-out runtime/public/cache.pub
   ```

4. Prepare or update a consumer config example so it includes:
   - `runtime/public/cache.pub`
   - `s3://<bucket>?endpoint=s3.<region>.scw.cloud&region=<region>&scheme=https`

   A tracked reference exists at `examples/consumer-config.example.nix`.

5. Validate the publish contract locally.

   ```sh
   cd cache/scaleway
   cargo run --manifest-path nix-cache-scaleway/Cargo.toml -- publish \
     --manifest runtime/infrastructure-manifest.json \
     --author-secret runtime/secrets/author.sops.yaml \
     --signing-secret runtime/secrets/signing.sops.yaml \
     --dry-run \
     /nix/store/example
   ```

6. Validate the full verify contract locally.

   ```sh
   cd cache/scaleway
   cargo run --manifest-path nix-cache-scaleway/Cargo.toml -- verify \
     --manifest runtime/infrastructure-manifest.json \
     --operator-secret runtime/secrets/operator.sops.yaml \
     --author-secret runtime/secrets/author.sops.yaml \
     --consumer-secret runtime/secrets/consumer.sops.yaml \
     --signing-secret runtime/secrets/signing.sops.yaml \
     --public-key runtime/public/cache.pub \
     --consumer-config examples/consumer-config.example.nix \
     --dry-run
   ```

## Future Live Flow

Once real Scaleway access is approved, the intended sequence is:

1. Validate OpenTofu config with `tofu fmt -check -recursive` and `tofu validate`.
2. Provision or import the real Scaleway project, bucket, IAM applications, and policies through OpenTofu.
3. Export the resulting manifest to `runtime/infrastructure-manifest.json`.
4. Obtain real operator, author, and consumer credentials and capture them in the role-scoped `sops` files.
5. Run `bootstrap` again only if signing material or local secret state must be refreshed.
6. Run `publish` with real credentials after live execution is implemented.
7. Run a real verification pass against the live bucket and consumer path once Scaleway API access and live checks are added.

## Notes

- The tracked example files under `examples/` are contract references, not live credentials.
- `runtime/` stays Git-ignored and is the reserved location for local generated artifacts.
- A future live verification step should confirm bucket access and an actual consumer fetch, but that is intentionally out of scope for this milestone.
