#!/usr/bin/env bash
# =============================================================================
# NFM Node Runner - Quick Launch Script (Linux/Mac)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CORE_DIR="$SCRIPT_DIR/../../core/blockchain"

RESTART_REQUESTED=0
HEALTH_ONLY=0
OPEN_DASHBOARD=0
DOCKER_MODE=0
JSON_MODE=0
QUIET_MODE=0
SCHEMA_MODE=0
for arg in "$@"; do
    case "$arg" in
        --restart)
            RESTART_REQUESTED=1
            ;;
        --health)
            HEALTH_ONLY=1
            ;;
        --open)
            OPEN_DASHBOARD=1
            ;;
        --docker)
            DOCKER_MODE=1
            ;;
        --json)
            JSON_MODE=1
            ;;
        --quiet)
            QUIET_MODE=1
            ;;
        --schema)
            SCHEMA_MODE=1
            ;;
    esac
done

log() {
    if [ "$QUIET_MODE" -eq 0 ] && [ "$JSON_MODE" -eq 0 ]; then
        echo "$1"
    fi
}

probe_api_health() {
    local mode="${1:-health}"
    local node_pid="${2:-null}"
    local now
    now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

    extract_number() {
        local key="$1"
        printf '%s' "$2" | sed -n "s/.*\"${key}\":[[:space:]]*\([0-9][0-9]*\).*/\1/p" | head -n 1
    }

    extract_string() {
        local key="$1"
        printf '%s' "$2" | sed -n "s/.*\"${key}\":[[:space:]]*\"\([^\"]*\)\".*/\1/p" | head -n 1
    }

    if command -v curl >/dev/null 2>&1; then
        local body
        local time_total
        if body="$(curl -sSf --max-time 2 "http://127.0.0.1:3000/api/status")" && time_total="$(curl -sSf --max-time 2 -o /dev/null -w "%{time_total}" "http://127.0.0.1:3000/api/status")"; then
            local latency_ms
            local blocks
            local peers
            local status
            local version
            latency_ms="$(awk -v t="$time_total" 'BEGIN { printf "%d", (t * 1000) }')"
            blocks="$(extract_number "blocks" "$body")"
            peers="$(extract_number "peers" "$body")"
            status="$(extract_string "status" "$body")"
            version="$(extract_string "version" "$body")"

            [ -z "$blocks" ] && blocks="null"
            [ -z "$peers" ] && peers="null"
            [ -z "$status" ] && status="unknown"
            [ -z "$version" ] && version="unknown"
            if [ "$JSON_MODE" -eq 1 ]; then
                echo "{\"ok\":true,\"mode\":\"${mode}\",\"endpoint\":\"http://127.0.0.1:3000/api/status\",\"api_port\":3000,\"p2p_port\":9000,\"timestamp\":\"${now}\",\"latency_ms\":${latency_ms},\"blocks\":${blocks},\"chain_height\":${blocks},\"peers\":${peers},\"status\":\"${status}\",\"version\":\"${version}\",\"pid\":${node_pid},\"error\":null}"
            else
                log "[HEALTH] API reachable at /api/status (${latency_ms}ms)"
            fi
            if [ "$OPEN_DASHBOARD" -eq 1 ]; then
                if command -v xdg-open >/dev/null 2>&1; then
                    xdg-open "http://127.0.0.1:3000" >/dev/null 2>&1 || true
                    log "[OPEN] Dashboard opened via xdg-open"
                elif command -v open >/dev/null 2>&1; then
                    open "http://127.0.0.1:3000" >/dev/null 2>&1 || true
                    log "[OPEN] Dashboard opened via open"
                fi
            fi
            return 0
        fi
    fi

    if [ "$JSON_MODE" -eq 1 ]; then
        echo "{\"ok\":false,\"mode\":\"${mode}\",\"endpoint\":\"http://127.0.0.1:3000/api/status\",\"api_port\":3000,\"p2p_port\":9000,\"timestamp\":\"${now}\",\"latency_ms\":null,\"blocks\":null,\"chain_height\":null,\"peers\":null,\"status\":null,\"version\":null,\"pid\":${node_pid},\"error\":\"API probe failed\"}"
    else
        log "[HEALTH] API probe failed at /api/status (node may be down or warming up)."
    fi
    return 1
}

