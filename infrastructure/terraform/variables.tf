# Terraform variables for decentralized decision vote infrastructure

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "dev"
  
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be one of: dev, staging, prod."
  }
}

variable "project_name" {
  description = "Project name"
  type        = string
  default     = "decentralized-decision-vote"
}

# VPC Configuration
variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "private_subnet_cidrs" {
  description = "CIDR blocks for private subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
}

# EKS Configuration
variable "kubernetes_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "node_instance_types" {
  description = "EC2 instance types for EKS nodes"
  type        = list(string)
  default     = ["t3.medium"]
}

variable "node_min_size" {
  description = "Minimum number of nodes"
  type        = number
  default     = 1
}

variable "node_max_size" {
  description = "Maximum number of nodes"
  type        = number
  default     = 10
}

variable "node_desired_size" {
  description = "Desired number of nodes"
  type        = number
  default     = 3
}

variable "node_disk_size" {
  description = "Disk size for nodes in GB"
  type        = number
  default     = 50
}

# RDS Configuration
variable "postgres_version" {
  description = "PostgreSQL version"
  type        = string
  default     = "15.4"
}

variable "rds_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.micro"
}

variable "rds_allocated_storage" {
  description = "RDS allocated storage in GB"
  type        = number
  default     = 20
}

variable "rds_backup_retention_period" {
  description = "RDS backup retention period in days"
  type        = number
  default     = 7
}

variable "database_name" {
  description = "Database name"
  type        = string
  default     = "ddv_db"
}

variable "database_username" {
  description = "Database username"
  type        = string
  default     = "ddv_user"
}

variable "database_password" {
  description = "Database password"
  type        = string
  sensitive   = true
}

# Redis Configuration
variable "redis_node_type" {
  description = "Redis node type"
  type        = string
  default     = "cache.t3.micro"
}

variable "redis_num_cache_nodes" {
  description = "Number of Redis cache nodes"
  type        = number
  default     = 2
}

# Application Configuration
variable "app_image_tag" {
  description = "Application Docker image tag"
  type        = string
  default     = "latest"
}

variable "app_replicas" {
  description = "Number of application replicas"
  type        = number
  default     = 2
}

variable "app_cpu_limit" {
  description = "CPU limit for application pods"
  type        = string
  default     = "500m"
}

variable "app_memory_limit" {
  description = "Memory limit for application pods"
  type        = string
  default     = "512Mi"
}

variable "app_cpu_request" {
  description = "CPU request for application pods"
  type        = string
  default     = "250m"
}

variable "app_memory_request" {
  description = "Memory request for application pods"
  type        = string
  default     = "256Mi"
}

# Monitoring Configuration
variable "enable_monitoring" {
  description = "Enable monitoring stack (Prometheus, Grafana)"
  type        = bool
  default     = true
}

variable "enable_logging" {
  description = "Enable logging stack (ELK)"
  type        = bool
  default     = true
}

variable "enable_tracing" {
  description = "Enable tracing (Jaeger)"
  type        = bool
  default     = true
}

# Security Configuration
variable "enable_ssl" {
  description = "Enable SSL/TLS"
  type        = bool
  default     = false
}

variable "ssl_certificate_arn" {
  description = "SSL certificate ARN"
  type        = string
  default     = ""
}

# Backup Configuration
variable "enable_backup" {
  description = "Enable backup"
  type        = bool
  default     = true
}

variable "backup_retention_days" {
  description = "Backup retention period in days"
  type        = number
  default     = 30
}

# Cost Optimization
variable "enable_spot_instances" {
  description = "Enable spot instances for cost optimization"
  type        = bool
  default     = false
}

variable "enable_autoscaling" {
  description = "Enable horizontal pod autoscaling"
  type        = bool
  default     = true
}

variable "min_replicas" {
  description = "Minimum number of replicas for HPA"
  type        = number
  default     = 1
}

variable "max_replicas" {
  description = "Maximum number of replicas for HPA"
  type        = number
  default     = 10
}

variable "target_cpu_utilization" {
  description = "Target CPU utilization for HPA"
  type        = number
  default     = 70
}

# Network Configuration
variable "enable_vpc_flow_logs" {
  description = "Enable VPC flow logs"
  type        = bool
  default     = true
}

variable "enable_cloudtrail" {
  description = "Enable CloudTrail"
  type        = bool
  default     = true
}

# Tags
variable "additional_tags" {
  description = "Additional tags to apply to resources"
  type        = map(string)
  default     = {}
}
