//#[macro_use]
//extern crate serde;

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

#[ic_cdk::init]
fn init() {

    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 200..210);

        // mount virtual memory as file for faster DB operations
        let memory = m.get(MemoryId::new(MOUNTED_MEMORY_ID));
        ic_wasi_polyfill::mount_memory_file(DB_FILE_NAME, Box::new(memory));
    });

    DB.with(|db| {
        let mut db = db.borrow_mut();
        *db = Some(Connection::open(DB_FILE_NAME).unwrap());
        let db = db.as_mut().unwrap();
        db.execute(
            "CREATE TABLE person (
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

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 200..210);

        // after upgrade we still need to explicitly mount the memory (otherwise the usual file is used)
        // its metadata infromation was stored in the file system and no other init steps are necessary
        let memory = m.get(MemoryId::new(MOUNTED_MEMORY_ID));
        ic_wasi_polyfill::mount_memory_file(DB_FILE_NAME, Box::new(memory));
    });

    DB.with(|db| {
        let mut db = db.borrow_mut();
        *db = Some(Connection::open(DB_FILE_NAME).unwrap());
    });
}


#[derive(CandidType, Deserialize)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

type QueryResult<T = Vec<Vec<String>>, E = Error> = std::result::Result<T, E>;
