#!/usr/bin/env bash
set -euo pipefail

slug=${1:-inventory-promise-hold}
repo_root=${2:-.}
dockerfile=${3:-Dockerfile}
registry=${AZURE_CONTAINER_REGISTRY:-sociobotregistry}
app_name="sf-${slug}"

[[ "$slug" =~ ^[a-z0-9]([a-z0-9-]{0,40}[a-z0-9])?$ ]] || {
  echo "Invalid product slug: $slug" >&2
  exit 2
}

if ((${#app_name} > 32)); then
  suffix=$(printf '%s' "$slug" | sha1sum | cut -c1-6)
  app_name="sf-${slug:0:22}-${suffix}"
  app_name=${app_name//--/-}
fi

source_sha=$(git -C "$repo_root" rev-parse HEAD)
tag="${app_name}:${source_sha:0:12}"

echo "Building $registry.azurecr.io/$tag" >&2
az acr build \
  --registry "$registry" \
  --image "$tag" \
  --file "$dockerfile" \
  --build-arg "BUILD_SHA=$source_sha" \
  --build-arg "GIT_SHA=$source_sha" \
  --build-arg "SOURCE_COMMIT=$source_sha" \
  "$repo_root" >&2

printf '%s\n' "$registry.azurecr.io/$tag"
