use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde_yaml::Value;

use crate::config::BootstrapConfig;
use crate::error::AppError;

const SCHEMA_VERSION: &str = "v1";
const OPERATOR_ORGANIZATION_ID_PLACEHOLDER: &str = "SCW_ORGANIZATION_ID_PLACEHOLDER";
const OPERATOR_ACCESS_KEY_PLACEHOLDER: &str = "SCW_OPERATOR_ACCESS_KEY_PLACEHOLDER";
const OPERATOR_SECRET_KEY_PLACEHOLDER: &str = "SCW_OPERATOR_SECRET_KEY_PLACEHOLDER";
const AUTHOR_ACCESS_KEY_PLACEHOLDER: &str = "SCW_AUTHOR_ACCESS_KEY_PLACEHOLDER";
const AUTHOR_SECRET_KEY_PLACEHOLDER: &str = "SCW_AUTHOR_SECRET_KEY_PLACEHOLDER";
const CONSUMER_ACCESS_KEY_PLACEHOLDER: &str = "SCW_CONSUMER_ACCESS_KEY_PLACEHOLDER";
const CONSUMER_SECRET_KEY_PLACEHOLDER: &str = "SCW_CONSUMER_SECRET_KEY_PLACEHOLDER";

static UNIQUE_ID: AtomicU64 = AtomicU64::new(0);

