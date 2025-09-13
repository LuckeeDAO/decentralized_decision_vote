#!/bin/bash

# Test script for decentralized decision vote system
# This script runs various tests to ensure system quality

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="decentralized-decision-vote"
TEST_TIMEOUT="${TEST_TIMEOUT:-300}"
COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-80}"

# Test types
UNIT_TESTS=true
INTEGRATION_TESTS=true
E2E_TESTS=true
PERFORMANCE_TESTS=false
SECURITY_TESTS=true

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
    
    if ! command -v kubectl &> /dev/null; then
        missing_deps+=("kubectl")
    fi
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install the missing dependencies and try again."
        exit 1
    fi
    
    log_success "All dependencies are installed"
}

# Run unit tests
run_unit_tests() {
    if [ "$UNIT_TESTS" != "true" ]; then
        log_warning "Skipping unit tests"
        return
    fi
    
    log_info "Running unit tests..."
    
    # Run Rust unit tests
    cargo test --lib --bins
    
    # Run tests with coverage
    if command -v cargo-tarpaulin &> /dev/null; then
        log_info "Running tests with coverage..."
        cargo tarpaulin --out Html --output-dir coverage/
        
        # Check coverage threshold
        local coverage=$(cargo tarpaulin --out Stdout | grep -o '[0-9]*\.[0-9]*%' | head -n 1 | sed 's/%//')
        if (( $(echo "$coverage < $COVERAGE_THRESHOLD" | bc -l) )); then
            log_error "Coverage $coverage% is below threshold $COVERAGE_THRESHOLD%"
            exit 1
        fi
        log_success "Coverage $coverage% meets threshold $COVERAGE_THRESHOLD%"
    fi
    
    log_success "Unit tests completed"
}

# Run integration tests
run_integration_tests() {
    if [ "$INTEGRATION_TESTS" != "true" ]; then
        log_warning "Skipping integration tests"
        return
    fi
    
    log_info "Running integration tests..."
    
    # Start test database
    start_test_database
    
    # Run integration tests
    cargo test --test '*'
    
    # Stop test database
    stop_test_database
    
    log_success "Integration tests completed"
}

# Start test database
start_test_database() {
    log_info "Starting test database..."
    
    docker run -d \
        --name test-postgres \
        -e POSTGRES_DB=test_db \
        -e POSTGRES_USER=test_user \
        -e POSTGRES_PASSWORD=test_pass \
        -p 5433:5432 \
        postgres:15
    
    # Wait for database to be ready
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if docker exec test-postgres pg_isready -U test_user -d test_db &> /dev/null; then
            log_success "Test database is ready"
            return 0
        fi
        
        log_info "Attempt $attempt/$max_attempts: Database not ready yet, waiting..."
        sleep 2
        ((attempt++))
    done
    
    log_error "Test database failed to start"
    exit 1
}

# Stop test database
stop_test_database() {
    log_info "Stopping test database..."
    
    docker stop test-postgres &> /dev/null || true
    docker rm test-postgres &> /dev/null || true
    
    log_success "Test database stopped"
}

# Run end-to-end tests
run_e2e_tests() {
    if [ "$E2E_TESTS" != "true" ]; then
        log_warning "Skipping end-to-end tests"
        return
    fi
    
    log_info "Running end-to-end tests..."
    
    # Start test environment
    start_test_environment
    
    # Run E2E tests
    if [ -d "tests/e2e" ]; then
        cd tests/e2e
        cargo test
        cd ../..
    fi
    
    # Stop test environment
    stop_test_environment
    
    log_success "End-to-end tests completed"
}

# Start test environment
start_test_environment() {
    log_info "Starting test environment..."
    
    # Start services with docker-compose
    if [ -f "infrastructure/docker/docker-compose.test.yml" ]; then
        docker-compose -f infrastructure/docker/docker-compose.test.yml up -d
        
        # Wait for services to be ready
        wait_for_services
    else
        log_warning "No test docker-compose file found, skipping E2E tests"
        E2E_TESTS=false
    fi
}

# Stop test environment
stop_test_environment() {
    log_info "Stopping test environment..."
    
    if [ -f "infrastructure/docker/docker-compose.test.yml" ]; then
        docker-compose -f infrastructure/docker/docker-compose.test.yml down
    fi
    
    log_success "Test environment stopped"
}

