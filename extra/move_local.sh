#!/usr/bin/env bash

set -e

BIN_NAME="comp1"

TARGET_DIR="$HOME/.local/bin"

echo "Building release $BIN_NAME... binary"
cargo build --release

echo "Installing $BIN_NAME to $TARGET_DIR..."
mkdir -p "$TARGET_DIR"

cp "target/release/$BIN_NAME" "$TARGET_DIR/$BIN_NAME"

chmod +x "$TARGET_DIR/$BIN_NAME"

echo "Done! $BIN_NAME installed to $TARGET_DIR"
