#!/bin/sh
sudo apt update
sudo apt install -y pkg-config iputils-ping protobuf-compiler openssl pkg-config
cargo install sqlx-cli@^0.7 cargo-watch
sudo chown -R vscode target
npm install
(cd components/server; sqlx migrate run; cargo build)
