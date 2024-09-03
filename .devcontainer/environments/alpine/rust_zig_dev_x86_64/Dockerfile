# arg for rust version
ARG RUST_VERSION=1.80.1
ARG ALPINE_VERSION=3.19

FROM rust:$RUST_VERSION-alpine$ALPINE_VERSION

# args for zig installation
ARG ARCH="x86_64"
ARG OS="linux"
ARG ZIG_VERSION="0.12.0"
ARG ZIG_DOWNLOAD_URL="https://ziglang.org/download/${ZIG_VERSION}/zig-${OS}-${ARCH}-${ZIG_VERSION}.tar.xz"
ARG INSTALL_DIR="/usr/local/zig/"


RUN apk update && \
    apk --no-cache add \
      musl-dev \
      git

RUN mkdir -p /tmp/build && \
  cd /tmp/build  && \
  wget "$ZIG_DOWNLOAD_URL" && \
  tar -xvf "zig-${OS}-${ARCH}-${ZIG_VERSION}.tar.xz" && \
  mv "zig-${OS}-${ARCH}-${ZIG_VERSION}" "$INSTALL_DIR" && \
  ln -sf "$INSTALL_DIR/zig" /usr/local/bin/zig && \
  echo " Zig version $ZIG_VERSION has been installed to $INSTALL_DIR" && \
  rm -rf /tmp/build

