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

cargo add hex

cargo add serde

cargo add serde_json

cargo add --git https://github.com/wasm-forge/ic-wasi-polyfill

cargo add --git https://github.com/rusqlite/rusqlite rusqlite -F wasm32-wasi-vfs,bundled
```

Modify the demo3/src/demo3_backend/src/lib.rs file containing the greet method so that it uses the rusqlite backend to store a list of persons:
```rust
#[macro_use]
extern crate serde;

use std::cell::RefCell;

use candid::CandidType;
use rusqlite::Connection;
use rusqlite::types::Type;

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

#[ic_cdk::query]
fn query(sql: String) -> QueryResult {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let mut stmt = db.prepare(&sql).unwrap();
        let cnt = stmt.column_count();
        let mut rows = stmt.query([]).unwrap();
        let mut res: Vec<Vec<String>> = Vec::new();
    
        loop {
            match rows.next() {
                Ok(row) => {
                    match row {
                        Some(row) => {
                            let mut vec: Vec<String> = Vec::new();
                            for idx in 0..cnt {
                                let v = row.get_ref_unwrap(idx);
                                match v.data_type() {
                                    Type::Null => {  vec.push(String::from("")) }
                                    Type::Integer => { vec.push(v.as_i64().unwrap().to_string()) }
                                    Type::Real => { vec.push(v.as_f64().unwrap().to_string()) }
                                    Type::Text => { vec.push(v.as_str().unwrap().parse().unwrap()) }
                                    Type::Blob => { vec.push(hex::encode(v.as_blob().unwrap())) }
                                }
                            }
                            res.push(vec)
                        },
                        None => break
                    }
                },
                Err(err) => return Err(Error::CanisterError {message: format!("{:?}", err) })
            }
        }
        Ok(res)
    })
}


#[ic_cdk::init]
fn init() {
    unsafe {
        ic_wasi_polyfill::init(&[0u8;32]);
    }

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


#[derive(CandidType, Deserialize)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}


type QueryResult<T = Vec<Vec<String>>, E = Error> = std::result::Result<T, E>;

```

## Deployment and testing

In a separate terminal start the `dfx` environment:
```bash
dfx start
```

Go to the `demo3` project folder and create the canister:
```bash
dfx canister create --all
```

Setup the environment variables to be able to compile using `clang` for WASI:
```bash
export CC=/opt/wasi-sdk/bin/clang
```

Now, build the wasm-wasi project with the command:
```bash
cargo build --release --target wasm32-wasi
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
dfx canister call demo3_backend add '("Amy","test1")'

dfx canister call demo3_backend add '("John","test2")'

dfx canister call demo3_backend add '("Mark","test3")'
```

To retrieve the persons stored, use:
```bash
dfx canister call demo3_backend list
```

## Performance benchmarks for SQL commands


The performance estimation was made similar to the one in the alternative [project](https://github.com/froghub-io/ic-sqlite/) implementing the sqlite in IC canister.
The benchmark source is located in the benchmark folder.

### Performance based on the stable structures:

| SQL <br/> commands               | performance counter <br/> 10K rows | performance counter <br/> 100K rows | performance counter <br/> 500K rows | performance counter <br/> 1M rows |
|----------------------------------|------------------------------------------------|-------------------------------------------------|-------------------------------------------------|--------------------------------------------------|
| create table                     | 5235895                                        | 6477727                                         | 9112574                                         | 12728842                                         | 
| create index <br/> (empty table) | 5077409                                        | 6313159                                         | 8611051                                         | 10063972                                         |
| count                            | 520789                                         | 102162132                                       | 570086368                                       | 1202162011                                       | 
| insert                           | 7083561                                        | 8429386                                         | 7670133                                         | 8038125                                          | 
| select <br/> (where primary key) | 569666                                         | 599329                                          | 632169                                          | 670414                                           | 
| select <br/> (where index field) | 610937                                         | 642083                                          | 675611                                          | 716432                                           | 
| select <br/> (where like field)  | 153412973                                      | 1637608328                                      | limit for single message execution              | limit for single message execution               | 
| update <br/> (where primary key) | 8100314                                        | 9622378                                         | 8991314                                         | 9690872                                          | 
| update <br/> (where index filed) | 531129                                         | 558131                                          | 591241                                          | 629889                                           | 
| delete <br/> (where primary key) | 10079160                                       | 8397133                                         | 11474854                                        | 11996039                                         |


### Performance based on the transient storage:

| SQL <br/> commands               | performance counter <br/> 10K rows | performance counter <br/> 100K rows | performance counter <br/> 500K rows | performance counter <br/> 1M rows |
|----------------------------------|------------------------------------------------|-------------------------------------------------|-------------------------------------------------|--------------------------------------------------|
| create table                     | 1355942                                        | 1590464                                         | 2701306                                         | 4254880                                          | 
| create index <br/> (empty table) | 1063520                                        | 1297837                                         | 2455461                                         | 3686834                                          |
| count                            | 243819                                         | 11838005                                        | 62710202                                        | 126003479                                        | 
| insert                           | 662632                                         | 819774                                          | 822305                                          | 830358                                           | 
| select <br/> (where primary key) | 293424                                         | 300705                                          | 320130                                          | 346628                                           | 
| select <br/> (where index field) | 334820                                         | 343590                                          | 363634                                          | 392090                                           | 
| select <br/> (where like field)  | 153135930                                      | 1541036699                                      | limit for single message execution              | limit for single message execution               | 
| update <br/> (where primary key) | 826795                                         | 915811                                          | 928460                                          | 935750                                           | 
| update <br/> (where index filed) | 254355                                         | 259787                                          | 279842                                          | 306150                                           | 
| delete <br/> (where primary key) | 1018120                                        | 737666                                          | 1101588                                         | 1342517                                          |


