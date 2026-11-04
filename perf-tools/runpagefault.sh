#!/bin/bash

cargo build --bin 'page-fault' --release
perf stat -e page-faults -- target/release/page-fault ../haversine/pairs.json
