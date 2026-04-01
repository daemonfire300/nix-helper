use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_yaml::Value;

use crate::cli::{BootstrapArgs, PublishArgs, VerifyArgs};
use crate::contract::{
    ensure_matches, load_author_secret, load_consumer_secret, load_manifest, load_operator_secret,
    load_signing_secret, parse_author_secret, parse_consumer_secret, parse_operator_secret,
    parse_signing_secret, validate_expected_file_name, AuthorSecretDocument,
    ConsumerSecretDocument, InfrastructureManifest, OperatorSecretDocument, SigningSecretDocument,
    AUTHOR_SECRET_FILE_NAME, CONSUMER_SECRET_FILE_NAME, MANIFEST_RUNTIME_FILE_NAME,
    OPERATOR_SECRET_FILE_NAME, PUBLIC_KEY_RUNTIME_FILE_NAME, SIGNING_SECRET_FILE_NAME,
};
use crate::error::AppError;

#[derive(Debug)]
pub struct BootstrapConfig {
    pub manifest_path: PathBuf,
    pub manifest: InfrastructureManifest,
    pub operator_secret: PathBuf,
    pub author_secret: PathBuf,
    pub consumer_secret: PathBuf,
    pub signing_secret: PathBuf,
    pub public_key_out: PathBuf,
    pub author_credentials_input: Option<PathBuf>,
    pub consumer_credentials_input: Option<PathBuf>,
}

#[derive(Debug)]
pub struct PublishConfig {
    pub manifest_path: PathBuf,
    pub manifest: InfrastructureManifest,
    pub author_secret: PathBuf,
    pub author_secret_data: AuthorSecretDocument,
    pub signing_secret: PathBuf,
    pub signing_secret_data: SigningSecretDocument,
    pub store_url: String,
    pub signing_key_path: PathBuf,
    pub store_paths: Vec<PathBuf>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct VerifyConfig {
    pub manifest_path: PathBuf,
    pub manifest: InfrastructureManifest,
    pub operator_secret: PathBuf,
    pub operator_secret_data: OperatorSecretDocument,
    pub author_secret: PathBuf,
    pub author_secret_data: AuthorSecretDocument,
    pub consumer_secret: PathBuf,
    pub consumer_secret_data: ConsumerSecretDocument,
    pub signing_secret: PathBuf,
    pub signing_secret_data: SigningSecretDocument,
    pub public_key: PathBuf,
    pub consumer_config: PathBuf,
    pub store_url: String,
    pub dry_run: bool,
}

impl TryFrom<BootstrapArgs> for BootstrapConfig {
    type Error = AppError;

    fn try_from(args: BootstrapArgs) -> Result<Self, Self::Error> {
        validate_expected_file_name(&args.manifest, "manifest", MANIFEST_RUNTIME_FILE_NAME)?;
        validate_expected_file_name(
            &args.operator_secret,
            "operator secret",
            OPERATOR_SECRET_FILE_NAME,
        )?;
        validate_expected_file_name(
            &args.author_secret,
            "author secret",
            AUTHOR_SECRET_FILE_NAME,
        )?;
        validate_expected_file_name(
            &args.consumer_secret,
            "consumer secret",
            CONSUMER_SECRET_FILE_NAME,
        )?;
        validate_expected_file_name(
            &args.signing_secret,
            "signing secret",
            SIGNING_SECRET_FILE_NAME,
        )?;

        let manifest = load_manifest(&args.manifest)?;

        validate_output_target(&args.operator_secret, "operator secret")?;
        validate_output_target(&args.author_secret, "author secret")?;
        validate_output_target(&args.consumer_secret, "consumer secret")?;
        validate_output_target(&args.signing_secret, "signing secret")?;
        validate_output_target(&args.public_key_out, "public key output")?;

        if let Some(path) = &args.author_credentials_input {
            validate_required_input_file(path, "author credentials input")?;
        }

        if let Some(path) = &args.consumer_credentials_input {
            validate_required_input_file(path, "consumer credentials input")?;
        }

        Ok(Self {
            manifest_path: args.manifest,
            manifest,
            operator_secret: args.operator_secret,
            author_secret: args.author_secret,
            consumer_secret: args.consumer_secret,
            signing_secret: args.signing_secret,
            public_key_out: args.public_key_out,
            author_credentials_input: args.author_credentials_input,
            consumer_credentials_input: args.consumer_credentials_input,
        })
    }
}

impl TryFrom<PublishArgs> for PublishConfig {
    type Error = AppError;