# Wait for services to be ready
wait_for_services() {
    log_info "Waiting for services to be ready..."
    
    local services=("vote-api:8080" "notification-service:8082" "admin-api:8081")
    
    for service in "${services[@]}"; do
        local name=$(echo $service | cut -d: -f1)
        local port=$(echo $service | cut -d: -f2)
        
        local max_attempts=60
        local attempt=1
        
        while [ $attempt -le $max_attempts ]; do
            if curl -s "http://localhost:$port/health" &> /dev/null; then
                log_success "Service $name is ready"
                break
            fi
            
            if [ $attempt -eq $max_attempts ]; then
                log_error "Service $name failed to start"
                exit 1
            fi
            
            log_info "Attempt $attempt/$max_attempts: Service $name not ready yet, waiting..."
            sleep 5
            ((attempt++))
        done
    done
}

# Run performance tests
run_performance_tests() {
    if [ "$PERFORMANCE_TESTS" != "true" ]; then
        log_warning "Skipping performance tests"
        return
    fi
    
    log_info "Running performance tests..."
    
    # Start performance test environment
    start_test_environment
    
    # Run performance tests
    if [ -d "tests/performance" ]; then
        cd tests/performance
        cargo test
        cd ../..
    fi
    
    # Stop test environment
    stop_test_environment
    
    log_success "Performance tests completed"
}

# Run security tests
run_security_tests() {
    if [ "$SECURITY_TESTS" != "true" ]; then
        log_warning "Skipping security tests"
        return
    fi
    
    log_info "Running security tests..."
    
    # Run cargo audit
    if command -v cargo-audit &> /dev/null; then
        log_info "Running cargo audit..."
        cargo audit
    else
        log_warning "cargo-audit not installed, skipping security audit"
    fi
    
    # Run clippy security checks
    log_info "Running clippy security checks..."
    cargo clippy -- -D warnings
    
    log_success "Security tests completed"
}

# Run code quality checks
run_code_quality_checks() {
    log_info "Running code quality checks..."
    
    # Format check
    log_info "Checking code formatting..."
    cargo fmt -- --check
    
    # Clippy check
    log_info "Running clippy..."
    cargo clippy -- -D warnings
    
    # Dead code check
    log_info "Checking for dead code..."
    cargo check --all-targets
    
    log_success "Code quality checks completed"
}

# Run API tests
run_api_tests() {
    log_info "Running API tests..."
    
    # Start test environment
    start_test_environment
    
    # Test vote API
    test_vote_api
    
    # Test notification service
    test_notification_service
    
    # Test admin API
    test_admin_api
    
    # Stop test environment
    stop_test_environment
    
    log_success "API tests completed"
}

# Test vote API
test_vote_api() {
    log_info "Testing vote API..."
    
    local base_url="http://localhost:8080"
    
    # Test health endpoint
    curl -f "$base_url/health" || {
        log_error "Vote API health check failed"
        exit 1
    }
    
    # Test session creation
    local session_response=$(curl -s -X POST "$base_url/sessions" \
        -H "Content-Type: application/json" \
        -d '{"name":"test-session","description":"Test session"}')
    
    if [ -z "$session_response" ]; then
        log_error "Vote API session creation failed"
        exit 1
    fi
    
    log_success "Vote API tests passed"
}

# Test notification service
test_notification_service() {
    log_info "Testing notification service..."
    
    local base_url="http://localhost:8082"
    
    # Test health endpoint
    curl -f "$base_url/health" || {
        log_error "Notification service health check failed"
        exit 1
    }
    
    log_success "Notification service tests passed"
}

# Test admin API
test_admin_api() {
    log_info "Testing admin API..."
    
    local base_url="http://localhost:8081"
    
    # Test health endpoint
    curl -f "$base_url/health" || {
        log_error "Admin API health check failed"
        exit 1
    }
    
    log_success "Admin API tests passed"
}

