#!/bin/sh
sudo apt update
sudo apt install -y pkg-config iputils-ping
cargo install sqlx-cli cargo-watch
sudo chown -R vscode /home/workspace/frontend/node_modules
sudo chown -R vscode fake-internet/node_modules
sudo chown -R vscode server/target
npm install
(cd server; cargo build)
