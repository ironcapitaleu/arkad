#!/bin/bash
echo "Testing updated queue implementation without defaults..."
cd /Users/demircatovic/Projects/gold/arkad
cargo test --lib -p utils queue::connection::tests