#!/bin/sh
# Muxwise installer — downloads the latest release binary for your platform.
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/caophuong2012/muxwise/main/install.sh | sh
#
# Installs to /usr/local/bin/muxwise (requires sudo) or ~/.local/bin/muxwise

set -e

REPO="caophuong2012/muxwise"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="muxwise"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS_TARGET="unknown-linux-musl" ;;
    Darwin) OS_TARGET="apple-darwin" ;;
    *)
        echo "Error: Unsupported operating system: $OS"
        echo "Muxwise supports Linux and macOS."
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)   ARCH_TARGET="x86_64" ;;
    aarch64|arm64)   ARCH_TARGET="aarch64" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        echo "Muxwise supports x86_64 and aarch64."
        exit 1
        ;;
esac

TARGET="${ARCH_TARGET}-${OS_TARGET}"
ASSET_NAME="zellij-${TARGET}.tar.gz"

echo "Muxwise installer"
echo "  Platform: ${OS} ${ARCH} (${TARGET})"

# Get latest release tag
echo "  Fetching latest release..."
LATEST_TAG=$(curl -sSL -H "Accept: application/vnd.github.v3+json" \
    "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "Error: Could not find a release. Please check https://github.com/${REPO}/releases"
    exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ASSET_NAME}"

echo "  Version:  ${LATEST_TAG}"
echo "  Download: ${ASSET_NAME}"

# Download to temp directory
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "  Downloading..."
HTTP_CODE=$(curl -sSL -w "%{http_code}" -o "${TMPDIR}/${ASSET_NAME}" "$DOWNLOAD_URL")

if [ "$HTTP_CODE" != "200" ]; then
    echo "Error: Download failed (HTTP ${HTTP_CODE})"
    echo "URL: ${DOWNLOAD_URL}"
    echo ""
    echo "Available assets at https://github.com/${REPO}/releases/tag/${LATEST_TAG}"
    exit 1
fi

# Extract
echo "  Extracting..."
tar xzf "${TMPDIR}/${ASSET_NAME}" -C "$TMPDIR"

if [ ! -f "${TMPDIR}/zellij" ]; then
    echo "Error: Binary not found in archive"
    exit 1
fi

chmod +x "${TMPDIR}/zellij"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "${TMPDIR}/zellij" "${INSTALL_DIR}/${BINARY_NAME}"
    echo "  Installed to ${INSTALL_DIR}/${BINARY_NAME}"
else
    # Try with sudo
    if command -v sudo >/dev/null 2>&1; then
        echo "  Installing to ${INSTALL_DIR}/${BINARY_NAME} (requires sudo)..."
        sudo mv "${TMPDIR}/zellij" "${INSTALL_DIR}/${BINARY_NAME}"
        echo "  Installed to ${INSTALL_DIR}/${BINARY_NAME}"
    else
        # Fall back to ~/.local/bin
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
        mv "${TMPDIR}/zellij" "${INSTALL_DIR}/${BINARY_NAME}"
        echo "  Installed to ${INSTALL_DIR}/${BINARY_NAME}"
        case ":$PATH:" in
            *":${INSTALL_DIR}:"*) ;;
            *)
                echo ""
                echo "  Note: Add ${INSTALL_DIR} to your PATH:"
                echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
                ;;
        esac
    fi
fi

echo ""
echo "Done! Run 'muxwise' to start."
