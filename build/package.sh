#!/bin/bash
# Package a self-contained ayanami build with bundled llc.
# Usage: ./build/package.sh [output-dir]
set -e

OUT="${1:-ayanami-bundle}"
VERSION=$(cargo metadata --format-version=1 --no-deps | python3 -c "import sys,json; print(json.load(sys.stdin)['packages'][0]['version'])")

echo "==> Building release..."
cargo build --release

mkdir -p "$OUT"
cp target/release/ayanami "$OUT/"
cp src/runtime.c "$OUT/"

# Bundle llc if available
if which llc &>/dev/null; then
    LLC=$(which llc)
    # Try to find and bundle libLLVM.so
    LLVM_LIB=$(ldd "$LLC" 2>/dev/null | grep libLLVM | awk '{print $3}')
    if [ -n "$LLVM_LIB" ] && [ -f "$LLVM_LIB" ]; then
        cp "$LLVM_LIB" "$OUT/"
        cp "$LLC" "$OUT/llc"
        # Set rpath so llc finds its .so
        if which patchelf &>/dev/null; then
            patchelf --set-rpath '$ORIGIN' "$OUT/llc"
        else
            echo "  (warning: install patchelf for standalone llc, or set LD_LIBRARY_PATH)"
        fi
        echo "  bundled llc + libLLVM"
    else
        echo "  (warning: llc found but libLLVM not detected, skipping bundle)"
    fi
else
    echo "  (llc not found in PATH; bundle will use system llc)"
fi

echo "==> Packaged: $OUT/"
ls -lh "$OUT/"
