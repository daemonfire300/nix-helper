resource "scaleway_account_project" "cache" {
  name            = var.project_name
  description     = var.project_description
  organization_id = var.organization_id
}

resource "scaleway_iam_application" "admin" {
  name            = var.admin_application_name
  description     = var.admin_application_description
  organization_id = var.organization_id
}

resource "scaleway_iam_application" "author" {
  name            = var.author_application_name
  description     = var.author_application_description
  organization_id = var.organization_id
}

resource "scaleway_iam_application" "consumer" {
  name            = var.consumer_application_name
  description     = var.consumer_application_description
  organization_id = var.organization_id
}

resource "scaleway_iam_policy" "admin" {
  name            = "${var.project_name}-cache-admin"
  description     = "Administrative access for the dedicated Nix cache project."
  organization_id = var.organization_id
  application_id  = scaleway_iam_application.admin.id

  rule {
    organization_id      = var.organization_id
    permission_set_names = var.admin_organization_permission_set_names
  }

  rule {
    project_ids          = [scaleway_account_project.cache.id]
    permission_set_names = var.admin_project_permission_set_names
  }
}

resource "scaleway_iam_policy" "author" {
  name            = "${var.project_name}-cache-author"
  description     = "Publishing access for the dedicated Nix cache project."
  organization_id = var.organization_id
  application_id  = scaleway_iam_application.author.id

  rule {
    project_ids          = [scaleway_account_project.cache.id]
    permission_set_names = var.author_permission_set_names
  }
}

resource "scaleway_iam_policy" "consumer" {
  name            = "${var.project_name}-cache-consumer"
  description     = "Read-only access for the dedicated Nix cache project."
  organization_id = var.organization_id
  application_id  = scaleway_iam_application.consumer.id

  rule {
    project_ids          = [scaleway_account_project.cache.id]
    permission_set_names = var.consumer_permission_set_names
  }
}

resource "scaleway_object_bucket" "cache" {
  name          = var.bucket_name
  project_id    = scaleway_account_project.cache.id
  region        = var.region
  force_destroy = false
}

resource "scaleway_object_bucket_acl" "cache" {
  bucket     = scaleway_object_bucket.cache.id
  project_id = scaleway_account_project.cache.id
  region     = var.region
  acl        = "private"
}

resource "aws_s3_bucket_server_side_encryption_configuration" "cache" {
  bucket = scaleway_object_bucket.cache.name

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = var.bucket_encryption_algorithm
    }
  }
}

resource "scaleway_object_bucket_policy" "cache" {
  bucket     = scaleway_object_bucket.cache.id
  project_id = scaleway_account_project.cache.id
  policy = templatefile("${path.module}/templates/bucket-policy.json.tftpl", {
    policy_version = var.bucket_policy_version
    policy_id      = local.bucket_policy_id
    statements     = local.bucket_policy_statements
  })
}
