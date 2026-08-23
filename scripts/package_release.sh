#!/usr/bin/env bash
set -euo pipefail

# Siphonix Release Packaging Automation
# Developed by: Chirag P Patil (cp099)
# https://github.com/cp099/Siphonix

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"
VERSION="$(grep '^version =' "${PROJECT_ROOT}/Cargo.toml" | head -n1 | cut -d'"' -f2)"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        TARGET_ARCH="arm64"
        ;;
    *)
        TARGET_ARCH="$ARCH"
        ;;
esac

RELEASE_NAME="siphonix-v${VERSION}-${OS}-${TARGET_ARCH}"
BUNDLE_DIR="${DIST_DIR}/${RELEASE_NAME}"
ARCHIVE_PATH="${DIST_DIR}/${RELEASE_NAME}.tar.gz"

echo "=== Packaging Siphonix v${VERSION} (${OS}-${TARGET_ARCH}) ==="

# 1. Clean & Prepare dist directory
rm -rf "${DIST_DIR}"
mkdir -p "${BUNDLE_DIR}"

# 2. Build release binary
echo "--> Compiling optimized release binary..."
cargo build --release --manifest-path "${PROJECT_ROOT}/Cargo.toml"

# 3. Strip binary for minimal footprint
echo "--> Stripping binary symbols..."
cp "${PROJECT_ROOT}/target/release/siphonix" "${BUNDLE_DIR}/siphonix"
if command -v strip >/dev/null 2>&1; then
    strip "${BUNDLE_DIR}/siphonix" 2>/dev/null || true
fi

# 4. Copy deployment scripts and docs
echo "--> Packaging deployment assets..."
cp "${PROJECT_ROOT}/scripts/install.sh" "${BUNDLE_DIR}/install.sh"
cp "${PROJECT_ROOT}/scripts/uninstall.sh" "${BUNDLE_DIR}/uninstall.sh"
cp "${PROJECT_ROOT}/README.md" "${BUNDLE_DIR}/README.md"
cp "${PROJECT_ROOT}/LICENSE" "${BUNDLE_DIR}/LICENSE"
chmod +x "${BUNDLE_DIR}/siphonix" "${BUNDLE_DIR}/install.sh" "${BUNDLE_DIR}/uninstall.sh"

# 5. Create compressed distribution tarball
echo "--> Generating tarball: ${ARCHIVE_PATH}..."
(cd "${DIST_DIR}" && tar -czf "${RELEASE_NAME}.tar.gz" "${RELEASE_NAME}")

# 6. Generate SHA256 Checksum
echo "--> Generating SHA-256 checksum..."
if command -v shasum >/dev/null 2>&1; then
    (cd "${DIST_DIR}" && shasum -a 256 "${RELEASE_NAME}.tar.gz" > "${RELEASE_NAME}.tar.gz.sha256")
elif command -v sha256sum >/dev/null 2>&1; then
    (cd "${DIST_DIR}" && sha256sum "${RELEASE_NAME}.tar.gz" > "${RELEASE_NAME}.tar.gz.sha256")
fi

echo ""
echo "=== Release Packaging Successful! ==="
echo "Artifact: ${ARCHIVE_PATH}"
if [ -f "${DIST_DIR}/${RELEASE_NAME}.tar.gz.sha256" ]; then
    echo "Checksum: $(cat "${DIST_DIR}/${RELEASE_NAME}.tar.gz.sha256")"
fi
echo ""
