#!/bin/bash

set -x

dfx canister create demo3_backend

dfx canister install --mode upgrade --upgrade-unchanged --wasm ./target/wasm32-wasi/release/no_wasi.wasm.gz demo3_backend

