terraform {
  required_version = "~> 1.10.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "= 6.21.0"
    }

    scaleway = {
      source  = "scaleway/scaleway"
      version = "= 2.63.0"
    }
  }
}

provider "scaleway" {
  organization_id = var.organization_id
  region          = var.region
}
