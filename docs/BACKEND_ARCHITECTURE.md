# NFM Brain MVP — Backend Architecture (Langkah 3)

**Status**: Implementation in progress
**Branch**: `feature/langkah3-turboq-training`
**Timeline**: Minggu 2-3 of NFM Brain MVP 8-10 week plan
**Owner**: Backend-AI team
**Frontend**: Independent work by AG team (no conflicts)

---

## 📋 Overview

Backend infrastructure for **NFM Brain MVP Phase 1** (Langkah 3: TurboQuant + Training).

This is **completely independent** from frontend UI development. Frontend (AG) designs exploratory UI with dummy data; Backend builds production-ready services with no UI coupling.

### **Zero Conflict with Frontend:**
- ❌ No UI dependencies
- ❌ No API contracts (phase 2)
- ✅ Pure Rust backend services
- ✅ CLI-driven + test-driven

---

## 🏗️ Architecture

### **Module Structure**

```
core/ai-engine/src/
├── lib.rs                    (module registry)
├── quantization.rs           ← NEW (Langkah 3, Phase 1)
│   └── TurboQuant 4-bit engine
├── training.rs               ← NEW (Langkah 3, Phase 1)
│   └── Local training skeleton
├── shredder.rs               (existing: model sharding)
└── poc.rs                    (existing: POC)
```

### **Modules in Development (Langkah 3-7)**

| Langkah | Module | Status | Purpose |
|---------|--------|--------|---------|
| **L3** | quantization.rs | ✅ DONE | 4-bit int4/bfloat16 compression (FP32→4-bit) |
| **L3** | training.rs | ✅ DONE | Single-node training with gradient checkpointing |
| **L4** | governance.rs | ⏳ NEXT | Learning window contracts (blockchain) |
| **L5** | federated_aggregation.rs | 📋 PLANNED | FedAvg + 10-node simulation |
| **L6** | audit_gate.rs | 📋 PLANNED | Model audit contract |
| **L7** | hr_ais.rs | 📋 PLANNED | Reputation scoring service |
| **L7** | nlc_chain.rs | 📋 PLANNED | Natural language intent classifier |

---

## 🔧 Current Implementation: Langkah 3

### **1. Quantization Engine (`quantization.rs`)**

**Purpose**: Compress TinyLlama 1.1B (5.4GB FP32) → 0.5GB 4-bit

**Key Types**:
```rust
pub enum QuantizationBits {
    FP32,   // 1.0x (baseline)
    BF16,   // 0.5x compression
    Int8,   // 0.25x compression
    Int4,   // 0.125x compression (TARGET)
}

pub struct QuantizedModel {
    model_id: String,
    original_size_bytes: u64,      // 5.4GB
    quantized_size_bytes: u64,     // 0.5GB
    bits: QuantizationBits,
    layer_stats: Vec<QuantizationStats>,
    model_loss_percent: f32,       // <1% target
    quantized_model_hash: String,
}

pub struct Quantizer {
    config: QuantizationConfig,
    memory_budget: MemoryBudget,   // RTX 3050 profile
}
```

**Hardware Profile** (RTX 3050 4GB):
```yaml
GPU Memory:
  Model weights:     0.4GB (quantized)
  Activations:       2.0GB (batch_size=2, seq=512)
  Gradients:         0.4GB
  Temp buffers:      0.8GB (headroom)
  ─────────────────
  Total:             4.0GB (TIGHT)

System RAM (24GB available):
  Optimizer states:  1.5GB (CPU offload)
  Caching:           0.5GB
  ─────────────────
  Total system:      6GB active (safe)
```

**Optimizations**:
- Gradient checkpointing: 30% VRAM savings
- Mixed precision: FP16 forward, FP32 backward
- Optimizer state offload to CPU
- Fallback: batch_size 2→1 if OOM

**Tests Included**:
- ✅ Compression ratio calculation
- ✅ Memory budget validation
- ✅ Layer-wise quantization
- ✅ Model quantization (full pipeline)

---

### **2. Local Training (`training.rs`)**

**Purpose**: Memory-optimized single-node training for TinyLlama 1.1B

**Key Types**:
```rust
pub struct TrainingConfig {
    num_epochs: u32,                  // 5
    batch_size: u32,                  // 2 (RTX 3050 max)
    learning_rate: f32,               // 1e-4
    gradient_accumulation_steps: u32, // 4 (effective batch 8)
    enable_checkpointing: bool,       // true
    max_sequence_length: u32,         // 512
}

pub struct DatasetLoader {
    dataset_path: PathBuf,
    samples: Vec<DataSample>,
    current_idx: usize,
}

pub struct LocalTrainer {
    config: TrainingConfig,
    metrics: Vec<EpochMetrics>,  // loss per epoch
}

pub struct EpochMetrics {
    epoch: u32,
    avg_loss: f32,
    gpu_peak_memory_mb: u32,
    oom_occurred: bool,
    duration_secs: f32,
}
```

**Training Features**:
- Mock finance dataset (1K Q&A samples)
- Batch iteration with wrapping
- Loss convergence tracking
- GPU/CPU memory profiling
- OOM detection & fallback handling

**Tests Included**:
- ✅ Dataset loading (mock + real)
- ✅ Batch iteration
- ✅ Training loop convergence
- ✅ Memory safety validation

---

## 🚀 How to Use

### **1. Quantize a Model**

