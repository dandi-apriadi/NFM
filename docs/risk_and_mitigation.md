# NFM Risk Analysis & Mitigation Plan

Dokumen ini mencatat tantangan nyata yang mungkin dihadapi project NFM dan strategi pengelolaannya.

## 1. Tantangan Teknis (The "Cold Start" Problem)
- **Risiko**: Dengan hanya 4 node awal (1 laptop + 3 HP), performa AI LLM besar akan sangat lambat karena minimnya paralelisme.
- **Mitigasi**: 
    - Gunakan **Hybrid-Small Model**: Gunakan model ultra-kecil (seperti Phi-2 atau TinyLlama) untuk fase Alpha.
    - **Caching Task**: Simpan hasil pertanyaan populer secara lokal di node untuk respon instan.

## 2. Tantangan OS Mobile (Background Killing)
- **Risiko**: Sistem operasi (Android/iOS) sering mematikan aplikasi yang berjalan di background untuk menghemat baterai, yang dapat memutuskan koneksi node.
- **Mitigasi**: 
    - Implementasikan **Foreground Service** dengan notifikasi persisten.
    - **Idle-Only Policy**: Izinkan node bekerja maksimal hanya saat HP sedang di-charge dan terhubung Wi-Fi.

## 3. Tantangan Likuiditas Token
- **Risiko**: Jika tidak ada pembeli NFM Gold/Credit di awal, node operator mungkin merasa tidak ada insentif nyata (Token tidak laku).
- **Mitigasi**: 
    - **Founder Buyback**: Alokasikan sebagian keuntungan dari gateway Xendit untuk membeli kembali (buyback) token dari pasar untuk mengisi Reward Pool.
    - **Utility First**: Pastikan AI NFM sangat berguna sehingga orang mau membeli NFM Credit hanya untuk menggunakan jasanya.

## 4. Tantangan Keamanan Biometrik
- **Risiko**: Perangkat keras sidik jari/retina yang berbeda-beda kualitasnya di setiap HP bisa menyebabkan kegagalan login (false rejection).
- **Mitigasi**: 
    - Gunakan **Multi-Sovereign Auth**: Tetap sediakan Seed Phrase atau Social Recovery (Guardian) sebagai cadangan jika hardware biometrik bermasalah.

## 5. Tantangan Regulasi
- **Risiko**: Integrasi pembayaran fiat (Xendit) dengan blockchain sovereign bisa memicu pengawasan regulasi keuangan.
- **Mitigasi**: 
    - Pisahkan layer pembayaran (Fiat Bridge) sebagai modul opsional yang mematuhi hukum lokal (KYC jika diperlukan untuk penarikan besar).
