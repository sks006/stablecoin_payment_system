provider "aws" {
  region = var.aws_region
}

resource "aws_db_instance" "postgres" {
  allocated_storage    = 50
  engine               = "postgres"
  engine_version       = "15"
  instance_class       = "db.r6g.xlarge"
  db_name              = "stablecoin"
  username             = "postgres"
  password             = var.db_password
  parameter_group_name = "default.postgres15"
  skip_final_snapshot  = true
}

resource "aws_elasticache_cluster" "redis" {
  cluster_id           = "stablecoin-redis"
  engine               = "redis"
  node_type            = "cache.m6g.large"
  num_cache_nodes      = 1
  parameter_group_name = "default.redis7"
  port                 = 6379
}

resource "aws_kms_key" "kms_signer" {
  description             = "Stablecoin Transaction KMS Signer Key"
  deletion_window_in_days = 10
}
