use std::fs;
use std::path::Path;

use crate::config::VerifyConfig;
use crate::error::AppError;

pub fn run(config: &VerifyConfig) -> Result<(), AppError> {
    let public_key = read_trimmed_file(&config.public_key, "public key")?;
    let public_key_name = parse_cache_key_name(&config.public_key, "public key", &public_key)?;
    let private_key_name = parse_cache_key_name(
        &config.signing_secret,
        "signing private key",
        &config.signing_secret_data.signing.private_key,
    )?;

    ensure_value_matches(
        "public key",
        &config.public_key,
        "cache key name",
        &public_key_name,
        &config.manifest.cache_name,
    )?;
    ensure_value_matches(
        "signing secret",
        &config.signing_secret,
        "signing.cache_name",
        &config.signing_secret_data.signing.cache_name,
        &config.manifest.cache_name,
    )?;
    ensure_value_matches(
        "signing secret",
        &config.signing_secret,
        "signing private key name",
        &private_key_name,
        &config.manifest.cache_name,
    )?;

    let consumer_config = read_trimmed_file(&config.consumer_config, "consumer config")?;
    validate_consumer_config(
        &config.consumer_config,
        &consumer_config,
        &config.store_url,
        &public_key,
    )?;

    println!("verify dry-run: offline validation completed");
    println!("dry_run: {}", config.dry_run);
    println!("manifest: {}", config.manifest_path.display());
    println!("cache_name: {}", config.manifest.cache_name);
    println!("project_id: {}", config.manifest.project_id);
    println!("bucket_name: {}", config.manifest.bucket_name);
    println!("region: {}", config.manifest.region);
    println!("endpoint: {}", config.manifest.endpoint);
    println!("store_url: {}", config.store_url);
    println!("operator_secret: {}", config.operator_secret.display());
    println!(
        "operator_secret_schema: {}",
        config.operator_secret_data.schema_version
    );
    println!("author_secret: {}", config.author_secret.display());
    println!(
        "author_application_id: {}",
        config.author_secret_data.author.application_id
    );
    println!("consumer_secret: {}", config.consumer_secret.display());
    println!(
        "consumer_application_id: {}",
        config.consumer_secret_data.consumer.application_id
    );
    println!("signing_secret: {}", config.signing_secret.display());
    println!(
        "signing_cache_name: {}",
        config.signing_secret_data.signing.cache_name
    );
    println!("public_key: {}", config.public_key.display());
    println!("public_key_name: {public_key_name}");
    println!("consumer_config: {}", config.consumer_config.display());

    Ok(())
}

fn read_trimmed_file(path: &Path, label: &'static str) -> Result<String, AppError> {
    let contents = fs::read_to_string(path).map_err(|source| AppError::FileRead {
        label,
        path: path.to_path_buf(),
        source,
    })?;
    let trimmed = contents.trim();

    if trimmed.is_empty() {
        return Err(AppError::InputValidation {
            label,
            path: path.to_path_buf(),
            message: "file must not be empty".to_string(),
        });
    }

    Ok(trimmed.to_string())
}

fn parse_cache_key_name(
    path: &Path,
    label: &'static str,
    key_value: &str,
) -> Result<String, AppError> {
    let Some((name, encoded_key)) = key_value.trim().split_once(':') else {
        return Err(AppError::InputValidation {
            label,
            path: path.to_path_buf(),
            message: "expected '<cache-name>:<encoded-key>'".to_string(),
        });
    };

    if name.trim().is_empty() || encoded_key.trim().is_empty() {
        return Err(AppError::InputValidation {
            label,
            path: path.to_path_buf(),
            message: "cache key name and encoded payload must both be non-empty".to_string(),
        });
    }

    Ok(name.trim().to_string())
}

fn ensure_value_matches(
    label: &'static str,
    path: &Path,
    field: &'static str,
    actual: &str,
    expected: &str,
) -> Result<(), AppError> {
    if actual == expected {
        return Ok(());
    }

    Err(AppError::InputValidation {
        label,
        path: path.to_path_buf(),
        message: format!("field '{field}' must match '{expected}'"),
    })
}

