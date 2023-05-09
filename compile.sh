#/bin/bash

export CC_wasm32_wasi="/opt/wasi-sdk/bin/clang --sysroot=/opt/wasi-sdk/share/wasi-sysroot" 

cargo build --release --target "wasm32-wasi"

wasi2ic ./target/wasm32-wasi/release/demo3_backend.wasm ./target/wasm32-wasi/release/nowasi.wasm && rm -f ./target/wasm32-wasi/release/nowasi.wasm.gz  && gzip ./target/wasm32-wasi/release/nowasi.wasm


