variable "organization_id" {
  description = "Scaleway organization that should own the dedicated cache project."
  type        = string

  validation {
    condition     = length(trimspace(var.organization_id)) > 0
    error_message = "organization_id must not be empty."
  }
}

variable "project_name" {
  description = "Name for the dedicated Scaleway Project that will own the cache bucket."
  type        = string

  validation {
    condition     = length(trimspace(var.project_name)) > 0
    error_message = "project_name must not be empty."
  }
}

variable "project_description" {
  description = "Description for the dedicated Scaleway Project."
  type        = string
  default     = "Dedicated project for the Scaleway-backed Nix binary cache."
}

variable "bucket_name" {
  description = "Private Object Storage bucket name for one cache environment."
  type        = string

  validation {
    condition     = length(trimspace(var.bucket_name)) > 0
    error_message = "bucket_name must not be empty."
  }
}

variable "cache_name" {
  description = "Logical Nix cache name used when generating signing keys; defaults to bucket_name."
  type        = string
  default     = null

  validation {
    condition     = var.cache_name == null || length(trimspace(var.cache_name)) > 0
    error_message = "cache_name must be null or a non-empty string."
  }
}

variable "region" {
  description = "Scaleway region for the cache bucket."
  type        = string
  default     = "fr-par"

  validation {
    condition = contains([
      "fr-par",
      "nl-ams",
      "pl-waw",
    ], var.region)
    error_message = "region must be one of fr-par, nl-ams, or pl-waw."
  }
}

variable "admin_application_name" {
  description = "IAM application name for administrative cache operations."
  type        = string
  default     = "nix-cache-admin"
}

variable "author_application_name" {
  description = "IAM application name for cache publishing operations."
  type        = string
  default     = "nix-cache-author"
}

variable "consumer_application_name" {
  description = "IAM application name for cache consumption and verification."
  type        = string
  default     = "nix-cache-consumer"
}

variable "admin_application_description" {
  description = "Description for the admin IAM application."
  type        = string
  default     = "Administrative principal for the Scaleway-backed Nix cache."
}

variable "author_application_description" {
  description = "Description for the author IAM application."
  type        = string
  default     = "Publishing principal for the Scaleway-backed Nix cache."
}

variable "consumer_application_description" {
  description = "Description for the consumer IAM application."
  type        = string
  default     = "Read-only consumer principal for the Scaleway-backed Nix cache."
}

variable "bucket_policy_version" {
  description = "Version field used in the rendered bucket policy document."
  type        = string
  default     = "2023-04-17"

  validation {
    condition     = var.bucket_policy_version == "2023-04-17"
    error_message = "bucket_policy_version must remain 2023-04-17 for this scaffold."
  }
}

variable "bucket_encryption_algorithm" {
  description = "Default server-side encryption algorithm for the cache bucket."
  type        = string
  default     = "AES256"

  validation {
    condition     = var.bucket_encryption_algorithm == "AES256"
    error_message = "bucket_encryption_algorithm must remain AES256 for this scaffold."
  }
}

variable "admin_organization_permission_set_names" {
  description = "Organization-scoped permission sets for the admin principal."
  type        = list(string)
  default = [
    "IAMManager",
    "ProjectManager",
  ]
}

variable "admin_project_permission_set_names" {
  description = "Project-scoped permission sets for the admin principal."
  type        = list(string)
  default = [
    "ObjectStorageFullAccess",
    "ObjectStorageBucketPolicyFullAccess",
  ]
}

variable "author_permission_set_names" {
  description = "Project-scoped permission sets for the author principal."
  type        = list(string)
  default = [
    "ObjectStorageBucketsRead",
    "ObjectStorageObjectsRead",
    "ObjectStorageObjectsWrite",
  ]
}

variable "consumer_permission_set_names" {
  description = "Project-scoped permission sets for the consumer principal."
  type        = list(string)
  default = [
    "ObjectStorageBucketsRead",
    "ObjectStorageObjectsRead",
  ]
}

variable "admin_bucket_policy_actions" {
  description = "S3 actions granted to the admin principal in the bucket policy."
  type        = list(string)
  default = [
    "s3:*",
  ]
}

variable "author_bucket_policy_actions" {
  description = "S3 actions granted to the author principal in the bucket policy."
  type        = list(string)
  default = [
    "s3:ListBucket",
    "s3:GetBucketLocation",
    "s3:GetObject",
    "s3:PutObject",
    "s3:AbortMultipartUpload",
    "s3:ListBucketMultipartUploads",
    "s3:ListMultipartUploadParts",
  ]
}

variable "consumer_bucket_policy_actions" {
  description = "S3 actions granted to the consumer principal in the bucket policy."
  type        = list(string)
  default = [
    "s3:ListBucket",
    "s3:GetBucketLocation",
    "s3:GetObject",
  ]
}
