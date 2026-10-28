# Derleme ve Geliştirme

Bu rehber AptozaQ3'ü kaynaktan derlemek, test etmek ve paketlemek için gereken
adımları açıklar.

## 1. Hedef platformlar

| Platform | Durum | Not |
|---|---|---|
| Windows 10/11 x64 | Birincil, tam destek | Ekran, webcam, ses, global kısayollar |
| Linux x64 | Deneysel | Derleme mümkün; yakalama backend'i sınırlı |
| macOS | Deneysel | Derleme mümkün; ekran kaydı izni gerekir |

Windows dışındaki platformlarda `global-hotkey`, dshow ses aygıtları ve bazı
Win32 katman özellikleri devre dışı kalabilir. Sorun bildirirken platformunuzu
belirtin.

## 2. Windows gereksinimleri

### Zorunlu

- Windows 10/11 (64-bit)
- [Rust](https://rustup.rs/) stable (rustup önerilir)
- Visual Studio Build Tools 2022 + **Desktop development with C++**
- Windows 10/11 SDK
- FFmpeg, ffprobe ve ffplay (`PATH`'te)
- Git

### Rust kurulumu

```powershell
winget install --id Rustlang.Rustup -e
rustup default stable
rustup component add rustfmt clippy
```

Yeni bir terminal açıp doğrulayın:

```powershell
rustc --version
cargo --version
```

### Visual Studio Build Tools

Rust, MSVC bağlayıcısını kullanır. Kurulum sırasında şu bileşenleri seçin:

- MSVC v143 — VS 2022 C++ x64/x86 build tools
- Windows 10/11 SDK
- C++ CMake tools (isteğe bağlı)

Alternatif olarak Visual Studio Community/Professional kurulabilir.

### FFmpeg

```powershell
winget install --id Gyan.FFmpeg -e
```

veya:

```powershell
choco install ffmpeg -y
```

veya:

```powershell
scoop install ffmpeg
```

Yeni terminalde doğrulayın:

```powershell
ffmpeg -version
ffprobe -version
ffplay -version
```

> `ffplay` ses önizlemesi için kullanılır. Eksikse uygulama açılır ancak bazı
> önizleme işlevleri çalışmaz.

## 3. Depoyu klonlama

```powershell
git clone https://github.com/emrullahalkac72-beep/AptozaQ3.git
cd AptozaQ3
```

## 4. Derleme

### Hızlı geliştirme derlemesi

```powershell
cargo build
.\target\debug\aptoza.exe
```

Debug derlemesinde konsol penceresi açılır; bu, log ve panik mesajlarını
görmek için yararlıdır.

### Release derlemesi

```powershell
cargo build --release --locked
.\target\release\aptoza.exe
```

`windows_subsystem = "windows"` yalnızca release derlemesinde etkindir; release
çalıştırılabilir dosyası konsol penceresi açmaz.

### Doğrudan çalıştırma

```powershell
cargo run --release
```

## 5. Geliştirme komutları

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets
cargo check --all-targets
cargo test --all-targets
cargo build --release
```

Yardımcı betik:

```powershell
.\scripts\check.ps1
```

Betik sırasıyla biçim, clippy, check ve test çalıştırır. Etkileşimli masaüstü
olmayan ortamda:

```powershell
.\scripts\check.ps1 -Headless
```

`-Headless`, `APTOZA_SKIP_SCREEN_TEST=1` ayarlar ve kısa ekran kaydı testini
atlar.

## 6. Testler

```powershell
cargo test --all-targets
```

Testler FFmpeg gerektirir. Kısa ekran kaydı testi etkileşimli masaüstü ister.
CI veya uzak oturumda:

```powershell
$env:APTOZA_SKIP_SCREEN_TEST = "1"
cargo test --all-targets
Remove-Item Env:APTOZA_SKIP_SCREEN_TEST
```

Belirli bir test:

```powershell
cargo test timeline::tests -- --nocapture
cargo test media::tests -- --nocapture
```

## 7. Paketleme

```powershell
.\scripts\package.ps1
```

Betik:

1. Sürümü `Cargo.toml` içinden okur (veya `-Version` parametresini kullanır).
2. `cargo build --release --locked` çalıştırır.
3. `dist/AptozaQ3-<sürüm>-windows-x64/` klasörüne şunları kopyalar:
   - `AptozaQ3.exe` (kaynak: `target/release/aptoza.exe`)
   - `README.md`, `LICENSE`, `KULLANIM_KILAVUZU.md`
   - `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`,
     `SUPPORT.md`, `ACKNOWLEDGEMENTS.md`
   - `docs/` klasörünün tamamı (mimari, SSS, kısayollar, sürüm notları, görseller)
4. `dist/AptozaQ3-<sürüm>-windows-x64.zip` üretir.

Örnek:

```powershell
.\scripts\package.ps1 -Version 1.0.6
```

FFmpeg pakete **dahil edilmez**; kullanıcı kendi kurmalıdır.

## 8. Linux

Örnek (Debian/Ubuntu):

```bash
sudo apt update
sudo apt install -y build-essential pkg-config ffmpeg \
  libx11-dev libxi-dev libgl1-mesa-dev libxkbcommon-dev \
  libwayland-dev libudev-dev libasound2-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo build --release
```

Ekran yakalama için `xcap` ve masaüstü ortamına bağlı ek bağımlılıklar
gerekebilir. Wayland'da ekran yakalama izinleri ve portal desteği dağıtıma göre
değişir. Ayrıntılar için `xcap` belgelerine bakın.

## 9. macOS

```bash
xcode-select --install
brew install ffmpeg
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cargo build --release
```

İlk kayıtta **System Settings → Privacy & Security → Screen Recording** ve
**Microphone** izinlerini verin. Uygulamayı yeniden başlatmanız gerekebilir.

## 10. Yaygın derleme hataları

| Hata | Çözüm |
|---|---|
| `linker 'link.exe' not found` | Visual Studio Build Tools + C++ iş yükünü kurun, yeni terminal açın |
| `error: linker not found` (Linux) | `build-essential` ve geliştirme paketlerini kurun |
| `ffmpeg not found` (çalışma zamanı) | FFmpeg'i `PATH`'e ekleyin; terminali yeniden açın |
| `failed to load manifest` | Doğru dizinde olduğunuzdan emin olun; `Cargo.toml` kökte olmalı |
| `permission denied` | Antivirüs/izinleri kontrol edin; `target` klasörünü temizleyin |
| `cargo test` ekran testinde takılıyor | `APTOZA_SKIP_SCREEN_TEST=1` kullanın |
| Türkçe karakter bozuk | Terminali UTF-8 olarak açın; dosyalar UTF-8'dir |

## 11. CI ve release

- `.github/workflows/ci.yml`: biçim, clippy, check, test ve release build.
- `.github/workflows/release.yml`: `v*` etiketiyle Windows ZIP üretir ve GitHub
  Release'e ekler.
- Sürüm notları `docs/releases/vX.Y.Z.md` içinde tutulur.
- Ayrıntılı süreç: [RELEASE.md](RELEASE.md).

## 12. Katkı

Kod göndermeden önce [CONTRIBUTING.md](../CONTRIBUTING.md) ve
[ARCHITECTURE.md](ARCHITECTURE.md) dosyalarını okuyun.
