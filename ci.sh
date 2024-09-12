#!/bin/bash

set -euxo pipefail

cargo test
cargo build --target thumbv6m-none-eabi --features portable-atomic/unsafe-assume-single-core
cargo build --target thumbv7em-none-eabi