    fn try_from(args: PublishArgs) -> Result<Self, Self::Error> {
        let PublishArgs {
            manifest,
            author_secret,
            signing_secret,
            dry_run: _,
            store_paths,
        } = args;

        validate_expected_file_name(&manifest, "manifest", MANIFEST_RUNTIME_FILE_NAME)?;
        validate_expected_file_name(&author_secret, "author secret", AUTHOR_SECRET_FILE_NAME)?;
        validate_expected_file_name(&signing_secret, "signing secret", SIGNING_SECRET_FILE_NAME)?;

        let manifest_data = load_manifest(&manifest)?;

        validate_required_input_file(&author_secret, "author secret")?;
        validate_required_input_file(&signing_secret, "signing secret")?;
        let author_secret_data = load_runtime_author_secret(&author_secret)?;
        let signing_secret_data = load_runtime_signing_secret(&signing_secret)?;

        ensure_matches(
            "author secret",
            &author_secret,
            "author.application_id",
            &author_secret_data.author.application_id,
            &manifest_data.author_application_id,
        )?;
        ensure_matches(
            "signing secret",
            &signing_secret,
            "signing.cache_name",
            &signing_secret_data.signing.cache_name,
            &manifest_data.cache_name,
        )?;
        validate_publish_manifest(&manifest, &manifest_data)?;
        let store_url = publish_store_url(
            &manifest_data.bucket_name,
            &manifest_data.endpoint,
            &manifest_data.region,
        );
        let signing_key_path = publish_signing_key_path(&manifest_data.cache_name);

        Ok(Self {
            manifest_path: manifest,
            manifest: manifest_data,
            author_secret,
            author_secret_data,
            signing_secret,
            signing_secret_data,
            store_url,
            signing_key_path,
            store_paths,
            dry_run: true,
        })
    }
}

impl TryFrom<VerifyArgs> for VerifyConfig {
    type Error = AppError;

