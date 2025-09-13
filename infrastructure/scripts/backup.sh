#!/bin/bash

# Backup script for decentralized decision vote system
# This script creates backups of databases, configurations, and application data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="decentralized-decision-vote"
BACKUP_DIR="${BACKUP_DIR:-./backups}"
RETENTION_DAYS="${RETENTION_DAYS:-30}"
NAMESPACE="${NAMESPACE:-ddv}"
ENVIRONMENT="${ENVIRONMENT:-dev}"

# Backup types
BACKUP_DATABASE=true
BACKUP_REDIS=true
BACKUP_CONFIGS=true
BACKUP_LOGS=true
BACKUP_VOLUMES=true

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    if ! command -v kubectl &> /dev/null; then
        missing_deps+=("kubectl")
    fi
    
    if ! command -v pg_dump &> /dev/null; then
        missing_deps+=("pg_dump")
    fi
    
    if ! command -v redis-cli &> /dev/null; then
        missing_deps+=("redis-cli")
    fi
    
    if ! command -v tar &> /dev/null; then
        missing_deps+=("tar")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install the missing dependencies and try again."
        exit 1
    fi
    
    log_success "All dependencies are installed"
}

# Create backup directory
create_backup_directory() {
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local backup_path="$BACKUP_DIR/$ENVIRONMENT/$timestamp"
    
    mkdir -p "$backup_path"
    echo "$backup_path"
}

# Backup PostgreSQL database
backup_database() {
    if [ "$BACKUP_DATABASE" != "true" ]; then
        log_warning "Skipping database backup"
        return
    fi
    
    log_info "Backing up PostgreSQL database..."
    
    local backup_path="$1"
    local db_backup_dir="$backup_path/database"
    mkdir -p "$db_backup_dir"
    
    # Get database connection details
    local db_host=$(kubectl get service postgresql -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || echo "localhost")
    local db_port=$(kubectl get service postgresql -n "$NAMESPACE" -o jsonpath='{.spec.ports[0].port}' 2>/dev/null || echo "5432")
    local db_name="${DATABASE_NAME:-ddv_db}"
    local db_user="${DATABASE_USER:-postgres}"
    local db_password="${DATABASE_PASSWORD:-changeme}"
    
    # Create database dump
    PGPASSWORD="$db_password" pg_dump \
        -h "$db_host" \
        -p "$db_port" \
        -U "$db_user" \
        -d "$db_name" \
        --verbose \
        --clean \
        --no-owner \
        --no-privileges \
        --format=custom \
        --file="$db_backup_dir/database.dump"
    
    # Create SQL dump as well
    PGPASSWORD="$db_password" pg_dump \
        -h "$db_host" \
        -p "$db_port" \
        -U "$db_user" \
        -d "$db_name" \
        --verbose \
        --clean \
        --no-owner \
        --no-privileges \
        --format=plain \
        --file="$db_backup_dir/database.sql"
    
    # Compress the backup
    tar -czf "$db_backup_dir/database.tar.gz" -C "$db_backup_dir" database.dump database.sql
    
    log_success "Database backup completed: $db_backup_dir/database.tar.gz"
}

