#!/usr/bin/env sh

set -eu

SCRIPT="$(basename "$0")"
cd "$(dirname "$0")"

if [ -z "${1:-}" ]; then
    echo "application name is not defined"
    echo "Usage: download.sh <app-name>"
    exit 1
fi

APP_NAME="$1"

# Detect OS
case "$(uname -s)" in
  Linux*)
    OS="linux"
    ;;
  Darwin*)
    OS="macos"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    OS="windows"
    ;;
  *)
    echo "Unsupported OS: $(uname -s)"
    exit 1
    ;;
esac

REPO="yantonov/alias"

# Get latest tag
LATEST_TAG=$(
  curl -fsSL "https://api.github.com/repos/${REPO}/tags" \
  | jq -r '.[0].name'
)

ALIAS_APP_NAME="alias"
ARCHIVE_NAME="${ALIAS_APP_NAME}-${OS}-${LATEST_TAG}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARCHIVE_NAME}"

echo "Latest tag: ${LATEST_TAG}"
echo "Downloading: ${DOWNLOAD_URL}"

TMP_DIR="$(mktemp -d)"
ARCHIVE_PATH="${TMP_DIR}/${ALIAS_APP_NAME}.tar.gz"

echo $ARCHIVE_PATH

# Download archive
curl -fL "${DOWNLOAD_URL}" -o "${ARCHIVE_PATH}"

# Extract archive
tar -xzf "${ARCHIVE_PATH}" -C "${TMP_DIR}"

# Find binary inside extracted files
BIN_PATH="$(find "${TMP_DIR}" -type f -exec sh -c 'test -x "$1"' _ {} \; -print | head -n 1)"

if [ -z "${BIN_PATH}" ]; then
  echo "Executable ${ALIAS_APP_NAME} is not found in the archive ${TMP_DIR}"
  rm -rf "${TMP_DIR}"
  exit 1
fi

# Copy binary to current directory
cp "${BIN_PATH}" "./${APP_NAME}"
chmod +x "./${APP_NAME}"

# Cleanup
rm -rf "${TMP_DIR}"

echo "Installed: ./${APP_NAME}"