    fn try_from(args: VerifyArgs) -> Result<Self, Self::Error> {
        let VerifyArgs {
            manifest,
            operator_secret,
            author_secret,
            consumer_secret,
            signing_secret,
            public_key,
            consumer_config,
            dry_run: _,
        } = args;

        validate_expected_file_name(&manifest, "manifest", MANIFEST_RUNTIME_FILE_NAME)?;
        validate_expected_file_name(
            &operator_secret,
            "operator secret",
            OPERATOR_SECRET_FILE_NAME,
        )?;
        validate_expected_file_name(&author_secret, "author secret", AUTHOR_SECRET_FILE_NAME)?;
        validate_expected_file_name(
            &consumer_secret,
            "consumer secret",
            CONSUMER_SECRET_FILE_NAME,
        )?;
        validate_expected_file_name(&signing_secret, "signing secret", SIGNING_SECRET_FILE_NAME)?;
        validate_expected_file_name(&public_key, "public key", PUBLIC_KEY_RUNTIME_FILE_NAME)?;

        let manifest_data = load_manifest(&manifest)?;

        validate_required_input_file(&operator_secret, "operator secret")?;
        validate_required_input_file(&author_secret, "author secret")?;
        validate_required_input_file(&consumer_secret, "consumer secret")?;
        validate_required_input_file(&signing_secret, "signing secret")?;
        validate_required_input_file(&public_key, "public key")?;
        validate_required_input_file(&consumer_config, "consumer config")?;

        let operator_secret_data = load_runtime_operator_secret(&operator_secret)?;
        let author_secret_data = load_runtime_author_secret(&author_secret)?;
        let consumer_secret_data = load_runtime_consumer_secret(&consumer_secret)?;
        let signing_secret_data = load_runtime_signing_secret(&signing_secret)?;

        ensure_matches(
            "author secret",
            &author_secret,
            "author.application_id",
            &author_secret_data.author.application_id,
            &manifest_data.author_application_id,
        )?;
        ensure_matches(
            "consumer secret",
            &consumer_secret,
            "consumer.application_id",
            &consumer_secret_data.consumer.application_id,
            &manifest_data.consumer_application_id,
        )?;
        ensure_matches(
            "signing secret",
            &signing_secret,
            "signing.cache_name",
            &signing_secret_data.signing.cache_name,
            &manifest_data.cache_name,
        )?;
        validate_publish_manifest(&manifest, &manifest_data)?;
        let store_url = publish_store_url(
            &manifest_data.bucket_name,
            &manifest_data.endpoint,
            &manifest_data.region,
        );

        Ok(Self {
            manifest_path: manifest,
            manifest: manifest_data,
            operator_secret,
            operator_secret_data,
            author_secret,
            author_secret_data,
            consumer_secret,
            consumer_secret_data,
            signing_secret,
            signing_secret_data,
            public_key,
            consumer_config,
            store_url,
            dry_run: true,
        })
    }
}

fn validate_required_input_file(path: &Path, label: &'static str) -> Result<(), AppError> {
    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::MissingRequiredFile {
                label,
                path: path.to_path_buf(),
            }
        } else {
            AppError::MetadataRead {
                label,
                path: path.to_path_buf(),
                source: error,
            }
        }
    })?;

    if !metadata.is_file() {
        return Err(AppError::WrongFileType {
            label,
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

fn load_runtime_author_secret(path: &Path) -> Result<AuthorSecretDocument, AppError> {
    load_runtime_secret_document(
        path,
        "author secret",
        parse_author_secret,
        load_author_secret,
    )
}

fn load_runtime_operator_secret(path: &Path) -> Result<OperatorSecretDocument, AppError> {
    load_runtime_secret_document(
        path,
        "operator secret",
        parse_operator_secret,
        load_operator_secret,
    )
}

fn load_runtime_consumer_secret(path: &Path) -> Result<ConsumerSecretDocument, AppError> {
    load_runtime_secret_document(
        path,
        "consumer secret",
        parse_consumer_secret,
        load_consumer_secret,
    )
}

fn load_runtime_signing_secret(path: &Path) -> Result<SigningSecretDocument, AppError> {
    load_runtime_secret_document(
        path,
        "signing secret",
        parse_signing_secret,
        load_signing_secret,
    )
}

fn load_runtime_secret_document<T>(
    path: &Path,
    kind: &'static str,
    parse: fn(&Path, &str) -> Result<T, AppError>,
    load_plain: fn(&Path) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let contents = fs::read_to_string(path).map_err(|source| AppError::SecretRead {
        kind,
        path: path.to_path_buf(),
        source,
    })?;

    if secret_uses_sops(&contents) {
        let decrypted = decrypt_sops_file(path)?;
        return parse(path, &decrypted);
    }

    load_plain(path)
}

fn secret_uses_sops(contents: &str) -> bool {
    serde_yaml::from_str::<Value>(contents)
        .ok()
        .and_then(|value| value.as_mapping().cloned())
        .map(|mapping| mapping.contains_key(Value::String("sops".to_string())))
        .unwrap_or(false)
}

fn decrypt_sops_file(path: &Path) -> Result<String, AppError> {
    let output = Command::new("sops")
        .arg("decrypt")
        .arg("--output-type")
        .arg("yaml")
        .arg(path)
        .output()
        .map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                AppError::MissingRequiredTool { tool: "sops" }
            } else {
                AppError::CommandFailed {
                    command: "sops decrypt",
                    message: source.to_string(),
                }
            }
        })?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let message = if stderr.is_empty() {
        format!("failed while decrypting {}", path.display())
    } else {
        stderr
    };

    Err(AppError::CommandFailed {
        command: "sops decrypt",
        message,
    })
}

