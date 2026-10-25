# Runtime image for the `stream_extract` binary.
#
# Build from the repository root: `docker build -t arkad-extract .`

# The build stage matches the runtime Debian release, so the binary links against the same glibc.
FROM rust:1.90-bookworm AS build
WORKDIR /arkad
COPY . .
RUN cargo build --release --locked -p sec --features tracing-logging --bin stream_extract

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /arkad/target/release/stream_extract /usr/local/bin/stream_extract
ENTRYPOINT ["/usr/local/bin/stream_extract"]
