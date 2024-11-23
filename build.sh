#!/bin/bash
set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

rustup target add wasm32-unknown-unknown

url="https://github.com/super-turbo-society/turbo-cli/releases/download/0.6.0/turbo-0.6.0-x86_64-unknown-linux-gnu.tar.gz"
curl -L "$url" > turbo.tar.gz
tar -xvf turbo.tar.gz --strip-components=1
rm -rf turbo.tar.gz
chmod +x turbo

./turbo export

