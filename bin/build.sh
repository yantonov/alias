#!/bin/sh

cd "$(dirname "$0")"

cd ..

EXECUTABLE_NAME="alias"

TARGET="$(pwd)/target/release/${EXECUTABLE_NAME}"

cargo build --release

echo "binary file is here: ${TARGET}"

# reduce binary size
strip "${TARGET}"
