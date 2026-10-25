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
RUN useradd --system --uid 10001 --user-group --create-home --home-dir /home/arkad arkad
COPY --from=build /arkad/target/release/stream_extract /usr/local/bin/stream_extract
# The default storage root `data` sits in the home directory, which the user owns.
USER 10001:10001
WORKDIR /home/arkad
ENTRYPOINT ["/usr/local/bin/stream_extract"]
