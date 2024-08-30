
use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};

use std::cell::RefCell;
use ic_cdk::api::call::RejectionCode;
use candid::CandidType;
use rusqlite::Connection;
use rusqlite::types::Type;

const WASI_MEMORY_ID: u8 = 50;

const MOUNTED_MEMORY_ID: u8 = 20;
const DB_FILE_NAME: &str = "db.db3";
const FAST_FILE_NAME: &str = "db.db3";

thread_local! {
    static DB: RefCell<Option<Connection>> = RefCell::new(None);
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}


#[ic_cdk::update]
fn execute(sql: String) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        return match db.execute(&sql, ()) {
            Ok(_) => Ok(format!("execute performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("execute {:?}", err) })
        }
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


#[ic_cdk::query]
fn count(table_name: String) -> Result {

    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let mut stmt = match db.prepare(&format!("select count(*) from {:?}", table_name)) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("{:?}", err) })
        };
        let mut iter = match stmt.query_map([], |row| {
            let count: u64 = row.get(0).unwrap();
            Ok(count)
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("count: {:?}", err) })
        };
        let count = iter.next().unwrap().unwrap();
    
        ic_cdk::eprintln!("count: {:?}", count);
    
        Ok(format!("count performance_counter: {:?}", ic_cdk::api::performance_counter(0)))
    
    })

}

#[ic_cdk::update]
fn bench1_insert_person(offset: usize, count: usize) -> Result {

    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();
        
        let mut i = 0;
        let step = 1000usize;
        
        while i < count {

            let id = offset + i + 1;
            let mut sql = String::from("insert into person (name, age, gender) values\n");
        
            for j in 0..step {
                sql.push_str(format!("('person{}', {}, {})", id + j, 18 + (id + j) % 10, (id + j) % 2).as_str());
                if j < step-1 {
                    sql.push_str(",\n")
                } else {
                    sql.push_str(";\n")
                }
            }

            match db.execute(
                sql.as_str(),
                ()
            ) {
                Ok(_) => {},
                Err(err) =>  return Err(Error::CanisterError {message: format!("bench1_insert_person: {:?}", err) })
            }

            i += step;
        }
        Ok(String::from("bench1_insert_person OK"))

    })
}

#[ic_cdk::update]
fn bench1_insert_person_one(offset: usize) -> Result {


    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        match db.execute(
            "insert into person (name, age, gender) values (?1, ?2, ?3);",
            (format!("person{:?}", id), 18 + id % 10, id % 2)
        ) {
            Ok(_) => Ok(format!("insert performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("insert: {:?}", err) })
        }
    })

}

