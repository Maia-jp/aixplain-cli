#!/bin/sh
# One-command installer for aix (aiXplain CLI)
# Usage: curl -fsSL https://raw.githubusercontent.com/aixplain/aixplain-cli/main/scripts/install.sh | sh

set -e

REPO="aixplain/aixplain-cli"
BINARY="aix"
INSTALL_DIR="${AIX_INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)   OS="unknown-linux-gnu" ;;
        Darwin)  OS="apple-darwin" ;;
        *)       echo "Error: Unsupported OS: $OS"; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64" ;;
        arm64|aarch64) ARCH="aarch64" ;;
        *)             echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    PLATFORM="${ARCH}-${OS}"
}

# Get latest release tag from GitHub
get_latest_version() {
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version"
        exit 1
    fi
}

main() {
    detect_platform
    get_latest_version

    ARCHIVE="${BINARY}-${VERSION}-${PLATFORM}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

    echo "Installing aix ${VERSION} for ${PLATFORM}..."
    echo "  Downloading: ${URL}"

    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    curl -fsSL "$URL" -o "${TMPDIR}/${ARCHIVE}"
    tar -xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    else
        echo "  Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    fi

    chmod +x "${INSTALL_DIR}/${BINARY}"

    echo ""
    echo "  ✓ aix installed to ${INSTALL_DIR}/${BINARY}"
    echo ""
    echo "  Get started:"
    echo "    export AIXPLAIN_API_KEY=your-key-here"
    echo "    aix models list"
    echo "    aix              # launch TUI browser"
    echo ""
}

main
