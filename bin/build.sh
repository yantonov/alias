#!/bin/sh

cd "$(dirname "$0")"

cd ..

TARGET="$(pwd)/target/release/alias"

cargo build --release

echo "binary file is here: ${TARGET}"

# reduce binary size
strip "${TARGET}"
