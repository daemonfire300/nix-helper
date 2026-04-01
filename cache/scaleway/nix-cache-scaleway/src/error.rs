use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("failed to read manifest at {path}: {source}")]
    ManifestRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse manifest at {path}: {source}")]
    ManifestParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("invalid manifest at {path}: {message}")]
    ManifestValidation { path: PathBuf, message: String },

    #[error("failed to read {kind} at {path}: {source}")]
    SecretRead {
        kind: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse {kind} at {path}: {source}")]
    SecretParse {
        kind: &'static str,
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("invalid {kind} at {path}: {message}")]
    SecretValidation {
        kind: &'static str,
        path: PathBuf,
        message: String,
    },

    #[error("failed to inspect {label} at {path}: {source}")]
    MetadataRead {
        label: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("missing required file for {label}: {path}")]
    MissingRequiredFile { label: &'static str, path: PathBuf },

    #[error("expected a regular file for {label}, but found a different file type: {path}")]
    WrongFileType { label: &'static str, path: PathBuf },

    #[error("invalid output target for {label}: {message} ({path})")]
    InvalidOutputTarget {
        label: &'static str,
        path: PathBuf,
        message: String,
    },

    #[error("refusing to overwrite existing {label}: {path}")]
    OutputConflict { label: &'static str, path: PathBuf },

    #[error("failed to create temporary workspace: {source}")]
    TemporaryWorkspaceCreate {
        #[source]
        source: std::io::Error,
    },

    #[error("required tool '{tool}' was not found in PATH")]
    MissingRequiredTool { tool: &'static str },

    #[error("{command} failed: {message}")]
    CommandFailed {
        command: &'static str,
        message: String,
    },

    #[error("failed to read {label} at {path}: {source}")]
    FileRead {
        label: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid {label} at {path}: {message}")]
    InputValidation {
        label: &'static str,
        path: PathBuf,
        message: String,
    },

    #[error("failed to write {label} at {path}: {source}")]
    FileWrite {
        label: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to persist {label} to {path}: {source}")]
    FilePersist {
        label: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid generated {label} at {path}: {message}")]
    OutputValidation {
        label: &'static str,
        path: PathBuf,
        message: String,
    },

    #[error("failed to locate .sops.yaml while searching from {start}")]
    SopsConfigNotFound { start: PathBuf },

    #[error("sops recipient configuration is missing or empty in {path}")]
    SopsRecipientsMissing { path: PathBuf },

    #[error("failed to read credential input for {label} at {path}: {source}")]
    CredentialInputRead {
        label: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse credential input for {label} at {path}: {source}")]
    CredentialInputParse {
        label: &'static str,
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("invalid credential input for {label} at {path}: {message}")]
    CredentialInputValidation {
        label: &'static str,
        path: PathBuf,
        message: String,
    },

    #[allow(dead_code)]
    #[error("{command} live execution is not supported in this milestone; only dry-run validation is implemented")]
    UnsupportedLiveExecution { command: &'static str },
}
