#!/bin/bash
set -e

echo "Starting DevContainer Environment Check..."
echo "----------------------------------------"

FAILED=0

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

warn() {
    local desc="$1"
    shift
    if eval "$@" > /dev/null 2>&1; then
        echo "[OK]   $desc"
    else
        echo "[WARN] $desc"
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

# Minimal compilation test (validates linker and toolchain, not the project)
COMPILE_TMP=$(mktemp -d 2>/dev/null)
if echo 'fn main() {}' > "$COMPILE_TMP/test.rs" \
    && rustc "$COMPILE_TMP/test.rs" -o "$COMPILE_TMP/test" 2>/dev/null; then
    echo "[OK]   compile + link"
else
    echo "[FAIL] compile + link"
    FAILED=1
fi
rm -rf "$COMPILE_TMP"

# 2. System Dependencies
echo ""
echo "System Dependencies:"
check "pkg-config"      "pkg-config --version"
check "python3"         "python3 --version"
check "git"             "git --version"
check "gh"              "gh --version"
check "openssl"         "pkg-config --exists openssl"

# 3. User Environment
echo ""
echo "User Environment:"

# Verify container user
if [ "$(id -u)" -eq 0 ]; then
    echo "[SKIP] sudo access (running as root)"
else
    EXPECTED_USER="vscode"
    CURRENT_USER=$(whoami)
    if [ "$CURRENT_USER" = "$EXPECTED_USER" ]; then
        echo "[OK]   user ($CURRENT_USER, uid=$(id -u))"
    else
        echo "[FAIL] user (expected $EXPECTED_USER, got $CURRENT_USER)"
        FAILED=1
    fi
    check "sudo access" "sudo -n true"
fi

# Workspace mount
if [ -d "$(pwd)/.git" ]; then
    echo "[OK]   workspace mounted ($(pwd))"
else
    echo "[FAIL] workspace not mounted or not a git repo"
    FAILED=1
fi

# Write permissions
if TMPFILE=$(mktemp 2>/dev/null) && rm -f "$TMPFILE"; then
    echo "[OK]   temp write"
else
    echo "[FAIL] temp write"
    FAILED=1
fi

# 4. Git
echo ""
echo "Git:"

check "git operations"  "git status"
check "safe.directory"  "git log --oneline -1"

warn  "git identity"    "git config user.name && git config user.email"

if [ -n "$SSH_AUTH_SOCK" ]; then
    if ssh-add -l > /dev/null 2>&1; then
        echo "[OK]   ssh agent ($(ssh-add -l 2>/dev/null | wc -l) key(s))"
    else
        echo "[WARN] ssh agent (socket set but no keys loaded)"
    fi
else
    echo "[WARN] ssh agent (SSH_AUTH_SOCK not set)"
fi

# 5. Resources
echo ""
echo "Resources:"
FREE_KB=$(df . | tail -1 | awk '{print $4}')
FREE_GB=$((FREE_KB / 1024 / 1024))
if [ "$FREE_GB" -ge 5 ]; then
    echo "[OK]   disk space (${FREE_GB}GB free)"
else
    echo "[WARN] disk space (${FREE_GB}GB free, recommend >= 5GB)"
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
