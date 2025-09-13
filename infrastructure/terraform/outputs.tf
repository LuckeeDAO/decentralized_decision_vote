# Terraform outputs for decentralized decision vote infrastructure

# VPC Outputs
output "vpc_id" {
  description = "ID of the VPC"
  value       = module.vpc.vpc_id
}

output "vpc_cidr_block" {
  description = "CIDR block of the VPC"
  value       = module.vpc.vpc_cidr_block
}

output "private_subnets" {
  description = "List of IDs of private subnets"
  value       = module.vpc.private_subnets
}

output "public_subnets" {
  description = "List of IDs of public subnets"
  value       = module.vpc.public_subnets
}

# EKS Outputs
output "cluster_id" {
  description = "EKS cluster ID"
  value       = module.eks.cluster_id
}

output "cluster_arn" {
  description = "EKS cluster ARN"
  value       = module.eks.cluster_arn
}

output "cluster_endpoint" {
  description = "Endpoint for EKS control plane"
  value       = module.eks.cluster_endpoint
}

output "cluster_security_group_id" {
  description = "Security group ids attached to the cluster control plane"
  value       = module.eks.cluster_security_group_id
}

output "cluster_iam_role_name" {
  description = "IAM role name associated with EKS cluster"
  value       = module.eks.cluster_iam_role_name
}

output "cluster_certificate_authority_data" {
  description = "Base64 encoded certificate data required to communicate with the cluster"
  value       = module.eks.cluster_certificate_authority_data
}

output "cluster_primary_security_group_id" {
  description = "The cluster primary security group ID created by EKS on the private subnets"
  value       = module.eks.cluster_primary_security_group_id
}

output "cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "cluster_version" {
  description = "EKS cluster version"
  value       = module.eks.cluster_version
}

output "node_groups" {
  description = "EKS node groups"
  value       = module.eks.eks_managed_node_groups
}

# RDS Outputs
output "rds_endpoint" {
  description = "RDS instance endpoint"
  value       = module.rds.db_instance_endpoint
  sensitive   = true
}

output "rds_port" {
  description = "RDS instance port"
  value       = module.rds.db_instance_port
}

output "rds_username" {
  description = "RDS instance root username"
  value       = module.rds.db_instance_username
  sensitive   = true
}

output "rds_database_name" {
  description = "RDS database name"
  value       = module.rds.db_instance_name
}

output "rds_arn" {
  description = "RDS instance ARN"
  value       = module.rds.db_instance_arn
}

# Redis Outputs
output "redis_endpoint" {
  description = "Redis cluster endpoint"
  value       = aws_elasticache_replication_group.main.primary_endpoint_address
}

output "redis_port" {
  description = "Redis cluster port"
  value       = aws_elasticache_replication_group.main.port
}

output "redis_arn" {
  description = "Redis cluster ARN"
  value       = aws_elasticache_replication_group.main.arn
}

# S3 Outputs
output "s3_bucket_id" {
  description = "S3 bucket ID"
  value       = aws_s3_bucket.app_data.id
}

output "s3_bucket_arn" {
  description = "S3 bucket ARN"
  value       = aws_s3_bucket.app_data.arn
}

output "s3_bucket_domain_name" {
  description = "S3 bucket domain name"
  value       = aws_s3_bucket.app_data.bucket_domain_name
}

# Load Balancer Outputs
output "alb_dns_name" {
  description = "ALB DNS name"
  value       = aws_lb.main.dns_name
}

output "alb_zone_id" {
  description = "ALB zone ID"
  value       = aws_lb.main.zone_id
}

output "alb_arn" {
  description = "ALB ARN"
  value       = aws_lb.main.arn
}

output "target_group_arn" {
  description = "Target group ARN"
  value       = aws_lb_target_group.main.arn
}

# IAM Outputs
output "eks_admin_role_arn" {
  description = "EKS admin role ARN"
  value       = aws_iam_role.eks_admin.arn
}

# Security Group Outputs
output "rds_security_group_id" {
  description = "RDS security group ID"
  value       = aws_security_group.rds.id
}

output "redis_security_group_id" {
  description = "Redis security group ID"
  value       = aws_security_group.redis.id
}

output "alb_security_group_id" {
  description = "ALB security group ID"
  value       = aws_security_group.alb.id
}

# Connection Information
output "kubectl_config" {
  description = "kubectl config command"
  value       = "aws eks update-kubeconfig --region ${var.aws_region} --name ${module.eks.cluster_name}"
}

output "database_connection_string" {
  description = "Database connection string"
  value       = "postgresql://${module.rds.db_instance_username}:${var.database_password}@${module.rds.db_instance_endpoint}:${module.rds.db_instance_port}/${module.rds.db_instance_name}"
  sensitive   = true
}

output "redis_connection_string" {
  description = "Redis connection string"
  value       = "${aws_elasticache_replication_group.main.primary_endpoint_address}:${aws_elasticache_replication_group.main.port}"
}

# Environment Information
output "environment" {
  description = "Environment name"
  value       = var.environment
}

output "aws_region" {
  description = "AWS region"
  value       = var.aws_region
}

output "project_name" {
  description = "Project name"
  value       = var.project_name
}

# Cost Information
output "estimated_monthly_cost" {
  description = "Estimated monthly cost in USD"
  value       = "~$200-500 (varies by usage and instance types)"
}

# Monitoring Information
output "monitoring_enabled" {
  description = "Whether monitoring is enabled"
  value       = var.enable_monitoring
}

output "logging_enabled" {
  description = "Whether logging is enabled"
  value       = var.enable_logging
}

output "tracing_enabled" {
  description = "Whether tracing is enabled"
  value       = var.enable_tracing
}

# Backup Information
output "backup_enabled" {
  description = "Whether backup is enabled"
  value       = var.enable_backup
}

output "backup_retention_days" {
  description = "Backup retention period in days"
  value       = var.backup_retention_days
}
