#!/usr/bin/env sh

set -eu

# The binary is installed into the current directory: install.sh changes into
# the target directory before piping this script into a shell.
if [ -z "${1:-}" ]; then
    echo "application name is not defined"
    echo "Usage: download.sh <app-name> [release]"
    exit 1
fi

APP_NAME="$1"
# install.sh passes the release it resolved, so that the scripts and the binary
# all come from one and the same release. Run on its own, this script falls
# back to the latest published one.
LATEST_TAG="${2:-}"

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
if [ -z "${LATEST_TAG}" ]; then
  LATEST_TAG="$(
    curl -fsSLo /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest" \
    | sed 's#.*/tag/##'
  )"
fi

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
# Named after the published asset, so that the downloaded file and the checksum
# beside it can also be verified by hand with the usual sha256sum -c.
ARCHIVE_PATH="${TMP_DIR}/${ARCHIVE_NAME}"
CHECKSUM_PATH="${ARCHIVE_PATH}.sha256"
UNPACK_DIR="${TMP_DIR}/unpacked"

# Download archive and the checksum published next to it
curl -fL "${DOWNLOAD_URL}" -o "${ARCHIVE_PATH}"
curl -fL "${DOWNLOAD_URL}.sha256" -o "${CHECKSUM_PATH}"

# Verify before unpacking, not after: linux and git bash carry sha256sum,
# macos carries shasum. Only the hash is compared, so the file name inside the
# checksum file does not have to match the temporary one.
if command -v sha256sum > /dev/null 2>&1; then
  ACTUAL_CHECKSUM="$(sha256sum "${ARCHIVE_PATH}" | awk '{print $1}')"
elif command -v shasum > /dev/null 2>&1; then
  ACTUAL_CHECKSUM="$(shasum -a 256 "${ARCHIVE_PATH}" | awk '{print $1}')"
else
  echo "Neither sha256sum nor shasum is available to verify the download"
  rm -rf "${TMP_DIR}"
  exit 1
fi

EXPECTED_CHECKSUM="$(awk '{print $1}' "${CHECKSUM_PATH}")"

if [ "${ACTUAL_CHECKSUM}" != "${EXPECTED_CHECKSUM}" ]; then
  echo "Checksum mismatch for ${ARCHIVE_NAME}"
  echo "  expected ${EXPECTED_CHECKSUM}"
  echo "  actual   ${ACTUAL_CHECKSUM}"
  rm -rf "${TMP_DIR}"
  exit 1
fi

echo "Checksum ok: ${ACTUAL_CHECKSUM}"

# Extract archive
mkdir -p "${UNPACK_DIR}"
tar -xzf "${ARCHIVE_PATH}" -C "${UNPACK_DIR}"

# Find binary inside extracted files
BIN_PATH="$(find "${UNPACK_DIR}" -type f -exec sh -c 'test -x "$1"' _ {} \; -print | head -n 1)"

if [ -z "${BIN_PATH}" ]; then
  echo "Executable ${ALIAS_APP_NAME} is not found in the archive ${ARCHIVE_NAME}"
  rm -rf "${TMP_DIR}"
  exit 1
fi

# Copy binary to current directory
cp "${BIN_PATH}" "./${APP_NAME}"
chmod +x "./${APP_NAME}"

# Cleanup
rm -rf "${TMP_DIR}"

echo "Installed: ./${APP_NAME}"
