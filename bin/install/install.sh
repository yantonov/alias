#!/usr/bin/env sh

set -eu

# Check that an application name was provided
if [ -z "${1:-}" ]; then
  echo "Usage: $0 <app-name>"
  exit 1
fi

APP_NAME="$1"
TARGET_DIR="${HOME}/bin/${APP_NAME}-aliases"
REPO="yantonov/alias"

# One release provides everything: the two scripts below and the binary they
# install. Taken from master instead, the scripts would be whatever landed
# there a minute ago, and could be paired with a binary they have never seen.
# Set ALIAS_VERSION to install a specific release.
VERSION="${ALIAS_VERSION:-$(
  curl -fsSLo /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest" \
  | sed 's#.*/tag/##'
)}"

case "${VERSION}" in
  ''|*/*)
    echo "Cannot detect the latest published release of ${REPO}"
    exit 1
    ;;
esac

SCRIPTS="https://raw.githubusercontent.com/${REPO}/${VERSION}/bin/install"

echo "Installing '${APP_NAME}' from release ${VERSION}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

# Fetched to a file rather than piped into a shell: in 'curl | sh' the exit
# code belongs to the shell, and an empty input is a script that succeeds, so a
# missing script would pass unnoticed and the failure would surface later as
# something unrelated.
fetch_script() {
  if ! curl -fsSL "${SCRIPTS}/$1" -o "${TMP_DIR}/$1"; then
    echo "Cannot fetch ${SCRIPTS}/$1"
    echo "Release ${VERSION} may not carry the installer scripts yet; set ALIAS_VERSION to a release that does"
    exit 1
  fi
}

fetch_script download.sh
fetch_script configure.sh

# 1. Create directory if it doesn't exist
mkdir -p "${TARGET_DIR}"

# 2. Go to the directory
cd "${TARGET_DIR}"

# 3. Download the binary of that release into the target directory
sh "${TMP_DIR}/download.sh" "${APP_NAME}" "${VERSION}"

# 4. Put the target directory on PATH of the current shell
sh "${TMP_DIR}/configure.sh" "${TARGET_DIR}"

# 5. Execute the application to generate default config
# (Assumes the executable name matches APP_NAME)
"./${APP_NAME}"

# 6. Print completion message
echo "Installation of '${APP_NAME}' completed successfully."
echo "Target directory: ${TARGET_DIR}"
