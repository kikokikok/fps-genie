# CS2-ML - Machine Learning Engine

CS2-ML is the machine learning engine for the FPS Genie project, providing AI-driven behavioral analysis and training capabilities for CS2 demo analysis.

## Platform-Specific Builds

### Default Configuration (Recommended)
By default, cs2-ml uses CPU-only mode to ensure cross-platform compatibility:

```bash
# Works on all platforms (Linux, macOS, Windows)
cargo build -p cs2-ml
```

### GPU Acceleration

#### macOS (Metal)
For GPU acceleration on macOS with Apple Silicon or Intel Macs:

```bash
# Enable Metal backend for macOS GPU acceleration
cargo build -p cs2-ml --features=metal
```

#### Linux/Windows (CUDA)
For NVIDIA GPU acceleration on Linux or Windows:

```bash
# Enable CUDA backend for NVIDIA GPUs
cargo build -p cs2-ml --features=cuda
```

## Quick Start

```bash
# 1. install LibTorch CPU (macOS example)
curl -L https://download.pytorch.org/libtorch/cpu/libtorch-macos-2.1.0.zip -o libtorch.zip
unzip libtorch.zip -d /opt

# 2. build (CPU-only by default)
export LIBTORCH=/opt/libtorch
export DYLD_LIBRARY_PATH=$LIBTORCH/lib:$DYLD_LIBRARY_PATH
cargo build --release

# 3. prepare dataset
./target/release/cs2-ml prepare "demos/*.dem" ./data

# 4. train
./target/release/cs2-ml train ./data/*.parquet ./policy.ot --epochs 1000

# 5. serve
./target/release/cs2-ml serve ./policy.ot --port 8123
```

The TCP server returns 8 bytes: two little-endian f32 (delta_yaw, delta_pitch) for each 14-f32 input vector.

## Important Notes

- **CPU-only mode** is the default to prevent build issues with platform-specific dependencies
- **Metal backend** requires macOS and may pull in Objective-C dependencies
- **CUDA backend** requires NVIDIA drivers and CUDA toolkit installation
- Use GPU acceleration only when you have the appropriate hardware and drivers installed
