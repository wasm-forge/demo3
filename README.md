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

cargo add --git https://github.com/wasm-forge/ic-wasi-polyfill

cargo add --git https://github.com/rusqlite/rusqlite rusqlite -F wasm32-wasi-vfs,bundled
```

Modify the demo3/src/demo3_backend/src/lib.rs file containing the greet method so that it uses the rusqlite backend to store a list of persons:
```rust
use std::cell::RefCell;

use rusqlite::Connection;

thread_local! {
    static DB: RefCell<Option<Connection>> = RefCell::new(None);
}

#[ic_cdk::update]
fn add(name: String, data: String) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        db.execute(
            "INSERT INTO person (name, data) VALUES (?1, ?2)",
            (&name, &data),
        )
        .unwrap();
    });
}

#[ic_cdk::query]
fn list() -> Vec<(u64, String, String)> {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        let mut stmt = db.prepare("SELECT id, name, data FROM person").unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0).unwrap(),
                    row.get(1).unwrap(),
                    row.get(2).unwrap(),
                ))
            })
            .unwrap();
        let mut result = vec![];
        for person in rows {
            result.push(person.unwrap());
        }
        result
    })
}

#[ic_cdk::init]
fn init() {
    ic_wasi_polyfill::init(0);

    DB.with(|db| {
        let mut db = db.borrow_mut();
        *db = Some(Connection::open("db.db3").unwrap());
        let db = db.as_mut().unwrap();
        db.execute(
            "CREATE TABLE person (
                id    INTEGER PRIMARY KEY,
                name  TEXT NOT NULL,
                data  TEXT
           )",
            (), // empty list of parameters.
        )
        .unwrap();
    });
}
```

Once the file is updated, setup the environment variables to be able to compile using `clang` for WASI:
```bash
export CC=/opt/wasi-sdk/bin/clang
```

Now, build the wasm-wasi project with the command:
```bash
cargo build --release --target wasm32-wasi
```

## Deployment and testing

In a separate terminal start the `dfx` environment:
```bash
dfx start
```

Go to the `demo3` project folder and deploy the canister:
```bash
dfx canister create --l
```

Now, use the `wasi2ic` tool to re-route the dependencies:
```bash
wasi2ic demo3_backend.wasm no_wasi.wasm
```

The file is likely to exceed the deployment limit, use compression:
```bash
gzip no_wasi.wasm
```
This creates a compressed file `no_wasi.wasm.gz`.

Now, deploy the canister:
```bash
dfx canister install --mode reinstall --wasm no_wasi.wasm.gz demo3_backend
```

Try running commands to update and query the database. To add a person, run:
```bash
dfx canister call demo3_backend add John Test1
```

To retrieve the persons stored, use:
```bash
dfx canister call demo3_backend greet test_hello
```

