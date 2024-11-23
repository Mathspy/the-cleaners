#!/bin/bash
set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

rustup target add wasm32-unknown-unknown

sudo curl -sSfL https://turbo.computer/install.sh | sh

turbo export

