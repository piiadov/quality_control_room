#!/usr/bin/env bash
#
# install.sh - Build and install xgbwrapper with XGBoost dependency
#
# Usage:
#   ./install.sh           # Build release version
#   ./install.sh --debug   # Build debug version
#   ./install.sh --clean   # Clean build directories and rebuild
#   ./install.sh --help    # Show this help
#
# Requirements:
#   - CMake >= 3.14
#   - GCC/G++ with C11/C++17 support
#   - Git
#

set -e  # Exit on error

# ==============================================================================
# Configuration
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LIB_DIR="$PROJECT_ROOT/lib"
XGBOOST_DIR="$PROJECT_ROOT/xgboost"
XGBOOST_REPO="https://github.com/dmlc/xgboost.git"
XGBOOST_VERSION="v3.0.0"  # Stable release

# Build configuration
BUILD_TYPE="release"
CLEAN_BUILD=0
JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ==============================================================================
# Helper Functions
# ==============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

die() {
    log_error "$1"
    exit 1
}

show_help() {
    cat << EOF
xgbwrapper Installation Script

Usage: $0 [OPTIONS]

Options:
    --debug     Build debug version (default: release)
    --clean     Clean build directories before building
    --jobs N    Number of parallel build jobs (default: $JOBS)
    --help      Show this help message

Examples:
    $0                  # Build release version
    $0 --debug          # Build debug version
    $0 --clean --debug  # Clean and build debug version

EOF
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing=()
    
    command -v git >/dev/null 2>&1 || missing+=("git")
    command -v cmake >/dev/null 2>&1 || missing+=("cmake")
    command -v gcc >/dev/null 2>&1 || missing+=("gcc")
    command -v g++ >/dev/null 2>&1 || missing+=("g++")
    command -v make >/dev/null 2>&1 || missing+=("make")
    
    if [ ${#missing[@]} -ne 0 ]; then
        die "Missing required tools: ${missing[*]}\nInstall with: sudo dnf install -y ${missing[*]} (Fedora) or sudo apt install -y ${missing[*]} (Ubuntu)"
    fi
    
    log_success "All dependencies found"
}

# ==============================================================================
# XGBoost Build
# ==============================================================================

clone_xgboost() {
    if [ -d "$XGBOOST_DIR" ]; then
        log_info "XGBoost directory exists, checking version..."
        cd "$XGBOOST_DIR"
        
        local current_tag=$(git describe --tags --exact-match 2>/dev/null || echo "unknown")
        if [ "$current_tag" = "$XGBOOST_VERSION" ]; then
            log_success "XGBoost $XGBOOST_VERSION already cloned"
            return 0
        else
            log_warn "XGBoost version mismatch (found: $current_tag, expected: $XGBOOST_VERSION)"
            log_info "Updating submodules..."
            git submodule update --init --recursive
        fi
    else
        log_info "Cloning XGBoost $XGBOOST_VERSION..."
        git clone --branch "$XGBOOST_VERSION" --recurse-submodules --depth 1 \
            "$XGBOOST_REPO" "$XGBOOST_DIR" || die "Failed to clone XGBoost"
        log_success "XGBoost cloned successfully"
    fi
}

build_xgboost() {
    log_info "Building XGBoost shared library..."
    
    local xgb_build_dir="$XGBOOST_DIR/build"
    
    if [ $CLEAN_BUILD -eq 1 ] && [ -d "$xgb_build_dir" ]; then
        log_info "Cleaning XGBoost build directory..."
        rm -rf "$xgb_build_dir"
    fi
    
    mkdir -p "$xgb_build_dir"
    cd "$xgb_build_dir"
    
    # Configure XGBoost
    cmake .. \
        -DCMAKE_BUILD_TYPE=Release \
        -DBUILD_SHARED_LIBS=ON \
        -DUSE_OPENMP=ON \
        -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
        || die "XGBoost CMake configuration failed"
    
    # Build
    cmake --build . --parallel "$JOBS" || die "XGBoost build failed"
    
    log_success "XGBoost built successfully"
}

install_xgboost_libs() {
    log_info "Installing XGBoost libraries to $LIB_DIR..."
    
    mkdir -p "$LIB_DIR"
    
    # XGBoost may place library in different locations depending on version
    local xgb_lib=""
    local search_paths=(
        "$XGBOOST_DIR/lib/libxgboost.so"
        "$XGBOOST_DIR/build/lib/libxgboost.so"
        "$XGBOOST_DIR/build/libxgboost.so"
    )
    
    for path in "${search_paths[@]}"; do
        if [ -f "$path" ]; then
            xgb_lib="$path"
            break
        fi
    done
    
    if [ -z "$xgb_lib" ]; then
        log_error "Cannot find libxgboost.so in any of:"
        for path in "${search_paths[@]}"; do
            log_error "  - $path"
        done
        die "XGBoost library not found"
    fi
    
    cp -v "$xgb_lib" "$LIB_DIR/"
    log_success "Copied $(basename "$xgb_lib") from $xgb_lib"
    
    # Find and copy DMLC library (optional, may be statically linked)
    local dmlc_lib=""
    local dmlc_paths=(
        "$XGBOOST_DIR/lib/libdmlc.so"
        "$XGBOOST_DIR/build/lib/libdmlc.so"
        "$XGBOOST_DIR/build/dmlc-core/libdmlc.so"
    )
    
    for path in "${dmlc_paths[@]}"; do
        if [ -f "$path" ]; then
            dmlc_lib="$path"
            break
        fi
    done
    
    if [ -n "$dmlc_lib" ]; then
        cp -v "$dmlc_lib" "$LIB_DIR/"
        log_success "Copied libdmlc.so"
    else
        log_warn "libdmlc.so not found (may be statically linked)"
    fi
    
    log_success "XGBoost libraries installed"
}

# ==============================================================================
# xgbwrapper Build
# ==============================================================================

build_xgbwrapper() {
    log_info "Building xgbwrapper ($BUILD_TYPE)..."
    
    cd "$SCRIPT_DIR"
    
    local preset="$BUILD_TYPE"
    local build_dir="$SCRIPT_DIR/build/$BUILD_TYPE"
    
    if [ $CLEAN_BUILD -eq 1 ] && [ -d "$build_dir" ]; then
        log_info "Cleaning xgbwrapper build directory..."
        rm -rf "$build_dir"
    fi
    
    # Check if presets are available
    if [ -f "$SCRIPT_DIR/CMakePresets.json" ]; then
        log_info "Using CMake preset: $preset"
        cmake --preset "$preset" || die "xgbwrapper CMake configuration failed"
        cmake --build --preset "$preset" --parallel "$JOBS" || die "xgbwrapper build failed"
    else
        # Fallback to manual CMake configuration
        log_warn "CMakePresets.json not found, using manual configuration"
        mkdir -p "$build_dir"
        cd "$build_dir"
        
        local cmake_build_type="Release"
        [ "$BUILD_TYPE" = "debug" ] && cmake_build_type="Debug"
        
        cmake "$SCRIPT_DIR" \
            -DCMAKE_BUILD_TYPE="$cmake_build_type" \
            -DXGBWRAPPER_BUILD_TESTS=ON \
            || die "xgbwrapper CMake configuration failed"
        
        cmake --build . --parallel "$JOBS" || die "xgbwrapper build failed"
    fi
    
    log_success "xgbwrapper built successfully"
}

run_tests() {
    log_info "Running xgbwrapper tests..."
    
    cd "$SCRIPT_DIR"
    
    if [ -f "$SCRIPT_DIR/CMakePresets.json" ]; then
        ctest --preset "$BUILD_TYPE" --output-on-failure || die "Tests failed"
    else
        cd "$SCRIPT_DIR/build/$BUILD_TYPE"
        ctest --output-on-failure || die "Tests failed"
    fi
    
    log_success "All tests passed"
}

# ==============================================================================
# Summary
# ==============================================================================

print_summary() {
    echo ""
    echo "=============================================="
    echo -e "${GREEN}Installation Complete!${NC}"
    echo "=============================================="
    echo ""
    echo "Libraries installed to: $LIB_DIR"
    echo ""
    ls -lh "$LIB_DIR"/*.so* 2>/dev/null || true
    echo ""
    echo "To use xgbwrapper in Quality Control Room project:"
    echo "  1. Add to LD_LIBRARY_PATH:"
    echo "     export LD_LIBRARY_PATH=\"$LIB_DIR:\$LD_LIBRARY_PATH\""
    echo ""
    echo "  2. Or add to /etc/ld.so.conf.d/ and run ldconfig"
    echo ""
    echo "  3. For Rust FFI, set in build.rs:"
    echo "     println!(\"cargo:rustc-link-search=$LIB_DIR\");"
    echo "     println!(\"cargo:rustc-link-lib=xgbwrapper\");"
    echo ""
}

# ==============================================================================
# Main
# ==============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --debug)
                BUILD_TYPE="debug"
                shift
                ;;
            --clean)
                CLEAN_BUILD=1
                shift
                ;;
            --jobs)
                JOBS="$2"
                shift 2
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    echo "=============================================="
    echo "xgbwrapper Installation Script"
    echo "=============================================="
    echo "Build type: $BUILD_TYPE"
    echo "Parallel jobs: $JOBS"
    echo "Project root: $PROJECT_ROOT"
    echo "=============================================="
    echo ""
    
    check_dependencies
    
    # Build XGBoost if not already installed
    if [ ! -f "$LIB_DIR/libxgboost.so" ] || [ $CLEAN_BUILD -eq 1 ]; then
        clone_xgboost
        build_xgboost
        install_xgboost_libs
    else
        log_success "XGBoost library already installed, skipping (use --clean to rebuild)"
    fi
    
    # Build xgbwrapper
    build_xgbwrapper
    run_tests
    
    print_summary
}

main "$@"
