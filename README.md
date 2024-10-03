# demo3 - Run Rusqlite on the IC

This project shows how to compilte the Rusqlite dependency in order to build the IC canister with the sqlite database.


## Prerequisites

It is assumed that you have [rust](https://doc.rust-lang.org/book/ch01-01-installation.html), [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install/), and [wasi2ic](https://github.com/wasm-forge/wasi2ic) installed.

You will also need the Wasm-oriented [clang](https://github.com/WebAssembly/wasi-sdk/releases/) installation. In this tutorial we use the `.deb` package [installation](https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-21/wasi-sdk_21.0_amd64.deb). Once installed the clang compiler is available from the path `/opt/wasi-sdk/bin/`. The additional builtins library will be found in `/opt/wasi-sdk/lib/clang/17/lib/wasi/`. 


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

cargo add ic-stable-structures

cargo add ic-wasi-polyfill

cargo add rusqlite rusqlite -F wasm32-wasi-vfs,bundled
```

Modify the demo3/src/demo3_backend/src/lib.rs file containing the greet method so that it uses the rusqlite backend to store a list of persons:
```rust
use std::cell::RefCell;

use candid::CandidType;
use candid::Deserialize;
use rusqlite::types::Type;
use rusqlite::Connection;

use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};

thread_local! {
    static DB: RefCell<Option<Connection>> = RefCell::new(None);
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

const MOUNTED_MEMORY_ID: u8 = 20;
const DB_FILE_NAME: &str = "db.db3";


#[ic_cdk::update]
fn add(name: String, data: String, age: u32) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        db.execute(
            "INSERT INTO person (name, data, age) VALUES (?1, ?2, ?3)",
            (&name, &data, age),
        )
        .unwrap();
    });
}

#[ic_cdk::query]
fn list() -> Vec<(u64, String, String, u32)> {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        let mut stmt = db
            .prepare("SELECT id, name, data, age FROM person")
            .unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0).unwrap(),
                    row.get(1).unwrap(),
                    row.get(2).unwrap(),
                    row.get(3).unwrap(),
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
                Ok(row) => match row {
                    Some(row) => {
                        let mut vec: Vec<String> = Vec::new();
                        for idx in 0..cnt {
                            let v = row.get_ref_unwrap(idx);
                            match v.data_type() {
                                Type::Null => vec.push(String::from("")),
                                Type::Integer => vec.push(v.as_i64().unwrap().to_string()),
                                Type::Real => vec.push(v.as_f64().unwrap().to_string()),
                                Type::Text => vec.push(v.as_str().unwrap().parse().unwrap()),
                                Type::Blob => vec.push(hex::encode(v.as_blob().unwrap())),
                            }
                        }
                        res.push(vec)
                    }
                    None => break,
                },
                Err(err) => {
                    return Err(Error::CanisterError {
                        message: format!("{:?}", err),
                    })
                }
            }
        }
        Ok(res)
    })
}


fn mount_memory_files() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 200..210);

        // mount virtual memory as file for faster DB operations
        let memory = m.get(MemoryId::new(MOUNTED_MEMORY_ID));
        ic_wasi_polyfill::mount_memory_file(DB_FILE_NAME, Box::new(memory));
    });
}

fn open_database() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        *db = Some(Connection::open(DB_FILE_NAME).unwrap());
    });

}

fn create_tables() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        db.execute(
            "CREATE TABLE person IF NOT EXISTS (
                id    INTEGER PRIMARY KEY,
                name  TEXT NOT NULL,
                data  TEXT,
                age   INTEGER
           )",
            (), // empty list of parameters.
        )
        .unwrap();
    });
}

fn set_pragmas() {
    // set pragmas
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        // do not create and destroy the journal file every time, set its size to 0 instead
        db.pragma_update(None, "journal_mode", &"TRUNCATE" as &dyn ToSql).unwrap();
        
        // reduce synchronizations
        db.pragma_update(None, "synchronous", &0 as &dyn ToSql).unwrap();
        
        // use fewer writes to disk with larger memory chunks
        // Note: values above 16384 cause I/O errors for some reason
        db.pragma_update(None, "page_size", &16384 as &dyn ToSql).unwrap();

        // reduce locks and unlocks
        db.pragma_update(None, "locking_mode", &"EXCLUSIVE" as &dyn ToSql).unwrap();
        
        // temp_store = MEMORY, disables creating temp files, improves performance, 
        // this workaround also avoids sqlite error on complex queries
        db.pragma_update(None, "temp_store", &2 as &dyn ToSql).unwrap();

        // add this option to minimize disk reads and work in canister memory instead
        //db.pragma_update(None, "cache_size", &1000000 as &dyn ToSql).unwrap();
    });    

}

#[ic_cdk::init]
fn init() {
    mount_memory_files();
    open_database();
    set_pragmas();
    create_tables();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    mount_memory_files();
    open_database();
    set_pragmas();
}


#[derive(CandidType, Deserialize)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

type QueryResult<T = Vec<Vec<String>>, E = Error> = std::result::Result<T, E>;

```

Finally, add the `build.rs` file into the `demo3/src/demo3_backend/` folder with the following content:
```rust
fn main() {
    println!("cargo:rustc-link-search=/opt/wasi-sdk/lib/clang/17/lib/wasi/");
    println!("cargo:rustc-link-arg=-lclang_rt.builtins-wasm32");
}
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
export CC_wasm_wasi=/opt/wasi-sdk/bin/clang --sysroot=/opt/wasi-sdk/share/wasi-sysroot
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
The benchmark source is located in the `benchmark` folder.

### Performance based on the stable structures:

| SQL <br/> commands               | performance counter <br/> 10K rows | performance counter <br/> 100K rows | performance counter <br/> 500K rows | performance counter <br/> 1M rows |
|----------------------------------|------------------------------------------------|-------------------------------------------------|-------------------------------------------------|--------------------------------------------------|
| create table                     | 5235895                                        | 6477727                                         | 9112574                                         | 12728842                                         | 
| create index <br/> (empty table) | 5077409                                        | 6313159                                         | 8611051                                         | 10063972                                         |
| count                            | 37626                                          | 99157                                          | 18184213                                        | 66793003                                         | 
| insert                           | 2900744                                         | 2892191                                          | 2889744                                          | 3037502                                           | 
| select <br/> (where primary key) | 51482                                         | 52810                                          | 106911                                          | 107700                                           | 
| select <br/> (where index field) | 66882                                         | 68559                                          | 69719                                          | 126714                                           | 
| select <br/> (where like field)  | 12422579                                      | 124389782                                      | 668933716                                      | 1338585422                                       | 
| update <br/> (where primary key) | 35237                                        | 35908                                         | 89981                                         | 90747                                          | 
| update <br/> (where index filed) | 65888                                         | 66842                                          | 121665                                          | 120353                                           | 
| delete <br/> (where primary key) | 26445                                          | 27123                                           | 81252                                           | 81983                                            |

