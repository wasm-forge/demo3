#!/bin/bash
export backend=ic_rusqlite_bench_backend
export target_path=./target/wasm32-wasi/release

dfx canister create $backend

echo "Compile"
cargo build --release --target wasm32-wasi

echo "Remove wasi dependencies"
rm -f $target_path/no_wasi.wasm $target_path/no_wasi.wasm.gz
wasi2ic $target_path/$backend.wasm $target_path/no_wasi.wasm
gzip $target_path/no_wasi.wasm

echo "install canister"
dfx canister install --mode reinstall --yes --wasm $target_path/no_wasi.wasm.gz $backend


