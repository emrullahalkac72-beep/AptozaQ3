# Betikler

Windows geliştirme ve paketleme yardımcıları.

| Betik | Amaç |
|---|---|
| `check.ps1` | Biçim, clippy, check ve test kontrolleri |
| `package.ps1` | Release EXE + ZIP + SHA256 paketi üretir |

## Kullanım

```powershell
# Tüm kontroller (etkileşimli masaüstü)
.\scripts\check.ps1

# Ekran kaydı testini atla (CI / uzak oturum)
.\scripts\check.ps1 -Headless

# Testleri tamamen atla
.\scripts\check.ps1 -SkipTests

# Sürümü Cargo.toml'dan okuyarak paketle
.\scripts\package.ps1

# Belirli sürümle paketle
.\scripts\package.ps1 -Version 1.0.6

# Yeniden derlemeden paketle
.\scripts\package.ps1 -NoBuild
```

## Gereksinimler

- PowerShell 5.1 veya PowerShell 7+
- Rust stable
- FFmpeg, ffprobe, ffplay (`PATH`'te)
- Visual Studio Build Tools + Windows SDK (Windows)

Script'ler `ExecutionPolicy` nedeniyle engellenirse:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check.ps1
```

## Çıktılar

- `check.ps1` herhangi bir dosya üretmez.
- `package.ps1` çıktıları `dist/` altındadır ve `.gitignore` ile hariç tutulur.
