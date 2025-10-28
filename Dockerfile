# Use a multi-stage build to keep the final image small
# First stage: Build the application for aarch64
FROM --platform=linux/aarch64 ubuntu:22.04 AS builder

# Install required build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc-aarch64-linux-gnu \
    pkg-config \
    curl \
    git \
    cmake \
    libdrm-dev:arm64 \
    && rm -rf /var/lib/apt/lists/*

# Install Rust for aarch64 target
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set up the environment for cross-compilation
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

# Create working directory
WORKDIR /usr/src/regmsg

# Copy the project files
COPY . .

# Add the aarch64 target and build the project
RUN rustup target add aarch64-unknown-linux-gnu
RUN cargo build --target aarch64-unknown-linux-gnu --release

# Second stage: Create a minimal runtime image
FROM --platform=linux/aarch64 ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libdrm2:arm64 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root/

# Copy the built binaries from the builder stage
COPY --from=builder /usr/src/regmsg/target/aarch64-unknown-linux-gnu/release/regmsg /usr/local/bin/regmsg
COPY --from=builder /usr/src/regmsg/target/aarch64-unknown-linux-gnu/release/regmsgd /usr/local/bin/regmsgd

# Copy the init script if needed
COPY --from=builder /usr/src/regmsg/init/S06regmsgd /etc/init.d/S06regmsgd

# Set up default environment
ENV RUST_BACKTRACE=1

# Default command is to show help
CMD ["/usr/local/bin/regmsg", "--help"]