# Dev (just for fast compilation)
# Build Stage
FROM rust:slim-buster AS builder

WORKDIR /usr/src/app

ENV CARGO_HOME=/usr/local/cargo
ENV CARGO_TARGET_DIR=/usr/src/app/target

COPY Cargo.lock .
COPY Cargo.toml .
COPY . .

# Install necessary system dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev

# Download the file using wget
RUN wget https://download.pytorch.org/libtorch/cpu/libtorch-shared-with-deps-2.1.1%2Bcpu.zip

# Extract the contents of the ZIP file to /usr/libtorch
RUN unzip libtorch-shared-with-deps-2.1.1+cpu.zip -d /usr/

# Clean up unnecessary files
RUN rm libtorch-shared-with-deps-2.1.1+cpu.zip

ENV LIBTORCH=/usr/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH

# Build the Rust application
RUN cargo build

# Runtime Stage
FROM fedora:34 AS runner

RUN dnf install -y libpq

EXPOSE 8080
COPY --from=builder /usr/src/app/target/debug/app /bin/app
ENTRYPOINT ["/bin/app"]
