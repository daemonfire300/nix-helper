use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error::AppError;

pub const MANIFEST_EXAMPLE_RELATIVE_PATH: &str = "examples/infrastructure-manifest.example.json";
pub const MANIFEST_RUNTIME_RELATIVE_PATH: &str = "runtime/infrastructure-manifest.json";
pub const OPERATOR_SECRET_RUNTIME_RELATIVE_PATH: &str = "runtime/secrets/operator.sops.yaml";
pub const AUTHOR_SECRET_RUNTIME_RELATIVE_PATH: &str = "runtime/secrets/author.sops.yaml";
pub const CONSUMER_SECRET_RUNTIME_RELATIVE_PATH: &str = "runtime/secrets/consumer.sops.yaml";
pub const SIGNING_SECRET_RUNTIME_RELATIVE_PATH: &str = "runtime/secrets/signing.sops.yaml";
pub const PUBLIC_KEY_RUNTIME_RELATIVE_PATH: &str = "runtime/public/cache.pub";

pub const MANIFEST_RUNTIME_FILE_NAME: &str = "infrastructure-manifest.json";
pub const OPERATOR_SECRET_FILE_NAME: &str = "operator.sops.yaml";
pub const AUTHOR_SECRET_FILE_NAME: &str = "author.sops.yaml";
pub const CONSUMER_SECRET_FILE_NAME: &str = "consumer.sops.yaml";
pub const SIGNING_SECRET_FILE_NAME: &str = "signing.sops.yaml";
pub const PUBLIC_KEY_RUNTIME_FILE_NAME: &str = "cache.pub";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct InfrastructureManifest {
    pub cache_name: String,
    pub project_id: String,
    pub bucket_name: String,
    pub region: String,
    pub endpoint: String,
    pub author_application_id: String,
    pub consumer_application_id: String,
}

