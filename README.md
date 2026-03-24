# 🧠 NFM (Neural Fragment Mesh)

![NFM Premium Logo](./nfm_logo_premium_1774358027664.png)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Made%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Phase-Active%20Development-green.svg)]()
[![Network](https://img.shields.io/badge/Node-v0.1.0--alpha-blueviolet.svg)]()

**Neural Fragment Mesh (NFM)** is a Sovereign AI-Blockchain Ecosystem designed as a decentralized foundation for collective intelligence. Unlike traditional blockchains, NFM is built from the ground up to integrate AI models directly into its consensus and reward mechanisms.

---

## 🏛️ Core Vision

NFM aims to solve the "Isolated AI" problem by creating a shared, immutable mesh where AI fragments can be trained, verified, and rewarded through a sovereign Layer 1 blockchain. This mesh is designed for:
- **Collective Learning**: Shared "Native Brain" curricula for decentralized intelligence.
- **Sovereign Performance**: High-efficiency Rust-based blockchain core.
- **AI-Native Governance**: Protocol rules driven by both biological and digital consensus.

## 🛠️ Technology Stack

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Blockchain Core** | [Rust](https://www.rust-lang.org/) | Memory-safe, high-performance L1 node. |
| **P2P Layer** | Gossip Protocol | Robust propagation & dynamic discovery. |
| **Database** | [Sled DB](https://github.com/spacejam/sled) | Embedded KV store for consensus state. |
| **UI Shell** | [Vite](https://vitejs.dev/) + [React](https://react.dev/) | High-speed, modern block explorer. |
| **AI Runtime** | Custom WASM/Native | Integrated model execution & sharding. |

## 🏗️ Architecture Overview

```mermaid
graph TD
    A[NFM Native Brain] --> B[Neural Fragment Mesh]
    B --> C[Blockchain Core (L1)]
    C --> D[P2P Network]
    C --> E[In-Mesh Storage]
    B --> F[AI Engine]
    F --> G[Curriculum Modules]
    G --> H[Collective Wisdom]
```

## 🚀 Getting Started

### Prerequisites
- **Rust**: [Installation Guide](https://rustup.rs/) (v1.75+)
- **Node.js**: [Download](https://nodejs.org/) (v18+)

### Launching the Node
Clone the repository and run the node using the automated script:
```bash
git clone https://github.com/dandi-apriadi/NFM.git
cd NFM/apps/node-runner
.\run.ps1
```

### Opening the Explorer
```bash
cd ../nfm-explorer
npm install
npm run dev
```

## 📜 Documentation Guide

| File | Description |
| :--- | :--- |
| [blueprint.txt](./blueprint.txt) | High-level engineering standards. |
| [docs/sovereign_chain_design.md](./docs/sovereign_chain_design.md) | L1 Architecture Specs. |
| [docs/security_audit.md](./docs/security_audit.md) | PQC & Bio-ZKP security. |
| [docs/implementation_roadmap.md](./docs/implementation_roadmap.md) | Phase-by-phase roadmap. |

## 🤝 Contributing

We welcome scientists, developers, and AI enthusiasts! Please read our [CONTRIBUTING.md](./CONTRIBUTING.md) to get started.

## ⚖️ License

Project NFM is released under the **MIT License**. See [LICENSE](./LICENSE) for details.

---
*Maintained by [Dandi Apriadi](https://github.com/dandi-apriadi) & the NFM Foundation.*
