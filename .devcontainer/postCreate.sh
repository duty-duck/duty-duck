#!/bin/sh
sudo apt update
sudo apt install -y pkg-config iputils-ping protobuf-compiler openssl pkg-config

# install pnpm
wget -qO- https://get.pnpm.io/install.sh | ENV="$HOME/.bashrc" SHELL="$(which bash)" bash -

# install sqlx and cargo watch
cargo install sqlx-cli@^0.7 cargo-watch

sudo chown -R vscode target
pnpm install
(cd components/server; sqlx migrate run; cargo build)