impl InfrastructureManifest {
    fn validate(&self, source_path: &Path) -> Result<(), AppError> {
        validate_required_values(
            source_path,
            "manifest",
            &[
                ("cache_name", self.cache_name.as_str()),
                ("project_id", self.project_id.as_str()),
                ("bucket_name", self.bucket_name.as_str()),
                ("region", self.region.as_str()),
                ("endpoint", self.endpoint.as_str()),
                ("author_application_id", self.author_application_id.as_str()),
                (
                    "consumer_application_id",
                    self.consumer_application_id.as_str(),
                ),
            ],
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct OperatorSecretDocument {
    pub schema_version: String,
    pub operator: OperatorSecret,
}

impl OperatorSecretDocument {
    fn validate(&self, source_path: &Path) -> Result<(), AppError> {
        validate_required_values(
            source_path,
            "operator secret",
            &[
                ("schema_version", self.schema_version.as_str()),
                (
                    "operator.organization_id",
                    self.operator.organization_id.as_str(),
                ),
                ("operator.access_key", self.operator.access_key.as_str()),
                ("operator.secret_key", self.operator.secret_key.as_str()),
            ],
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct OperatorSecret {
    pub organization_id: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AuthorSecretDocument {
    pub schema_version: String,
    pub author: AuthorSecret,
}

impl AuthorSecretDocument {
    fn validate(&self, source_path: &Path) -> Result<(), AppError> {
        validate_required_values(
            source_path,
            "author secret",
            &[
                ("schema_version", self.schema_version.as_str()),
                ("author.application_id", self.author.application_id.as_str()),
                ("author.access_key", self.author.access_key.as_str()),
                ("author.secret_key", self.author.secret_key.as_str()),
            ],
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct AuthorSecret {
    pub application_id: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ConsumerSecretDocument {
    pub schema_version: String,
    pub consumer: ConsumerSecret,
}

impl ConsumerSecretDocument {
    fn validate(&self, source_path: &Path) -> Result<(), AppError> {
        validate_required_values(
            source_path,
            "consumer secret",
            &[
                ("schema_version", self.schema_version.as_str()),
                (
                    "consumer.application_id",
                    self.consumer.application_id.as_str(),
                ),
                ("consumer.access_key", self.consumer.access_key.as_str()),
                ("consumer.secret_key", self.consumer.secret_key.as_str()),
            ],
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ConsumerSecret {
    pub application_id: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SigningSecretDocument {
    pub schema_version: String,
    pub signing: SigningSecret,
}

impl SigningSecretDocument {
    fn validate(&self, source_path: &Path) -> Result<(), AppError> {
        validate_required_values(
            source_path,
            "signing secret",
            &[
                ("schema_version", self.schema_version.as_str()),
                ("signing.cache_name", self.signing.cache_name.as_str()),
                ("signing.private_key", self.signing.private_key.as_str()),
            ],
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct SigningSecret {
    pub cache_name: String,
    pub private_key: String,
}

pub fn load_manifest(path: &Path) -> Result<InfrastructureManifest, AppError> {
    let contents = fs::read_to_string(path).map_err(|source| AppError::ManifestRead {
        path: path.to_path_buf(),
        source,
    })?;

    let manifest: InfrastructureManifest =
        serde_json::from_str(&contents).map_err(|source| AppError::ManifestParse {
            path: path.to_path_buf(),
            source,
        })?;

    manifest.validate(path)?;

    Ok(manifest)
}

pub fn load_operator_secret(path: &Path) -> Result<OperatorSecretDocument, AppError> {
    load_yaml_document(path, "operator secret", OperatorSecretDocument::validate)
}

pub fn load_author_secret(path: &Path) -> Result<AuthorSecretDocument, AppError> {
    load_yaml_document(path, "author secret", AuthorSecretDocument::validate)
}

pub fn parse_operator_secret(
    path: &Path,
    contents: &str,
) -> Result<OperatorSecretDocument, AppError> {
    parse_yaml_document(
        contents,
        path,
        "operator secret",
        OperatorSecretDocument::validate,
    )
}

pub fn load_consumer_secret(path: &Path) -> Result<ConsumerSecretDocument, AppError> {
    load_yaml_document(path, "consumer secret", ConsumerSecretDocument::validate)
}

pub fn parse_consumer_secret(
    path: &Path,
    contents: &str,
) -> Result<ConsumerSecretDocument, AppError> {
    parse_yaml_document(
        contents,
        path,
        "consumer secret",
        ConsumerSecretDocument::validate,
    )
}

pub fn load_signing_secret(path: &Path) -> Result<SigningSecretDocument, AppError> {
    load_yaml_document(path, "signing secret", SigningSecretDocument::validate)
}

pub fn parse_author_secret(path: &Path, contents: &str) -> Result<AuthorSecretDocument, AppError> {
    parse_yaml_document(
        contents,
        path,
        "author secret",
        AuthorSecretDocument::validate,
    )
}

pub fn parse_signing_secret(
    path: &Path,
    contents: &str,
) -> Result<SigningSecretDocument, AppError> {
    parse_yaml_document(
        contents,
        path,
        "signing secret",
        SigningSecretDocument::validate,
    )
}

pub fn validate_expected_file_name(
    path: &Path,
    label: &'static str,
    expected_file_name: &'static str,
) -> Result<(), AppError> {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return Err(AppError::InvalidOutputTarget {
            label,
            path: path.to_path_buf(),
            message: "path must end with a file name".to_string(),
        });
    };

    if file_name != expected_file_name {
        return Err(AppError::InvalidOutputTarget {
            label,
            path: path.to_path_buf(),
            message: format!("expected file name {expected_file_name}"),
        });
    }

    Ok(())
}

pub fn ensure_matches(
    kind: &'static str,
    path: &Path,
    field: &'static str,
    actual: &str,
    expected: &str,
) -> Result<(), AppError> {
    if actual == expected {
        return Ok(());
    }

    Err(AppError::SecretValidation {
        kind,
        path: path.to_path_buf(),
        message: format!("field '{field}' must match manifest value '{expected}'"),
    })
}

fn load_yaml_document<T>(
    path: &Path,
    kind: &'static str,
    validate: fn(&T, &Path) -> Result<(), AppError>,
) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    let contents = fs::read_to_string(path).map_err(|source| AppError::SecretRead {
        kind,
        path: path.to_path_buf(),
        source,
    })?;

    parse_yaml_document(&contents, path, kind, validate)
}

fn parse_yaml_document<T>(
    contents: &str,
    path: &Path,
    kind: &'static str,
    validate: fn(&T, &Path) -> Result<(), AppError>,
) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    let document: T = serde_yaml::from_str(contents).map_err(|source| AppError::SecretParse {
        kind,
        path: path.to_path_buf(),
        source,
    })?;

    validate(&document, path)?;

    Ok(document)
}

fn validate_required_values(
    source_path: &Path,
    kind: &'static str,
    fields: &[(&'static str, &str)],
) -> Result<(), AppError> {
    for (label, value) in fields {
        if value.trim().is_empty() {
            return Err(match kind {
                "manifest" => AppError::ManifestValidation {
                    path: source_path.to_path_buf(),
                    message: format!("required field '{label}' must not be empty"),
                },
                _ => AppError::SecretValidation {
                    kind,
                    path: source_path.to_path_buf(),
                    message: format!("required field '{label}' must not be empty"),
                },
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        load_author_secret, load_consumer_secret, load_manifest, load_operator_secret,
        load_signing_secret, InfrastructureManifest, AUTHOR_SECRET_RUNTIME_RELATIVE_PATH,
        CONSUMER_SECRET_RUNTIME_RELATIVE_PATH, MANIFEST_EXAMPLE_RELATIVE_PATH,
        MANIFEST_RUNTIME_FILE_NAME, MANIFEST_RUNTIME_RELATIVE_PATH, OPERATOR_SECRET_FILE_NAME,
        OPERATOR_SECRET_RUNTIME_RELATIVE_PATH, PUBLIC_KEY_RUNTIME_FILE_NAME,
        PUBLIC_KEY_RUNTIME_RELATIVE_PATH, SIGNING_SECRET_FILE_NAME,
        SIGNING_SECRET_RUNTIME_RELATIVE_PATH,
    };

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    #[test]
    fn tracked_example_manifest_matches_schema() {
        let manifest_path = project_root().join(MANIFEST_EXAMPLE_RELATIVE_PATH);
        let manifest = load_manifest(&manifest_path).expect("tracked example manifest should load");

        assert_eq!(
            manifest,
            InfrastructureManifest {
                cache_name: "nix-cache-sandbox".to_string(),
                project_id: "11111111-1111-1111-1111-111111111111".to_string(),
                bucket_name: "nix-cache-scaleway-sandbox-example".to_string(),
                region: "fr-par".to_string(),
                endpoint: "s3.fr-par.scw.cloud".to_string(),
                author_application_id: "22222222-2222-2222-2222-222222222222".to_string(),
                consumer_application_id: "33333333-3333-3333-3333-333333333333".to_string(),
            }
        );
    }

    #[test]
    fn tracked_example_secret_documents_match_schema() {
        let root = project_root().join("examples/secrets");

        load_operator_secret(&root.join("operator.example.sops.yaml"))
            .expect("operator example should load");
        load_author_secret(&root.join("author.example.sops.yaml"))
            .expect("author example should load");
        load_consumer_secret(&root.join("consumer.example.sops.yaml"))
            .expect("consumer example should load");
        load_signing_secret(&root.join("signing.example.sops.yaml"))
            .expect("signing example should load");
    }

    #[test]
    fn runtime_relative_paths_stay_stable() {
        assert_eq!(
            MANIFEST_RUNTIME_RELATIVE_PATH,
            "runtime/infrastructure-manifest.json"
        );
        assert_eq!(MANIFEST_RUNTIME_FILE_NAME, "infrastructure-manifest.json");
        assert_eq!(
            OPERATOR_SECRET_RUNTIME_RELATIVE_PATH,
            "runtime/secrets/operator.sops.yaml"
        );
        assert_eq!(OPERATOR_SECRET_FILE_NAME, "operator.sops.yaml");
        assert_eq!(
            AUTHOR_SECRET_RUNTIME_RELATIVE_PATH,
            "runtime/secrets/author.sops.yaml"
        );
        assert_eq!(
            CONSUMER_SECRET_RUNTIME_RELATIVE_PATH,
            "runtime/secrets/consumer.sops.yaml"
        );
        assert_eq!(
            SIGNING_SECRET_RUNTIME_RELATIVE_PATH,
            "runtime/secrets/signing.sops.yaml"
        );
        assert_eq!(SIGNING_SECRET_FILE_NAME, "signing.sops.yaml");
        assert_eq!(PUBLIC_KEY_RUNTIME_RELATIVE_PATH, "runtime/public/cache.pub");
        assert_eq!(PUBLIC_KEY_RUNTIME_FILE_NAME, "cache.pub");
    }
}
