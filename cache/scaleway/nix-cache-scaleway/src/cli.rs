use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "nix-cache-scaleway",
    version,
    about = "Offline-safe scaffolding for the Scaleway-backed Nix binary cache workflows."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prepare local signing and secret outputs without contacting Scaleway.
    Bootstrap(BootstrapArgs),
    /// Validate publish inputs and render the dry-run-only publish placeholder.
    Publish(PublishArgs),
    /// Validate verification inputs and render the dry-run-only verify placeholder.
    Verify(VerifyArgs),
}

#[derive(Debug, Args)]
pub struct BootstrapArgs {
    /// Path to the OpenTofu-generated infrastructure manifest JSON.
    #[arg(long)]
    pub manifest: PathBuf,

    /// Target path for the operator secret file.
    #[arg(long)]
    pub operator_secret: PathBuf,

    /// Target path for the author secret file.
    #[arg(long)]
    pub author_secret: PathBuf,

    /// Target path for the consumer secret file.
    #[arg(long)]
    pub consumer_secret: PathBuf,

    /// Target path for the signing secret file.
    #[arg(long)]
    pub signing_secret: PathBuf,

    /// Output path for the consumer-facing public key.
    #[arg(long)]
    pub public_key_out: PathBuf,

    /// Optional path to manually captured author credentials for later tasks.
    #[arg(long)]
    pub author_credentials_input: Option<PathBuf>,

    /// Optional path to manually captured consumer credentials for later tasks.
    #[arg(long)]
    pub consumer_credentials_input: Option<PathBuf>,
}

#[derive(Debug, Args)]
#[command(
    long_about = "Validate publish inputs and render the placeholder workflow. \
The current milestone is dry-run-only, and running publish without --dry-run still stays in dry-run mode."
)]
pub struct PublishArgs {
    /// Path to the OpenTofu-generated infrastructure manifest JSON.
    #[arg(long)]
    pub manifest: PathBuf,

    /// Path to the author secret file.
    #[arg(long)]
    pub author_secret: PathBuf,

    /// Path to the signing secret file.
    #[arg(long)]
    pub signing_secret: PathBuf,

    /// Accepted for compatibility; publish is already dry-run-only in this milestone.
    #[arg(long)]
    pub dry_run: bool,

    /// Nix store paths that would be published later.
    #[arg(value_name = "STORE_PATH", required = true, num_args = 1..)]
    pub store_paths: Vec<PathBuf>,
}

#[derive(Debug, Args)]
#[command(
    long_about = "Validate verification inputs and render the placeholder workflow. \
The current milestone is dry-run-only, and running verify without --dry-run still stays in dry-run mode."
)]
pub struct VerifyArgs {
    /// Path to the OpenTofu-generated infrastructure manifest JSON.
    #[arg(long)]
    pub manifest: PathBuf,

    /// Path to the operator secret file.
    #[arg(long)]
    pub operator_secret: PathBuf,

    /// Path to the author secret file.
    #[arg(long)]
    pub author_secret: PathBuf,

    /// Path to the consumer secret file.
    #[arg(long)]
    pub consumer_secret: PathBuf,

    /// Path to the signing secret file.
    #[arg(long)]
    pub signing_secret: PathBuf,

    /// Path to the public key that consumers should trust.
    #[arg(long)]
    pub public_key: PathBuf,

    /// Path to the consumer configuration example to validate.
    #[arg(long)]
    pub consumer_config: PathBuf,

    /// Accepted for compatibility; verify is already dry-run-only in this milestone.
    #[arg(long)]
    pub dry_run: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap::Parser;

    use super::Cli;

    #[test]
    fn help_includes_required_subcommands() {
        let help = Cli::command().render_long_help().to_string();

        assert!(help.contains("bootstrap"));
        assert!(help.contains("publish"));
        assert!(help.contains("verify"));
    }

    #[test]
    fn publish_accepts_dry_run_flag() {
        let parsed = Cli::try_parse_from([
            "nix-cache-scaleway",
            "publish",
            "--manifest",
            "manifest.json",
            "--author-secret",
            "author.sops.yaml",
            "--signing-secret",
            "signing.sops.yaml",
            "--dry-run",
            "/nix/store/example",
        ]);

        assert!(parsed.is_ok());
    }

    #[test]
    fn verify_accepts_dry_run_flag() {
        let parsed = Cli::try_parse_from([
            "nix-cache-scaleway",
            "verify",
            "--manifest",
            "manifest.json",
            "--operator-secret",
            "operator.sops.yaml",
            "--author-secret",
            "author.sops.yaml",
            "--consumer-secret",
            "consumer.sops.yaml",
            "--signing-secret",
            "signing.sops.yaml",
            "--public-key",
            "cache.pub",
            "--consumer-config",
            "consumer.nix",
            "--dry-run",
        ]);

        assert!(parsed.is_ok());
    }

    #[test]
    fn publish_requires_store_paths() {
        let parsed = Cli::try_parse_from([
            "nix-cache-scaleway",
            "publish",
            "--manifest",
            "manifest.json",
            "--author-secret",
            "author.sops.yaml",
            "--signing-secret",
            "signing.sops.yaml",
        ]);

        assert!(parsed.is_err());
    }
}
