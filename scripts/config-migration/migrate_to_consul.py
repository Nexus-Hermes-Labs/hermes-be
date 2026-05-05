import os
import requests
import json

CONSUL_URL = os.environ.get("CONSUL_URL", "http://127.0.0.1:8500")

def parse_env_file(file_path):
    config = {}
    if not os.path.exists(file_path):
        return config
    
    with open(file_path, 'r') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            if '=' not in line:
                continue
            
            key, value = line.split('=', 1)
            # Remove quotes if present
            value = value.strip().strip('"').strip("'")
            
            # We only care about APP_ prefixed variables for the structured config
            if key.startswith('APP_'):
                parts = key[4:].lower().split('__') # Skip 'APP_' and split by '__'
                current = config
                for i, part in enumerate(parts):
                    if i == len(parts) - 1:
                        # Convert to int/bool if possible
                        if value.lower() == 'true': value = True
                        elif value.lower() == 'false': value = False
                        else:
                            try:
                                if '.' in value: value = float(value)
                                else: value = int(value)
                            except ValueError:
                                pass
                        current[part] = value
                    else:
                        if part not in current:
                            current[part] = {}
                        current = current[part]
    return config

def upload_to_consul(key, data):
    url = f"{CONSUL_URL}/v1/kv/config/{key}/data"
    response = requests.put(url, json=data)
    if response.status_code == 200:
        print(f"✅ Successfully uploaded {key} configuration to Consul.")
    else:
        print(f"❌ Failed to upload {key}. Status code: {response.status_code}")

def migrate():
    # 1. Migrate Global Config (prioritize .env over .env.example)
    global_env = ".env" if os.path.exists(".env") else ".env.example"
    print(f"Processing global config: {global_env}")
    global_config = parse_env_file(global_env)
    if global_config:
        upload_to_consul("application", global_config)

    # 2. Migrate Service Specific Configs
    services_dir = "services"
    for service_name in os.listdir(services_dir):
        service_path = os.path.join(services_dir, service_name)
        if os.path.isdir(service_path):
            # Prioritize service-specific .env if it exists
            env_path = os.path.join(service_path, ".env")
            if not os.path.exists(env_path):
                env_path = os.path.join(service_path, ".env.example")
            
            if os.path.exists(env_path):
                print(f"Processing service config: {service_name} ({os.path.basename(env_path)})")
                service_config = parse_env_file(env_path)
                if service_config:
                    upload_to_consul(service_name, service_config)

if __name__ == "__main__":
    try:
        migrate()
    except Exception as e:
        print(f"An error occurred: {e}")
        print("\nTip: Make sure Consul is running (infra/consul) and 'requests' library is installed (pip install requests).")