pub fn run(config: &BootstrapConfig) -> Result<(), AppError> {
    validate_output_absent(&config.operator_secret, "operator secret")?;
    validate_output_absent(&config.author_secret, "author secret")?;
    validate_output_absent(&config.consumer_secret, "consumer secret")?;
    validate_output_absent(&config.signing_secret, "signing secret")?;
    validate_output_absent(&config.public_key_out, "public key output")?;

    let author_credentials = load_role_credentials(
        config.author_credentials_input.as_deref(),
        "author credentials input",
        "author",
        &config.manifest.author_application_id,
        AUTHOR_ACCESS_KEY_PLACEHOLDER,
        AUTHOR_SECRET_KEY_PLACEHOLDER,
    )?;
    let consumer_credentials = load_role_credentials(
        config.consumer_credentials_input.as_deref(),
        "consumer credentials input",
        "consumer",
        &config.manifest.consumer_application_id,
        CONSUMER_ACCESS_KEY_PLACEHOLDER,
        CONSUMER_SECRET_KEY_PLACEHOLDER,
    )?;

    let sops_config_path = find_sops_config(&config.signing_secret)?;
    validate_sops_recipients(&sops_config_path)?;
    let project_root = sops_config_path
        .parent()
        .expect(".sops.yaml should always have a parent directory");

    let temp_workspace = TemporaryWorkspace::new()?;
    let raw_private_key_path = temp_workspace.path().join("cache-priv-key");
    let raw_public_key_path = temp_workspace.path().join("cache-pub-key");

    generate_binary_cache_key(
        &config.manifest.cache_name,
        &raw_private_key_path,
        &raw_public_key_path,
    )?;

    let private_key = load_generated_key(&raw_private_key_path, "private signing key")?;
    let public_key = load_generated_key(&raw_public_key_path, "public signing key")?;

    let public_key_temp = write_temp_output(
        &config.public_key_out,
        "public key output",
        format!("{public_key}\n").as_bytes(),
    )?;
    let operator_secret_temp = encrypt_secret_document(
        project_root,
        &config.operator_secret,
        "operator secret",
        &render_operator_secret_yaml(),
    )?;
    let author_secret_temp = encrypt_secret_document(
        project_root,
        &config.author_secret,
        "author secret",
        &render_role_secret_yaml(
            "author",
            &author_credentials.application_id,
            &author_credentials.access_key,
            &author_credentials.secret_key,
        ),
    )?;
    let consumer_secret_temp = encrypt_secret_document(
        project_root,
        &config.consumer_secret,
        "consumer secret",
        &render_role_secret_yaml(
            "consumer",
            &consumer_credentials.application_id,
            &consumer_credentials.access_key,
            &consumer_credentials.secret_key,
        ),
    )?;
    let signing_secret_temp = encrypt_secret_document(
        project_root,
        &config.signing_secret,
        "signing secret",
        &render_signing_secret_yaml(&config.manifest.cache_name, &private_key),
    )?;

    persist_output(
        &public_key_temp,
        &config.public_key_out,
        "public key output",
    )?;
    persist_output(
        &operator_secret_temp,
        &config.operator_secret,
        "operator secret",
    )?;
    persist_output(&author_secret_temp, &config.author_secret, "author secret")?;
    persist_output(
        &consumer_secret_temp,
        &config.consumer_secret,
        "consumer secret",
    )?;
    persist_output(
        &signing_secret_temp,
        &config.signing_secret,
        "signing secret",
    )?;

    println!("bootstrap completed without contacting Scaleway");
    println!("manifest: {}", config.manifest_path.display());
    println!("cache_name: {}", config.manifest.cache_name);
    println!("public_key_out: {}", config.public_key_out.display());
    println!("operator_secret: {}", config.operator_secret.display());
    println!("author_secret: {}", config.author_secret.display());
    println!("consumer_secret: {}", config.consumer_secret.display());
    println!("signing_secret: {}", config.signing_secret.display());

    if let Some(path) = &config.author_credentials_input {
        println!("author_credentials_input: {}", path.display());
    } else {
        println!("author_credentials_input: placeholder values written");
    }

    if let Some(path) = &config.consumer_credentials_input {
        println!("consumer_credentials_input: {}", path.display());
    } else {
        println!("consumer_credentials_input: placeholder values written");
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoleCredentials {
    application_id: String,
    access_key: String,
    secret_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ManualCredentialInput {
    Flat(FlatCredentialInput),
    WrappedAuthor {
        author: FlatCredentialInput,
        #[allow(dead_code)]
        schema_version: Option<String>,
    },
    WrappedConsumer {
        consumer: FlatCredentialInput,
        #[allow(dead_code)]
        schema_version: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct FlatCredentialInput {
    application_id: Option<String>,
    access_key: String,
    secret_key: String,
}

struct TemporaryWorkspace {
    path: PathBuf,
}

impl TemporaryWorkspace {
    fn new() -> Result<Self, AppError> {
        let path =
            std::env::temp_dir().join(format!("nix-cache-scaleway-bootstrap-{}", unique_id()));
        fs::create_dir_all(&path)
            .map_err(|source| AppError::TemporaryWorkspaceCreate { source })?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn validate_output_absent(path: &Path, label: &'static str) -> Result<(), AppError> {
    if path.exists() {
        return Err(AppError::OutputConflict {
            label,
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

fn find_sops_config(secret_path: &Path) -> Result<PathBuf, AppError> {
    let start = secret_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    for ancestor in start.ancestors() {
        let candidate = ancestor.join(".sops.yaml");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(AppError::SopsConfigNotFound { start })
}

fn validate_sops_recipients(path: &Path) -> Result<(), AppError> {
    let contents = fs::read_to_string(path).map_err(|source| AppError::FileRead {
        label: ".sops.yaml",
        path: path.to_path_buf(),
        source,
    })?;
    let value: Value =
        serde_yaml::from_str(&contents).map_err(|source| AppError::CredentialInputParse {
            label: ".sops.yaml",
            path: path.to_path_buf(),
            source,
        })?;

    if sops_value_has_recipient(&value) {
        return Ok(());
    }

    Err(AppError::SopsRecipientsMissing {
        path: path.to_path_buf(),
    })
}

fn sops_value_has_recipient(value: &Value) -> bool {
    match value {
        Value::Mapping(mapping) => mapping.iter().any(|(key, value)| {
            let key = key.as_str().unwrap_or_default();
            match key {
                "age" | "pgp" | "kms" | "gcp_kms" => value.as_sequence().is_some_and(|items| {
                    items
                        .iter()
                        .any(|item| item.as_str().is_some_and(|entry| !entry.trim().is_empty()))
                }),
                "hc_vault_transit_uri" => {
                    value.as_str().is_some_and(|entry| !entry.trim().is_empty())
                }
                "azure_keyvault" => value.as_sequence().is_some_and(|items| !items.is_empty()),
                _ => sops_value_has_recipient(value),
            }
        }),
        Value::Sequence(values) => values.iter().any(sops_value_has_recipient),
        _ => false,
    }
}

fn generate_binary_cache_key(
    cache_name: &str,
    private_key_path: &Path,
    public_key_path: &Path,
) -> Result<(), AppError> {
    let output = Command::new("nix-store")
        .arg("--generate-binary-cache-key")
        .arg(cache_name)
        .arg(private_key_path)
        .arg(public_key_path)
        .output()
        .map_err(|source| map_command_spawn_error("nix-store", source))?;

    if output.status.success() {
        return Ok(());
    }

    Err(AppError::CommandFailed {
        command: "nix-store --generate-binary-cache-key",
        message: command_message(&output.stderr, output.status.code()),
    })
}

fn load_generated_key(path: &Path, label: &'static str) -> Result<String, AppError> {
    let contents = fs::read_to_string(path).map_err(|source| AppError::FileRead {
        label,
        path: path.to_path_buf(),
        source,
    })?;
    let trimmed = contents.trim();

    if trimmed.is_empty() {
        return Err(AppError::OutputValidation {
            label,
            path: path.to_path_buf(),
            message: "generated file must not be empty".to_string(),
        });
    }

    Ok(trimmed.to_string())
}

fn load_role_credentials(
    input_path: Option<&Path>,
    input_label: &'static str,
    role: &'static str,
    expected_application_id: &str,
    placeholder_access_key: &'static str,
    placeholder_secret_key: &'static str,
) -> Result<RoleCredentials, AppError> {
    match input_path {
        Some(path) => {
            load_manual_role_credentials(path, input_label, role, expected_application_id)
        }
        None => Ok(RoleCredentials {
            application_id: expected_application_id.to_string(),
            access_key: placeholder_access_key.to_string(),
            secret_key: placeholder_secret_key.to_string(),
        }),
    }
}

fn load_manual_role_credentials(
    path: &Path,
    input_label: &'static str,
    role: &'static str,
    expected_application_id: &str,
) -> Result<RoleCredentials, AppError> {
    let contents = fs::read_to_string(path).map_err(|source| AppError::CredentialInputRead {
        label: input_label,
        path: path.to_path_buf(),
        source,
    })?;
    let parsed: ManualCredentialInput =
        serde_yaml::from_str(&contents).map_err(|source| AppError::CredentialInputParse {
            label: input_label,
            path: path.to_path_buf(),
            source,
        })?;

    let flat = match parsed {
        ManualCredentialInput::Flat(flat) => flat,
        ManualCredentialInput::WrappedAuthor { author, .. } => {
            if role != "author" {
                return Err(AppError::CredentialInputValidation {
                    label: input_label,
                    path: path.to_path_buf(),
                    message: "wrapped author credentials were provided for a consumer input"
                        .to_string(),
                });
            }
            author
        }
        ManualCredentialInput::WrappedConsumer { consumer, .. } => {
            if role != "consumer" {
                return Err(AppError::CredentialInputValidation {
                    label: input_label,
                    path: path.to_path_buf(),
                    message: "wrapped consumer credentials were provided for an author input"
                        .to_string(),
                });
            }
            consumer
        }
    };

    if flat.access_key.trim().is_empty() {
        return Err(AppError::CredentialInputValidation {
            label: input_label,
            path: path.to_path_buf(),
            message: "required field 'access_key' must not be empty".to_string(),
        });
    }

    if flat.secret_key.trim().is_empty() {
        return Err(AppError::CredentialInputValidation {
            label: input_label,
            path: path.to_path_buf(),
            message: "required field 'secret_key' must not be empty".to_string(),
        });
    }

    let application_id = flat
        .application_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| expected_application_id.to_string());

    if application_id != expected_application_id {
        return Err(AppError::CredentialInputValidation {
            label: input_label,
            path: path.to_path_buf(),
            message: format!(
                "application_id '{application_id}' must match manifest value '{expected_application_id}'"
            ),
        });
    }

    Ok(RoleCredentials {
        application_id,
        access_key: flat.access_key.trim().to_string(),
        secret_key: flat.secret_key.trim().to_string(),
    })
}

fn encrypt_secret_document(
    project_root: &Path,
    target_path: &Path,
    label: &'static str,
    plaintext_yaml: &str,
) -> Result<PathBuf, AppError> {
    let mut child = Command::new("sops")
        .current_dir(project_root)
        .arg("encrypt")
        .arg("--filename-override")
        .arg(target_path)
        .arg("--input-type")
        .arg("yaml")
        .arg("--output-type")
        .arg("yaml")
        .arg("/dev/stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| map_command_spawn_error("sops", source))?;

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(plaintext_yaml.as_bytes())
        .map_err(|source| AppError::CommandFailed {
            command: "sops encrypt",
            message: format!("failed to write plaintext to sops stdin: {source}"),
        })?;

    let output = child
        .wait_with_output()
        .map_err(|source| AppError::CommandFailed {
            command: "sops encrypt",
            message: format!("failed to wait for sops: {source}"),
        })?;

    if !output.status.success() {
        return Err(AppError::CommandFailed {
            command: "sops encrypt",
            message: command_message(&output.stderr, output.status.code()),
        });
    }

    write_temp_output(target_path, label, &output.stdout)
}

fn write_temp_output(
    target_path: &Path,
    label: &'static str,
    contents: &[u8],
) -> Result<PathBuf, AppError> {
    let temporary_path = temp_output_path(target_path);
    fs::write(&temporary_path, contents).map_err(|source| AppError::FileWrite {
        label,
        path: temporary_path.clone(),
        source,
    })?;
    Ok(temporary_path)
}

fn persist_output(
    temporary_path: &Path,
    destination_path: &Path,
    label: &'static str,
) -> Result<(), AppError> {
    fs::rename(temporary_path, destination_path).map_err(|source| AppError::FilePersist {
        label,
        path: destination_path.to_path_buf(),
        source,
    })
}

fn temp_output_path(target_path: &Path) -> PathBuf {
    let file_name = target_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    target_path.with_file_name(format!(".{file_name}.tmp-{}", unique_id()))
}

fn unique_id() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos() as u64;
    nanos ^ UNIQUE_ID.fetch_add(1, Ordering::Relaxed)
}

fn map_command_spawn_error(command: &'static str, source: std::io::Error) -> AppError {
    if source.kind() == std::io::ErrorKind::NotFound {
        AppError::MissingRequiredTool { tool: command }
    } else {
        AppError::CommandFailed {
            command,
            message: source.to_string(),
        }
    }
}

fn command_message(stderr: &[u8], exit_code: Option<i32>) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if stderr.is_empty() {
        match exit_code {
            Some(code) => format!("process exited with status code {code}"),
            None => "process terminated without an exit code".to_string(),
        }
    } else {
        stderr
    }
}

fn render_operator_secret_yaml() -> String {
    format!(
        "schema_version: '{}'\noperator:\n  organization_id: '{}'\n  access_key: '{}'\n  secret_key: '{}'\n",
        escape_yaml_scalar(SCHEMA_VERSION),
        escape_yaml_scalar(OPERATOR_ORGANIZATION_ID_PLACEHOLDER),
        escape_yaml_scalar(OPERATOR_ACCESS_KEY_PLACEHOLDER),
        escape_yaml_scalar(OPERATOR_SECRET_KEY_PLACEHOLDER),
    )
}

fn render_role_secret_yaml(
    role: &'static str,
    application_id: &str,
    access_key: &str,
    secret_key: &str,
) -> String {
    format!(
        "schema_version: '{}'\n{role}:\n  application_id: '{}'\n  access_key: '{}'\n  secret_key: '{}'\n",
        escape_yaml_scalar(SCHEMA_VERSION),
        escape_yaml_scalar(application_id),
        escape_yaml_scalar(access_key),
        escape_yaml_scalar(secret_key),
    )
}

fn render_signing_secret_yaml(cache_name: &str, private_key: &str) -> String {
    format!(
        "schema_version: '{}'\nsigning:\n  cache_name: '{}'\n  private_key: '{}'\n",
        escape_yaml_scalar(SCHEMA_VERSION),
        escape_yaml_scalar(cache_name),
        escape_yaml_scalar(private_key),
    )
}

fn escape_yaml_scalar(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::BootstrapConfig;
    use crate::contract::{
        load_author_secret, load_consumer_secret, load_operator_secret, load_signing_secret,
        InfrastructureManifest,
    };
    use crate::test_support::ENV_LOCK;

    use super::run;
    use super::AppError;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path(name: &str) -> PathBuf {
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "nix-cache-scaleway-bootstrap-tests-{}-{nanos}-{nonce}",
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

    fn with_fake_tools(root: &Path) -> OsString {
        let bin_dir = root.join("bin");
        fs::create_dir_all(&bin_dir).expect("bin dir should be creatable");

        let nix_store_script = r#"#!/bin/sh
secret_path="$3"
public_path="$4"
printf "%s:%s\n" "$2" "fake-secret-key" > "$secret_path"
printf "%s:%s\n" "$2" "fake-public-key" > "$public_path"
"#;
        make_executable(&bin_dir.join("nix-store"), nix_store_script);

        let sops_script = format!(
            r##"#!/bin/sh
log_path="{}"
filename_override=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--filename-override" ]; then
    shift
    filename_override="$1"
  fi
  shift
done
printf "%s\n" "$filename_override" >> "$log_path"
printf "# fake-sops\n"
cat
"##,
            root.join("sops.log").display()
        );
        make_executable(&bin_dir.join("sops"), &sops_script);

        let original_path = std::env::var_os("PATH").unwrap_or_default();
        let mut new_path = OsString::from(bin_dir.as_os_str());
        if !original_path.is_empty() {
            new_path.push(":");
            new_path.push(original_path);
        }
        new_path
    }

    fn manifest() -> InfrastructureManifest {
        InfrastructureManifest {
            cache_name: "nix-cache-sandbox".to_string(),
            project_id: "project-123".to_string(),
            bucket_name: "cache-bucket".to_string(),
            region: "fr-par".to_string(),
            endpoint: "s3.fr-par.scw.cloud".to_string(),
            author_application_id: "author-app".to_string(),
            consumer_application_id: "consumer-app".to_string(),
        }
    }

    fn bootstrap_config(root: &Path) -> BootstrapConfig {
        let runtime_dir = root.join("runtime");
        fs::create_dir_all(runtime_dir.join("secrets")).expect("secret dir should exist");
        fs::create_dir_all(runtime_dir.join("public")).expect("public dir should exist");

        let manifest_path = runtime_dir.join("infrastructure-manifest.json");
        write_file(
            &manifest_path,
            r#"{
  "cache_name": "nix-cache-sandbox",
  "project_id": "project-123",
  "bucket_name": "cache-bucket",
  "region": "fr-par",
  "endpoint": "s3.fr-par.scw.cloud",
  "author_application_id": "author-app",
  "consumer_application_id": "consumer-app"
}"#,
        );
        write_file(
            &root.join(".sops.yaml"),
            r#"creation_rules:
  - path_regex: .*
    key_groups:
      - age:
          - age1testrecipient
"#,
        );

        BootstrapConfig {
            manifest_path,
            manifest: manifest(),
            operator_secret: runtime_dir.join("secrets/operator.sops.yaml"),
            author_secret: runtime_dir.join("secrets/author.sops.yaml"),
            consumer_secret: runtime_dir.join("secrets/consumer.sops.yaml"),
            signing_secret: runtime_dir.join("secrets/signing.sops.yaml"),
            public_key_out: runtime_dir.join("public/cache.pub"),
            author_credentials_input: None,
            consumer_credentials_input: None,
        }
    }

    #[test]
    fn bootstrap_generates_runtime_outputs_with_placeholders_and_manual_credentials() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = temp_path("workspace");
        let mut config = bootstrap_config(&root);

        let author_input = root.join("manual-author.yaml");
        write_file(
            &author_input,
            r#"access_key: AUTHOR-ACCESS
secret_key: AUTHOR-SECRET
"#,
        );
        let consumer_input = root.join("manual-consumer.yaml");
        write_file(
            &consumer_input,
            r#"consumer:
  application_id: consumer-app
  access_key: CONSUMER-ACCESS
  secret_key: CONSUMER-SECRET
"#,
        );
        config.author_credentials_input = Some(author_input);
        config.consumer_credentials_input = Some(consumer_input);

        let fake_path = with_fake_tools(&root);
        let original_path = std::env::var_os("PATH");
        std::env::set_var("PATH", &fake_path);

        let result = run(&config);

        match original_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        result.expect("bootstrap should succeed");

        assert_eq!(
            fs::read_to_string(&config.public_key_out).expect("public key should exist"),
            "nix-cache-sandbox:fake-public-key\n"
        );

        let operator = load_operator_secret(&config.operator_secret)
            .expect("operator placeholder secret should parse");
        assert_eq!(
            operator.operator.organization_id,
            "SCW_ORGANIZATION_ID_PLACEHOLDER"
        );

        let author = load_author_secret(&config.author_secret).expect("author secret should parse");
        assert_eq!(author.author.application_id, "author-app");
        assert_eq!(author.author.access_key, "AUTHOR-ACCESS");
        assert_eq!(author.author.secret_key, "AUTHOR-SECRET");

        let consumer =
            load_consumer_secret(&config.consumer_secret).expect("consumer secret should parse");
        assert_eq!(consumer.consumer.application_id, "consumer-app");
        assert_eq!(consumer.consumer.access_key, "CONSUMER-ACCESS");
        assert_eq!(consumer.consumer.secret_key, "CONSUMER-SECRET");

        let signing =
            load_signing_secret(&config.signing_secret).expect("signing secret should parse");
        assert_eq!(signing.signing.cache_name, "nix-cache-sandbox");
        assert_eq!(
            signing.signing.private_key,
            "nix-cache-sandbox:fake-secret-key"
        );

        let sops_log = fs::read_to_string(root.join("sops.log")).expect("sops log should exist");
        assert!(sops_log.contains("operator.sops.yaml"));
        assert!(sops_log.contains("author.sops.yaml"));
        assert!(sops_log.contains("consumer.sops.yaml"));
        assert!(sops_log.contains("signing.sops.yaml"));
    }

    #[test]
    fn bootstrap_fails_when_output_already_exists() {
        let root = temp_path("workspace-conflict");
        let config = bootstrap_config(&root);
        write_file(&config.public_key_out, "already here");

        let result = run(&config);

        assert!(matches!(
            result,
            Err(AppError::OutputConflict { label, path })
                if label == "public key output" && path == config.public_key_out
        ));
    }

    #[test]
    fn bootstrap_fails_without_recipient_configuration() {
        let root = temp_path("workspace-no-recipients");
        let config = bootstrap_config(&root);
        write_file(&root.join(".sops.yaml"), "creation_rules: []\n");

        let result = run(&config);

        assert!(matches!(
            result,
            Err(AppError::SopsRecipientsMissing { path }) if path == root.join(".sops.yaml")
        ));
    }

    #[test]
    fn bootstrap_fails_when_manual_credentials_do_not_match_manifest() {
        let root = temp_path("workspace-manual-mismatch");
        let mut config = bootstrap_config(&root);
        let author_input = root.join("manual-author.yaml");
        write_file(
            &author_input,
            r#"application_id: different-author
access_key: AUTHOR-ACCESS
secret_key: AUTHOR-SECRET
"#,
        );
        config.author_credentials_input = Some(author_input);

        let result = run(&config);

        assert!(matches!(
            result,
            Err(AppError::CredentialInputValidation { label, .. })
                if label == "author credentials input"
        ));
    }

    #[test]
    fn bootstrap_reports_missing_sops_tool() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let root = temp_path("workspace-missing-sops");
        let config = bootstrap_config(&root);

        let bin_dir = root.join("bin");
        fs::create_dir_all(&bin_dir).expect("bin dir should be creatable");
        make_executable(
            &bin_dir.join("nix-store"),
            r#"#!/bin/sh
printf "%s:%s\n" "$2" "fake-secret-key" > "$3"
printf "%s:%s\n" "$2" "fake-public-key" > "$4"
"#,
        );

        let original_path = std::env::var_os("PATH");
        std::env::set_var("PATH", bin_dir.as_os_str());

        let result = run(&config);

        match original_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }

        assert!(matches!(
            result,
            Err(AppError::MissingRequiredTool { tool }) if tool == "sops"
        ));
    }
}