# Backup Redis data
backup_redis() {
    if [ "$BACKUP_REDIS" != "true" ]; then
        log_warning "Skipping Redis backup"
        return
    fi
    
    log_info "Backing up Redis data..."
    
    local backup_path="$1"
    local redis_backup_dir="$backup_path/redis"
    mkdir -p "$redis_backup_dir"
    
    # Get Redis connection details
    local redis_host=$(kubectl get service redis-master -n "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null || echo "localhost")
    local redis_port=$(kubectl get service redis-master -n "$NAMESPACE" -o jsonpath='{.spec.ports[0].port}' 2>/dev/null || echo "6379")
    
    # Create Redis backup
    redis-cli -h "$redis_host" -p "$redis_port" --rdb "$redis_backup_dir/dump.rdb"
    
    # Also create a text backup of all keys
    redis-cli -h "$redis_host" -p "$redis_port" --scan > "$redis_backup_dir/keys.txt"
    
    # Compress the backup
    tar -czf "$redis_backup_dir/redis.tar.gz" -C "$redis_backup_dir" dump.rdb keys.txt
    
    log_success "Redis backup completed: $redis_backup_dir/redis.tar.gz"
}

# Backup Kubernetes configurations
backup_configs() {
    if [ "$BACKUP_CONFIGS" != "true" ]; then
        log_warning "Skipping configuration backup"
        return
    fi
    
    log_info "Backing up Kubernetes configurations..."
    
    local backup_path="$1"
    local config_backup_dir="$backup_path/configs"
    mkdir -p "$config_backup_dir"
    
    # Backup all resources in the namespace
    kubectl get all -n "$NAMESPACE" -o yaml > "$config_backup_dir/all-resources.yaml"
    
    # Backup ConfigMaps
    kubectl get configmaps -n "$NAMESPACE" -o yaml > "$config_backup_dir/configmaps.yaml"
    
    # Backup Secrets (without sensitive data)
    kubectl get secrets -n "$NAMESPACE" -o yaml > "$config_backup_dir/secrets.yaml"
    
    # Backup Services
    kubectl get services -n "$NAMESPACE" -o yaml > "$config_backup_dir/services.yaml"
    
    # Backup Deployments
    kubectl get deployments -n "$NAMESPACE" -o yaml > "$config_backup_dir/deployments.yaml"
    
    # Backup Ingress
    kubectl get ingress -n "$NAMESPACE" -o yaml > "$config_backup_dir/ingress.yaml"
    
    # Backup PersistentVolumeClaims
    kubectl get pvc -n "$NAMESPACE" -o yaml > "$config_backup_dir/pvc.yaml"
    
    # Compress the backup
    tar -czf "$config_backup_dir/configs.tar.gz" -C "$config_backup_dir" *.yaml
    
    log_success "Configuration backup completed: $config_backup_dir/configs.tar.gz"
}

# Backup application logs
backup_logs() {
    if [ "$BACKUP_LOGS" != "true" ]; then
        log_warning "Skipping logs backup"
        return
    fi
    
    log_info "Backing up application logs..."
    
    local backup_path="$1"
    local logs_backup_dir="$backup_path/logs"
    mkdir -p "$logs_backup_dir"
    
    # Get all pods in the namespace
    local pods=$(kubectl get pods -n "$NAMESPACE" -o jsonpath='{.items[*].metadata.name}')
    
    for pod in $pods; do
        log_info "Backing up logs for pod: $pod"
        
        # Get pod logs
        kubectl logs "$pod" -n "$NAMESPACE" --previous > "$logs_backup_dir/${pod}_previous.log" 2>/dev/null || true
        kubectl logs "$pod" -n "$NAMESPACE" > "$logs_backup_dir/${pod}_current.log" 2>/dev/null || true
        
        # Get pod description
        kubectl describe pod "$pod" -n "$NAMESPACE" > "$logs_backup_dir/${pod}_description.txt" 2>/dev/null || true
    done
    
    # Compress the backup
    tar -czf "$logs_backup_dir/logs.tar.gz" -C "$logs_backup_dir" *.log *.txt
    
    log_success "Logs backup completed: $logs_backup_dir/logs.tar.gz"
}

# Backup persistent volumes
backup_volumes() {
    if [ "$BACKUP_VOLUMES" != "true" ]; then
        log_warning "Skipping volumes backup"
        return
    fi
    
    log_info "Backing up persistent volumes..."
    
    local backup_path="$1"
    local volumes_backup_dir="$backup_path/volumes"
    mkdir -p "$volumes_backup_dir"
    
    # Get all PVCs in the namespace
    local pvcs=$(kubectl get pvc -n "$NAMESPACE" -o jsonpath='{.items[*].metadata.name}')
    
    for pvc in $pvcs; do
        log_info "Backing up volume: $pvc"
        
        # Create a temporary pod to access the volume
        local pod_name="backup-pod-$(date +%s)"
        
        kubectl run "$pod_name" -n "$NAMESPACE" \
            --image=busybox \
            --restart=Never \
            --rm \
            --overrides="{
                \"spec\": {
                    \"containers\": [{
                        \"name\": \"backup\",
                        \"image\": \"busybox\",
                        \"command\": [\"sleep\", \"3600\"],
                        \"volumeMounts\": [{
                            \"name\": \"data\",
                            \"mountPath\": \"/data\"
                        }]
                    }],
                    \"volumes\": [{
                        \"name\": \"data\",
                        \"persistentVolumeClaim\": {
                            \"claimName\": \"$pvc\"
                        }
                    }]
                }
            }" &
        
        # Wait for pod to be ready
        kubectl wait --for=condition=Ready pod/"$pod_name" -n "$NAMESPACE" --timeout=60s
        
        # Copy data from the volume
        kubectl cp "$NAMESPACE/$pod_name:/data" "$volumes_backup_dir/$pvc" 2>/dev/null || true
        
        # Clean up the pod
        kubectl delete pod "$pod_name" -n "$NAMESPACE" --ignore-not-found=true
    done
    
    # Compress the backup
    if [ -d "$volumes_backup_dir" ] && [ "$(ls -A "$volumes_backup_dir")" ]; then
        tar -czf "$volumes_backup_dir/volumes.tar.gz" -C "$volumes_backup_dir" .
    fi
    
    log_success "Volumes backup completed: $volumes_backup_dir/volumes.tar.gz"
}

