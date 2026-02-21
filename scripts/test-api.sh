#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────
# Schemathesis API test runner for Hermes
# Usage:
#   ./scripts/test-api.sh              # all services
#   ./scripts/test-api.sh auth         # auth-service only
#   ./scripts/test-api.sh user         # user-service only
#   ./scripts/test-api.sh guild        # guild-service only
# ─────────────────────────────────────────────

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Seed credentials (from services/auth-service/seeds/dev/001_seed_auth_credentials.sql)
TEST_EMAIL="${TEST_EMAIL:-admin@example.com}"
TEST_PASSWORD="${TEST_PASSWORD:-Password123!}"

MAX_EXAMPLES="${MAX_EXAMPLES:-50}"
TARGET="${1:-all}"

# ─────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────

log()  { echo -e "${BLUE}[test-api]${NC} $*" >&2; }
ok()   { echo -e "${GREEN}[test-api]${NC} $*" >&2; }
err()  { echo -e "${RED}[test-api]${NC} $*" >&2; }

check_service() {
    local name=$1
    local url=$2
    # Always check on localhost — host.docker.internal is only for the container
    local check_url="${url/host.docker.internal/localhost}"
    if ! curl -sf "$check_url/health" &>/dev/null; then
        err "Service '$name' is not reachable at $check_url"
        err "Start it with: make run-${name}"
        exit 1
    fi
}

# ─────────────────────────────────────────────
# Detect runner: local st → docker-compose → raw Docker
# Sets AUTH_URL/USER_URL/GUILD_URL and REPORT_DIR accordingly
# ─────────────────────────────────────────────

if command -v st &>/dev/null; then
    log "Using local schemathesis (st)"
    run_st() { st "$@"; }
    AUTH_URL="${AUTH_URL:-http://localhost:8081}"
    USER_URL="${USER_URL:-http://localhost:8082}"
    GUILD_URL="${GUILD_URL:-http://localhost:8086}"
    REPORT_DIR="reports"
elif command -v schemathesis &>/dev/null; then
    log "Using local schemathesis"
    run_st() { schemathesis "$@"; }
    AUTH_URL="${AUTH_URL:-http://localhost:8081}"
    USER_URL="${USER_URL:-http://localhost:8082}"
    GUILD_URL="${GUILD_URL:-http://localhost:8086}"
    REPORT_DIR="reports"
elif command -v docker &>/dev/null && docker compose version &>/dev/null 2>&1; then
    log "Using schemathesis via docker compose (testing profile)"
    run_st() {
        docker compose --profile testing run --rm schemathesis "$@"
    }
    # Container reaches host services via host.docker.internal
    AUTH_URL="${AUTH_URL:-http://host.docker.internal:8081}"
    USER_URL="${USER_URL:-http://host.docker.internal:8082}"
    GUILD_URL="${GUILD_URL:-http://host.docker.internal:8086}"
    REPORT_DIR="/reports"
elif command -v docker &>/dev/null; then
    log "Using schemathesis via Docker (raw)"
    run_st() {
        docker run --rm --network host \
            -v "$(pwd)/reports:/reports" \
            schemathesis/schemathesis:stable "$@"
    }
    AUTH_URL="${AUTH_URL:-http://localhost:8081}"
    USER_URL="${USER_URL:-http://localhost:8082}"
    GUILD_URL="${GUILD_URL:-http://localhost:8086}"
    REPORT_DIR="/reports"
else
    err "schemathesis not found. Install with:"
    echo "  pip install schemathesis"
    echo "  or start Docker and re-run (image is pulled automatically)"
    exit 1
fi

command -v curl &>/dev/null || { err "curl is required"; exit 1; }
command -v jq  &>/dev/null  || { err "jq is required (apt/brew install jq)"; exit 1; }

mkdir -p reports

# ─────────────────────────────────────────────
# Acquire JWT token
# ─────────────────────────────────────────────

get_token() {
    log "Logging in as $TEST_EMAIL to get access token..."
    local curl_auth_url="${AUTH_URL/host.docker.internal/localhost}"
    local response
    response=$(curl -sf -X POST "$curl_auth_url/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASSWORD\"}") || {
        err "Login failed — is auth-service running at $curl_auth_url?"
        exit 1
    }

    local token
    token=$(echo "$response" | jq -r '.access_token // .token // empty')

    if [[ -z "$token" || "$token" == "null" ]]; then
        err "Could not extract access_token from login response:"
        echo "$response" | jq . 2>/dev/null || echo "$response"
        exit 1
    fi

    ok "Token acquired"
    echo "$token"
}

# ─────────────────────────────────────────────
# Schemathesis runners
# ─────────────────────────────────────────────

run_auth() {
    check_service "auth" "$AUTH_URL"
    log "Running Schemathesis against Auth Service ($AUTH_URL)..."

    run_st run "$AUTH_URL/api-docs/openapi.json" \
        --checks all \
        --exclude-checks not_a_server_error \
        --max-examples "$MAX_EXAMPLES" \
        --report junit \
        --report-junit-path "$REPORT_DIR/auth-report.xml"

    ok "Auth Service tests done → reports/auth-report.xml"
}

run_user() {
    check_service "auth" "$AUTH_URL"
    check_service "user" "$USER_URL"
    local token
    token=$(get_token)

    log "Running Schemathesis against User Service ($USER_URL)..."

    run_st run "$USER_URL/api-docs/openapi.json" \
        --checks all \
        --exclude-checks not_a_server_error \
        --max-examples "$MAX_EXAMPLES" \
        -H "Authorization: Bearer $token" \
        --report junit \
        --report-junit-path "$REPORT_DIR/user-report.xml"

    ok "User Service tests done → reports/user-report.xml"
}

run_guild() {
    check_service "auth" "$AUTH_URL"
    check_service "guild" "$GUILD_URL"
    local token
    token=$(get_token)

    log "Running Schemathesis against Guild Service ($GUILD_URL)..."

    run_st run "$GUILD_URL/api-docs/openapi.json" \
        --checks all \
        --exclude-checks not_a_server_error \
        --max-examples "$MAX_EXAMPLES" \
        -H "Authorization: Bearer $token" \
        --report junit \
        --report-junit-path "$REPORT_DIR/guild-report.xml"

    ok "Guild Service tests done → reports/guild-report.xml"
}

# ─────────────────────────────────────────────
# Entry point
# ─────────────────────────────────────────────

case "$TARGET" in
    auth)  run_auth  ;;
    user)  run_user  ;;
    guild) run_guild ;;
    all)
        run_auth
        run_user
        run_guild
        ok "All API tests complete. Reports saved to ./reports/"
        ;;
    *)
        err "Unknown target: $TARGET"
        echo "Usage: $0 [all|auth|user|guild]"
        exit 1
        ;;
esac
