#!/bin/bash

# Deployment script for decentralized decision vote system
# This script deploys the application to various environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="decentralized-decision-vote"
NAMESPACE="${NAMESPACE:-ddv}"
ENVIRONMENT="${ENVIRONMENT:-dev}"
VERSION="${VERSION:-latest}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-localhost:5000}"

# Services to deploy
SERVICES=(
    "vote-api"
    "notification-service"
    "admin-api"
)

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
    
    if ! command -v helm &> /dev/null; then
        missing_deps+=("helm")
    fi
    
    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install the missing dependencies and try again."
        exit 1
    fi
    
    log_success "All dependencies are installed"
}

# Check Kubernetes cluster connection
check_k8s_connection() {
    log_info "Checking Kubernetes cluster connection..."
    
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        log_error "Please ensure kubectl is configured correctly"
        exit 1
    fi
    
    local cluster_info=$(kubectl cluster-info | head -n 1)
    log_success "Connected to cluster: $cluster_info"
}

# Create namespace
create_namespace() {
    log_info "Creating namespace: $NAMESPACE"
    
    if kubectl get namespace "$NAMESPACE" &> /dev/null; then
        log_warning "Namespace $NAMESPACE already exists"
    else
        kubectl create namespace "$NAMESPACE"
        log_success "Namespace $NAMESPACE created"
    fi
}

# Deploy infrastructure components
deploy_infrastructure() {
    log_info "Deploying infrastructure components..."
    
    # Deploy PostgreSQL
    if [ "$SKIP_DATABASE" != "true" ]; then
        deploy_postgresql
    fi
    
    # Deploy Redis
    if [ "$SKIP_REDIS" != "true" ]; then
        deploy_redis
    fi
    
    # Deploy monitoring stack
    if [ "$ENABLE_MONITORING" = "true" ]; then
        deploy_monitoring
    fi
    
    # Deploy logging stack
    if [ "$ENABLE_LOGGING" = "true" ]; then
        deploy_logging
    fi
    
    log_success "Infrastructure components deployed"
}

# Deploy PostgreSQL
deploy_postgresql() {
    log_info "Deploying PostgreSQL..."
    
    helm upgrade --install postgresql oci://registry-1.docker.io/bitnamicharts/postgresql \
        --namespace "$NAMESPACE" \
        --set auth.postgresPassword="$POSTGRES_PASSWORD" \
        --set auth.database="$DATABASE_NAME" \
        --set primary.persistence.size=20Gi \
        --set metrics.enabled=true \
        --wait
    
    log_success "PostgreSQL deployed"
}

# Deploy Redis
deploy_redis() {
    log_info "Deploying Redis..."
    
    helm upgrade --install redis oci://registry-1.docker.io/bitnamicharts/redis \
        --namespace "$NAMESPACE" \
        --set auth.enabled=false \
        --set master.persistence.size=10Gi \
        --set metrics.enabled=true \
        --wait
    
    log_success "Redis deployed"
}

# Deploy monitoring stack
deploy_monitoring() {
    log_info "Deploying monitoring stack..."
    
    # Add Prometheus Helm repository
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update
    
    # Deploy Prometheus
    helm upgrade --install prometheus prometheus-community/kube-prometheus-stack \
        --namespace "$NAMESPACE" \
        --set grafana.adminPassword="$GRAFANA_PASSWORD" \
        --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=20Gi \
        --wait
    
    log_success "Monitoring stack deployed"
}

# Deploy logging stack
deploy_logging() {
    log_info "Deploying logging stack..."
    
    # Add Elastic Helm repository
    helm repo add elastic https://helm.elastic.co
    helm repo update
    
    # Deploy Elasticsearch
    helm upgrade --install elasticsearch elastic/elasticsearch \
        --namespace "$NAMESPACE" \
        --set replicas=1 \
        --set volumeClaimTemplate.resources.requests.storage=20Gi \
        --wait
    
    # Deploy Kibana
    helm upgrade --install kibana elastic/kibana \
        --namespace "$NAMESPACE" \
        --set elasticsearchHosts="http://elasticsearch-master:9200" \
        --wait
    
    # Deploy Logstash
    helm upgrade --install logstash elastic/logstash \
        --namespace "$NAMESPACE" \
        --wait
    
    log_success "Logging stack deployed"
}

# Deploy application services
deploy_services() {
    log_info "Deploying application services..."
    
    for service in "${SERVICES[@]}"; do
        deploy_service "$service"
    done
    
    log_success "All application services deployed"
}

# Deploy individual service
deploy_service() {
    local service="$1"
    log_info "Deploying service: $service"
    
    # Create ConfigMap
    kubectl create configmap "$service-config" \
        --namespace "$NAMESPACE" \
        --from-file="infrastructure/k8s/configs/$service.yaml" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create Secret
    kubectl create secret generic "$service-secret" \
        --namespace "$NAMESPACE" \
        --from-literal=database-url="$DATABASE_URL" \
        --from-literal=redis-url="$REDIS_URL" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Deploy service
    envsubst < "infrastructure/k8s/deployments/$service.yaml" | kubectl apply -f -
    
    # Deploy service
    envsubst < "infrastructure/k8s/services/$service.yaml" | kubectl apply -f -
    
    # Wait for deployment
    kubectl rollout status deployment/"$service" --namespace "$NAMESPACE" --timeout=300s
    
    log_success "Service $service deployed successfully"
}

