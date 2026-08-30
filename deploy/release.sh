#!/usr/bin/env bash
set -euo pipefail

# The factory container deployer is the only component allowed to provision
# Azure Files. This repository asks it for the work-order data directory and
# then verifies only this product's Container App topology.
slug=${1:-inventory-promise-hold}
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)
deploy_command=${FACTORY_CONTAINER_DEPLOY_SCRIPT:-/opt/fleet/lib/deploy-container.sh}
verify_command=${PERSISTENT_DATA_VERIFY_SCRIPT:-$script_dir/verify-persistent-data.sh}
public_origin=${PUBLIC_ORIGIN:-https://${slug}.sociobot.in}

[[ "$slug" == "inventory-promise-hold" ]] || { echo "This release script is scoped to inventory-promise-hold." >&2; exit 2; }
[[ -x "$deploy_command" ]] || { echo "Factory deployment command is not executable: $deploy_command" >&2; exit 1; }
if [[ "${ALLOW_DIRTY_RELEASE:-0}" != 1 ]] && [[ -n "$(git -C "$repo_root" status --porcelain)" ]]; then
  echo "Refusing to deploy a dirty source tree; commit the release first." >&2
  exit 1
fi

expected_sha=$(git -C "$repo_root" rev-parse HEAD)
WO_DATA_DIR=/data "$deploy_command" "$slug" "$repo_root" Dockerfile 8080
"$verify_command" "$slug"

health=$(curl --fail --silent --show-error --retry 10 --retry-delay 3 "$public_origin/health")
live_sha=$(jq -er '.build_sha' <<<"$health")
if [[ "$live_sha" != "$expected_sha" ]]; then
  echo "Live build identity mismatch: expected $expected_sha, received $live_sha" >&2
  exit 1
fi

echo "Release verified: $public_origin has durable /data, one replica, and build $live_sha."
