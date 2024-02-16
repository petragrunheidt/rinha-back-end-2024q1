FROM rust:latest

WORKDIR /rinha
COPY . .
RUN cargo build --release

CMD ["target/release/rinha"]