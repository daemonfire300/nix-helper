locals {
  bucket_endpoint = "s3.${var.region}.scw.cloud"
  cache_name      = coalesce(var.cache_name, var.bucket_name)

  bucket_policy_resources = [
    scaleway_object_bucket.cache.name,
    "${scaleway_object_bucket.cache.name}/*",
  ]

  bucket_policy_id = "${var.bucket_name}-private-cache-policy"

  bucket_policy_statements = [
    {
      sid            = "AdminFullControl"
      application_id = scaleway_iam_application.admin.id
      actions        = var.admin_bucket_policy_actions
      resources      = local.bucket_policy_resources
    },
    {
      sid            = "AuthorReadWriteNoDelete"
      application_id = scaleway_iam_application.author.id
      actions        = var.author_bucket_policy_actions
      resources      = local.bucket_policy_resources
    },
    {
      sid            = "ConsumerReadOnly"
      application_id = scaleway_iam_application.consumer.id
      actions        = var.consumer_bucket_policy_actions
      resources      = local.bucket_policy_resources
    },
  ]

  manifest = {
    cache_name              = local.cache_name
    project_id              = scaleway_account_project.cache.id
    bucket_name             = scaleway_object_bucket.cache.name
    region                  = var.region
    endpoint                = local.bucket_endpoint
    author_application_id   = scaleway_iam_application.author.id
    consumer_application_id = scaleway_iam_application.consumer.id
  }
}
