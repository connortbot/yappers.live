FROM rust:1.87 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./

COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/wukong /usr/local/bin/wukong

EXPOSE 8080

ENTRYPOINT ["wukong"]
