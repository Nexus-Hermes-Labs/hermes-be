#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

status=0
traefik_routes="../infra/traefik/dynamic/routes.yml"

read_toml_value() {
  local file="$1"
  local key="$2"

  awk -F '=' -v key="$key" '
    $1 ~ "^[[:space:]]*" key "[[:space:]]*$" {
      value = $2
      gsub(/[[:space:]]/, "", value)
      gsub(/"/, "", value)
      print value
      exit
    }
  ' "$file"
}

require_contains() {
  local file="$1"
  local pattern="$2"
  local label="$3"

  if ! grep -Fq "$pattern" "$file"; then
    printf 'config drift: %s missing %s in %s\n' "$label" "$pattern" "$file" >&2
    status=1
  fi
}

for toml in config/services/*-service.toml; do
  service="$(basename "$toml" .toml)"
  compose="services/${service}/docker-compose.yml"

  http_port="$(read_toml_value "$toml" port)"
  grpc_port="$(read_toml_value "$toml" grpc_port)"

  if [[ -z "$http_port" ]]; then
    printf 'config drift: %s has no service.port\n' "$toml" >&2
    status=1
    continue
  fi

  if [[ ! -f "$compose" ]]; then
    printf 'config drift: missing compose file %s\n' "$compose" >&2
    status=1
    continue
  fi

  require_contains "$compose" "\"${http_port}:${http_port}\"" "${service} HTTP port"

  if [[ -n "$grpc_port" ]]; then
    require_contains "$compose" "\"${grpc_port}:${grpc_port}\"" "${service} gRPC port"
  fi

  if [[ -f "$traefik_routes" ]]; then
    require_contains "$traefik_routes" "http://hermes-${service}:${http_port}" "${service} Traefik HTTP route"
  fi
done

exit "$status"
