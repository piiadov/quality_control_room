# Quality Control Room - Build Instructions

Complete guide for building all components on Debian 13.

## System Requirements

- **OS**: Debian 13 (Trixie)
- **Architecture**: x86_64
- **Disk Space**: ~500 MB for dependencies + build artifacts
- **Memory**: 2 GB minimum recommended

## Prerequisites Installation

### 1. System Dependencies

```bash
# Update package lists
sudo apt update

# Install build essentials and dependencies
sudo apt install -y \
  cmake \
  build-essential \
  pkg-config \
  libssl-dev \
  curl \
  git
```

**Installed versions (as of build):**
- CMake 3.31.6
- GCC 14.2.0
- pkg-config 1.8.1
- libssl-dev 3.5.1

### 2. Rust Installation

Rust is required for the engine component.

```bash
# If not installed, use rustup (official installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add Cargo to PATH (if newly installed)
source ~/.cargo/env

# Verify installation
rustc --version  # Should be 1.90.0 or newer
cargo --version
```

**Current version:** rustc 1.90.0

### 3. Node.js Installation

Node.js is required for the UI component.

```bash
# Add NodeSource repository for Node.js 20 LTS
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo bash -

# Install Node.js
sudo apt install -y nodejs

# Verify installation
node --version  # Should be v20.19.5 or newer
npm --version   # Should be 10.8.2 or newer
```

**Current versions:**
- Node.js v20.19.5
- npm 10.8.2

### 4. XGBoost Headers

The xgbwrapper needs XGBoost header files (the library is already in `lib/`).

```bash
# Clone XGBoost repository for headers (shallow clone)
cd /home/vp
git clone --depth 1 --branch master https://github.com/dmlc/xgboost.git xgboost-headers
```

**Note:** The build expects headers at `/home/vp/xgboost-headers/include/`

## Building Components

### Component 1: xgbwrapper (C Library)

The C wrapper around XGBoost for data manipulation and model training.

```bash
cd /home/vp/quality_control_room/xgbwrapper

# Create build directory
mkdir -p build
cd build

# Configure with CMake
cmake ..

# Compile
make

# Verify build
ls -lh lib/libxgbwrapper.so
ls -lh test_xgbwrapper
```

**Output:**
- `lib/libxgbwrapper.so` - Shared library (automatically copied to `/home/vp/quality_control_room/lib/`)
- `test_xgbwrapper` - Test executable

**Build time:** ~5 seconds

### Component 2: Engine (Rust Workspace)

The core processing engine with models and web server.

```bash
cd /home/vp/quality_control_room/engine

# Build all workspace members in release mode
cargo build --release
```

**Build includes:**
- `models` crate - Statistical models and ML integration
- `server` crate - Warp-based web API server
- `tests` crate - Integration tests

**Output location:** `engine/target/release/`

**Build time:** ~1.5 minutes (first build)

**Note:** You may see 5 warnings about unused imports in the tests module. These are non-critical.

### Component 3: UI (Vue.js Application)

The web frontend dashboard.

```bash
cd /home/vp/quality_control_room/ui

# Install dependencies
npm install

# Build for production
npm run build
```

**Output:** `ui/dist/` directory with production-ready static files

**Build time:** ~10 seconds (after npm install)

**Note:** 
- npm install takes ~8 seconds and installs 273 packages
- There may be 1 moderate security vulnerability - run `npm audit fix` if needed
- CSS minifier may show a warning about syntax - this is non-critical

## Verification

### Check All Built Artifacts

```bash
# C library
ls -lh /home/vp/quality_control_room/lib/libxgbwrapper.so

# Rust binaries (if main executables exist)
ls -lh /home/vp/quality_control_room/engine/target/release/

# UI distribution
ls -lh /home/vp/quality_control_room/ui/dist/index.html
```

### Run Tests (Optional)

```bash
# Test xgbwrapper
cd /home/vp/quality_control_room/xgbwrapper/build
ctest

# Test Rust components
cd /home/vp/quality_control_room/engine
cargo test --release
```

## Troubleshooting

### Issue: XGBoost headers not found

**Error:** `fatal error: xgboost/c_api.h: No such file or directory`

**Solution:**
1. Ensure XGBoost headers are cloned to `/home/vp/xgboost-headers/`
2. Verify CMakeLists.txt points to correct path: `include_directories(/home/vp/xgboost-headers/include)`

### Issue: libxgboost.so not found

**Error:** Linker cannot find `libxgboost.so`

**Solution:**
- The library should be in `/home/vp/quality_control_room/lib/libxgboost.so`
- Check CMakeLists.txt links to correct path: `/home/vp/quality_control_room/lib/libxgboost.so`

### Issue: SSL certificate error during UI build

**Error:** `EACCES: permission denied, open '/etc/ssl/private/quality-control.io.key'`

**Solution:**
- The vite.config.js has been updated to only load SSL certs in dev mode
- For production builds, no certificates are needed
- If you see this error, update vite.config.js to make HTTPS conditional

### Issue: Rust compilation errors

**Solution:**
- Ensure Rust 1.90.0 or newer is installed
- Try `cargo clean` and rebuild
- Check internet connection (Cargo needs to download crates)

### Issue: Node.js build warnings

**Note:** Warnings about browserslist data being old are non-critical. You can update with:
```bash
npx update-browserslist-db@latest
```

## Running the Application

### Start the Server

```bash
cd /home/vp/quality_control_room/engine
cargo run --release --bin server
```

### Start the UI Development Server

```bash
cd /home/vp/quality_control_room/ui
npm run dev
```

**Note:** Dev server will try to use HTTPS if certificates exist at:
- Key: `/etc/ssl/private/quality-control.io.key`
- Cert: `/etc/ssl/certs/quality-control.io-fullchain.crt`

If certificates don't exist, it will fall back to HTTP.

### Serve Production UI

For production, serve the `ui/dist/` directory with your preferred web server (nginx, Apache, etc.)

## Clean Build (Start Fresh)

```bash
# Clean xgbwrapper
cd /home/vp/quality_control_room/xgbwrapper
rm -rf build

# Clean Rust engine
cd /home/vp/quality_control_room/engine
cargo clean

# Clean UI
cd /home/vp/quality_control_room/ui
rm -rf node_modules dist
```

Then rebuild following the instructions above.

## Summary of Build Artifacts

| Component | Output | Location |
|-----------|--------|----------|
| xgbwrapper | libxgbwrapper.so | `lib/libxgbwrapper.so` |
| xgbwrapper tests | test_xgbwrapper | `xgbwrapper/build/test_xgbwrapper` |
| Engine (models) | libmodels.rlib | `engine/target/release/` |
| Engine (server) | libserver.rlib | `engine/target/release/` |
| UI | Static files | `ui/dist/` |

## Development Environment

For active development:

```bash
# Terminal 1: Run Rust server with auto-reload
cd /home/vp/quality_control_room/engine
cargo watch -x 'run --bin server'

# Terminal 2: Run Vite dev server
cd /home/vp/quality_control_room/ui
npm run dev
```

**Note:** Install `cargo-watch` for auto-reload: `cargo install cargo-watch`

## System Service (Production)

A systemd service file is available at `systemd/quality-engine.service` for production deployment.

---

**Last updated:** October 20, 2025  
**Build environment:** Debian 13 (Trixie)  
**Build status:** ✅ All components built successfully
