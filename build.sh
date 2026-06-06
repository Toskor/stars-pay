#!/bin/bash
# Build all five Mini App / overlay targets into tma-client/dist/.
# Run this on the `frontend` branch, then copy the resulting HTML to
# `server/static/` on `main` (see README).
set -euo pipefail

cd "$(dirname "$0")/tma-client"

rm -rf dist
mkdir -p dist

for target in main_bot_mini_app mini_app layer blocked_app goal_app; do
    echo "→ $target"
    TARGET="$target" bun run build
done

echo "Build complete. Output: tma-client/dist/src/pages/*.html"
