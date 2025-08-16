#!/bin/bash
# Local CI checks - Run the same checks as GitHub Actions CI

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[CHECK]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Track if any check fails
FAILED=0

echo "========================================="
echo "       Local CI Checks for Dott         "
echo "========================================="
echo ""

# 1. Format check
print_status "Checking code formatting with rustfmt..."
if cargo fmt -- --check; then
    print_success "Code formatting check passed"
else
    print_error "Code formatting check failed - run 'cargo fmt' to fix"
    FAILED=1
fi
echo ""

# 2. Clippy check with CI flags
print_status "Running clippy with strict warnings..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "Clippy check passed"
else
    print_error "Clippy check failed - fix the warnings above"
    FAILED=1
fi
echo ""

# 3. Build check
print_status "Building project..."
if cargo build --all-features; then
    print_success "Build succeeded"
else
    print_error "Build failed"
    FAILED=1
fi
echo ""

# 4. Test check
print_status "Running tests..."
if cargo test --all-features; then
    print_success "All tests passed"
else
    print_error "Tests failed"
    FAILED=1
fi
echo ""

# 5. Test with no default features
print_status "Running tests with no default features..."
if cargo test --no-default-features; then
    print_success "Tests with no default features passed"
else
    print_error "Tests with no default features failed"
    FAILED=1
fi
echo ""

# 6. Documentation check
print_status "Building documentation..."
if cargo doc --all-features --no-deps --document-private-items; then
    print_success "Documentation build succeeded"
else
    print_error "Documentation build failed"
    FAILED=1
fi
echo ""

# 7. Check for security vulnerabilities (if cargo-audit is installed)
if command -v cargo-audit >/dev/null 2>&1; then
    print_status "Running security audit..."
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security vulnerabilities found"
        # Don't fail the build for audit issues as they might be in dependencies
    fi
else
    print_warning "cargo-audit not installed - skipping security audit"
    print_warning "Install with: cargo install cargo-audit"
fi
echo ""

# 8. Check MSRV (Minimum Supported Rust Version)
print_status "Checking MSRV (1.75.0)..."
CURRENT_RUST_VERSION=$(rustc --version | cut -d' ' -f2)
MSRV="1.75.0"

# Simple version comparison (may need refinement for edge cases)
if [ "$(printf '%s\n' "$MSRV" "$CURRENT_RUST_VERSION" | sort -V | head -n1)" = "$MSRV" ]; then
    print_success "Rust version $CURRENT_RUST_VERSION meets MSRV $MSRV"
else
    print_warning "Rust version $CURRENT_RUST_VERSION is older than MSRV $MSRV"
fi
echo ""

# Summary
echo "========================================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All CI checks passed!${NC} ✨"
    echo "Your code is ready for commit and push."
else
    echo -e "${RED}Some CI checks failed!${NC}"
    echo "Please fix the issues above before pushing."
    exit 1
fi
echo "========================================="