fn validate_publish_manifest(
    manifest_path: &Path,
    manifest: &InfrastructureManifest,
) -> Result<(), AppError> {
    let expected_endpoint = scaleway_endpoint(&manifest.region);
    if manifest.endpoint != expected_endpoint {
        return Err(AppError::ManifestValidation {
            path: manifest_path.to_path_buf(),
            message: format!(
                "endpoint '{}' must match region-derived endpoint '{}'",
                manifest.endpoint, expected_endpoint
            ),
        });
    }

    Ok(())
}

fn scaleway_endpoint(region: &str) -> String {
    format!("s3.{region}.scw.cloud")
}

fn publish_store_url(bucket_name: &str, endpoint: &str, region: &str) -> String {
    format!("s3://{bucket_name}?endpoint={endpoint}&region={region}&scheme=https")
}

fn publish_signing_key_path(cache_name: &str) -> PathBuf {
    std::env::temp_dir()
        .join("nix-cache-scaleway")
        .join("publish")
        .join(format!("{cache_name}.sec"))
}

fn validate_output_target(path: &Path, label: &'static str) -> Result<(), AppError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));

    if !parent.exists() {
        return Err(AppError::InvalidOutputTarget {
            label,
            path: path.to_path_buf(),
            message: format!("parent directory does not exist: {}", parent.display()),
        });
    }

    if !parent.is_dir() {
        return Err(AppError::InvalidOutputTarget {
            label,
            path: path.to_path_buf(),
            message: format!("parent path is not a directory: {}", parent.display()),
        });
    }

    if path.exists() && !path.is_file() {
        return Err(AppError::InvalidOutputTarget {
            label,
            path: path.to_path_buf(),
            message: "target exists but is not a regular file".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::cli::{BootstrapArgs, PublishArgs, VerifyArgs};
    use crate::error::AppError;
    use crate::test_support::ENV_LOCK;

    use super::{
        publish_signing_key_path, publish_store_url, BootstrapConfig, InfrastructureManifest,
        PublishConfig, VerifyConfig,
    };

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path(name: &str) -> PathBuf {
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "nix-cache-scaleway-tests-{}-{nanos}-{nonce}",
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

    fn make_executable(path: &Path, contents: &str) {
        write_file(path, contents);
        let mut permissions = fs::metadata(path)
            .expect("script should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("script permissions should be writable");
    }

    fn manifest_json() -> &'static str {
        r#"{
  "cache_name": "nix-cache-sandbox",
  "project_id": "project-123",
  "bucket_name": "cache-bucket",
  "region": "fr-par",
  "endpoint": "s3.fr-par.scw.cloud",
  "author_application_id": "author-app",
  "consumer_application_id": "consumer-app"
}"#
    }

    fn make_manifest_file() -> PathBuf {
        let path = temp_path("infrastructure-manifest.json");
        write_file(&path, manifest_json());
        path
    }

    fn make_regular_file(name: &str) -> PathBuf {
        let path = temp_path(name);
        write_file(&path, "placeholder");
        path
    }

    fn make_output_target(name: &str) -> PathBuf {
        let path = temp_path(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be creatable");
        }
        path
    }

    fn make_operator_secret_file() -> PathBuf {
        let path = temp_path("operator.sops.yaml");
        write_file(
            &path,
            r#"schema_version: "v1"
operator:
  organization_id: "org-id"
  access_key: "access"
  secret_key: "secret"
"#,
        );
        path
    }

    fn make_author_secret_file(application_id: &str) -> PathBuf {
        let path = temp_path("author.sops.yaml");
        write_file(
            &path,
            &format!(
                "schema_version: \"v1\"\nauthor:\n  application_id: \"{application_id}\"\n  access_key: \"access\"\n  secret_key: \"secret\"\n"
            ),
        );
        path
    }

    fn make_consumer_secret_file(application_id: &str) -> PathBuf {
        let path = temp_path("consumer.sops.yaml");
        write_file(
            &path,
            &format!(
                "schema_version: \"v1\"\nconsumer:\n  application_id: \"{application_id}\"\n  access_key: \"access\"\n  secret_key: \"secret\"\n"
            ),
        );
        path
    }

    fn make_signing_secret_file(cache_name: &str) -> PathBuf {
        let path = temp_path("signing.sops.yaml");
        write_file(
            &path,
            &format!(
                "schema_version: \"v1\"\nsigning:\n  cache_name: \"{cache_name}\"\n  private_key: \"private-key\"\n"
            ),
        );
        path
    }

    fn make_fake_sops_encrypted_secret(name: &str) -> PathBuf {
        let path = temp_path(name);
        write_file(
            &path,
            r#"schema_version: ENC[AES256_GCM,data:placeholder,type:str]
sops:
  age:
    - recipient: age1testrecipient
"#,
        );
        path
    }

    fn with_fake_sops(root: &Path, script_body: &str) -> OsString {
        let bin_dir = root.join("bin");
        fs::create_dir_all(&bin_dir).expect("bin dir should be creatable");
        make_executable(&bin_dir.join("sops"), script_body);

        let original_path = std::env::var_os("PATH").unwrap_or_default();
        let mut new_path = OsString::from(bin_dir.as_os_str());
        if !original_path.is_empty() {
            new_path.push(":");
            new_path.push(original_path);
        }
        new_path
    }

    #[test]
    fn manifest_parses_successfully() {
        let manifest_path = make_manifest_file();
        let config = BootstrapConfig::try_from(BootstrapArgs {
            manifest: manifest_path,
            operator_secret: make_output_target("operator.sops.yaml"),
            author_secret: make_output_target("author.sops.yaml"),
            consumer_secret: make_output_target("consumer.sops.yaml"),
            signing_secret: make_output_target("signing.sops.yaml"),
            public_key_out: make_output_target("cache.pub"),
            author_credentials_input: None,
            consumer_credentials_input: None,
        })
        .expect("bootstrap config should parse");

        assert_eq!(
            config.manifest,
            InfrastructureManifest {
                cache_name: "nix-cache-sandbox".to_string(),
                project_id: "project-123".to_string(),
                bucket_name: "cache-bucket".to_string(),
                region: "fr-par".to_string(),
                endpoint: "s3.fr-par.scw.cloud".to_string(),
                author_application_id: "author-app".to_string(),
                consumer_application_id: "consumer-app".to_string(),
            }
        );
    }

    #[test]
    fn malformed_manifest_fails() {
        let manifest_path = temp_path("infrastructure-manifest.json");
        write_file(&manifest_path, "{ invalid json");

        let result = BootstrapConfig::try_from(BootstrapArgs {
            manifest: manifest_path.clone(),
            operator_secret: temp_path("operator.sops.yaml"),
            author_secret: temp_path("author.sops.yaml"),
            consumer_secret: temp_path("consumer.sops.yaml"),
            signing_secret: temp_path("signing.sops.yaml"),
            public_key_out: temp_path("cache.pub"),
            author_credentials_input: None,
            consumer_credentials_input: None,
        });

        assert!(matches!(
            result,
            Err(AppError::ManifestParse { path, .. }) if path == manifest_path
        ));
    }

    #[test]
    fn missing_required_manifest_key_fails() {
        let manifest_path = temp_path("infrastructure-manifest.json");
        write_file(
            &manifest_path,
            r#"{
  "cache_name": "nix-cache-sandbox",
  "project_id": "project-123",
  "bucket_name": "cache-bucket",
  "region": "fr-par",
  "endpoint": "s3.fr-par.scw.cloud",
  "author_application_id": "author-app"
}"#,
        );

        let result = BootstrapConfig::try_from(BootstrapArgs {
            manifest: manifest_path.clone(),
            operator_secret: temp_path("operator.sops.yaml"),
            author_secret: temp_path("author.sops.yaml"),
            consumer_secret: temp_path("consumer.sops.yaml"),
            signing_secret: temp_path("signing.sops.yaml"),
            public_key_out: temp_path("cache.pub"),
            author_credentials_input: None,
            consumer_credentials_input: None,
        });

        assert!(matches!(
            result,
            Err(AppError::ManifestParse { path, .. }) if path == manifest_path
        ));
    }

    #[test]
    fn empty_required_manifest_value_fails() {
        let manifest_path = temp_path("infrastructure-manifest.json");
        write_file(
            &manifest_path,
            r#"{
  "cache_name": "nix-cache-sandbox",
  "project_id": "",
  "bucket_name": "cache-bucket",
  "region": "fr-par",
  "endpoint": "s3.fr-par.scw.cloud",
  "author_application_id": "author-app",
  "consumer_application_id": "consumer-app"
}"#,
        );

        let result = BootstrapConfig::try_from(BootstrapArgs {
            manifest: manifest_path.clone(),
            operator_secret: temp_path("operator.sops.yaml"),
            author_secret: temp_path("author.sops.yaml"),
            consumer_secret: temp_path("consumer.sops.yaml"),
            signing_secret: temp_path("signing.sops.yaml"),
            public_key_out: temp_path("cache.pub"),
            author_credentials_input: None,
            consumer_credentials_input: None,
        });

        assert!(matches!(
            result,
            Err(AppError::ManifestValidation { path, .. }) if path == manifest_path
        ));
    }

    #[test]
    fn missing_manifest_file_fails() {
        let manifest_path = temp_path("infrastructure-manifest.json");

        let result = BootstrapConfig::try_from(BootstrapArgs {
            manifest: manifest_path.clone(),
            operator_secret: temp_path("operator.sops.yaml"),
            author_secret: temp_path("author.sops.yaml"),
            consumer_secret: temp_path("consumer.sops.yaml"),
            signing_secret: temp_path("signing.sops.yaml"),
            public_key_out: temp_path("cache.pub"),
            author_credentials_input: None,
            consumer_credentials_input: None,
        });

        assert!(matches!(
            result,
            Err(AppError::ManifestRead { path, .. }) if path == manifest_path
        ));
    }

    #[test]
    fn missing_publish_secret_file_fails() {
        let manifest_path = make_manifest_file();
        let missing_secret = temp_path("author.sops.yaml");

        let result = PublishConfig::try_from(PublishArgs {
            manifest: manifest_path,
            author_secret: missing_secret.clone(),
            signing_secret: make_signing_secret_file("nix-cache-sandbox"),
            dry_run: false,
            store_paths: vec![PathBuf::from("/nix/store/example")],
        });

        assert!(matches!(
            result,
            Err(AppError::MissingRequiredFile { label, path }) if label == "author secret" && path == missing_secret
        ));
    }

    #[test]
    fn directory_as_input_file_fails() {
        let manifest_path = make_manifest_file();
        let directory = temp_path("operator.sops.yaml");
        fs::create_dir_all(&directory).expect("directory should be creatable");

        let result = VerifyConfig::try_from(VerifyArgs {
            manifest: manifest_path,
            operator_secret: directory.clone(),
            author_secret: make_author_secret_file("author-app"),
            consumer_secret: make_consumer_secret_file("consumer-app"),
            signing_secret: make_signing_secret_file("nix-cache-sandbox"),
            public_key: make_regular_file("cache.pub"),
            consumer_config: make_regular_file("consumer.nix"),
            dry_run: false,
        });

        assert!(matches!(
            result,
            Err(AppError::WrongFileType { label, path }) if label == "operator secret" && path == directory
        ));
    }

    #[test]
    fn missing_bootstrap_output_parent_fails() {
        let manifest_path = make_manifest_file();
        let output_root = temp_path("missing-parent");
        let public_key_out = output_root.join("cache.pub");

        let result = BootstrapConfig::try_from(BootstrapArgs {
            manifest: manifest_path,
            operator_secret: output_root.join("operator.sops.yaml"),
            author_secret: output_root.join("author.sops.yaml"),
            consumer_secret: output_root.join("consumer.sops.yaml"),
            signing_secret: output_root.join("signing.sops.yaml"),
            public_key_out: public_key_out.clone(),
            author_credentials_input: None,
            consumer_credentials_input: None,
        });

        assert!(matches!(
            result,
            Err(AppError::InvalidOutputTarget { label, path, .. }) if label == "operator secret" && path == output_root.join("operator.sops.yaml")
        ));
    }

    #[test]
    fn publish_defaults_to_dry_run() {
        let manifest_path = make_manifest_file();
        let config = PublishConfig::try_from(PublishArgs {
            manifest: manifest_path,
            author_secret: make_author_secret_file("author-app"),
            signing_secret: make_signing_secret_file("nix-cache-sandbox"),
            dry_run: false,
            store_paths: vec![PathBuf::from("/nix/store/example")],
        })
        .expect("publish config should parse");

        assert!(config.dry_run);
    }

    #[test]
    fn verify_defaults_to_dry_run() {
        let manifest_path = make_manifest_file();
        let config = VerifyConfig::try_from(VerifyArgs {
            manifest: manifest_path,
            operator_secret: make_operator_secret_file(),
            author_secret: make_author_secret_file("author-app"),
            consumer_secret: make_consumer_secret_file("consumer-app"),
            signing_secret: make_signing_secret_file("nix-cache-sandbox"),
            public_key: make_regular_file("cache.pub"),
            consumer_config: make_regular_file("consumer.nix"),
            dry_run: false,
        })
        .expect("verify config should parse");

        assert!(config.dry_run);
    }

    #[test]
    fn verify_accepts_sops_encrypted_runtime_secrets() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = temp_path("verify-sops");
        let manifest_path = make_manifest_file();
        let fake_path = with_fake_sops(
            &root,
            r#"#!/bin/sh
case "$(basename "$4")" in
  operator.sops.yaml)
    cat <<'EOF'
schema_version: "v1"
operator:
  organization_id: "org-id"
  access_key: "access"
  secret_key: "secret"
EOF
    ;;
  author.sops.yaml)
    cat <<'EOF'
schema_version: "v1"
author:
  application_id: "author-app"
  access_key: "access"
  secret_key: "secret"
EOF
    ;;
  consumer.sops.yaml)
    cat <<'EOF'
