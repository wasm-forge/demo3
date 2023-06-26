#!/bin/bash
set -x

dfx canister call demo3_backend add '("Amy","test1", 25: nat32)'

dfx canister call demo3_backend add '("John","test2", 34: nat32)'

dfx canister call demo3_backend add '("Mark","test3", 19: nat32)'

