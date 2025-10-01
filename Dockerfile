# Builder
ARG RUST_VERSION=1.90.0
ARG BIN_NAME=RustLettreSMTP

FROM rust:${RUST_VERSION}-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
      pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

ARG BIN_NAME
RUN cargo build --release --bin ${BIN_NAME}

# Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates tzdata openssl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 10001 appuser
WORKDIR /app

ARG BIN_NAME
COPY --from=builder /app/target/release/${BIN_NAME} /usr/local/bin/app

RUN touch /app/email_sent.log && chmod +w /app/email_sent.log

ENV RUST_LOG=info
EXPOSE 8080
USER 10001

ENTRYPOINT ["/usr/local/bin/app"]
