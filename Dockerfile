FROM rust:1-slim AS builder
ARG TARGETARCH=amd64
RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN case "$TARGETARCH" in \
      amd64) echo x86_64-unknown-linux-musl > /musl-target ;; \
      arm64) echo aarch64-unknown-linux-musl > /musl-target ;; \
      *) echo "unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac
RUN rustup target add "$(cat /musl-target)"
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --target "$(cat /musl-target)" \
    && cp "target/$(cat /musl-target)/release/ccprt" /app/ccprt

FROM scratch
COPY --from=builder /app/ccprt /ccprt
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/ccprt"]
