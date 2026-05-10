#!/usr/bin/env sh

set -eu

# Check that an application name was provided
if [ -z "${1:-}" ]; then
  echo "Usage: $0 <app-name>"
  exit 1
fi

APP_NAME="$1"
TARGET_DIR="${HOME}/bin/${APP_NAME}"

# 1. Create directory if it doesn't exist
mkdir -p "${TARGET_DIR}"

# 2. Go to the directory
cd "${TARGET_DIR}"

# 3. Download and execute first script with app name
curl -fsSL "https://raw.githubusercontent.com/yantonov/alias/master/bin/download.sh" | sh -s -- "${APP_NAME}"

# 4. Download and execute second script with target directory
curl -fsSL "https://raw.githubusercontent.com/yantonov/alias/master/bin/configure.sh" | sh -s -- "${TARGET_DIR}"

# 5. Execute the application to generate default config
# (Assumes the executable name matches APP_NAME)
"./${APP_NAME}"

# 6. Print completion message
echo "Installation of '${APP_NAME}' completed successfully."
echo "Target directory: ${TARGET_DIR}"
