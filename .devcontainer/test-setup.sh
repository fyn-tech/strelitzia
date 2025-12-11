#!/bin/bash
set -e

echo "Starting DevContainer Environment Check..."
echo "----------------------------------------"

FAILED=0

# Function for status output
check() {
    local desc="$1"
    shift
    if eval "$@" > /dev/null 2>&1; then
        echo "[OK]   $desc"
    else
        echo "[FAIL] $desc"
        FAILED=1
    fi
}

# 1. Rust Toolchain
echo ""
echo "Rust Toolchain:"
check "rustc"           "rustc --version"
check "cargo"           "cargo --version"
check "clippy"          "cargo clippy --version"
check "rustfmt"         "rustfmt --version"
check "rust-src"        "rustup component list --installed | grep -q rust-src"

# 2. System Dependencies
echo ""
echo "System Dependencies:"
check "pkg-config"      "pkg-config --version"
check "python3"         "python3 --version"
check "git"             "git --version"
check "openssl"         "pkg-config --exists openssl"

# 3. User Environment
echo ""
echo "User Environment:"
check "sudo access"     "sudo -n true"
check "workspace write" "touch .write-test && rm .write-test"

# 4. Disk Space (warn if < 5GB free)
echo ""
echo "Resources:"
FREE_KB=$(df . | tail -1 | awk '{print $4}')
FREE_GB=$((FREE_KB / 1024 / 1024))
if [ "$FREE_GB" -ge 5 ]; then
    echo "[OK]   Disk space (${FREE_GB}GB free)"
else
    echo "[WARN] Disk space (${FREE_GB}GB free, recommend >= 5GB)"
fi

# 5. Project Build Tests (only if Cargo.toml exists)
echo ""
echo "Project Build:"
if [ -f "Cargo.toml" ]; then
    check "cargo check"         "cargo check --quiet"
    check "cargo clippy"        "cargo clippy --quiet -- -D warnings 2>/dev/null || cargo clippy --quiet"
    check "cargo test --no-run" "cargo test --no-run --quiet 2>/dev/null"
else
    echo "[SKIP] Cargo.toml not found"
fi

# Summary
echo ""
echo "----------------------------------------"
if [ "$FAILED" -eq 0 ]; then
    echo "DevContainer environment is healthy."
    exit 0
else
    echo "DevContainer environment has issues."
    exit 1
fi
