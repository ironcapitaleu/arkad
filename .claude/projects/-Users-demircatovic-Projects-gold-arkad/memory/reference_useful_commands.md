---
name: Useful development commands
description: Frequently used commands for running binaries, viewing logs, and profiling pipeline performance
type: reference
---

## Running the streaming extraction binary

```sh
# Build and run with colored JSON logs
cargo build --release --bin stream_extract && ./target/release/stream_extract 2>&1 | jq -C .

# Run and show only pipeline summaries sorted by slowest
cargo run --release --bin stream_extract 2>&1 | grep "pipeline_complete" | python3 -c "
import sys, json
lines = []
for line in sys.stdin:
    obj = json.loads(line)
    ctx = json.loads(obj['context'])
    lines.append((int(ctx['duration_ms']), ctx['cik'], obj['message']))
lines.sort(key=lambda x: -x[0])
for dur, cik, msg in lines[:10]:
    print(f'{dur:>6}ms  CIK {cik:>10}  {msg}')
print(f'\nTotal: {len(lines)} pipelines')
"

# Run and follow one execution ID through the logs
cargo build --release --bin stream_extract && ./target/release/stream_extract 2>&1 | grep "<execution_id>" | jq -C .
```
