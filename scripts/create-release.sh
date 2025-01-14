#!/bin/bash

cd "$(dirname "$0")/.."

cargo build --release

rm -r build/dsv 2> /dev/null
rm build/dsv.tar.gz 2> /dev/null

mkdir -p build/dsv
cp target/release/dsv build/dsv
cd build
tar czvf dsv.tar.gz dsv