#[ic_cdk::query]
fn bench1_query_person_by_id(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        let mut stmt = match db.prepare("select * from person where id=?1") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_id: {:?}", err) })
        };
        let iter = match stmt.query_map((id,), |row| {
            Ok(Person {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_id: {:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_id: {:?}", res);
        Ok(format!("query_by_id performance_counter: {:?}", ic_cdk::api::performance_counter(0)))
    
    })


}

#[ic_cdk::query]
fn bench1_query_person_by_name(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let name = format!("person{:?}", offset + 1);
        let mut stmt = match db.prepare("select * from person where name=?1") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_name: {:?}", err) })
        };
        let iter = match stmt.query_map((name,), |row| {
            Ok(Person {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_name: {:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_name: {:?}", res);
        Ok(format!("query_by_name performance_counter: {:?}", ic_cdk::api::performance_counter(0)))

    })


}

#[ic_cdk::query]
fn bench1_query_person_by_like_name(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();


        let name = format!("person{:?}", offset + 1);
        let mut stmt = match db.prepare("select * from person where name like ?1") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("{:?}", err) })
        };
        let iter = match stmt.query_map((format!("{:?}%", name),), |row| {
            Ok(Person {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("{:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_like_name: {:?}", res);
        Ok(format!("query_by_like_name performance_counter: {:?}", ic_cdk::api::performance_counter(0)))
            
    })

}

#[ic_cdk::query]
fn bench1_query_person_by_limit_offset(limit: usize, offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();


        let mut stmt = match db.prepare("select * from person limit ?1 offset ?2") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_limit_offset: {:?}", err) })
        };
        let iter = match stmt.query_map((limit, offset), |row| {
            Ok(Person {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_limit_offset: {:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_limit_offset: {:?}", res);
        Ok(format!("query_by_limit_offset performance_counter: {:?}", ic_cdk::api::performance_counter(0)))

    })

}

#[ic_cdk::update]
fn bench1_update_person_by_id(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        return match db.execute(
            "update person set name=?1 where id=?2",
            (String::from("person_id"), id)
        ) {
            Ok(_) => Ok(format!("update_by_id performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("{:?}", err) })
        }
    })
}

#[ic_cdk::update]
fn bench1_update_person_by_name(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let name = format!("{:?}", offset + 1);
        return match db.execute(
            "update person set name=?1 where name=?2",
            (String::from("person_name"), name)
        ) {
            Ok(_) => Ok(format!("update_by_name performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("update_by_name: {:?}", err) })
        }
    })
}

#[ic_cdk::update]
fn bench1_delete_person_by_id(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        return match db.execute(
            "delete from person where id=?1",
            (id,)
        ) {
            Ok(_) => Ok(format!("delete performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("delete: {:?}", err) })
        }
    })

}

#[ic_cdk::update]
fn bench2_insert_person2(offset: usize, count: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();


        for i in 0..count {

            let id = offset + i + 1;
            let data = "0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a";
            match db.execute(
                "insert into person2 (name, age, gender, data) values (?1, ?2, ?3, ?4);",
                (format!("person2{:?}", id), 18 + id % 10, id % 2, &data)
            ) {
                Ok(_) => {},
                Err(err) =>  return Err(Error::CanisterError {message: format!("bench2_insert_person2: {:?}", err) })
            }
        }
        Ok(String::from("bench2_insert_person2 OK"))
    })

}

#[ic_cdk::update]
fn bench2_insert_person2_one(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        let data = "0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a";
        match db.execute(
            "insert into person2 (name, age, gender, data) values (?1, ?2, ?3, ?4);",
            (format!("person2{:?}", id), 18 + id % 10, id % 2, &data)
        ) {
            Ok(_) => Ok(format!("insert performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("insert: {:?}", err) })
        }
    })

}

#[ic_cdk::query]
fn bench2_query_person2_by_id(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        let mut stmt = match db.prepare("select * from person2 where id=?1") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_id: {:?}", err) })
        };
        let iter = match stmt.query_map((id,), |row| {
            Ok(Person2 {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap(),
                data: row.get(4).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_id: {:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_id: {:?}", res);
        Ok(format!("query_by_id performance_counter: {:?}", ic_cdk::api::performance_counter(0)))
    })
}

#[ic_cdk::query]
fn bench2_query_person2_by_name(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let name = format!("person2{:?}", offset + 1);
        let mut stmt = match db.prepare("select * from person2 where name=?1") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_name: {:?}", err) })
        };
        let iter = match stmt.query_map((name,), |row| {
            Ok(Person2 {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap(),
                data: row.get(4).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_name: {:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_name: {:?}", res);
        Ok(format!("query_by_name performance_counter: {:?}", ic_cdk::api::performance_counter(0)))
    })

}

#[ic_cdk::query]
fn bench2_query_person2_by_like_name(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let name = format!("person2{:?}", offset + 1);
        let mut stmt = match db.prepare("select * from person2 where name like ?1") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("{:?}", err) })
        };
        let iter = match stmt.query_map((format!("{:?}%", name),), |row| {
            Ok(Person2 {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap(),
                data: row.get(4).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("{:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_like_name: {:?}", res);
        Ok(format!("query_by_like_name performance_counter: {:?}", ic_cdk::api::performance_counter(0)))
    })

}

#[ic_cdk::query]
fn bench2_query_person2_by_limit_offset(limit: usize, offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let mut stmt = match db.prepare("select * from person2 limit ?1 offset ?2") {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_limit_offset: {:?}", err) })
        };
        let iter = match stmt.query_map((limit, offset), |row| {
            Ok(Person2 {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                age: row.get(2).unwrap(),
                gender: row.get(3).unwrap(),
                data: row.get(4).unwrap()
            })
        }) {
            Ok(e) => e,
            Err(err) => return Err(Error::CanisterError {message: format!("query_by_limit_offset: {:?}", err) })
        };
        let mut arr = Vec::new();
        for ite in iter {
            arr.push(ite.unwrap());
        }
        let res = serde_json::to_string(&arr).unwrap();
        ic_cdk::eprintln!("query_by_limit_offset: {:?}", res);
        Ok(format!("query_by_limit_offset performance_counter: {:?}", ic_cdk::api::performance_counter(0)))

    })

}

#[ic_cdk::update]
fn bench2_update_person2_by_id(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        return match db.execute(
            "update person2 set name=?1 where id=?2",
            (String::from("person2_id"), id)
        ) {
            Ok(_) => Ok(format!("update_by_id performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("{:?}", err) })
        }
    })


}

#[ic_cdk::update]
fn bench2_update_person2_by_name(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let name = format!("{:?}", offset + 1);
        return match db.execute(
            "update person2 set name=?1 where name=?2",
            (String::from("person2_name"), name)
        ) {
            Ok(_) => Ok(format!("update_by_name performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("update_by_name: {:?}", err) })
        }
    })


}

#[ic_cdk::update]
fn bench2_delete_person2_by_id(offset: usize) -> Result {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        let db = db.as_mut().unwrap();

        let id = offset + 1;
        return match db.execute(
            "delete from person2 where id=?1",
            (id,)
        ) {
            Ok(_) => Ok(format!("delete performance_counter: {:?}", ic_cdk::api::performance_counter(0))),
            Err(err) => Err(Error::CanisterError {message: format!("delete: {:?}", err) })
        }
    })


}

#[derive(CandidType, Debug, Serialize, Deserialize, Default)]
struct Person {
    id: u64,
    name: String,
    age: u32,
    gender: u8
}

#[derive(CandidType, Debug, Serialize, Deserialize, Default)]
struct Person2 {
    id: u64,
    name: String,
    age: u32,
    gender: u8,
    data: String,
}


fn open_database() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        *db = Some(Connection::open(DB_FILE_NAME).unwrap());
    });
}

