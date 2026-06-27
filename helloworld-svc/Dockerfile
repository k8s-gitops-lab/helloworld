FROM rust:1.88-slim AS builder

WORKDIR /src

COPY certs /usr/local/share/ca-certificates/zscaler

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml .
COPY Cargo.lock .
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY certs /usr/local/share/ca-certificates/zscaler
COPY --from=builder /src/target/release/helloworld-svc /usr/local/bin/helloworld-svc

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

CMD ["helloworld-svc"]
