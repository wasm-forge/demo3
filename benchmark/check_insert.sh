#!/bin/bash
export backend=ic_rusqlite_bench_backend

set -e

./build.sh

./deploy.sh

# create table person
dfx canister call $backend execute 'create table if not exists person ( id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER, gender INTEGER )'

dfx canister call $backend bench1_insert_person "(0, 510000)"

# create person name index
dfx canister call $backend execute 'create index if not exists name on person(name)'

#dfx canister call $backend query '("select count(*) from person")'
