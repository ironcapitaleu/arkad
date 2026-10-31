---
last-verified: 2026-10-29
update-frequency: on-new-measurement
source: files-in-flight runs of `notes_backfill` on 2026-10-29, recorded in the Linear document "notes_backfill Network and Stream Cut-Off: Findings"
---

# Network Profiling: Where the Notes Download Is Limited

This reference answers one question: what limits the download of `notes_backfill`? The candidates
are the network link, a cap of the SEC on each stream, the CPU, the disk, and memory.

The script `scripts/fif_experiment.py` runs the whole method and cleans up after itself. The
results live in the Linear document
[notes_backfill Network and Stream Cut-Off: Findings](https://linear.app/state-machine/document/notes-backfill-network-and-stream-cut-off-findings-2238f0c1e060),
not here. See "Reporting" in `SKILL.md`.

## Method

Run both steps on the same machine, at the same time of day.

1. **curl test.** Download one Notes file alone. Then download four Notes files in parallel. Note
   the total MB/s of each.
2. **Files-in-flight runs.** Build `notes_backfill` from a scratch worktree of `origin/main`.
   Replace the computed periods in `main.rs` with four large periods, one `tokio::spawn` per
   period. Run it with
   `FILES_IN_FLIGHT` 2, 3, and 4, two runs each, in the order 2, 3, 4, 4, 3, 2. The order spreads
   the change in network speed over the day across all three values.

For each run, record the total duration, the peak resident memory (`ru_maxrss` via `os.wait4`), the
download time of each file from its `state_completed` event, and every failure. The script prints
all four: the table, then each failure below it.

```sh
ARKAD_SEC_USER_AGENT="Company Name email@domain.com" \
  python3 .claude/skills/performance/scripts/fif_experiment.py --workdir "$HOME/arkad-fif-experiment"
```

The `--workdir` must not exist yet, because the script deletes it at the end. `--periods` and
`--order` change the files and the runs. Two small periods, such as `2009q1 2009q2`, check the
script in about two minutes of build time.

The script downloads about 2 GB per run from the live SEC. This is the exploratory exception that
the invariant in `SKILL.md` allows. Never assert against these numbers.

## How to Read the Results

- **The link is the limit** if four parallel curl downloads add up to about the speed of one alone.
  More files in flight then only make each file slower, and the total time stays the same.
- **The SEC limits each stream** if each parallel stream gets about the speed of one alone. The total
  then grows with the number of streams, and a higher `FILES_IN_FLIGHT` pays off.
- **Our code is the limit** if `notes_backfill` stays well below the curl speed on the same machine.
  Profile the CPU next, with `cpu.md`.
- **Memory** grows with each file in flight, because each one holds its body in memory. Read the
  peak per extra file from the runs, and compare it with the memory of the Cloud Run job.

## The Decision

`notes_backfill` keeps `FILES_IN_FLIGHT = 2`. On 2026-10-29 the network was the limit in the cloud
container and on the lead's Mac, so more files in flight raised the memory and not the speed.
Measure once on Cloud Run before you change the value. The design doc records the same decision in
`sec/design/pipeline/ingestion_paths.md`, section "Memory and the Cap on Files in Flight".

## Testing a Faster Machine

The findings document lists open hypotheses for a wired 10 Gbit machine, and its hardware
checklist. Run the curl test first on such a machine. If one stream gets well above the speed of the
lead's Wi-Fi, a wired line pays off, and the files-in-flight runs are worth repeating there.

## Peak Memory on macOS and on Linux

`ru_maxrss` is in bytes on macOS and in KB on Linux. The script converts both to KB. Cloud Run
runs Linux, so plan its memory from a Linux run.

## Rules

- Never commit downloads. The script writes them under its `--workdir` and deletes it at the end.
- Use a real User-Agent in `ARKAD_SEC_USER_AGENT`. The SEC blocks requests without one.
- Give every path with `$HOME` or as an absolute path. Never write a quoted `~`: the shell does not
  expand it, and the files land in a folder named `~` inside the repository.
- Clean up afterwards: the downloads, the worktree, and the build directory. The script does all
  three, also when a run fails.
