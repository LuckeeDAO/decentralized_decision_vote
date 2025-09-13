#!/bin/bash

# Build script for decentralized decision vote system
# This script builds all services and creates Docker images

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="decentralized-decision-vote"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-localhost:5000}"
VERSION="${VERSION:-latest}"
BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Services to build
SERVICES=(
    "services/vote-api"
    "services/notification-service"
    "services/admin-api"
    "clients/cli"
)

# Core libraries to build
CORE_LIBS=(
    "core/vote-engine"
    "core/template-system"
    "core/commitment-engine"
    "shared/types"
    "shared/config"
    "shared/utils"
    "shared/logging"
    "storage/vote-store"
    "storage/config-store"
    "storage/event-store"
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
    
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi
    
    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi
    
    if ! command -v git &> /dev/null; then
        missing_deps+=("git")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install the missing dependencies and try again."
        exit 1
    fi
    
    log_success "All dependencies are installed"
}

# Clean build artifacts
clean() {
    log_info "Cleaning build artifacts..."
    
    # Clean Rust build artifacts
    cargo clean
    
    # Clean Docker images
    if [ "$CLEAN_DOCKER" = "true" ]; then
        log_info "Cleaning Docker images..."
        docker system prune -f
    fi
    
    log_success "Build artifacts cleaned"
}

# Build Rust workspace
build_rust() {
    log_info "Building Rust workspace..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Build the workspace
    if [ "$RELEASE" = "true" ]; then
        log_info "Building in release mode..."
        cargo build --release
    else
        log_info "Building in debug mode..."
        cargo build
    fi
    
    log_success "Rust workspace built successfully"
}

# Run tests
run_tests() {
    if [ "$SKIP_TESTS" = "true" ]; then
        log_warning "Skipping tests"
        return
    fi
    
    log_info "Running tests..."
    
    # Run unit tests
    cargo test --lib
    
    # Run integration tests
    cargo test --test '*'
    
    log_success "All tests passed"
}

# Build Docker images
build_docker_images() {
    log_info "Building Docker images..."
    
    for service in "${SERVICES[@]}"; do
        if [ -f "$service/Dockerfile" ]; then
            log_info "Building Docker image for $service..."
            
            local image_name="$DOCKER_REGISTRY/$PROJECT_NAME-$(basename $service):$VERSION"
            local build_args="--build-arg BUILD_DATE=$BUILD_DATE --build-arg GIT_COMMIT=$GIT_COMMIT"
            
            if [ "$RELEASE" = "true" ]; then
                build_args="$build_args --build-arg BUILD_MODE=release"
            else
                build_args="$build_args --build-arg BUILD_MODE=debug"
            fi
            
            docker build $build_args -t "$image_name" "$service/"
            
            if [ "$PUSH_IMAGES" = "true" ]; then
                log_info "Pushing image $image_name..."
                docker push "$image_name"
            fi
            
            log_success "Built image: $image_name"
        else
            log_warning "No Dockerfile found for $service, skipping..."
        fi
    done
    
    log_success "All Docker images built successfully"
}

# Build CLI binary
build_cli() {
    log_info "Building CLI binary..."
    
    cd clients/cli
    
    if [ "$RELEASE" = "true" ]; then
        cargo build --release
        cp target/release/ddv ../../bin/
    else
        cargo build
        cp target/debug/ddv ../../bin/
    fi
    
    cd ../..
    
    log_success "CLI binary built successfully"
}

# Generate documentation
generate_docs() {
    if [ "$SKIP_DOCS" = "true" ]; then
        log_warning "Skipping documentation generation"
        return
    fi
    
    log_info "Generating documentation..."
    
    cargo doc --no-deps --document-private-items
    
    log_success "Documentation generated"
}

# Create release package
create_release_package() {
    if [ "$RELEASE" != "true" ]; then
        return
    fi
    
    log_info "Creating release package..."
    
    local release_dir="releases/$VERSION"
    mkdir -p "$release_dir"
    
    # Copy binaries
    if [ -d "bin" ]; then
        cp -r bin "$release_dir/"
    fi
    
    # Copy Docker images
    for service in "${SERVICES[@]}"; do
        local image_name="$DOCKER_REGISTRY/$PROJECT_NAME-$(basename $service):$VERSION"
        docker save "$image_name" | gzip > "$release_dir/$(basename $service).tar.gz"
    done
    
    # Copy documentation
    if [ -d "target/doc" ]; then
        cp -r target/doc "$release_dir/"
    fi
    
    # Create release notes
    cat > "$release_dir/RELEASE_NOTES.md" << EOF
# Release $VERSION

## Build Information
- Build Date: $BUILD_DATE
- Git Commit: $GIT_COMMIT
- Build Mode: Release

## Services
$(for service in "${SERVICES[@]}"; do echo "- $(basename $service)"; done)

## Installation
See the documentation in the doc/ directory for installation instructions.

## Docker Images
$(for service in "${SERVICES[@]}"; do echo "- $DOCKER_REGISTRY/$PROJECT_NAME-$(basename $service):$VERSION"; done)
EOF
    
    log_success "Release package created at $release_dir"
}

# Main build function
main() {
    log_info "Starting build process for $PROJECT_NAME"
    log_info "Version: $VERSION"
    log_info "Docker Registry: $DOCKER_REGISTRY"
    log_info "Build Date: $BUILD_DATE"
    log_info "Git Commit: $GIT_COMMIT"
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                CLEAN="true"
                shift
                ;;
            --clean-docker)
                CLEAN_DOCKER="true"
                shift
                ;;
            --release)
                RELEASE="true"
                shift
                ;;
            --skip-tests)
                SKIP_TESTS="true"
                shift
                ;;
            --skip-docs)
                SKIP_DOCS="true"
                shift
                ;;
            --push-images)
                PUSH_IMAGES="true"
                shift
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --registry)
                DOCKER_REGISTRY="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --clean              Clean build artifacts"
                echo "  --clean-docker       Clean Docker images"
                echo "  --release            Build in release mode"
                echo "  --skip-tests         Skip running tests"
                echo "  --skip-docs          Skip documentation generation"
                echo "  --push-images        Push Docker images to registry"
                echo "  --version VERSION    Set version (default: latest)"
                echo "  --registry REGISTRY  Set Docker registry (default: localhost:5000)"
                echo "  --help               Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Create necessary directories
    mkdir -p bin
    mkdir -p releases
    
    # Execute build steps
    check_dependencies
    
    if [ "$CLEAN" = "true" ]; then
        clean
    fi
    
    build_rust
    run_tests
    build_docker_images
    build_cli
    generate_docs
    
    if [ "$RELEASE" = "true" ]; then
        create_release_package
    fi
    
    log_success "Build completed successfully!"
    
    # Show summary
    echo ""
    log_info "Build Summary:"
    echo "  Version: $VERSION"
    echo "  Build Mode: $([ "$RELEASE" = "true" ] && echo "Release" || echo "Debug")"
    echo "  Docker Registry: $DOCKER_REGISTRY"
    echo "  Git Commit: $GIT_COMMIT"
    echo ""
    
    if [ "$RELEASE" = "true" ]; then
        log_info "Release package created at: releases/$VERSION"
    fi
}

# Run main function
main "$@"
