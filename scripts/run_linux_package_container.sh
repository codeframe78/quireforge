#!/usr/bin/env bash
set -euo pipefail

repository_root="$(git rev-parse --show-toplevel)"
if [[ "$(pwd -P)" != "$(cd "$repository_root" && pwd -P)" ]]; then
  echo "Run this script from the QuireForge repository root." >&2
  exit 1
fi

builder_image="quireforge-packaging:ubuntu-22.04"
builder_source="ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982"
cache_root="$repository_root/.cache/packaging"
source_epoch="$(git log -1 --format=%ct)"

mkdir -p \
  "$cache_root/cargo" \
  "$cache_root/home" \
  "$cache_root/node_modules/desktop" \
  "$cache_root/node_modules/root" \
  "$cache_root/node_modules/website" \
  "$cache_root/pnpm-store"

docker build \
  --file packaging/linux/Dockerfile \
  --tag "$builder_image" \
  packaging/linux

docker run \
  --rm \
  --init \
  --user "$(id -u):$(id -g)" \
  --volume "$repository_root:/workspace" \
  --volume "$cache_root:/cache" \
  --volume "$cache_root/node_modules/root:/workspace/node_modules" \
  --volume "$cache_root/node_modules/desktop:/workspace/apps/desktop/node_modules" \
  --volume "$cache_root/node_modules/website:/workspace/apps/website/node_modules" \
  --env CARGO_BUILD_JOBS=4 \
  --env CARGO_HOME=/cache/cargo \
  --env CARGO_TARGET_DIR=/workspace/target/ubuntu-22.04 \
  --env CI=true \
  --env HOME=/cache/home \
  --env QUIRE_FORGE_BUILD_DISTRIBUTION=ubuntu \
  --env QUIRE_FORGE_BUILD_IMAGE="$builder_source" \
  --env QUIRE_FORGE_BUILD_VERSION=22.04 \
  --env QUIRE_FORGE_TAURI_CACHE_DIR=/cache/home/.cache/tauri \
  --env SOURCE_DATE_EPOCH="$source_epoch" \
  --workdir /workspace \
  "$builder_image" \
  /bin/bash -c \
  "pnpm install --frozen-lockfile --store-dir /cache/pnpm-store \
    && pnpm package:linux \
    && python3 scripts/validate_release_artifacts.py \
      --artifact-dir target/ubuntu-22.04/release/packages \
      --lifecycle \
      --smoke"