fn validate_consumer_config(
    path: &Path,
    contents: &str,
    expected_store_url: &str,
    expected_public_key: &str,
) -> Result<(), AppError> {
    let active_contents = contents
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    if !contains_any(&active_contents, &["extra-substituters", "substituters"]) {
        return Err(AppError::InputValidation {
            label: "consumer config",
            path: path.to_path_buf(),
            message: "must define a substituter list".to_string(),
        });
    }

    if !active_contents.contains(expected_store_url) {
        return Err(AppError::InputValidation {
            label: "consumer config",
            path: path.to_path_buf(),
            message: format!(
                "must include the Scaleway substituter URL '{}'",
                expected_store_url
            ),
        });
    }

    if !contains_any(
        &active_contents,
        &["extra-trusted-public-keys", "trusted-public-keys"],
    ) {
        return Err(AppError::InputValidation {
            label: "consumer config",
            path: path.to_path_buf(),
            message: "must define a trusted public key list".to_string(),
        });
    }

    if !active_contents.contains(expected_public_key) {
        return Err(AppError::InputValidation {
            label: "consumer config",
            path: path.to_path_buf(),
            message: "must include the exact trusted public key from cache.pub".to_string(),
        });
    }

    Ok(())
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::VerifyConfig;
    use crate::contract::{
        AuthorSecret, AuthorSecretDocument, ConsumerSecret, ConsumerSecretDocument,
        InfrastructureManifest, OperatorSecret, OperatorSecretDocument, SigningSecret,
        SigningSecretDocument,
    };
    use crate::error::AppError;

    use super::run;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path(name: &str) -> PathBuf {
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "nix-cache-scaleway-verify-tests-{}-{nanos}-{nonce}",
                std::process::id()
            ))
            .join(name)
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be creatable");
        }
        fs::write(path, contents).expect("file should be writable");
    }

    fn verify_config(root: &Path) -> VerifyConfig {
        let public_key = root.join("runtime/public/cache.pub");
        let consumer_config = root.join("examples/consumer-config.nix");

        write_file(&public_key, "nix-cache-sandbox:fake-public-key\n");
        write_file(
            &consumer_config,
            r#"{
  nix.settings = {
    extra-substituters = [
      "s3://cache-bucket?endpoint=s3.fr-par.scw.cloud&region=fr-par&scheme=https"
    ];
    extra-trusted-public-keys = [
      "nix-cache-sandbox:fake-public-key"
    ];
  };
}
"#,
        );

        VerifyConfig {
            manifest_path: root.join("runtime/infrastructure-manifest.json"),
            manifest: InfrastructureManifest {
                cache_name: "nix-cache-sandbox".to_string(),
                project_id: "project-123".to_string(),
                bucket_name: "cache-bucket".to_string(),
                region: "fr-par".to_string(),
                endpoint: "s3.fr-par.scw.cloud".to_string(),
                author_application_id: "author-app".to_string(),
                consumer_application_id: "consumer-app".to_string(),
            },
            operator_secret: root.join("runtime/secrets/operator.sops.yaml"),
            operator_secret_data: OperatorSecretDocument {
                schema_version: "v1".to_string(),
                operator: OperatorSecret {
                    organization_id: "org-id".to_string(),
                    access_key: "access".to_string(),
                    secret_key: "secret".to_string(),
                },
            },
            author_secret: root.join("runtime/secrets/author.sops.yaml"),
            author_secret_data: AuthorSecretDocument {
                schema_version: "v1".to_string(),
                author: AuthorSecret {
                    application_id: "author-app".to_string(),
                    access_key: "access".to_string(),
                    secret_key: "secret".to_string(),
                },
            },
            consumer_secret: root.join("runtime/secrets/consumer.sops.yaml"),
            consumer_secret_data: ConsumerSecretDocument {
                schema_version: "v1".to_string(),
                consumer: ConsumerSecret {
                    application_id: "consumer-app".to_string(),
                    access_key: "access".to_string(),
                    secret_key: "secret".to_string(),
                },
            },
            signing_secret: root.join("runtime/secrets/signing.sops.yaml"),
            signing_secret_data: SigningSecretDocument {
                schema_version: "v1".to_string(),
                signing: SigningSecret {
                    cache_name: "nix-cache-sandbox".to_string(),
                    private_key: "nix-cache-sandbox:fake-secret-key".to_string(),
                },
            },
            public_key,
            consumer_config,
            store_url: "s3://cache-bucket?endpoint=s3.fr-par.scw.cloud&region=fr-par&scheme=https"
                .to_string(),
            dry_run: true,
        }
    }

    #[test]
    fn verify_validation_succeeds() {
        let root = temp_path("success");
        let result = run(&verify_config(&root));

        assert!(result.is_ok());
    }

    #[test]
    fn verify_rejects_mismatched_public_key_name() {
        let root = temp_path("mismatched-public-key");
        let config = verify_config(&root);
        write_file(&config.public_key, "other-cache:fake-public-key\n");

        let result = run(&config);

        assert!(matches!(
            result,
            Err(AppError::InputValidation { label, path, .. })
                if label == "public key" && path == config.public_key
        ));
    }

    #[test]
    fn verify_rejects_missing_substituter_entry() {
        let root = temp_path("missing-substituter");
        let config = verify_config(&root);
        write_file(
            &config.consumer_config,
            r#"{
  nix.settings = {
    extra-trusted-public-keys = [
      "nix-cache-sandbox:fake-public-key"
    ];
  };
}
"#,
        );

        let result = run(&config);

        assert!(matches!(
            result,
            Err(AppError::InputValidation { label, path, .. })
                if label == "consumer config" && path == config.consumer_config
        ));
    }

    #[test]
    fn verify_rejects_missing_public_key_entry() {
        let root = temp_path("missing-public-key");
        let config = verify_config(&root);
        write_file(
            &config.consumer_config,
            r#"{
  nix.settings = {
    extra-substituters = [
      "s3://cache-bucket?endpoint=s3.fr-par.scw.cloud&region=fr-par&scheme=https"
    ];
    extra-trusted-public-keys = [];
  };
}
"#,
        );

        let result = run(&config);

        assert!(matches!(
            result,
            Err(AppError::InputValidation { label, path, .. })
                if label == "consumer config" && path == config.consumer_config
        ));
    }
}
