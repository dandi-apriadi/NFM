# NFM Developer Debug & Reset Protocol (v7.5)

**PERINGATAN**: Dokumen ini hanya berlaku selama tahap **In-Dev Alpha/Beta**. Seluruh fitur di sini harus dihapus total sebelum melangkah ke **Mainnet Launch**.

## 1. The Nuclear Option (Dashboard Reset Button)
Fitur untuk membersihkan seluruh data jaringan guna pengujian ulang siklus ekonomi.

- **Lokasi**: Dashboard Admin Khusus (Hanya muncul untuk ID Founder).
- **Fungsi**: Menghapus seluruh Ledger, Inventori Item, dan NFM-ID pengguna lain.
- **Proteksi**: Sistem secara permanen mengecualikan (Hard-Coded Bypass) ID `nfm_0001...` (Founder) agar tidak ikut terhapus.

## 2. Arsitektur "No-Trace" (Modular Cleanup)
Untuk memastikan fitur ini tidak meninggalkan jejak keamanan di hari ke depan:
- **Modular Toggle**: Seluruh logika Reset dibungkus dalam modul `nfm_core::dev_tools`.
- **Feature Flag**: Hanya dapat dikompilasi jika flag `--features dev-reset` aktif.
- **Cleanup Instruction**: Sebelum produksi, cukup hapus folder `dev_tools/` dan baris deklarasi modul di `main.rs`.

## 3. Alur Kerja Reset
1. Founder menekan tombol **[NUKE PROJECT]**.
2. Sistem meminta konfirmasi biometrik/KYC ulang (Brain Check).
3. Database Sharding & Ledger Blockchain diatur kembali ke `Genesis Block`.
4. Founder tetap memegang status "The Founder" dan item "Legacy Core".
