#/bin/bash

set -x

export CC_wasm32_wasi="/opt/wasi-sdk/bin/clang --sysroot=/opt/wasi-sdk/share/wasi-sysroot" 

cargo build --release --target "wasm32-wasi"

rm -f ./target/wasm32-wasi/release/no_wasi.wasm.gz

wasi2ic ./target/wasm32-wasi/release/demo3_backend.wasm ./target/wasm32-wasi/release/no_wasi.wasm

gzip ./target/wasm32-wasi/release/no_wasi.wasm


