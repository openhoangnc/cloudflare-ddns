FROM rust:1.93-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev
COPY Cargo.toml ./
COPY src ./src
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") TARGET=x86_64-unknown-linux-musl ;; \
    "linux/arm64") TARGET=aarch64-unknown-linux-musl ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac && \
    rustup target add $TARGET && \
    cargo build --release --target $TARGET && \
    mkdir -p /app/output && \
    cp /app/target/$TARGET/release/cloudflare-ddns /app/output/cloudflare-ddns

FROM scratch
COPY --from=builder /app/output/cloudflare-ddns /cloudflare-ddns
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENTRYPOINT ["/cloudflare-ddns"]
