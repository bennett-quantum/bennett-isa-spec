#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
cp scripts/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
echo 'installed pre-push hook'