#[ic_cdk::init]
pub fn init() {
    ic_wasi_polyfill::init(&[0u8;32], &[]);

    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, WASI_MEMORY_ID..WASI_MEMORY_ID + 10);

        let memory = m.get(MemoryId::new(MOUNTED_MEMORY_ID));
        ic_wasi_polyfill::mount_memory_file(FAST_FILE_NAME, Box::new(memory));
    });

    open_database();
}

#[ic_cdk::post_upgrade]
pub fn post_upgrade() {

    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, WASI_MEMORY_ID..WASI_MEMORY_ID + 10);

        let memory = m.get(MemoryId::new(MOUNTED_MEMORY_ID));
        ic_wasi_polyfill::mount_memory_file(FAST_FILE_NAME, Box::new(memory));
    });

    open_database();
}


#[derive(CandidType, Deserialize)]
enum Error {
    InvalidCanister,
    CanisterError { message: String },
}

type Result<T = String, E = Error> = std::result::Result<T, E>;

type QueryResult<T = Vec<Vec<String>>, E = Error> = std::result::Result<T, E>;

impl From<(RejectionCode, String)> for Error {
    fn from((code, message): (RejectionCode, String)) -> Self {
        match code {
            RejectionCode::CanisterError => Self::CanisterError { message },
            _ => Self::InvalidCanister,
        }
    }
}

