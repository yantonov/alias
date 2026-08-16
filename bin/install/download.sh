#!/usr/bin/env sh

set -eu

# The binary is installed into the current directory: install.sh changes into
# the target directory before piping this script into a shell.
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

# The version comes from the latest published release rather than from the tag
# list. A tag exists the moment it is pushed, while the release built from it
# stays a draft until someone publishes it, so the newest tag can easily point
# at assets that cannot be downloaded yet. Following the redirect of the
# 'latest release' page also keeps this free of a json parser and of the
# unauthenticated api rate limit.
LATEST_TAG="$(
  curl -fsSLo /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest" \
  | sed 's#.*/tag/##'
)"

case "${LATEST_TAG}" in
  ''|*/*)
    echo "Cannot detect the latest published release of ${REPO}"
    exit 1
    ;;
esac

ALIAS_APP_NAME="alias"
# Release assets carry the architecture as uname reports it, so no mapping is
# needed here: x86_64 and aarch64 on linux, x86_64 and arm64 on macos.
ARCH="$(uname -m)"

ARCHIVE_NAME="${ALIAS_APP_NAME}-${OS}-${ARCH}-${LATEST_TAG}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARCHIVE_NAME}"

echo "Latest tag: ${LATEST_TAG}"
echo "Downloading: ${DOWNLOAD_URL}"

TMP_DIR="$(mktemp -d)"
ARCHIVE_PATH="${TMP_DIR}/${ALIAS_APP_NAME}.tar.gz"

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
