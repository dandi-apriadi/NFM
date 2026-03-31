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

# Windows
.\run.ps1

# Windows (force restart if already running)
.\run.ps1 --restart
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

