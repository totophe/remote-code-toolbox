#!/usr/bin/env sh
# Install the latest dcon binary from GitHub Releases.
#
# Usage:
#   ./scripts/install.sh                 # installs to ~/.local/bin
#   INSTALL_DIR=/usr/local/bin ./scripts/install.sh

set -e

REPO="totophe/remote-code-toolbox"
TOOL="dcon"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# ── detect OS / arch ──────────────────────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Linux)
    case "${ARCH}" in
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "error: unsupported Linux architecture: ${ARCH}" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    case "${ARCH}" in
      x86_64)  TARGET="x86_64-apple-darwin" ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      *) echo "error: unsupported macOS architecture: ${ARCH}" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "error: unsupported OS: ${OS}" >&2
    exit 1
    ;;
esac

# ── download ──────────────────────────────────────────────────────────────────

BINARY="${TOOL}-${TARGET}"
URL="https://github.com/${REPO}/releases/download/latest/${BINARY}"

echo "Downloading ${BINARY} …"
mkdir -p "${INSTALL_DIR}"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "${URL}" -o "${INSTALL_DIR}/${TOOL}"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "${INSTALL_DIR}/${TOOL}" "${URL}"
else
  echo "error: neither curl nor wget found" >&2
  exit 1
fi

chmod +x "${INSTALL_DIR}/${TOOL}"

echo "Installed to ${INSTALL_DIR}/${TOOL}"

# ── PATH hint ─────────────────────────────────────────────────────────────────

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "hint: ${INSTALL_DIR} is not in your PATH."
    echo "      Add this to your shell profile:"
    echo "        export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac
