# demo3 - Run Rusqlite on the IC

This project shows how to compilte the Rusqlite dependency in order to build the IC canister with the sqlite database.


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/), and [wasi2ic](https://github.com/wasm-forge/wasi2ic) installed.

You will also need the Wasm-oriented [clang](https://github.com/WebAssembly/wasi-sdk/releases/) installation. In this tutorial we use the `.deb` package [installation](https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-19/wasi-sdk_19.0_amd64.deb). Once installed the clang compiler is available from the path `/opt/wasi-sdk/bin/`.


## Building project from scratch

Creare a new project using `dfx`:

```bash
dfx new --type=rust --no-frontend demo3
```

Enter the backend source folder and add a few dependencies:

```bash
cd demo3/src/demo3_backend/

cargo add --git https://github.com/wasm-forge/ic_polyfill

cargo add --git https://github.com/wasm-forge/rusqlite rusqlite -F bundled
```


## Deployment and testing

In a separate terminal start the `dfx` environment:
```bash
dfx start
```

Go to the `demo3` project folder and deploy the project:
```bash
dfx deploy
```