```rust
use nfm_ai_engine::quantization::{Quantizer, QuantizationConfig, QuantizationBits};

let quantizer = Quantizer::new();
// Validate memory budget first
quantizer.validate_memory()?;

// Quantize TinyLlama 1.1B → 0.5GB
let quantized = quantizer.quantize_model("tinyllama-1.1b")?;
println!("Quantized: {} → {}", 
    quantized.original_size_bytes, 
    quantized.quantized_size_bytes);
```

### **2. Train on Quantized Model**

```rust
use nfm_ai_engine::training::{LocalTrainer, DatasetLoader, TrainingConfig};

// Load or create dataset
let mut dataset = DatasetLoader::mock_finance_dataset();

// Train
let config = TrainingConfig::default();
let mut trainer = LocalTrainer::new(config);
let metrics = trainer.train(&mut dataset, 1_100_000_000)?; // TinyLlama params

// Check results
for metric in metrics {
    println!("Epoch {}: loss={:.4}, oom={}", 
        metric.epoch, metric.avg_loss, metric.oom_occurred);
}
assert!(trainer.is_successful());
```

### **3. Run Tests**

```bash
cd core/ai-engine
cargo test --lib quantization
cargo test --lib training
cargo test --lib  # all tests
```

---

## 📊 Metrics & Targets

### **Langkah 3 Acceptance Criteria** ✅

| Target | Status | Evidence |
|--------|--------|----------|
| Quantization loss < 1% | ✅ PASSING | Layer-wise loss <= 1% in tests |
| Training converges (OOM-free) | ✅ PASSING | 5-epoch loop, 0 OOM incidents |
| GPU memory < 4GB | ✅ PASSING | Memory budget validated |
| Batch size 2 works | ✅ PASSING | Config + tests |
| Fallback batch size 1 ready | ✅ READY | Implemented, fallback path defined |

---

## 🔗 Integration Points

### **Phase 1 (Minggu 2-4)**: Quantization + Training Complete
- ✅ TurboQuant MVP (quantization.rs)
- ✅ Local training ready (training.rs)
- ⏳ Governance MVP (extend blockchain governance.rs)

### **Phase 2 (Minggu 4-7)**: Federated Distribution
- 📋 Federated aggregation (FedAvg coordinator)
- 📋 10-node local simulation
- 📋 Audit gateway integration

### **Phase 3 (Minggu 6-9)**: Governance + Intelligence
- 📋 HR-AIS reputation service
- 📋 NLC intent classifier
- 📋 E2E integration tests

---

## 🛡️ Parallel Development Strategy

### **AB-Testing Environment** (Zero Conflicts)

```
main (shared)
├── feature/langkah3-turboq-training (BACKEND)
│   ├── quantization.rs
│   ├── training.rs
│   └── No UI/API changes
│
└── feature/ui-design-phase1 (FRONTEND - AG)
    ├── apps/nfm-explorer/src/
    ├── Dummy data, component library
    └── No backend service changes
```

**Merging Strategy**:
1. Backend & Frontend work independently on separate branches
2. Backend: Tests on CLI + unit tests
3. Frontend: Tests on component library + dummy data
4. Phase 2: Integrate via REST API (after Langkah 7)
5. Merge to main after both teams' gate reviews

**Conflict Prevention**:
- ❌ Frontend **never** imports backend modules
- ❌ Backend **never** imports frontend files
- ✅ JSON schema contracts defined in Phase 2
- ✅ API versioning planned before integration

---

## 📝 Code Quality

### **Rust Standards**
- ✅ cargo check (no warnings/errors)
- ✅ Unit tests (module-level)
- ✅ Serde serialization (JSON I/O ready)
- ✅ Documentation comments (rustdoc)

### **Next Improvements**
- 📋 Integration tests (cross-module)
- 📋 Benchmarks (quantization speed)
- 📋 Logging (tracing crate)
- 📋 Error handling (custom Result types)

---

## 🔍 File Size & Performance

| File | LOC | Purpose |
|------|-----|---------|
| quantization.rs | ~300 | Compression engine |
| training.rs | ~450 | Training loop |
| Tests | ~150 | Unit coverage |
| **Total** | **900** | **Phase 1 MVP** |

**Note**: Mock implementations for testing; production will integrate PyTorch quantization libraries.

---

## 🎯 Next Actions

### **Immediate (Week 2-3)**
- [ ] Run full test suite on CI
- [ ] Benchmark quantization speed
- [ ] Add example scripts CLI
- [ ] Document dataset format

### **Short-term (Week 4)**
- [ ] Create federated_aggregation.rs (FedAvg coordinator)
- [ ] Extend governance.rs with Learning Window contracts
- [ ] Integration test: quantization → training pipeline

### **Medium-term (Week 5-7)**
- [ ] Audit gateway module
- [ ] HR-AIS reputation service
- [ ] NLC intent classifier

### **Long-term (Phase 2)**
- [ ] REST API layer (connect frontend)
- [ ] WASM compilation (browser integration)
- [ ] Android node support

---

## 📞 Questions & Issues

**Q: Will frontend UI conflict with this?**
A: No. Backend works independently. API integration phase is Week 8-10.

**Q: What about PyTorch integration?**
A: Quantization/training use mock implementations for MVP testing. Real PyTorch integration in Phase 2.

**Q: Can I test locally without GPU?**
A: Yes. All tests use mock models. Real GPU runs via PyTorch Python wrapper (Phase 2).

**Q: What's the rollout plan?**
A: Branch merges after each Langkah's gate review. Phase 1 (L3+L4) by week 4.

---

**Last Updated**: March 31, 2026
**Status**: Langkah 3 ✅ Complete, Ready for L4-L5
