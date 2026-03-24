# NFM Security Audit Report v0.4
**Auditor**: Antigravity AI (Automated Static Analysis)  
**Target**: `core/blockchain/src/` (18 modules, ~110KB)  
**Tanggal**: 2026-03-19  
**Status Blueprint**: v7.9.2 (Final Audit Approved)

---

## Ringkasan Eksekutif

| Kategori | Jumlah |
|----------|--------|
| 🔴 Kritis | 2 |
| 🟠 Sedang | 4 |
| 🟡 Rendah | 3 |
| ✅ Poin Positif | 5 |

---

## ✅ Poin Positif (Sudah Benar)

1. **Ed25519 Digital Signatures** (`wallet.rs`, `transfer.rs`): Transfer NVCoin wajib tanda tangan kriptografis. Pencocokan `verifying_key → address` sudah diterapkan (mencegah replay attack).
2. **Anti-Sybil Defense** (`security.rs`): Batas 3 akun per perangkat (device fingerprint). MFA threshold untuk transaksi besar (>50 NVC).
3. **Block Integrity Validation** (`block.rs`, `network.rs`): P2P node melakukan validasi `previous_hash`, index, re-hash, dan PoW difficulty sebelum menerima block.
4. **Admin Audit Trail** (`admin.rs`): Admin actions (freeze/unfreeze/emergency) dicatat dengan timestamp, target, dan alasan.
5. **Soulbound Founder ID** (`identity.rs`): Founder ID (#1) tidak bisa diperdagangkan (`can_be_traded() = false`).

---

## 🔴 Temuan Kritis

### K-01: API Tidak Memiliki Autentikasi
**File**: `api.rs` (Seluruh endpoint)  
**Risiko**: Siapa saja yang bisa mengakses port 3000 dapat memanggil endpoint admin seperti `/api/admin/freeze`, `/api/admin/nuke`, dan `/api/nlc`.  
**Dampak**: Penyerang dapat membekukan akun, mengaktifkan emergency lockdown, atau memanipulasi staking tanpa autentikasi.

**Rekomendasi**:
- Tambahkan middleware autentikasi (API Key atau JWT) untuk endpoint admin.
- Minimal: verifikasi `node_address` via signed request header.
- Endpoint `/api/admin/nuke` (emergency shutdown) WAJIB memerlukan konfirmasi multi-sig.

---

### K-02: Tidak Ada Rate Limiting pada API & P2P
**File**: `api.rs`, `network.rs`  
**Risiko**: Serangan DDoS dapat membanjiri node dengan request. P2P listener menerima koneksi tanpa batas.

**Rekomendasi**:
- Implementasi rate limiter per IP (misalnya max 60 req/menit).
- P2P: Batas koneksi per peer, timeout pada `handle_connection`.

---

## 🟠 Temuan Sedang

### S-01: Floating-Point untuk Saldo Keuangan
**File**: `transfer.rs`, `reward.rs`, `contract.rs`  
**Risiko**: `f64` menyebabkan kesalahan pembulatan pada operasi aritmatika keuangan.

**Rekomendasi**: Migrasi ke integer (satuan terkecil, misal `u64` dalam satoshi/wei) atau gunakan `rust_decimal` crate.

---

### S-02: Device Fingerprint Mudah Dipalsukan
**File**: `security.rs` line 18-22  
**Risiko**: `generate_id()` hanya menggunakan IP + User-Agent. Mudah di-spoof.

**Rekomendasi**: Tambahkan faktor tambahan (hardware signature, TLS certificate fingerprint) dan implement challenge-response saat registrasi.

---

### S-03: P2P Peer Tidak Divalidasi
**File**: `network.rs` line 148-152  
**Risiko**: Siapa saja bisa mengirim pesan `Hello` dan langsung ditambahkan ke daftar peer tanpa verifikasi identitas.

**Rekomendasi**: Implementasi handshake kriptografis (mutual TLS atau signed Hello dengan NFM-ID).

---

### S-04: `can_transact()` Tidak Dipanggil di Transfer
**File**: `transfer.rs` → `fn transfer()`  
**Risiko**: Fungsi `admin.can_transact()` (cek freeze & emergency) tidak dipanggil di dalam `WalletEngine::transfer()`. Akun yang di-freeze masih bisa transfer jika flow tidak melewati admin check.

**Rekomendasi**: Integrasikan `AdminEngine` reference ke `WalletEngine`, atau pastikan caller selalu memanggil `can_transact()` sebelum `transfer()`.

---

## 🟡 Temuan Rendah

### R-01: MFA Challenge Deterministik
**File**: `security.rs` line 84-96  
**Catatan**: Challenge MFA hanya hash dari address (statis). Bukan masalah karena ini simulasi, tapi harus diganti dengan OTP/biometrik di production.

---

### R-02: Hardcoded Difficulty & Port
**File**: `block.rs` (difficulty=2), `api.rs` (port binding `0.0.0.0`)  
**Catatan**: Sebaiknya konfigurasi via file `config.yaml` atau environment variable.

---

### R-03: Panic pada Empty Chain
**File**: `main.rs` line 80-82  
**Catatan**: `get_latest_block()` menggunakan `.expect()` yang akan crash jika chain kosong. Meskipun genesis block selalu dibuat, error handling yang lebih graceful direkomendasikan.

---

## Rekomendasi Prioritas

| Prioritas | Item | Estimasi |
|-----------|------|----------|
| 1 | K-01: API Authentication | 2-3 hari |
| 2 | K-02: Rate Limiting | 1-2 hari |
| 3 | S-04: Admin check di transfer flow | 0.5 hari |
| 4 | S-01: Decimal precision | 2-3 hari |
| 5 | S-03: P2P handshake | 2-3 hari |