find_listener_pid() {
    local port="$1"

    if command -v lsof >/dev/null 2>&1; then
        lsof -tiTCP:"${port}" -sTCP:LISTEN 2>/dev/null | head -n 1
        return
    fi

    if command -v ss >/dev/null 2>&1; then
        ss -ltnp 2>/dev/null \
            | awk -v p=":${port}" '$4 ~ p"$" {print $NF}' \
            | sed -n 's/.*pid=\([0-9][0-9]*\).*/\1/p' \
            | head -n 1
        return
    fi

    if command -v netstat >/dev/null 2>&1; then
        netstat -anv 2>/dev/null \
            | awk -v p=".${port}" '$0 ~ p" .*LISTEN" {print $9}' \
            | head -n 1
        return
    fi
}

log "=========================================="
log "  NFM Node Runner - Quick Launch"
log "=========================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    if [ "$JSON_MODE" -eq 1 ]; then
        echo '{"ok":false,"error":"Rust/Cargo not found"}'
    else
        echo "[ERROR] Rust/Cargo not found. Install from https://rustup.rs"
    fi
    exit 1
fi

if [ "$SCHEMA_MODE" -eq 1 ]; then
    echo '{"ok":true,"mode":"health","endpoint":"http://127.0.0.1:3000/api/status","api_port":3000,"p2p_port":9000,"timestamp":"2026-01-01T00:00:00Z","latency_ms":5,"blocks":1,"chain_height":1,"peers":0,"status":"running","version":"1.0.0-mesh","pid":null,"error":null}'
    exit 0
fi

# Check if Docker is preferred
if [ "$DOCKER_MODE" -eq 1 ]; then
    log "[MODE] Docker"
    cd "$CORE_DIR"
    docker build -t nfm-node .
    docker run -it --rm \
        --name nfm-node \
        -p 3000:3000 \
        -p 9000:9000 \
        -v nfm-data:/home/nfm/nfm_main.db \
        nfm-node
    exit 0
fi

if [ "$HEALTH_ONLY" -eq 1 ]; then
    log "[MODE] Health Check"
    probe_api_health "health" "null"
    exit $?
fi

log "[MODE] Native Rust"
cd "$CORE_DIR"

EXISTING_PID="$(pgrep -f nfm-core-blockchain | head -n 1 || true)"
if [ -n "$EXISTING_PID" ]; then
    if [ "$RESTART_REQUESTED" -eq 1 ]; then
        log "[INFO] Existing nfm-core-blockchain process found (PID ${EXISTING_PID}). Restart requested."
        kill "$EXISTING_PID"
        sleep 1
    else
        log "[INFO] NFM Node is already running (PID ${EXISTING_PID})."
        log "       API: http://127.0.0.1:3000"
        log "       P2P: 127.0.0.1:9000"
        probe_api_health "native" "${EXISTING_PID}" || true
        log "       Use './run.sh --restart' to restart the node."
        exit 0
    fi
fi

PORT3000_PID="$(find_listener_pid 3000 || true)"
PORT9000_PID="$(find_listener_pid 9000 || true)"
if [ -n "$PORT3000_PID" ] || [ -n "$PORT9000_PID" ]; then
    if [ "$JSON_MODE" -eq 1 ]; then
        now="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
        json_conflicts=""
        if [ -n "$PORT3000_PID" ]; then
            json_conflicts="{\"port\":3000,\"pid\":${PORT3000_PID}}"
        fi
        if [ -n "$PORT9000_PID" ]; then
            if [ -n "$json_conflicts" ]; then
                json_conflicts="${json_conflicts},"
            fi
            json_conflicts="${json_conflicts}{\"port\":9000,\"pid\":${PORT9000_PID}}"
        fi
        echo "{\"ok\":false,\"mode\":\"native\",\"timestamp\":\"${now}\",\"error\":\"Required ports are already in use\",\"conflicts\":[${json_conflicts}]}"
    else
        echo "[ERROR] Required ports are already in use (3000/9000)."
        if [ -n "$PORT3000_PID" ]; then
            echo "       Port 3000 is used by PID: ${PORT3000_PID}"
        fi
        if [ -n "$PORT9000_PID" ]; then
            echo "       Port 9000 is used by PID: ${PORT9000_PID}"
        fi
        echo "[HINT] Stop the process above or run './run.sh --restart' if it is nfm-core-blockchain."
    fi
    exit 1
fi

log "[BUILD] Compiling NFM Core..."
cargo build --release 2>&1

log "[RUN] Starting NFM Node..."
./target/release/nfm-core-blockchain
