FROM mcr.microsoft.com/devcontainers/base:trixie

ARG USERNAME=vscode
ARG RUST_TOOLCHAIN=nightly

ENV DEBIAN_FRONTEND=noninteractive \
    PATH=/home/${USERNAME}/.cargo/bin:${PATH}

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        lld \
        llvm \
        make \
        pkg-config \
        python3 \
        rustup \
        tmux \
    && rm -rf /var/lib/apt/lists/*

USER ${USERNAME}

RUN rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal --component clippy --component rustfmt \
    && rustup default "${RUST_TOOLCHAIN}" \
    && cargo install --locked cargo-make cargo-audit cargo-afl afl_runner

WORKDIR /workspaces/lexord
