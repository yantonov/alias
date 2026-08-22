#!/bin/sh

cd "$(dirname "$0")/../.."

EXECUTABLE_NAME="$(basename $(pwd))"

TARGET="$(pwd)/target/release/${EXECUTABLE_NAME}"

cargo build --release

echo "binary file is here: ${TARGET}"
