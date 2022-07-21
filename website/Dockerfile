ARG RUST_VERSION=1
ARG ALPINE_VERSION

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} as builder

RUN set -xeu;\
    apk update; \
    apk add musl-dev; \
    rm -rf /var/cache/apt/*

WORKDIR /

RUN cargo new --bin website

WORKDIR /website

# Only build dependencies at first to be more cache-friendly
COPY Cargo.toml Cargo.lock .cargo ./
RUN set -xeu; \
    cargo build --release --locked --verbose; \
    rm src/*.rs ./target/release/deps/website*

# Time to build for real
COPY . .
RUN cargo build --release --locked --verbose

FROM alpine:${ALPINE_VERSION:-latest}

COPY --from=builder /website/target/release/website /website

ENTRYPOINT [ "/website" ]
