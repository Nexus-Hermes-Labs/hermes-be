#!/usr/bin/env python3
"""
Migrate per-service `.env.example` (and the workspace-root `.env`/`.env.example`)
into Consul KV under `config/{service_name}/data` and `config/application/data`.

Run from the workspace root that contains `services/`:

    cd hermes-be
    CONSUL_URL=http://127.0.0.1:8500 python3 scripts/config-migration/migrate_to_consul.py

Idempotent: re-running overwrites existing keys with the latest .env content.
Uses only the standard library — no `requests` install needed.
"""

import json
import os
import sys
import urllib.error
import urllib.request

CONSUL_URL = os.environ.get("CONSUL_URL", "http://127.0.0.1:8500")
HTTP_TIMEOUT = 5  # seconds


def parse_env_file(file_path):
    """Parse a `.env`-style file into a nested dict suitable for Consul.

    Only `APP_*` keys are extracted; the `APP_` prefix is stripped, the
    remainder is lowercased and split on `__` to produce nested objects.
    Values that look like ints/floats/bools are coerced.
    """
    config = {}
    if not os.path.exists(file_path):
        return config

    with open(file_path, "r") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" not in line:
                continue

            key, value = line.split("=", 1)
            value = value.strip().strip('"').strip("'")

            if not key.startswith("APP_"):
                continue

            parts = key[4:].lower().split("__")
            current = config
            for i, part in enumerate(parts):
                if i == len(parts) - 1:
                    current[part] = coerce_scalar(value)
                else:
                    if part not in current or not isinstance(current[part], dict):
                        current[part] = {}
                    current = current[part]
    return config


def coerce_scalar(value):
    if value.lower() == "true":
        return True
    if value.lower() == "false":
        return False
    try:
        if "." in value:
            return float(value)
        return int(value)
    except ValueError:
        return value


def upload_to_consul(key, data):
    """PUT a JSON-encoded value to Consul KV at `config/{key}/data`."""
    url = f"{CONSUL_URL}/v1/kv/config/{key}/data"
    body = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        url, data=body, method="PUT",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=HTTP_TIMEOUT) as resp:
            if resp.status == 200:
                print(f"[migrate] OK    {key}  ({len(data)} top-level keys)")
                return True
            print(f"[migrate] FAIL  {key}  status={resp.status}")
            return False
    except urllib.error.URLError as e:
        print(f"[migrate] FAIL  {key}  ({e.reason})")
        return False


def consul_reachable():
    """Probe the Consul leader endpoint; returns True if Consul responds."""
    try:
        with urllib.request.urlopen(
            f"{CONSUL_URL}/v1/status/leader", timeout=HTTP_TIMEOUT
        ) as resp:
            return resp.status == 200
    except urllib.error.URLError:
        return False


def migrate():
    if not consul_reachable():
        print(f"[migrate] FATAL Consul not reachable at {CONSUL_URL}")
        print("[migrate]       Start it with: docker compose -f hermes-be/infra/docker-compose.yml up -d consul")
        sys.exit(1)

    # 1. Workspace-root global config (shared defaults)
    #
    # `.env.example` is the committed baseline that should land in Consul.
    # `.env` is the developer's local overrides (sqlx-cli URLs, host-specific
    # tweaks) and is git-ignored — we read it on top of .env.example so a
    # developer can still customise without committing.
    global_config = parse_env_file(".env.example")
    overrides = parse_env_file(".env")
    deep_merge(global_config, overrides)

    if global_config:
        print(f"[migrate] Reading global config: .env.example (+ .env overrides if present)")
        upload_to_consul("application", global_config)
    else:
        print(f"[migrate] SKIP  application  (no APP_* keys in .env.example or .env)")

    # 2. Per-service configs
    services_dir = "services"
    if not os.path.isdir(services_dir):
        print(f"[migrate] FATAL `{services_dir}/` not found — run from the workspace root")
        sys.exit(1)

    skipped = []
    for service_name in sorted(os.listdir(services_dir)):
        service_path = os.path.join(services_dir, service_name)
        if not os.path.isdir(service_path):
            continue

        baseline = os.path.join(service_path, ".env.example")
        local = os.path.join(service_path, ".env")

        service_config = parse_env_file(baseline)
        deep_merge(service_config, parse_env_file(local))

        if not service_config:
            skipped.append(service_name)
            continue

        upload_to_consul(service_name, service_config)

    if skipped:
        print(f"[migrate] SKIPPED no APP_* keys in .env / .env.example: {', '.join(skipped)}")


def deep_merge(target, overrides):
    """Recursively merge `overrides` into `target` in place."""
    for key, value in overrides.items():
        if isinstance(value, dict) and isinstance(target.get(key), dict):
            deep_merge(target[key], value)
        else:
            target[key] = value


if __name__ == "__main__":
    try:
        migrate()
    except KeyboardInterrupt:
        sys.exit(130)