# Generate test report
generate_test_report() {
    log_info "Generating test report..."
    
    local report_dir="test-reports"
    mkdir -p "$report_dir"
    
    # Generate HTML report
    cat > "$report_dir/index.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>Test Report - $PROJECT_NAME</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; }
        .success { color: green; }
        .error { color: red; }
        .warning { color: orange; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Test Report - $PROJECT_NAME</h1>
        <p>Generated on: $(date)</p>
        <p>Version: $VERSION</p>
    </div>
    
    <div class="section">
        <h2>Test Summary</h2>
        <ul>
            <li>Unit Tests: $([ "$UNIT_TESTS" = "true" ] && echo '<span class="success">✓ Passed</span>' || echo '<span class="warning">⚠ Skipped</span>')</li>
            <li>Integration Tests: $([ "$INTEGRATION_TESTS" = "true" ] && echo '<span class="success">✓ Passed</span>' || echo '<span class="warning">⚠ Skipped</span>')</li>
            <li>E2E Tests: $([ "$E2E_TESTS" = "true" ] && echo '<span class="success">✓ Passed</span>' || echo '<span class="warning">⚠ Skipped</span>')</li>
            <li>Performance Tests: $([ "$PERFORMANCE_TESTS" = "true" ] && echo '<span class="success">✓ Passed</span>' || echo '<span class="warning">⚠ Skipped</span>')</li>
            <li>Security Tests: $([ "$SECURITY_TESTS" = "true" ] && echo '<span class="success">✓ Passed</span>' || echo '<span class="warning">⚠ Skipped</span>')</li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Coverage Report</h2>
        <p>Coverage threshold: $COVERAGE_THRESHOLD%</p>
        <p><a href="coverage/index.html">View detailed coverage report</a></p>
    </div>
</body>
</html>
EOF
    
    log_success "Test report generated at $report_dir/index.html"
}

# Main test function
main() {
    log_info "Starting test process for $PROJECT_NAME"
    log_info "Version: $VERSION"
    log_info "Test timeout: $TEST_TIMEOUT seconds"
    log_info "Coverage threshold: $COVERAGE_THRESHOLD%"
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-unit)
                UNIT_TESTS=false
                shift
                ;;
            --skip-integration)
                INTEGRATION_TESTS=false
                shift
                ;;
            --skip-e2e)
                E2E_TESTS=false
                shift
                ;;
            --skip-performance)
                PERFORMANCE_TESTS=false
                shift
                ;;
            --skip-security)
                SECURITY_TESTS=false
                shift
                ;;
            --enable-performance)
                PERFORMANCE_TESTS=true
                shift
                ;;
            --coverage-threshold)
                COVERAGE_THRESHOLD="$2"
                shift 2
                ;;
            --timeout)
                TEST_TIMEOUT="$2"
                shift 2
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --skip-unit              Skip unit tests"
                echo "  --skip-integration       Skip integration tests"
                echo "  --skip-e2e               Skip end-to-end tests"
                echo "  --skip-performance       Skip performance tests"
                echo "  --skip-security          Skip security tests"
                echo "  --enable-performance     Enable performance tests"
                echo "  --coverage-threshold N   Set coverage threshold (default: 80)"
                echo "  --timeout N              Set test timeout in seconds (default: 300)"
                echo "  --version VERSION        Set version (default: latest)"
                echo "  --help                   Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Execute test steps
    check_dependencies
    run_code_quality_checks
    run_unit_tests
    run_integration_tests
    run_e2e_tests
    run_performance_tests
    run_security_tests
    run_api_tests
    generate_test_report
    
    log_success "All tests completed successfully!"
    
    # Show summary
    echo ""
    log_info "Test Summary:"
    echo "  Unit Tests: $([ "$UNIT_TESTS" = "true" ] && echo "✓ Passed" || echo "⚠ Skipped")"
    echo "  Integration Tests: $([ "$INTEGRATION_TESTS" = "true" ] && echo "✓ Passed" || echo "⚠ Skipped")"
    echo "  E2E Tests: $([ "$E2E_TESTS" = "true" ] && echo "✓ Passed" || echo "⚠ Skipped")"
    echo "  Performance Tests: $([ "$PERFORMANCE_TESTS" = "true" ] && echo "✓ Passed" || echo "⚠ Skipped")"
    echo "  Security Tests: $([ "$SECURITY_TESTS" = "true" ] && echo "✓ Passed" || echo "⚠ Skipped")"
    echo ""
    log_info "Test report generated at: test-reports/index.html"
}

# Run main function
main "$@"
