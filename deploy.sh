#!/bin/bash

dfx canister create --all

dfx canister install --mode reinstall --wasm ./target/wasm32-wasi/release/no_wasi.wasm.gz demo3_backend

