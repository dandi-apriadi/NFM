# NFM Quest, Syndicates, and Item Economy

Sistem ini dirancang untuk meningkatkan interaksi sosial dan loyalitas pengguna melalui gamifikasi.

## 1. NFM Quest (Misi & Tantangan)
Daftar misi yang bisa dijalankan node operator:
- **[Awal] The First Fragment**: Selesaikan komputasi shard pertama Anda. Reward: Badge "Pioneer" (Untradable).
- **[Waktu Terbatas] Sovereign Spark**: Berhasil menyelesaikan 1.000 task dalam 24 jam. Reward: **Overclock Chip (Item)** yang menambah +20% reward selama 7 hari.
- **[Kolaboratif] Syndicate Alpha**: Bergabung dengan sindikat dan lalui 10.000 task bersama tim. Reward: **Syndicate Banner** (Item) yang memberikan bonus staking +5% bagi tim.
- **[Harian] The Educator**: Berikan 5 feedback berkualitas pada chat AI. Reward: **Intelligence Boost** (+10% reward per task berikutnya).

## 2. Item & Boost Economy
Item-item spesial yang bisa didapatkan dari Quest atau dibeli:
- **Overclock Chip (Consumable)**: Menambah performa komputasi tanpa panas berlebih sementara. Durasi: 24 Jam (Aktif).
- **VRAM Booster**: Memungkinkan node Lite menerima tugas kelas Pro sementara. Durasi: 48 Jam (Aktif).
- **Legacy Fragment (NFT)**: Item koleksi permanen.
- **Trade Mechanism**: Item yang belum digunakan (In-Box) dapat dijual/dikirim. Item yang sudah diaktifkan akan terkunci (Soulbound) ke ID tersebut hingga durasinya habis.

## 3. Aturan Anti-Eksploitasi (Item Lifecycle)
Untuk mencegah pengiriman item yang hampir habis masa gunanya:
- **Binding Mechanism**: Begitu item "Digunakan" (Activated), item tersebut **tidak bisa dikirim atau dijual** (Soulbound). Pengguna harus menghabiskan durasinya sendiri.
- **Real-time Decay**: Durasi item dihitung berdasarkan waktu server. Jika item berdurasi 24 jam diaktifkan jam 12 siang, maka jam 12 siang besok item tersebut akan hancur (Burn) otomatis.
- **Transparency**: Marketplace hanya menampilkan item yang masih dalam kondisi **"Mint/Sealed"** (belum pernah diaktifkan) untuk menjamin pembeli mendapatkan durasi penuh.

## 3. AI Syndicates (Social Staking)
- **Grup Komputasi**: Pengguna dapat bergabung menjadi satu kelompok besar untuk mengamankan satu cluster node virtual yang lebih besar.
- **Shared Rewards**: Hasil komputasi sindikat dibagi otomatis secara proporsional berdasarkan kontribusi hardware masing-masing anggota.
- **Team Leader Board**: Sindikat terbaik mendapatkan kuota "Tugas VIP" eksklusif dari jaringan.

## 4. Tatakelola & Validasi Quest (Quest Governance)
Untuk menjaga keseimbangan ekonomi, penambahan quest baru mengikuti aturan ketat:

### A. Tipe Misi (Quest Constraints)
- **One-Time Quest**: Misi yang hanya bisa diselesaikan sekali seumur hidup per NFM-ID (misal: "First Fragment").
- **Limited-Time Quest**: Misi yang hanya muncul pada event tertentu (misal: "Sovereign Spark") dan akan hilang setelah durasi event habis.
- **Recurring Quest**: Misi harian/mingguan untuk menjaga keaktifan node.

### B. Mekanisme Penambahan Quest (DAO Voting)
- **Quest Proposal**: Founder atau pemegang token terbanyak dapat mengajukan proposal quest baru, lengkap dengan deskripsi tugas dan hadiah yang diinginkan.
- **AI Reward Validator (NFM Brain)**: Sebelum masuk ke tahap voting, **NFM Brain** secara otomatis akan melakukan analisa:
    - Apakah tingkat kesulitan misi sebanding dengan hadiah (reward) yang diminta?
    - Apakah hadiah tersebut akan menyebabkan inflasi token yang berlebihan?
- **Veto AI**: Jika AI mendeteksi potensi eksploitasi (misal: misi terlalu mudah tapi hadiah terlalu besar), proposal akan ditolak secara otomatis atau dikembalikan untuk diperbaiki sebelum bisa di-vote oleh komunitas.
