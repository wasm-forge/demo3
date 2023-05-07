#![no_std]
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