schema_version: "v1"
consumer:
  application_id: "consumer-app"
  access_key: "access"
  secret_key: "secret"
EOF
    ;;
  signing.sops.yaml)
    cat <<'EOF'
schema_version: "v1"
signing:
  cache_name: "nix-cache-sandbox"
  private_key: "nix-cache-sandbox:private-key"
EOF
    ;;
  *)
    echo "unexpected path: $4" >&2
    exit 1
    ;;
esac
"#,
        );
        let original_path = std::env::var_os("PATH");
        std::env::set_var("PATH", &fake_path);

        let result = VerifyConfig::try_from(VerifyArgs {
            manifest: manifest_path,
            operator_secret: make_fake_sops_encrypted_secret("operator.sops.yaml"),
            author_secret: make_fake_sops_encrypted_secret("author.sops.yaml"),
            consumer_secret: make_fake_sops_encrypted_secret("consumer.sops.yaml"),
            signing_secret: make_fake_sops_encrypted_secret("signing.sops.yaml"),
            public_key: make_regular_file("cache.pub"),
            consumer_config: make_regular_file("consumer.nix"),
            dry_run: true,
        });

        match original_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        assert!(result.is_ok());
    }

    #[test]
    fn verify_reports_sops_decrypt_failures() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = temp_path("verify-sops-fail");
        let manifest_path = make_manifest_file();
        let fake_path = with_fake_sops(
            &root,
            r#"#!/bin/sh
echo "boom" >&2
exit 1
"#,
        );
        let original_path = std::env::var_os("PATH");
        std::env::set_var("PATH", &fake_path);

        let result = VerifyConfig::try_from(VerifyArgs {
            manifest: manifest_path,
            operator_secret: make_fake_sops_encrypted_secret("operator.sops.yaml"),
            author_secret: make_fake_sops_encrypted_secret("author.sops.yaml"),
            consumer_secret: make_fake_sops_encrypted_secret("consumer.sops.yaml"),
            signing_secret: make_fake_sops_encrypted_secret("signing.sops.yaml"),
            public_key: make_regular_file("cache.pub"),
            consumer_config: make_regular_file("consumer.nix"),
            dry_run: true,
        });

        match original_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        assert!(matches!(
            result,
            Err(AppError::CommandFailed { command, message })
                if command == "sops decrypt" && message.contains("boom")
        ));
    }

    #[test]
    fn mismatched_author_application_id_fails() {
        let manifest_path = make_manifest_file();
        let author_secret = make_author_secret_file("other-author-app");

        let result = PublishConfig::try_from(PublishArgs {
            manifest: manifest_path,
            author_secret: author_secret.clone(),
            signing_secret: make_signing_secret_file("nix-cache-sandbox"),
            dry_run: true,
            store_paths: vec![PathBuf::from("/nix/store/example")],
        });

        assert!(matches!(
            result,
            Err(AppError::SecretValidation { kind, path, .. })
                if kind == "author secret" && path == author_secret
        ));
    }

    #[test]
    fn publish_derives_store_url_and_signing_key_path() {
        let manifest_path = make_manifest_file();
        let config = PublishConfig::try_from(PublishArgs {
            manifest: manifest_path,
            author_secret: make_author_secret_file("author-app"),
            signing_secret: make_signing_secret_file("nix-cache-sandbox"),
            dry_run: true,
            store_paths: vec![PathBuf::from("/nix/store/example")],
        })
        .expect("publish config should parse");

        assert_eq!(
            config.store_url,
            publish_store_url("cache-bucket", "s3.fr-par.scw.cloud", "fr-par")
        );
        assert_eq!(
            config.signing_key_path,
            publish_signing_key_path("nix-cache-sandbox")
        );
    }

    #[test]
    fn publish_rejects_endpoint_that_does_not_match_region() {
        let manifest_path = temp_path("infrastructure-manifest.json");
        write_file(
            &manifest_path,
            r#"{
  "cache_name": "nix-cache-sandbox",
  "project_id": "project-123",
  "bucket_name": "cache-bucket",
  "region": "fr-par",
  "endpoint": "s3.nl-ams.scw.cloud",
  "author_application_id": "author-app",
  "consumer_application_id": "consumer-app"
}"#,
        );

        let result = PublishConfig::try_from(PublishArgs {
            manifest: manifest_path.clone(),
            author_secret: make_author_secret_file("author-app"),
            signing_secret: make_signing_secret_file("nix-cache-sandbox"),
            dry_run: true,
            store_paths: vec![PathBuf::from("/nix/store/example")],
        });

        assert!(matches!(
            result,
            Err(AppError::ManifestValidation { path, .. }) if path == manifest_path
        ));
    }
}
