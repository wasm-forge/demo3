#!/bin/bash
export backend=ic_rusqlite_bench_backend
export target_path="./target/wasm32-wasi/release"

echo "Install canister"
dfx canister create $backend || (echo "Make sure dfx is running..." && false)

dfx canister install --mode reinstall --yes --wasm $target_path/$backend.wasm.gz $backend
