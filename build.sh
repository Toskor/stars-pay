#!/bin/bash

cd tma-client

rm -rf dist
mkdir -p dist

echo "main_bot_mini_app"
TARGET="main_bot_mini_app" bun run build

echo "mini_app"
TARGET="mini_app" bun run build

echo "Build complete!"