#!/usr/bin/env bash
set -euo pipefail

slug=${1:-inventory-promise-hold}
resource_group=${AZURE_RESOURCE_GROUP:-sociobot}
subscription=${AZURE_SUBSCRIPTION_ID:?AZURE_SUBSCRIPTION_ID is required}
app_name="sf-${slug}"
storage_name="sf-${slug}-data"
volume_name="data"

[[ "$slug" =~ ^[a-z0-9]([a-z0-9-]{0,40}[a-z0-9])?$ ]] || {
  echo "Invalid product slug: $slug" >&2
  exit 2
}

if ((${#app_name} > 32)); then
  suffix=$(printf '%s' "$slug" | sha1sum | cut -c1-6)
  app_name="sf-${slug:0:22}-${suffix}"
  app_name=${app_name//--/-}
fi

app=$(az containerapp show \
  --subscription "$subscription" \
  --resource-group "$resource_group" \
  --name "$app_name" \
  --output json)

if ! jq -e \
  --arg storage "$storage_name" \
  --arg volume "$volume_name" '
    .properties.provisioningState == "Succeeded" and
    (.properties.latestRevisionName | length > 0) and
    .properties.latestRevisionName == .properties.latestReadyRevisionName and
    .properties.template.scale.minReplicas == 1 and
    .properties.template.scale.maxReplicas == 1 and
    ([.properties.template.volumes[]? |
      select(.name == $volume and .storageType == "AzureFile" and .storageName == $storage)] | length) == 1 and
    ([.properties.template.containers[]? |
      select(.name == "app") |
      .volumeMounts[]? |
      select(.volumeName == $volume and .mountPath == "/data")] | length) == 1
  ' <<<"$app" >/dev/null; then
  jq '{
    provisioning: .properties.provisioningState,
    latest: .properties.latestRevisionName,
    ready: .properties.latestReadyRevisionName,
    minReplicas: .properties.template.scale.minReplicas,
    maxReplicas: .properties.template.scale.maxReplicas,
    volumes: .properties.template.volumes,
    mounts: [.properties.template.containers[]? | select(.name == "app") | .volumeMounts]
  }' <<<"$app" >&2
  echo "Unsafe deployment: Stock Promise requires one ready replica and its Azure Files volume mounted at /data." >&2
  exit 1
fi

jq '{
  revision: .properties.latestReadyRevisionName,
  minReplicas: .properties.template.scale.minReplicas,
  maxReplicas: .properties.template.scale.maxReplicas,
  volumes: .properties.template.volumes,
  mounts: [.properties.template.containers[]? | select(.name == "app") | .volumeMounts]
}' <<<"$app"
