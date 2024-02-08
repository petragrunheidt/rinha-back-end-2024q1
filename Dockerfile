FROM rust:latest AS base

WORKDIR /rinha
COPY . .
RUN cargo build --release
CMD ["target/release/rinha"]