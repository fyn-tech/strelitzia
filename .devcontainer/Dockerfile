FROM rust:1.91.0

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

# Install additional Rust components
RUN rustup component add clippy rustfmt rust-src

# Note: Modern Cargo (1.62+) has built-in cargo add/remove commands
# No need to install cargo-edit separately

# Set the default user
USER $USERNAME

# Set working directory
WORKDIR /workspace

# Verify installation
RUN rustc --version && cargo --version
