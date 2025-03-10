# FROM mcr.microsoft.com/devcontainers/rust:latest
# Step 1: build solana
FROM rust:1.76.0 AS solana-builder
RUN apt-get update && apt-get install -y \
    clang \
    libudev-dev \
    protobuf-compiler
# RUN sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)" 
# && export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH" && cargo install --git https://github.com/coral-xyz/anchor avm --force && avm install latest && avm use latest && npm install --global yarn
RUN git config --global http.postBuffer 524288000 && \
    git clone https://github.com/solana-labs/solana.git && \
    cd solana && \
    cargo build --release && \
    cp target/release/solana /usr/local/bin/ && \
    cp target/release/solana-genesis /usr/local/bin/ && \
    cp target/release/solana-keygen /usr/local/bin/ && \
    cp target/release/solana-test-validator /usr/local/bin/ && \
    cp target/release/solana-validator /usr/local/bin/

# Step 2: build anchor
FROM rust:latest AS anchor-builder
RUN cargo install --git https://github.com/coral-xyz/anchor --tag v0.30.1 anchor-cli
# RUN cargo install --git https://github.com/coral-xyz/anchor avm --force \
#    && avm install latest \
#    && avm use latest

# Step 3: app image
FROM rust:latest
COPY --from=solana-builder /usr/local/bin/solana /usr/local/bin/solana
COPY --from=solana-builder /usr/local/bin/solana-genesis /usr/local/bin/solana-genesis
COPY --from=solana-builder /usr/local/bin/solana-keygen /usr/local/bin/solana-keygen
COPY --from=solana-builder /usr/local/bin/solana-test-validator /usr/local/bin/solana-test-validator
COPY --from=solana-builder /usr/local/bin/solana-validator /usr/local/bin/solana-validator
COPY --from=anchor-builder /usr/local/cargo/bin/anchor /usr/local/bin/anchor
# install node and yarn
RUN curl -fsSL https://deb.nodesource.com/setup_23.x -o nodesource_setup.sh &&  \
    bash nodesource_setup.sh && \
    apt-get install -y nodejs && \
    npm install --global yarn

