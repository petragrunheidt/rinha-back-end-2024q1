FROM rust:latest

# Set the working directory inside the container
WORKDIR /rinha

# Copy the current directory (containing your Rust source code) into the container
COPY . .

# Build your Rust application
RUN cargo build --release

# Set the command to run your application when the container starts
CMD ["target/release/rinha"]