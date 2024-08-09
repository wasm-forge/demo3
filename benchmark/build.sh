#!/bin/bash
export backend=ic_rusqlite_bench_backend
export target_path="./target/wasm32-wasi/release"
export CC_wasm32_wasi="/opt/wasi-sdk/bin/clang"
export CFLAGS_wasm32_wasi="--sysroot=/opt/wasi-sdk/share/wasi-sysroot"

set -e

dfx canister create $backend || (echo "Make sure dfx is running..." && false)

echo "Compile"
cargo build --release --target wasm32-wasi
rm -f $target_path/no_wasi.wasm $target_path/no_wasi.wasm.gz
wasi2ic $target_path/$backend.wasm $target_path/$backend.wasm
ic-wasm $target_path/$backend.wasm -o $target_path/$backend.wasm metadata candid:service -f src/$backend/$backend.did -v public
gzip -c $target_path/$backend.wasm > $target_path/$backend.wasm.gz

echo "Install canister"
dfx canister install --mode reinstall --yes --wasm $target_path/$backend.wasm.gz $backend

