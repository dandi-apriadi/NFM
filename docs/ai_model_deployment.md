# NFM AI Model Deployment (Uplink Process)

Mekanisme desentralisasi untuk memasukkan kecerdasan baru ke dalam jaringan NFM.

## 1. Tahap Persiapan (Developer Side)
- **Model Compression**: Developer menggunakan **NFM Terminal** untuk mengompresi model (misal dari format GGUF/Safetensors) ke format internal NFM.
- **Metadata Tagging**: Menentukan spesifikasi hardware minimum (VRAM/NPU) yang dibutuhkan node untuk menjalankan model ini.

## 2. Proses Sharding (The Shredder)
- **DSMO Fragmentation**: Model dipecah menjadi ribuan "Neural Shards". 
- **ZK-Proof Embedding**: Setiap shard diberikan tanda air digital (Watermark) dan ZK-Proof untuk memastikan integritas data saat disebar.

## 3. Distribusi (P2P Mesh)
- **Genesis Uplink**: Shards pertama kali diunggah ke cluster node **Genesis** (Staker tertinggi).
- **Profound Propagation**: Node Genesis kemudian menyebarkan replika shards ke node **Pro** di seluruh dunia secara asinkron.
- **Redundancy**: Setiap shard disimpan di minimal 10-20 node berbeda untuk menjamin ketersediaan 100%.

## 4. Verifikasi & Aktivasi
- **Consensus Check**: Jaringan melakukan pengecekan apakah model tersebut aman (tidak berisi malware atau kode berbahaya).
- **Marketplace Listing**: Setelah verifikasi sukses, model muncul sebagai "Skill" baru di **NFM Marketplace** dan siap dipanggil oleh user melalui Super-App.

## 5. Ekonomi Uplink
- **Deployment Fee**: Developer membayar sejumlah NFM Credit untuk biaya distribusi data ke ribuan node.
- **Royalty System**: Setiap kali model tersebut digunakan oleh user, developer mendapatkan bagi hasil (Royalty) yang otomatis masuk ke NFM-ID mereka.