# Deploy ingress
deploy_ingress() {
    if [ "$SKIP_INGRESS" = "true" ]; then
        log_warning "Skipping ingress deployment"
        return
    fi
    
    log_info "Deploying ingress..."
    
    # Deploy NGINX Ingress Controller
    helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
        --namespace "$NAMESPACE" \
        --set controller.service.type=LoadBalancer \
        --wait
    
    # Deploy application ingress
    envsubst < "infrastructure/k8s/ingress/app-ingress.yaml" | kubectl apply -f -
    
    log_success "Ingress deployed"
}

# Run health checks
run_health_checks() {
    log_info "Running health checks..."
    
    for service in "${SERVICES[@]}"; do
        check_service_health "$service"
    done
    
    log_success "All health checks passed"
}

# Check service health
check_service_health() {
    local service="$1"
    log_info "Checking health of service: $service"
    
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if kubectl get pods --namespace "$NAMESPACE" -l app="$service" --field-selector=status.phase=Running | grep -q "$service"; then
            log_success "Service $service is healthy"
            return 0
        fi
        
        log_info "Attempt $attempt/$max_attempts: Service $service not ready yet, waiting..."
        sleep 10
        ((attempt++))
    done
    
    log_error "Service $service failed health check"
    return 1
}

# Show deployment status
show_status() {
    log_info "Deployment Status:"
    echo ""
    
    # Show pods
    kubectl get pods --namespace "$NAMESPACE"
    echo ""
    
    # Show services
    kubectl get services --namespace "$NAMESPACE"
    echo ""
    
    # Show ingress
    kubectl get ingress --namespace "$NAMESPACE"
    echo ""
    
    # Show endpoints
    log_info "Service Endpoints:"
    for service in "${SERVICES[@]}"; do
        local endpoint=$(kubectl get service "$service" --namespace "$NAMESPACE" -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
        if [ -n "$endpoint" ]; then
            echo "  $service: http://$endpoint"
        else
            echo "  $service: Not available (check with 'kubectl get service $service -n $NAMESPACE')"
        fi
    done
}

# Rollback deployment
rollback() {
    log_info "Rolling back deployment..."
    
    for service in "${SERVICES[@]}"; do
        log_info "Rolling back service: $service"
        kubectl rollout undo deployment/"$service" --namespace "$NAMESPACE"
    done
    
    log_success "Rollback completed"
}

# Clean up deployment
cleanup() {
    log_info "Cleaning up deployment..."
    
    # Delete services
    for service in "${SERVICES[@]}"; do
        kubectl delete deployment "$service" --namespace "$NAMESPACE" --ignore-not-found=true
        kubectl delete service "$service" --namespace "$NAMESPACE" --ignore-not-found=true
        kubectl delete configmap "$service-config" --namespace "$NAMESPACE" --ignore-not-found=true
        kubectl delete secret "$service-secret" --namespace "$NAMESPACE" --ignore-not-found=true
    done
    
    # Delete ingress
    kubectl delete ingress app-ingress --namespace "$NAMESPACE" --ignore-not-found=true
    
    log_success "Cleanup completed"
}

# Main deployment function
main() {
    log_info "Starting deployment process for $PROJECT_NAME"
    log_info "Environment: $ENVIRONMENT"
    log_info "Namespace: $NAMESPACE"
    log_info "Version: $VERSION"
    log_info "Docker Registry: $DOCKER_REGISTRY"
    
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
            --version)
                VERSION="$2"
                shift 2
                ;;
            --registry)
                DOCKER_REGISTRY="$2"
                shift 2
                ;;
            --skip-database)
                SKIP_DATABASE="true"
                shift
                ;;
            --skip-redis)
                SKIP_REDIS="true"
                shift
                ;;
            --skip-ingress)
                SKIP_INGRESS="true"
                shift
                ;;
            --enable-monitoring)
                ENABLE_MONITORING="true"
                shift
                ;;
            --enable-logging)
                ENABLE_LOGGING="true"
                shift
                ;;
            --rollback)
                rollback
                exit 0
                ;;
            --cleanup)
                cleanup
                exit 0
                ;;
            --status)
                show_status
                exit 0
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --environment ENV     Set environment (default: dev)"
                echo "  --namespace NS        Set namespace (default: ddv)"
                echo "  --version VERSION     Set version (default: latest)"
                echo "  --registry REGISTRY   Set Docker registry (default: localhost:5000)"
                echo "  --skip-database       Skip database deployment"
                echo "  --skip-redis          Skip Redis deployment"
                echo "  --skip-ingress        Skip ingress deployment"
                echo "  --enable-monitoring   Enable monitoring stack"
                echo "  --enable-logging      Enable logging stack"
                echo "  --rollback            Rollback deployment"
                echo "  --cleanup             Clean up deployment"
                echo "  --status              Show deployment status"
                echo "  --help                Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Set default values
    POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-changeme}"
    DATABASE_NAME="${DATABASE_NAME:-ddv_db}"
    DATABASE_URL="${DATABASE_URL:-postgresql://postgres:$POSTGRES_PASSWORD@postgresql:5432/$DATABASE_NAME}"
    REDIS_URL="${REDIS_URL:-redis://redis-master:6379}"
    GRAFANA_PASSWORD="${GRAFANA_PASSWORD:-admin}"
    
    # Execute deployment steps
    check_dependencies
    check_k8s_connection
    create_namespace
    deploy_infrastructure
    deploy_services
    deploy_ingress
    run_health_checks
    show_status
    
    log_success "Deployment completed successfully!"
}

# Run main function
main "$@"
