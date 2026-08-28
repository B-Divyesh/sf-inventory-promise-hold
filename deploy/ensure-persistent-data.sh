#!/usr/bin/env bash
set -euo pipefail

# Apply the image and durable SQLite topology in the same Container Apps revision.
slug=${1:-inventory-promise-hold}
release_image=${2:-}
resource_group=${AZURE_RESOURCE_GROUP:-sociobot}
environment_name=${AZURE_CONTAINER_ENV:-factory-env}
storage_account=${AZURE_STORAGE_ACCOUNT:-sociobotblob}
subscription=${AZURE_SUBSCRIPTION_ID:?AZURE_SUBSCRIPTION_ID is required}
app_name="sf-${slug}"
share_name="sf-${slug}"
storage_name="data-${slug}"
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
readiness_attempts=${PERSISTENT_READY_ATTEMPTS:-120}

[[ "$slug" =~ ^[a-z0-9]([a-z0-9-]{0,40}[a-z0-9])?$ ]] || {
  echo "Invalid product slug: $slug" >&2
  exit 2
}

if ((${#app_name} > 32)); then
  suffix=$(printf '%s' "$slug" | sha1sum | cut -c1-6)
  app_name="sf-${slug:0:22}-${suffix}"
  app_name=${app_name//--/-}
fi

storage_key=$(az storage account keys list \
  --subscription "$subscription" \
  --resource-group "$resource_group" \
  --account-name "$storage_account" \
  --query '[0].value' -o tsv)

az storage share create \
  --account-name "$storage_account" \
  --account-key "$storage_key" \
  --name "$share_name" \
  --quota 5 \
  --output none

environment_id="/subscriptions/$subscription/resourceGroups/$resource_group/providers/Microsoft.App/managedEnvironments/$environment_name"
az rest --method put \
  --url "https://management.azure.com${environment_id}/storages/${storage_name}?api-version=2024-03-01" \
  --body "$(jq -n \
    --arg location eastus2 \
    --arg account "$storage_account" \
    --arg key "$storage_key" \
    --arg share "$share_name" \
    '{location:$location,properties:{azureFile:{accountName:$account,accountKey:$key,shareName:$share,accessMode:"ReadWrite"}}}')" \
  --output none

template=$(az containerapp show \
  --subscription "$subscription" \
  --resource-group "$resource_group" \
  --name "$app_name" \
  --query properties.template -o json)
template=$(jq \
  --arg storage "$storage_name" \
  --arg release_image "$release_image" \
  '{containers: [.containers[] | select(.name == "app") |
      {name, image: (if $release_image == "" then .image else $release_image end),
       resources: {cpu: .resources.cpu, memory: .resources.memory}, env,
       volumeMounts: [{volumeName:"stock-promise-data",mountPath:"/data"}]}],
    scale: {minReplicas: 1, maxReplicas: 1},
    volumes: [{name:"stock-promise-data",storageType:"AzureFile",storageName:$storage}]}' \
  <<<"$template")

az rest --method patch \
  --url "https://management.azure.com/subscriptions/${subscription}/resourceGroups/${resource_group}/providers/Microsoft.App/containerApps/${app_name}?api-version=2024-03-01" \
  --body "$(jq -n --argjson template "$template" '{properties:{template:$template}}')" \
  --output none

ready=false
for _ in $(seq 1 "$readiness_attempts"); do
  if "$script_dir/verify-persistent-data.sh" "$slug" >/dev/null 2>&1; then
    ready=true
    break
  fi
  sleep 5
done

if [[ "$ready" != true ]]; then
  echo "Timed out waiting for the durable revision to become ready after $((readiness_attempts * 5)) seconds." >&2
  exit 1
fi

"$script_dir/verify-persistent-data.sh" "$slug"
