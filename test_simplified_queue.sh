#!/bin/bash
echo "Running simplified Queue tests..."
cd /Users/demircatovic/Projects/gold/arkad
cargo test --lib -p utils queue
echo "Compiling simplified queue example..."
cargo build --bin simplified_queue_example -p utils