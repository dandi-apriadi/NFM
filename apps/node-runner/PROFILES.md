# NFM Node Runner — Performance Profiles

Konfigurasi mode performa sesuai `docs/app_suite_definition.md`.

## Profiles

### Quiet (< 10% Resource)
```yaml
profile: quiet
cpu_limit_percent: 10
gpu_limit_percent: 0
ram_limit_mb: 256
network_priority: low
description: "Penggunaan resource sangat rendah, tidak mengganggu pekerjaan lain."
```

### Balanced (50% Resource) — Default
```yaml
profile: balanced
cpu_limit_percent: 50
gpu_limit_percent: 30
ram_limit_mb: 1024
network_priority: normal
description: "Keseimbangan optimal untuk pemakaian harian."
```

### Turbo (80-90% Resource)
```yaml
profile: turbo
cpu_limit_percent: 85
gpu_limit_percent: 70
ram_limit_mb: 4096
network_priority: high
description: "Performa tinggi untuk user yang jarang memakai laptop/HP."
```

### Hardcore (100% Resource)
```yaml
profile: hardcore
cpu_limit_percent: 100
gpu_limit_percent: 100
ram_limit_mb: 0  # Unlimited
network_priority: highest
description: "100% Resource didedikasikan untuk NFM. Reward maksimal."
```

## Mobile-Safe Mode
```yaml
mobile_safe:
  only_on_charge: true
  only_on_wifi: true
  battery_threshold: 20  # Stop jika baterai < 20%
  max_temperature_c: 40  # Throttle jika suhu > 40°C
```

## Cara Pakai
```bash
# Jalankan dengan profile tertentu
nfm-node --profile balanced

# Atau via environment variable
NFM_PROFILE=turbo nfm-node

# Docker
docker run -e NFM_PROFILE=quiet nfm-node
```
