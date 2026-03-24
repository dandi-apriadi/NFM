//! DSMO (Distributed Secure Model Orchestration) — Neural Sharding Engine
//!
//! Memecah model AI besar menjadi "Neural Shards" yang dapat didistribusikan
//! ke node-node dalam jaringan NFM secara aman.
//!
//! Sesuai dengan: docs/ai_model_deployment.md

use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// Ukuran shard default (dalam bytes) — 1 MB
const DEFAULT_SHARD_SIZE: usize = 1_048_576;

/// Minimum jumlah replika per shard untuk redundansi
const MIN_REPLICAS: usize = 10;

/// Metadata shard yang disimpan di blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMetadata {
    /// ID unik shard (hash dari konten)
    pub shard_id: String,
    /// Index shard dalam model asli (untuk reassembly)
    pub index: u32,
    /// Ukuran shard dalam bytes
    pub size: usize,
    /// Hash integritas konten shard
    pub content_hash: String,
    /// ZK-Proof watermark (simulasi)
    pub zk_watermark: String,
    /// ID model induk
    pub parent_model_id: String,
}

/// Metadata model AI yang didaftarkan ke jaringan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    /// ID unik model
    pub model_id: String,
    /// Nama model
    pub name: String,
    /// Developer / uploader address
    pub developer_address: String,
    /// Total shard yang terbentuk
    pub total_shards: u32,
    /// Ukuran asli model (bytes)
    pub original_size: usize,
    /// Minimum hardware requirement
    pub min_vram_mb: u32,
    /// Manifest hash (untuk verifikasi integritas)
    pub manifest_hash: String,
    /// Biaya deployment (NVC)
    pub deployment_fee: f64,
}

/// Hasil proses sharding
#[derive(Debug, Clone)]
pub struct ShredResult {
    pub manifest: ModelManifest,
    pub shards: Vec<ShardMetadata>,
}

/// DSMO Shredder Engine
pub struct Shredder {
    pub shard_size: usize,
    pub min_replicas: usize,
}

impl Shredder {
    pub fn new() -> Self {
        Self {
            shard_size: DEFAULT_SHARD_SIZE,
            min_replicas: MIN_REPLICAS,
        }
    }

    /// Konfigurasi ukuran shard kustom
    pub fn with_shard_size(mut self, size: usize) -> Self {
        self.shard_size = size;
        self
    }

    /// Simulasi proses sharding model besar
    ///
    /// Di production: membaca file model dari disk dan memecahnya.
    /// Di alpha: menerima raw bytes dan menghasilkan metadata.
    pub fn shred_model(
        &self,
        model_data: &[u8],
        model_name: &str,
        developer_address: &str,
        min_vram_mb: u32,
    ) -> ShredResult {
        let total_shards = (model_data.len() + self.shard_size - 1) / self.shard_size;

        // Generate model ID
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}:{}", model_name, developer_address, model_data.len()).as_bytes());
        let model_id = format!("model_{}", &hex::encode(hasher.finalize())[..24]);

        let mut shards = Vec::new();

        for i in 0..total_shards {
            let start = i * self.shard_size;
            let end = std::cmp::min(start + self.shard_size, model_data.len());
            let chunk = &model_data[start..end];

            // Hash konten shard
            let mut content_hasher = Sha256::new();
            content_hasher.update(chunk);
            let content_hash = hex::encode(content_hasher.finalize());

            // ZK-Proof watermark (simulasi: hash of model_id + index)
            let mut zk_hasher = Sha256::new();
            zk_hasher.update(format!("zk:{}:{}", model_id, i).as_bytes());
            let zk_watermark = format!("zkp_{}", &hex::encode(zk_hasher.finalize())[..16]);

            // Shard ID
            let mut shard_hasher = Sha256::new();
            shard_hasher.update(format!("{}:{}:{}", model_id, i, content_hash).as_bytes());
            let shard_id = format!("shard_{}", &hex::encode(shard_hasher.finalize())[..20]);

            shards.push(ShardMetadata {
                shard_id,
                index: i as u32,
                size: chunk.len(),
                content_hash,
                zk_watermark,
                parent_model_id: model_id.clone(),
            });
        }

