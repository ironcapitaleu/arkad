# ADR 0002: Static Binaries for Every Container

| Field | Value |
| ----- | ----- |
| Status | Accepted by the lead: the build and the base on 2026-11-01, one image per binary on 2026-11-03 |
| Date | 2026-11-03 |
| Deciders | The project lead |
| Ticket | [STA-437](https://linear.app/state-machine/issue/STA-437). Decisions recorded on [STA-422](https://linear.app/state-machine/issue/STA-422) and [STA-431](https://linear.app/state-machine/issue/STA-431) |

## Context

The plan runs `backfill` and `daily_filings` as Cloud Run jobs, and later `api` as a Cloud Run
service. Today the root `Dockerfile` builds one deprecated example on `rust:1.90-bookworm` and runs
it on `debian:bookworm-slim`.

Two facts of the code make a static build possible:

- TLS goes through `rustls`, with `aws-lc-rs` underneath. No crate in `Cargo.lock` links OpenSSL
  or `native-tls`.
- `daily_filings` needs the New York time zone. `chrono-tz` compiles the zone data into the binary,
  so no binary reads `/usr/share/zoneinfo`. Log timestamps are UTC.

A dynamic glibc binary must match the glibc of its runtime image. A binary built on Ubuntu 24.04
(glibc 2.39) needs `GLIBC_2.38` and does not start on `debian:bookworm-slim` (glibc 2.36). A build
stage on the same Debian release avoids this, but couples the build to the runtime. A static
binary removes this coupling.

## Decision

1. **Every binary that runs in a container is a static binary.** The build uses
   `cargo zigbuild --release --locked --target x86_64-unknown-linux-musl`. zig is the linker and
   the C compiler for the musl target, so the C code of `aws-lc-sys` builds without a separate musl
   toolchain.
2. **The runtime base is `gcr.io/distroless/static-debian13`.** Google maintains it. It holds the
   CA certificates, `/etc/passwd`, and `/tmp`, and it has no shell and no package manager.
3. **One container runs one binary.** The root `Dockerfile` has one shared build stage, which
   compiles every binary in one call, and one final stage per binary, for example
   `--target backfill`. Each image holds only its own binary, and its entrypoint runs that binary.
   The repository has one `Dockerfile`.
4. **Every image runs as user `10001:10001`.** The `/etc/passwd` and `/etc/group` of the image hold
   the distroless entries plus `arkad` (uid and gid 10001), with the home `/home/arkad`. That user
   owns `/home/arkad`. The image does not use the distroless user `nonroot` (65532), so the uid
   stays the same as in the current image. The image has no shell. After STA-431, the container
   CI check reads the user from the image config and these files from the exported file system.
5. **Fallback:** if a binary cannot build static, it uses `gcr.io/distroless/cc-debian13` with a
   glibc build on a `trixie` build stage. The PR that takes the fallback names the reason.

The pinned tool versions of the first build are zig 0.17.0 and `cargo-zigbuild` 0.24.0, on
`rust:1.97-trixie`. `cargo-zigbuild` 0.24.0 needs a newer Rust than the CI toolchain 1.90.

## Options Considered

Measured on 2026-11-01 on `main` at `8afab52`, with `backfill` and `daily_filings` in each image.
The live run was `daily_filings` for 2026-10-25 against the SEC.

| Option | Runtime base | Binaries | Pull size | Local size | Live run |
| ------ | ------------ | -------- | --------- | ---------- | -------- |
| A | `debian:bookworm-slim` | glibc, built on the host | 34.7 MB | 136 MB | Failed: the host binary needs `GLIBC_2.38`, and bookworm has 2.36. A bookworm build stage avoids this, but couples build and runtime |
| B | `gcr.io/distroless/cc-debian13` | glibc | 17.1 MB | 65 MB | Passed, 1.00 s |
| **C** | **`gcr.io/distroless/static-debian13`** | **static musl, zig** | **7.4 MB** | **28.3 MB** | **Passed, 0.35 s** |
| D | `scratch` | static musl, zig | 6.6 MB | 22 MB | Passed, 0.49 s |

- C against D: D saves 0.8 MB. But `scratch` has no `/etc/passwd`, no `/tmp`, and no maintained CA
  bundle. Its CA bundle is a copy from the build stage and changes only on a rebuild.
- B is the fallback. It keeps glibc and needs a build stage on the same Debian release.
- Not considered further: `distroless/base-debian13` (no `libgcc_s`), `alpine` (a shell and a
  package manager), and SlimToolkit (it can drop a file that only a rare path needs).

The static binaries were 7.3 MiB (`backfill`) and 7.2 MiB (`daily_filings`). `readelf -d` listed
no shared library. In image D, `backfill` stored 28 Notes ZIPs over HTTPS, so the static binary
handles the large-file path.

## Consequences

- An image is about 7 MB to pull, and no glibc version couples the build to the runtime.
- No image has a shell. A check inside an image cannot use `sh`. Container CI reads the image
  config and the exported file system instead.
- A dependency that links OpenSSL or `native-tls`, or that needs a system library at runtime,
  breaks the build. A new dependency must use `rustls` for TLS.
- A binary compiles in the data it needs, such as time zones, and reads no data file of the image.
  `distroless/static` ships `tzdata`, but no binary depends on it.
- musl has a slower allocator than glibc. The full `backfill` run of STA-422 measures memory and
  runtime. If either suffers, the binaries add `mimalloc`.
- A cold build installs zig and `cargo-zigbuild` first, about 1 min 44 s of 4 min 11 s on
  2026-11-01. A builder image with both tools removes that step.

## Open Questions

- A CI check that fails when a dependency adds `openssl-sys` or `native-tls`. Not built yet.
- An `aarch64` target, if Cloud Run moves to Arm machines. Not needed today.

## Sources

- STA-422, comments "Image size notes", "Runtime image comparison", and "Decided (lead,
  2026-11-01)"
- STA-431, Additional Notes (one image per binary)
- Proposal "ETL Deployment Shape after STA-304" (one `Dockerfile`, `--target`s per image):
  https://linear.app/state-machine/document/etl-deployment-shape-after-sta-304-proposal-4ce8a78e92b7
