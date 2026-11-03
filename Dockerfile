# Container images for the binaries `backfill` and `daily_filings`, one image per binary.
#
# Build from the repository root, one target per image:
#   docker build --target backfill -t arkad-backfill .
#   docker build --target daily-filings -t arkad-daily-filings .
#
# The binaries are static musl binaries, linked by zig through `cargo zigbuild`.
# See `docs/adr/0002-static-binaries.md`.

# The build stage compiles both binaries in one call. Both targets share its cache.
# `cargo-zigbuild` 0.24.0 needs a newer Rust than the CI toolchain 1.90.
FROM rust:1.97-trixie AS build
ARG ZIGLANG_VERSION=0.17.0
ARG CARGO_ZIGBUILD_VERSION=0.24.0
# zig comes from the `ziglang` wheel on PyPI. The image has no pip, so the wheel is unpacked as a zip.
RUN wheel_url="$(curl -fsSL "https://pypi.org/pypi/ziglang/${ZIGLANG_VERSION}/json" \
        | python3 -c 'import json, sys; print(next(u["url"] for u in json.load(sys.stdin)["urls"] if "manylinux" in u["filename"] and "x86_64" in u["filename"]))')" \
    && curl -fsSL "$wheel_url" -o /tmp/ziglang.whl \
    && python3 -m zipfile -e /tmp/ziglang.whl /opt/ \
    && rm /tmp/ziglang.whl \
    && chmod +x /opt/ziglang/zig \
    && ln -s /opt/ziglang/zig /usr/local/bin/zig \
    && cargo install --locked "cargo-zigbuild@${CARGO_ZIGBUILD_VERSION}" \
    && rustup target add x86_64-unknown-linux-musl
WORKDIR /arkad
COPY . .
RUN cargo zigbuild --release --locked -p sec --features tracing-logging \
        --bin backfill --bin daily_filings --target x86_64-unknown-linux-musl
# An empty home directory to copy with its owner into the runtime stage.
RUN mkdir -p /out/home/arkad

# The runtime base holds the CA certificates, `/etc/passwd`, and `/tmp`, and has no shell.
FROM gcr.io/distroless/static-debian13 AS runtime
# Without `--chown`, root owns `/home/arkad`, and user 10001 cannot create the storage root `data`.
COPY --from=build --chown=10001:10001 /out/home/arkad /home/arkad
USER 10001:10001
WORKDIR /home/arkad

FROM runtime AS backfill
COPY --from=build /arkad/target/x86_64-unknown-linux-musl/release/backfill /usr/local/bin/backfill
ENTRYPOINT ["/usr/local/bin/backfill"]

FROM runtime AS daily-filings
COPY --from=build /arkad/target/x86_64-unknown-linux-musl/release/daily_filings /usr/local/bin/daily_filings
ENTRYPOINT ["/usr/local/bin/daily_filings"]