        // Manifest hash
        let mut manifest_hasher = Sha256::new();
        for shard in &shards {
            manifest_hasher.update(shard.content_hash.as_bytes());
        }
        let manifest_hash = hex::encode(manifest_hasher.finalize());

        // Biaya deployment: proporsional terhadap ukuran model
        let deployment_fee = (model_data.len() as f64 / 1_000_000.0) * 5.0; // 5 NVC per MB

        let manifest = ModelManifest {
            model_id,
            name: model_name.to_string(),
            developer_address: developer_address.to_string(),
            total_shards: total_shards as u32,
            original_size: model_data.len(),
            min_vram_mb,
            manifest_hash,
            deployment_fee,
        };

        ShredResult { manifest, shards }
    }

    /// Verifikasi integritas shard berdasarkan metadata
    pub fn verify_shard(shard_data: &[u8], metadata: &ShardMetadata) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(shard_data);
        let computed_hash = hex::encode(hasher.finalize());
        computed_hash == metadata.content_hash
    }

    /// Verifikasi ZK watermark (simulasi)
    pub fn verify_watermark(metadata: &ShardMetadata) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(format!("zk:{}:{}", metadata.parent_model_id, metadata.index).as_bytes());
        let expected = format!("zkp_{}", &hex::encode(hasher.finalize())[..16]);
        expected == metadata.zk_watermark
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shred_small_model() {
        let shredder = Shredder::new().with_shard_size(100); // 100 bytes per shard
        let model_data = vec![42u8; 350]; // 350 bytes = 4 shards

        let result = shredder.shred_model(&model_data, "TestModel", "nfm_dev_001", 2048);

        assert_eq!(result.manifest.total_shards, 4);
        assert_eq!(result.shards.len(), 4);
        assert_eq!(result.shards[0].size, 100);
        assert_eq!(result.shards[3].size, 50); // Sisa
        assert!(result.manifest.model_id.starts_with("model_"));
    }

    #[test]
    fn test_shard_integrity_verification() {
        let shredder = Shredder::new().with_shard_size(100);
        let model_data = vec![99u8; 200];

        let result = shredder.shred_model(&model_data, "IntegrityTest", "nfm_dev", 1024);

        // Verify first shard
        let first_shard_data = &model_data[0..100];
        assert!(Shredder::verify_shard(first_shard_data, &result.shards[0]));

        // Tampered data should fail
        let tampered = vec![0u8; 100];
        assert!(!Shredder::verify_shard(&tampered, &result.shards[0]));
    }

    #[test]
    fn test_zk_watermark_verification() {
        let shredder = Shredder::new().with_shard_size(100);
        let model_data = vec![1u8; 150];

        let result = shredder.shred_model(&model_data, "WatermarkTest", "nfm_dev", 512);

        for shard in &result.shards {
            assert!(Shredder::verify_watermark(shard));
            assert!(shard.zk_watermark.starts_with("zkp_"));
        }
    }

    #[test]
    fn test_deployment_fee_calculation() {
        let shredder = Shredder::new();
        let model_data = vec![0u8; 2_000_000]; // 2 MB

        let result = shredder.shred_model(&model_data, "FeeTest", "nfm_dev", 4096);

        // 2 MB * 5 NVC/MB = 10 NVC
        assert!((result.manifest.deployment_fee - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_deterministic_sharding() {
        let shredder = Shredder::new().with_shard_size(50);
        let data = b"Hello NFM Neural Fragment Mesh Sovereign AI";

        let r1 = shredder.shred_model(data, "DetTest", "nfm_dev", 256);
        let r2 = shredder.shred_model(data, "DetTest", "nfm_dev", 256);

        assert_eq!(r1.manifest.model_id, r2.manifest.model_id);
        assert_eq!(r1.manifest.manifest_hash, r2.manifest.manifest_hash);
        for (s1, s2) in r1.shards.iter().zip(r2.shards.iter()) {
            assert_eq!(s1.content_hash, s2.content_hash);
        }
    }
}
