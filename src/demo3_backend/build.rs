fn main() {
    println!("cargo:rustc-link-search=/opt/wasi-sdk/lib/clang/17/lib/wasi/");
    println!("cargo:rustc-link-arg=-lclang_rt.builtins-wasm32");
}
