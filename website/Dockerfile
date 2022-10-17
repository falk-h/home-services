ARG RUST_VERSION=1
ARG ALPINE_VERSION=""

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} as builder

RUN set -xeu;\
    apk update; \
    apk add musl-dev; \
    rm -rf /var/cache/apt/*

WORKDIR /

RUN cargo new --bin website

WORKDIR /website

ARG RUST_PROFILE=dev

# Incremental builds aren't helpful here.
ENV CARGO_INCREMENTAL=0

# Only copy what we need to build our dependencies at first, and build
# dependencies separately from the binary so that they don't have to be
# recompiled if only our code changes.
COPY Cargo.toml Cargo.lock .cargo ./
RUN cargo fetch -v
RUN set -xeu; \
    cargo build \
        --profile "${RUST_PROFILE}" \
        --no-default-features \
        --offline \
        --locked \
        --verbose; \
    rm src/*.rs ./target/*/deps/website*

# Time to build for real. Use `cargo install` instead of `cargo build` here so
# that the binary is always placed in the same location, regardless of profile.
COPY src src
RUN cargo install \
    --path . \
    --profile "${RUST_PROFILE}" \
    --no-default-features \
    --offline \
    --locked \
    --verbose

FROM alpine:${ALPINE_VERSION:-latest}

ENV DIR=/app/ \
    USER=appuser

RUN set -xeu; \
    addgroup -S "$USER"; \
    adduser -S -G "$USER" "$USER"

COPY --from=builder /usr/local/cargo/bin/website $DIR

RUN chown -R "$USER:$USER" "$DIR"

USER $USER
WORKDIR $DIR
EXPOSE 80

ENV RUST_BACKTRACE=1

VOLUME /markdown
VOLUME /static
VOLUME /templates

ENTRYPOINT [ "./website" ]
CMD [ \
    "--markdown-dir", "/markdown", \
    "--static-dir", "/static", \
    "--template-dir", "/templates" \
]
