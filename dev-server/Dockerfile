# ---- Build ----
FROM rust:1.86-slim AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN cargo build --release

# ---- Runtime ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/dev-server /usr/local/bin/dev-server

ENV PORT=8080
EXPOSE 8080

CMD ["dev-server"]
