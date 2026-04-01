use std::path::PathBuf;

use crate::config::PublishConfig;
use crate::error::AppError;

pub fn run(config: &PublishConfig) -> Result<(), AppError> {
    let plan = PublishPlan::from_config(config);

    println!("publish dry-run: live execution remains disabled in this milestone");
    println!("dry_run: {}", config.dry_run);
    println!("manifest: {}", config.manifest_path.display());
    println!("cache_name: {}", config.manifest.cache_name);
    println!("bucket_name: {}", config.manifest.bucket_name);
    println!("region: {}", config.manifest.region);
    println!("endpoint: {}", config.manifest.endpoint);
    println!("store_url: {}", plan.store_url);
    println!("author_secret: {}", config.author_secret.display());
    println!(
        "author_application_id: {}",
        config.author_secret_data.author.application_id
    );
    println!("signing_secret: {}", config.signing_secret.display());
    println!(
        "signing_cache_name: {}",
        config.signing_secret_data.signing.cache_name
    );
    println!("signing_key_path: {}", plan.signing_key_path.display());
    println!(
        "publish_environment: AWS_ACCESS_KEY_ID <- {}#author.access_key",
        config.author_secret.display()
    );
    println!(
        "publish_environment: AWS_SECRET_ACCESS_KEY <- {}#author.secret_key",
        config.author_secret.display()
    );
    println!(
        "publish_environment: AWS_REGION <- {}#region",
        config.manifest_path.display()
    );
    println!(
        "publish_environment: NIX secret-key-files <- {}",
        plan.signing_key_path.display()
    );
    println!("nix_copy_command: {}", plan.render_command());
    println!("store_paths:");
    for store_path in &config.store_paths {
        println!("  {}", store_path.display());
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishPlan {
    store_url: String,
    signing_key_path: PathBuf,
    command: Vec<String>,
}

impl PublishPlan {
    fn from_config(config: &PublishConfig) -> Self {
        let mut command = Vec::with_capacity(config.store_paths.len() + 6);
        command.push("nix".to_string());
        command.push("copy".to_string());
        command.push("--option".to_string());
        command.push("secret-key-files".to_string());
        command.push(config.signing_key_path.display().to_string());
        command.push("--to".to_string());
        command.push(config.store_url.clone());
        command.extend(
            config
                .store_paths
                .iter()
                .map(|store_path| store_path.display().to_string()),
        );

        Self {
            store_url: config.store_url.clone(),
            signing_key_path: config.signing_key_path.clone(),
            command,
        }
    }

    fn render_command(&self) -> String {
        self.command
            .iter()
            .map(|arg| shell_quote(arg))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn shell_quote(arg: &str) -> String {
    if !needs_shell_quote(arg) {
        return arg.to_string();
    }

    format!("'{}'", arg.replace('\'', "'\"'\"'"))
}

fn needs_shell_quote(arg: &str) -> bool {
    arg.is_empty()
        || arg
            .chars()
            .any(|character| !matches!(character, 'A'..='Z' | 'a'..='z' | '0'..='9' | '/' | '.' | '_' | '-' | ':' | '=' | '?'
                | '&'))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::PublishConfig;
    use crate::contract::{
        AuthorSecret, AuthorSecretDocument, InfrastructureManifest, SigningSecret,
        SigningSecretDocument,
    };

    use super::{run, PublishPlan};

    fn publish_config() -> PublishConfig {
        PublishConfig {
            manifest_path: PathBuf::from("runtime/infrastructure-manifest.json"),
            manifest: InfrastructureManifest {
                cache_name: "nix-cache-sandbox".to_string(),
                project_id: "project-123".to_string(),
                bucket_name: "cache-bucket".to_string(),
                region: "fr-par".to_string(),
                endpoint: "s3.fr-par.scw.cloud".to_string(),
                author_application_id: "author-app".to_string(),
                consumer_application_id: "consumer-app".to_string(),
            },
            author_secret: PathBuf::from("runtime/secrets/author.sops.yaml"),
            author_secret_data: AuthorSecretDocument {
                schema_version: "v1".to_string(),
                author: AuthorSecret {
                    application_id: "author-app".to_string(),
                    access_key: "access".to_string(),
                    secret_key: "secret".to_string(),
                },
            },
            signing_secret: PathBuf::from("runtime/secrets/signing.sops.yaml"),
            signing_secret_data: SigningSecretDocument {
                schema_version: "v1".to_string(),
                signing: SigningSecret {
                    cache_name: "nix-cache-sandbox".to_string(),
                    private_key: "private-key".to_string(),
                },
            },
            store_url: "s3://cache-bucket?endpoint=s3.fr-par.scw.cloud&region=fr-par&scheme=https"
                .to_string(),
            signing_key_path: std::env::temp_dir()
                .join("nix-cache-scaleway")
                .join("publish")
                .join("nix-cache-sandbox.sec"),
            store_paths: vec![PathBuf::from("/nix/store/example")],
            dry_run: true,
        }
    }

    #[test]
    fn publish_command_renders_expected_nix_copy_invocation() {
        let config = publish_config();
        let plan = PublishPlan::from_config(&config);

        assert_eq!(
            plan.render_command(),
            format!(
                "nix copy --option secret-key-files {} --to {} /nix/store/example",
                plan.signing_key_path.display(),
                plan.store_url
            )
        );
    }

    #[test]
    fn publish_command_shell_quotes_special_arguments() {
        let mut config = publish_config();
        config.store_paths = vec![PathBuf::from("/nix/store/with space")];
        let plan = PublishPlan::from_config(&config);

        assert!(plan.render_command().contains("'/nix/store/with space'"));
    }

    #[test]
    fn publish_workflow_succeeds() {
        let result = run(&publish_config());

        assert!(result.is_ok());
    }
}
