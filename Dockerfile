# Dev (just for fast compilation)
# Build Stage
FROM rust AS builder

WORKDIR /usr/src/app

ENV CARGO_HOME=/usr/local/cargo
ENV CARGO_TARGET_DIR=/usr/src/app/target

COPY Cargo.lock .
COPY Cargo.toml .
COPY . .

# Install necessary system dependencies
RUN apt update &&\
    rm -rf ~/.cache &&\
    apt clean all &&\
    apt install -y cmake &&\
    apt install -y clang &&\
    apt install -y pkgconf &&\
    apt-get install -y pkg-config libssl-dev libpq-dev wget unzip

# Download the file using wget
RUN wget https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.1.0%2Bcpu.zip -O libtorch.zip
# Extract the contents of the ZIP file to /usr/libtorch
RUN unzip -o libtorch.zip
# Clean up unnecessary files
RUN rm libtorch.zip

# Set the environment variables
ENV LIBTORCH=/usr/src/app/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH

# Build the Rust application
RUN cargo build

# Runtime Stage
FROM fedora:34 AS runner

# Set the environment variables in runner
ENV LIBTORCH=/usr/src/app/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH

# Postgres
RUN dnf install -y libpq

# Expose the port
EXPOSE 8080
# Copy the binary from the builder stage to the runner stage
COPY --from=builder /usr/src/app/libtorch /usr/src/app/libtorch
# Copy the binary from the builder stage to the runner stage
COPY --from=builder /usr/src/app/target/debug/app /bin/app
ENTRYPOINT ["/bin/app"]
