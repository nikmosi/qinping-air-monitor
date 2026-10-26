FROM rust:1.99.1 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

RUN echo "app:x:10001:10001:app:/nonexistent:/bin/false" > /etc/passwd_app

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl --bin qinping-air-monitor

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /etc/passwd_app /etc/passwd
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/qinping-air-monitor /qinping-air-monitor

USER 10001:10001
EXPOSE 8000

ENTRYPOINT ["/qinping-air-monitor"]