# Create backup metadata
create_backup_metadata() {
    local backup_path="$1"
    local metadata_file="$backup_path/backup-metadata.json"
    
    cat > "$metadata_file" << EOF
{
    "project": "$PROJECT_NAME",
    "environment": "$ENVIRONMENT",
    "namespace": "$NAMESPACE",
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "backup_types": {
        "database": $BACKUP_DATABASE,
        "redis": $BACKUP_REDIS,
        "configs": $BACKUP_CONFIGS,
        "logs": $BACKUP_LOGS,
        "volumes": $BACKUP_VOLUMES
    },
    "kubernetes_version": "$(kubectl version --short 2>/dev/null | grep Server | cut -d' ' -f3 || echo 'unknown')",
    "backup_size": "$(du -sh "$backup_path" | cut -f1)"
}
EOF
    
    log_success "Backup metadata created: $metadata_file"
}

# Clean up old backups
cleanup_old_backups() {
    log_info "Cleaning up old backups (older than $RETENTION_DAYS days)..."
    
    find "$BACKUP_DIR" -type d -name "20*" -mtime +$RETENTION_DAYS -exec rm -rf {} \; 2>/dev/null || true
    
    log_success "Old backups cleaned up"
}

# Verify backup integrity
verify_backup() {
    local backup_path="$1"
    
    log_info "Verifying backup integrity..."
    
    # Check if backup directory exists and is not empty
    if [ ! -d "$backup_path" ] || [ -z "$(ls -A "$backup_path")" ]; then
        log_error "Backup directory is empty or does not exist"
        return 1
    fi
    
    # Check if metadata file exists
    if [ ! -f "$backup_path/backup-metadata.json" ]; then
        log_error "Backup metadata file is missing"
        return 1
    fi
    
    # Check backup size
    local backup_size=$(du -sh "$backup_path" | cut -f1)
    log_info "Backup size: $backup_size"
    
    log_success "Backup integrity verified"
}

