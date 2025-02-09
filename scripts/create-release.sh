#!/bin/bash

cd "$(dirname "$0")/.."

rm -r target/release 2> /dev/null
# docker run --rm -v "$PWD":/app -w /app rust:latest cargo build --release # Don't use docker for now
RUSTFLAGS="--remap-path-prefix /home/$(whoami)=/app" cargo build --release

rm -r build/dsv 2> /dev/null
rm build/dsv.tar.gz 2> /dev/null

mkdir -p build/dsv
cp target/release/dsv build/dsv
cd build
tar czvf dsv.tar.gz dsv
