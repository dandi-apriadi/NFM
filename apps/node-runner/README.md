# NFM Node Runner

Scripts and configurations to easily run an NFM Blockchain Node.

## Quick Start

### Option 1: Docker (Recommended)
```bash
# Build image
docker build -t nfm-node ../core/blockchain/

# Run node
docker run -d \
  --name nfm-node \
  -p 3000:3000 \
  -p 9000:9000 \
  -v nfm-data:/home/nfm/nfm_main.db \
  nfm-node

# Check status
curl http://localhost:3000/api/status
```

### Option 2: Native (Rust)
```bash
cd ../../core/blockchain
cargo run --release
```

### Option 3: Automated Scripts
```bash
# Linux/Mac
./run.sh

# Linux/Mac (force restart if already running)
./run.sh --restart

# Linux/Mac (health check only)
./run.sh --health

# Linux/Mac (open dashboard after successful health check)
./run.sh --health --open

# Linux/Mac (machine-readable health JSON)
./run.sh --health --json

# Linux/Mac (print JSON schema example)
./run.sh --schema

# Notes:
# JSON mode includes UTC timestamp.
# Port conflict errors include per-port PID details.
# Health JSON includes mode and latency_ms.
# Health JSON also includes api_port, p2p_port, and pid (native mode).
# Health JSON also includes blocks, chain_height, peers, status, and version.
# JSON key order is stable across launcher outputs.

# Linux/Mac (quiet mode)
./run.sh --health --quiet

# Windows
.\run.ps1

# Windows (force restart if already running)
.\run.ps1 --restart

# Windows (health check only)
.\run.ps1 --health

# Windows (open dashboard after successful health check)
.\run.ps1 --health --open

# Windows (machine-readable health JSON)
.\run.ps1 --health --json

# Windows (print JSON schema example)
.\run.ps1 --schema

# Notes:
# JSON mode includes UTC timestamp.
# Port conflict errors include per-port PID details.
# Health JSON includes mode and latency_ms.
# Health JSON also includes api_port, p2p_port, and pid (native mode).
# Health JSON also includes blocks, chain_height, peers, status, and version.
# JSON key order is stable across launcher outputs.

# Windows (quiet mode)
.\run.ps1 --health --quiet
```

## Dashboard
Once the node is running, open **http://localhost:3000** in your browser.

## Ports
| Port | Function |
|------|--------|
| 3000 | REST API & Dashboard |
| 9000 | P2P Network |

## Data Persistence
Blockchain data is stored in `nfm_main.db/` (Sled DB). If using Docker, ensure you use a volume mount so data is not lost when the container is deleted.

