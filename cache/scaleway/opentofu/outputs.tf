output "cache_name" {
  description = "Logical Nix cache name used for signing key material and consumer trust."
  value       = local.manifest.cache_name
}

output "project_id" {
  description = "Dedicated Scaleway Project ID for the cache environment."
  value       = local.manifest.project_id
}

output "bucket_name" {
  description = "Private Object Storage bucket name for the cache environment."
  value       = local.manifest.bucket_name
}

output "region" {
  description = "Scaleway region hosting the cache bucket."
  value       = local.manifest.region
}

output "endpoint" {
  description = "S3-compatible endpoint hostname for the cache bucket."
  value       = local.manifest.endpoint
}

output "author_application_id" {
  description = "IAM application ID reserved for publishing credentials."
  value       = local.manifest.author_application_id
}

output "consumer_application_id" {
  description = "IAM application ID reserved for consumer credentials."
  value       = local.manifest.consumer_application_id
}

output "manifest" {
  description = "Combined OpenTofu-to-Rust manifest contract for nix-cache-scaleway."
  value       = local.manifest
}
