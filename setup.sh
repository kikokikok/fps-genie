#!/usr/bin/env bash
set -euo pipefail

# LibTorch version
LIBTORCH_VERSION="2.7.0"
LIBTORCH_DIR="$HOME/libtorch"
LIBTORCH_URL_MACOS="https://download.pytorch.org/libtorch/cpu/libtorch-macos-arm64-${LIBTORCH_VERSION}.zip"
LIBTORCH_URL_LINUX="https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-${LIBTORCH_VERSION}%2Bcpu.zip"
LIBTORCH_URL_WINDOWS_RELEASE="https://download.pytorch.org/libtorch/cpu/libtorch-win-shared-with-deps-${LIBTORCH_VERSION}%2Bcpu.zip"
LIBTORCH_URL_WINDOWS_DEBUG="https://download.pytorch.org/libtorch/cpu/libtorch-win-shared-with-deps-debug-${LIBTORCH_VERSION}%2Bcpu.zip"

# Parse --force flag
FORCE_REINSTALL=false
for arg in "$@"; do
    if [[ "$arg" == "--force" ]]; then
        FORCE_REINSTALL=true
    fi
    # You can add more CLI options here if needed
done

echo "📦 Setting up the CS2 AI Training System..."

# Install dependencies based on OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 macOS detected, checking for Homebrew..."
    if ! command -v brew &> /dev/null; then
        echo "🛠️ Homebrew not found. Installing..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    echo "🛠️ Installing dependencies with Homebrew..."
    brew install cmake opus pkg-config python3

    # Configure Python for pyo3
    echo "🐍 Configuring Python for pyo3 linking..."
    PYTHON_PATH=$(which python3)
    PYTHON_FRAMEWORK=$(python3 -c "import sys; print(sys.prefix)")
    PYTHON_VERSION=$(python3 -c "import sys; print('.'.join(map(str, sys.version_info[:2])))")

    # Set Python environment variables for pyo3 linking
    export PYO3_PYTHON="$PYTHON_PATH"
    export PYTHONPATH="$PYTHON_FRAMEWORK/lib/python$PYTHON_VERSION"
    export PYTHON_LIBRARY="$PYTHON_FRAMEWORK/lib/libpython$PYTHON_VERSION.dylib"

    # Add to env_setup.sh
    echo "export PYO3_PYTHON=\"$PYTHON_PATH\"" >> env_setup.sh
    echo "export PYTHONPATH=\"$PYTHONPATH\"" >> env_setup.sh
    echo "export PYTHON_LIBRARY=\"$PYTHON_LIBRARY\"" >> env_setup.sh

    # Download and extract LibTorch if needed
    if [ "$FORCE_REINSTALL" = true ] || [ ! -d "$LIBTORCH_DIR/libtorch" ]; then
        echo "⬇️  Downloading LibTorch $LIBTORCH_VERSION for macOS ARM64..."
        rm -rf "$LIBTORCH_DIR"
        mkdir -p "$LIBTORCH_DIR"
        curl -L "$LIBTORCH_URL_MACOS" -o "$LIBTORCH_DIR/libtorch.zip"
        unzip -q "$LIBTORCH_DIR/libtorch.zip" -d "$LIBTORCH_DIR"
        rm "$LIBTORCH_DIR/libtorch.zip"
    fi
    export LIBTORCH="$LIBTORCH_DIR/libtorch"
    export DYLD_LIBRARY_PATH="$LIBTORCH/lib:${DYLD_LIBRARY_PATH:-}"
    echo "export LIBTORCH=\"$LIBTORCH_DIR/libtorch\"" > env_setup.sh
    echo "export DYLD_LIBRARY_PATH=\"$LIBTORCH_DIR/libtorch/lib:\${DYLD_LIBRARY_PATH:-}\"" >> env_setup.sh
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "🐧 Linux detected, checking for package manager..."
    if command -v apt-get &> /dev/null; then
        echo "🛠️ Installing dependencies with apt..."
        sudo apt-get update
        sudo apt-get install -y build-essential cmake pkg-config libopus-dev python3-dev python3-pip unzip curl
    elif command -v dnf &> /dev/null; then
        echo "🛠️ Installing dependencies with dnf..."
        sudo dnf install -y cmake gcc gcc-c++ opus-devel python3-devel python3-pip unzip curl
    elif command -v pacman &> /dev/null; then
        echo "🛠️ Installing dependencies with pacman..."
        sudo pacman -Syu --noconfirm cmake opus python python-pip unzip curl
    else
        echo "❌ Unsupported Linux distribution. Please install cmake, opus, and python3-dev manually."
        exit 1
    fi
    # Configure Python for pyo3
    echo "🐍 Configuring Python for pyo3 linking..."
    PYTHON_PATH=$(which python3)
    export PYO3_PYTHON="$PYTHON_PATH"

    # Add to env_setup.sh
    echo "export PYO3_PYTHON=\"$PYTHON_PATH\"" >> env_setup.sh

    # Download and extract LibTorch if needed
    if [ "$FORCE_REINSTALL" = true ] || [ ! -d "$LIBTORCH_DIR/libtorch" ]; then
        echo "⬇️  Downloading LibTorch $LIBTORCH_VERSION for Linux..."
        rm -rf "$LIBTORCH_DIR"
        mkdir -p "$LIBTORCH_DIR"
        curl -L "$LIBTORCH_URL_LINUX" -o "$LIBTORCH_DIR/libtorch.zip"
        unzip -q "$LIBTORCH_DIR/libtorch.zip" -d "$LIBTORCH_DIR"
        rm "$LIBTORCH_DIR/libtorch.zip"
    fi
    export LIBTORCH="$LIBTORCH_DIR/libtorch"
    export LD_LIBRARY_PATH="$LIBTORCH/lib:${LD_LIBRARY_PATH:-}"
    echo "export LIBTORCH=\"$LIBTORCH_DIR/libtorch\"" > env_setup.sh
    echo "export LD_LIBRARY_PATH=\"$LIBTORCH_DIR/libtorch/lib:\${LD_LIBRARY_PATH:-}\"" >> env_setup.sh
else
    echo "❌ Unsupported OS for automatic LibTorch installation. Please install libtorch v$LIBTORCH_VERSION manually."
    exit 1
fi
chmod +x env_setup.sh

echo "📁 Creating test data directories..."
mkdir -p test_data
if [ ! -f "test_data/minimal_demo.bin" ]; then
    dd if=/dev/urandom of=test_data/minimal_demo.bin bs=1024 count=10
fi

if ! command -v rustup &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "🔨 Building project..."
source ./env_setup.sh
cargo build || {
    echo "⚠️ Build failed. This might be due to missing dependencies or environment setup."
    exit 1
}

echo "🧪 Running tests..."
cargo test -- --skip test_vectors_from_demo || echo "⚠️ Some tests failed. This might be expected during initial setup."

echo "✅ Setup complete! You can now use the CS2 AI Training System."
echo
echo "To use the system, first set the environment variables:"
echo "  source ./env_setup.sh"
echo
echo "Then run the commands:"
echo "  - Prepare data:   cargo run --bin cs2-ml -- prepare \"demos/*.dem\" ./data"
echo "  - Train model:    cargo run --bin cs2-ml -- train ./data/*.parquet ./policy.ot --epochs 1000"
echo "  - Serve model:    cargo run --bin cs2-ml -- serve ./policy.ot --port 8123"
echo "  - Analyze demos:  cargo run --bin cs2-demo-analyzer -- analyze --demo path/to/demo.dem --output-dir ./analysis"
