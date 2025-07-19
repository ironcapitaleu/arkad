#!/bin/bash
echo "Testing Queue with impl Into<String> trait bound..."
cd /Users/demircatovic/Projects/gold/arkad
cargo test --lib -p utils queue::tests::should_initialize_queue_correctly
echo "Compilation check..."
cargo check --lib -p utils