#!/bin/bash

cd "$(dirname "$0")/.."

cargo build --release

mkdir -p build/dsv
cp target/release/dsv build/dsv
cd build
tar czvf dsv.tar.gz dsv
