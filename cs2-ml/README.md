CS2-ML (prototype)

Quick start

```bash
# 1. install LibTorch CPU (macOS example)
curl -L https://download.pytorch.org/libtorch/cpu/libtorch-macos-2.1.0.zip -o libtorch.zip
unzip libtorch.zip -d /opt

# 2. build
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
