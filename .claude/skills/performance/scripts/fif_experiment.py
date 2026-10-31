#!/usr/bin/env python3
"""Files-in-flight experiment for `notes_backfill`.

Method and how to read the results: `../references/profiling/network.md`.

1. A curl test: one Notes file alone, then the same files in parallel.
2. A scratch worktree from `origin/main`, one release build, and the runs in the order 2, 3, 4,
   4, 3, 2. Each run sets ARKAD_FILES_IN_FLIGHT to its value. Each run reports its duration, its peak resident memory
   (`os.wait4`), the download time of each file, and its failures.
3. Cleanup: every download, the worktree, and the build directory.

The script downloads from the live SEC. It needs a real User-Agent in `ARKAD_SEC_USER_AGENT`.
Standard library only, so it runs with any Python 3.9 or later on macOS or Linux (`os.wait4` is
Unix only).
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

NOTES_URL = "https://www.sec.gov/files/dera/data/financial-statement-notes-data-sets/{}_notes.zip"
DEFAULT_PERIODS = ["2025q1", "2024q4", "2024q3", "2024q1"]
DEFAULT_ORDER = [2, 3, 4, 4, 3, 2]
# .claude/skills/performance/scripts/fif_experiment.py -> the repository root
REPO_ROOT = Path(__file__).resolve().parents[4]


def main():
    args = parse_args()
    user_agent = os.environ.get("ARKAD_SEC_USER_AGENT")
    if not user_agent:
        sys.exit("Set ARKAD_SEC_USER_AGENT to 'Company Name email@domain.com' first.")
    workdir = Path(args.workdir).expanduser().resolve()
    # The cleanup deletes the whole workdir, so it must be a new directory.
    if workdir.exists():
        sys.exit(f"{workdir} exists. Pass a --workdir that does not exist yet.")
    workdir.mkdir(parents=True)
    try:
        if not args.skip_curl:
            curl_test(args.periods, user_agent, workdir / "curl")
        if not args.skip_runs:
            binary = build_binary(args, workdir)
            rows = [run_once(binary, n, workdir, user_agent) for n in args.order]
            print_table(rows)
    finally:
        cleanup(args.repo, workdir)


def parse_args():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--repo", default=str(REPO_ROOT), help="arkad checkout with an origin remote")
    parser.add_argument("--workdir", default=str(Path.home() / "arkad-fif-experiment"),
                        help="new scratch directory, created here and deleted at the end")
    parser.add_argument("--periods", nargs="+", default=DEFAULT_PERIODS)
    parser.add_argument("--order", nargs="+", type=int, default=DEFAULT_ORDER,
                        help="ARKAD_FILES_IN_FLIGHT value of each run, in run order")
    parser.add_argument("--skip-curl", action="store_true")
    parser.add_argument("--skip-runs", action="store_true")
    return parser.parse_args()


def curl_test(periods, user_agent, directory):
    directory.mkdir(parents=True, exist_ok=True)
    alone = curl_downloads(periods[:1], user_agent, directory)
    parallel = curl_downloads(periods, user_agent, directory)
    print(f"curl, 1 file alone: {mb_per_s(*alone):.1f} MB/s")
    print(f"curl, {len(periods)} files in parallel: {mb_per_s(*parallel):.1f} MB/s in total")
    shutil.rmtree(directory)


def curl_downloads(periods, user_agent, directory):
    """Downloads the periods at once and returns (total bytes, wall seconds)."""
    start = time.time()
    processes = [
        subprocess.Popen(
            ["curl", "-sSf", "-A", user_agent, "-o", str(directory / f"{period}.zip"),
             "-w", "%{size_download}", NOTES_URL.format(period)],
            stdout=subprocess.PIPE, text=True)
        for period in periods
    ]
    sizes = [int(float(process.communicate()[0] or 0)) for process in processes]
    if any(process.returncode != 0 for process in processes):
        sys.exit("A curl download failed. Check the User-Agent and the network.")
    return sum(sizes), time.time() - start


def mb_per_s(total_bytes, seconds):
    return total_bytes / seconds / 1e6


def build_binary(args, workdir):
    tree, target = workdir / "tree", workdir / "target"
    git(args.repo, "fetch", "-q", "origin", "main")
    git(args.repo, "worktree", "add", "-q", "--detach", str(tree), "origin/main")
    set_periods(tree, args.periods)
    subprocess.run(
        ["cargo", "build", "--release", "--features", "tracing-logging", "--bin", "notes_backfill"],
        cwd=tree, env=dict(os.environ, CARGO_TARGET_DIR=str(target)), check=True)
    binary = workdir / "notes_backfill"
    shutil.copy2(target / "release" / "notes_backfill", binary)
    return binary


def set_periods(tree, periods):
    path = tree / "sec/src/bin/notes_backfill/main.rs"
    listed = ", ".join(f'"{period}".to_string()' for period in periods)
    replace_once(path, r"let periods = notes_periods\(Utc::now\(\)\.date_naive\(\)\);",
                 f"let periods: Vec<String> = vec![{listed}];")


def replace_once(path, pattern, replacement):
    text = path.read_text()
    new_text, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        sys.exit(f"Pattern not found in {path}. The file changed on main, so update this script.")
    path.write_text(new_text)


def run_once(binary, files_in_flight, workdir, user_agent):
    data, log = workdir / "data", workdir / "run.jsonl"
    data.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, ARKAD_STORAGE_ROOT=str(data), ARKAD_SEC_USER_AGENT=user_agent,
               ARKAD_FILES_IN_FLIGHT=str(files_in_flight))
    start = time.time()
    with open(log, "w") as out:
        process = subprocess.Popen([str(binary)], env=env, stdout=out, stderr=subprocess.STDOUT)
        _, status, usage = os.wait4(process.pid, 0)
    seconds = time.time() - start
    # ru_maxrss is in bytes on macOS and in KB on Linux.
    peak_kb = usage.ru_maxrss // 1024 if sys.platform == "darwin" else usage.ru_maxrss
    downloads, failures, used = read_log(log)
    shutil.rmtree(data)
    log.unlink()
    # A build without ARKAD_FILES_IN_FLIGHT ignores the variable and logs no value, so `used` is
    # "none". A run that dies before its batch event (an OOM kill) leaves `used` as None, and its
    # row keeps the exit code.
    if used is not None and used != files_in_flight:
        sys.exit(f"The binary ran with {used} files in flight, not {files_in_flight}. "
                 "Build a commit that reads ARKAD_FILES_IN_FLIGHT.")
    return {"files_in_flight": files_in_flight, "exit": os.waitstatus_to_exitcode(status),
            "seconds": seconds, "peak_kb": peak_kb, "downloads": downloads, "failures": failures}


def read_log(log):
    """Returns the download seconds per period, the error lines, and the files in flight of one run."""
    downloads, failures, used = {}, [], None
    for line in log.read_text(errors="replace").splitlines():
        try:
            event = json.loads(line)
            context = json.loads(event.get("context") or "{}")
        except (json.JSONDecodeError, TypeError):
            continue
        if event.get("event") in ("batch_complete", "batch_failed"):
            used = context.get("files_in_flight", "none")
        if event.get("event") == "state_completed" and context.get("state") == "Execute SEC Request":
            downloads[context.get("period")] = context.get("event_duration_ms", 0) / 1000
        # `batch_failed` is the summary of the failed periods, not a failure of its own.
        if event.get("level") == "ERROR" and event.get("event") != "batch_failed":
            failures.append(line)
    return downloads, failures, used


def print_table(rows):
    print("| ARKAD_FILES_IN_FLIGHT | Exit | Duration | Peak | Mean download per file "
          "| Download per file | Failures |")
    print("| --- | --- | --- | --- | --- | --- | --- |")
    for row in rows:
        times = row["downloads"].values()
        mean = f"{sum(times) / len(times):.1f} s" if times else "-"
        per_file = ", ".join(
            f"{period} {seconds:.1f} s" for period, seconds in sorted(row["downloads"].items())
        ) or "-"
        print(f"| {row['files_in_flight']} | {row['exit']} | {row['seconds']:.1f} s "
              f"| {row['peak_kb']:,} KB | {mean} | {per_file} | {len(row['failures'])} |")
    for row in rows:
        for failure in row["failures"]:
            print(f"\nFailure, ARKAD_FILES_IN_FLIGHT {row['files_in_flight']}:\n{failure}")


def cleanup(repo, workdir):
    # No `check=True`: a failed git call must not stop the deletion or hide the original error.
    tree = workdir / "tree"
    if tree.exists():
        subprocess.run(["git", "-C", str(repo), "worktree", "remove", "--force", str(tree)])
    shutil.rmtree(workdir, ignore_errors=True)
    subprocess.run(["git", "-C", str(repo), "worktree", "prune"])
    if workdir.exists():
        print(f"Could not delete all of {workdir}. Delete it by hand.")
    else:
        print(f"Deleted {workdir} and the worktree.")


def git(repo, *arguments):
    subprocess.run(["git", "-C", str(repo), *arguments], check=True)


if __name__ == "__main__":
    main()