# Upload backup to remote storage
upload_backup() {
    local backup_path="$1"
    
    if [ -z "$REMOTE_BACKUP_URL" ]; then
        log_warning "No remote backup URL configured, skipping upload"
        return
    fi
    
    log_info "Uploading backup to remote storage..."
    
    # Compress the entire backup
    local backup_name="backup_${ENVIRONMENT}_$(basename "$backup_path").tar.gz"
    tar -czf "$backup_name" -C "$(dirname "$backup_path")" "$(basename "$backup_path")"
    
    # Upload to remote storage (example with S3)
    if command -v aws &> /dev/null && [ -n "$AWS_S3_BUCKET" ]; then
        aws s3 cp "$backup_name" "s3://$AWS_S3_BUCKET/backups/$backup_name"
        log_success "Backup uploaded to S3: s3://$AWS_S3_BUCKET/backups/$backup_name"
    else
        log_warning "AWS CLI not available or S3 bucket not configured, skipping upload"
    fi
    
    # Clean up local compressed file
    rm -f "$backup_name"
}

# Main backup function
main() {
    log_info "Starting backup process for $PROJECT_NAME"
    log_info "Environment: $ENVIRONMENT"
    log_info "Namespace: $NAMESPACE"
    log_info "Backup directory: $BACKUP_DIR"
    log_info "Retention days: $RETENTION_DAYS"
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            --namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            --backup-dir)
                BACKUP_DIR="$2"
                shift 2
                ;;
            --retention-days)
                RETENTION_DAYS="$2"
                shift 2
                ;;
            --skip-database)
                BACKUP_DATABASE=false
                shift
                ;;
            --skip-redis)
                BACKUP_REDIS=false
                shift
                ;;
            --skip-configs)
                BACKUP_CONFIGS=false
                shift
                ;;
            --skip-logs)
                BACKUP_LOGS=false
                shift
                ;;
            --skip-volumes)
                BACKUP_VOLUMES=false
                shift
                ;;
            --upload)
                UPLOAD_BACKUP=true
                shift
                ;;
            --cleanup)
                cleanup_old_backups
                exit 0
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --environment ENV     Set environment (default: dev)"
                echo "  --namespace NS        Set namespace (default: ddv)"
                echo "  --backup-dir DIR      Set backup directory (default: ./backups)"
                echo "  --retention-days N    Set retention days (default: 30)"
                echo "  --skip-database       Skip database backup"
                echo "  --skip-redis          Skip Redis backup"
                echo "  --skip-configs        Skip configuration backup"
                echo "  --skip-logs           Skip logs backup"
                echo "  --skip-volumes        Skip volumes backup"
                echo "  --upload              Upload backup to remote storage"
                echo "  --cleanup             Clean up old backups"
                echo "  --help                Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Execute backup steps
    check_dependencies
    
    local backup_path=$(create_backup_directory)
    log_info "Backup path: $backup_path"
    
    backup_database "$backup_path"
    backup_redis "$backup_path"
    backup_configs "$backup_path"
    backup_logs "$backup_path"
    backup_volumes "$backup_path"
    create_backup_metadata "$backup_path"
    verify_backup "$backup_path"
    
    if [ "$UPLOAD_BACKUP" = "true" ]; then
        upload_backup "$backup_path"
    fi
    
    cleanup_old_backups
    
    log_success "Backup completed successfully!"
    
    # Show summary
    echo ""
    log_info "Backup Summary:"
    echo "  Environment: $ENVIRONMENT"
    echo "  Namespace: $NAMESPACE"
    echo "  Backup Path: $backup_path"
    echo "  Backup Size: $(du -sh "$backup_path" | cut -f1)"
    echo "  Database: $([ "$BACKUP_DATABASE" = "true" ] && echo "✓ Backed up" || echo "⚠ Skipped")"
    echo "  Redis: $([ "$BACKUP_REDIS" = "true" ] && echo "✓ Backed up" || echo "⚠ Skipped")"
    echo "  Configs: $([ "$BACKUP_CONFIGS" = "true" ] && echo "✓ Backed up" || echo "⚠ Skipped")"
    echo "  Logs: $([ "$BACKUP_LOGS" = "true" ] && echo "✓ Backed up" || echo "⚠ Skipped")"
    echo "  Volumes: $([ "$BACKUP_VOLUMES" = "true" ] && echo "✓ Backed up" || echo "⚠ Skipped")"
}

# Run main function
main "$@"
