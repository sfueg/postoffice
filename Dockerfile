FROM rust:1.85 AS builder
WORKDIR /usr/src/app
COPY Cargo* ./
COPY src ./src/
RUN cargo build --release

FROM rust:1.85.0-slim-bookworm
WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/postoffice /usr/src/app/postoffice
CMD ["./postoffice", "--file", "/config.json"]
