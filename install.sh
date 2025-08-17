#!/bin/bash
# Dotf installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/k1-c/dotf/main/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="k1-c"
REPO_NAME="dotf"
BINARY_NAME="dotf"
INSTALL_DIR="${DOTF_INSTALL_DIR:-$HOME/.local/bin}"

# Functions
print_error() {
    echo -e "${RED}Error: $1${NC}" >&2
}

print_success() {
    echo -e "${GREEN}$1${NC}"
}

print_info() {
    echo -e "${BLUE}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}$1${NC}"
}

# Detect OS and architecture
detect_platform() {
    local os=""
    local arch=""
    
    # Detect OS
    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="macos"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            if [ "$os" = "macos" ]; then
                arch="aarch64"
            else
                arch="aarch64"
            fi
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${os}-${arch}"
}

# Get the latest release version
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Download the binary
download_binary() {
    local version="$1"
    local platform="$2"
    local download_url=""
    local asset_name=""
    
    # Determine asset name based on platform
    case "$platform" in
        linux-x86_64)
            asset_name="${BINARY_NAME}-linux-x86_64"
            ;;
        linux-aarch64)
            asset_name="${BINARY_NAME}-linux-aarch64"
            ;;
        macos-x86_64)
            asset_name="${BINARY_NAME}-macos-x86_64"
            ;;
        macos-aarch64)
            asset_name="${BINARY_NAME}-macos-aarch64"
            ;;
        *)
            print_error "Unknown platform: $platform"
            exit 1
            ;;
    esac
    
    download_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${asset_name}"
    
    print_info "Downloading ${BINARY_NAME} ${version} for ${platform}..."
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap 'rm -rf '"$temp_dir" EXIT
    
    local temp_file="${temp_dir}/${BINARY_NAME}"
    
    # Download the binary
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL -o "$temp_file" "$download_url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$temp_file" "$download_url"
    fi
    
    if [ ! -f "$temp_file" ]; then
        print_error "Failed to download binary"
        exit 1
    fi
    
    # Make binary executable
    chmod +x "$temp_file"
    
    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"
    
    # Move binary to install directory
    mv "$temp_file" "${INSTALL_DIR}/${BINARY_NAME}"
    
    print_success "Successfully installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

# Check if install directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        print_warning "\nWarning: ${INSTALL_DIR} is not in your PATH."
        print_info "Add the following line to your shell configuration file (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        print_info "Then reload your shell configuration:"
        echo "  source ~/.bashrc  # or ~/.zshrc"
        echo ""
    fi
}

# Main installation flow
main() {
    print_info "Installing ${BINARY_NAME}..."
    echo ""
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    print_info "Detected platform: ${platform}"
    
    # Get latest version
    local version
    version=$(get_latest_version)
    
    if [ -z "$version" ]; then
        print_error "Failed to get latest version"
        exit 1
    fi
    
    print_info "Latest version: ${version}"
    
    # Check if already installed
    if [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        local current_version
        current_version=$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>/dev/null | awk '{print $2}' || echo "unknown")
        print_warning "Found existing installation (version: ${current_version})"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Installation cancelled"
            exit 0
        fi
    fi
    
    # Download and install
    download_binary "$version" "$platform"
    
    # Check PATH
    check_path
    
    # Verify installation
    if "${INSTALL_DIR}/${BINARY_NAME}" --version >/dev/null 2>&1; then
        print_success "\n✨ ${BINARY_NAME} has been successfully installed!"
        echo ""
        "${INSTALL_DIR}/${BINARY_NAME}" --version
        echo ""
        print_info "Get started with:"
        echo "  ${BINARY_NAME} --help"
    else
        print_error "Installation verification failed"
        exit 1
    fi
}

# Run main function
main "$@"