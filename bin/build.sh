#!/bin/sh

cd "$(dirname "$0")"

cd ..

cargo build --release
