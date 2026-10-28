# GitHub Deposu Kurulumu

Bu belge depoyu GitHub'da yayınlarken önerilen ayarları ve kontrol listesini
içerir. Dosyalar hazırdır; buradaki adımlar GitHub arayüzünde yapılacak
ayarlardır.

## 1. Depo oluşturma

- **Repository name:** `AptozaQ3`
- **Description:** `Modern ekran kaydedici, video ve ses editörü — Rust/egui + FFmpeg`
- **Visibility:** Public (açık kaynak için)
- **Initialize:** README, .gitignore veya license **eklemeyin**; bunlar zaten
  depoda var.

Yerel klasörden ilk push:

```powershell
git init
git branch -M main
git add .
git commit -m "chore: ilk commit"
git remote add origin https://github.com/emralkac21/AptozaQ3.git
git push -u origin main
```

## 2. About bölümü

Depo ana sayfasında **About → Edit**:

- **Description:** yukarıdaki açıklama
- **Website:** isteğe bağlı (YouTube tanıtımı veya proje sayfası)
- **Topics:**

```text
screen-recorder
video-editor
audio-editor
timeline
rust
egui
ffmpeg
windows
screen-capture
desktop-app
```

## 3. Sosyal önizleme

GitHub, bağlantı paylaşımlarında 1280×640 PNG/JPG ister.

- Hazır PNG: `docs/assets/social-preview.png` (1280×640)
- Düzenlenebilir SVG: `docs/assets/banner.svg`
- PNG'yi **Settings → Social preview** bölümüne yükleyin.
- Öneri: 1280×640, 1 MB altı, metin güvenli alanda.

## 4. Dallar ve koruma

**Settings → Branches → Add branch protection rule:**

- Branch name pattern: `main`
- ✅ Require a pull request before merging
- ✅ Require status checks to pass (CI)
- ✅ Require branches to be up to date
- ✅ Do not allow bypassing the above settings (mümkünse)
- ✅ Restrict force pushes
- ✅ Restrict deletions

Tek kişilik projede doğrudan push gerekiyorsa en azından force-push ve silme
korumasını açık tutun.

## 5. Actions ayarları

**Settings → Actions → General:**

- **Actions permissions:** Allow all actions and reusable workflows
- **Workflow permissions:** Read and write permissions
- ✅ Allow GitHub Actions to create and approve pull requests (release için
  gerekli olabilir)

Release iş akışı `contents: write` iznini kendi içinde ister.

## 6. Güvenlik

**Settings → Code security and analysis:**

- ✅ Dependency graph
- ✅ Dependabot alerts
- ✅ Dependabot security updates
- ✅ Secret scanning
- ✅ Push protection
- ✅ Private vulnerability reporting

`SECURITY.md` ve GitHub Security Advisories bağlantısı hazırdır.

## 7. Issue ve PR şablonları

`.github/ISSUE_TEMPLATE/` altındaki formlar otomatik görünür:

- Bug report
- Feature request
- Documentation
- Question (config üzerinden)
- Boş issue'lar kapalı

`.github/PULL_REQUEST_TEMPLATE.md` otomatik yüklenir.

## 8. Etiketler

Önerilen etiketler:

| Etiket | Renk | Açıklama |
|---|---|---|
| `bug` | `#d73a4a` | Hata |
| `enhancement` | `#a2eeef` | Yeni özellik |
| `documentation` | `#0075ca` | Dokümantasyon |
| `good first issue` | `#7057ff` | Yeni katkıcı dostu |
| `help wanted` | `#008672` | Yardım isteniyor |
| `security` | `#b60205` | Güvenlik |
| `performance` | `#fbca04` | Performans |
| `windows` | `#0078d4` | Windows'a özgü |
| `question` | `#d876e3` | Soru |
| `dependencies` | `#0366d6` | Bağımlılık güncellemesi |
| `github-actions` | `#2088ff` | CI / Actions |
| `needs-triage` | `#ededed` | İncelenecek |
| `duplicate` | `#cfd3d7` | Yinelenen |
| `wontfix` | `#ffffff` | Düzeltilmeyecek |

`.github/release.yml` bu etiketleri sürüm notu kategorilerine ayırır.

## 9. Discussions

**Settings → General → Features → Discussions** etkinleştirin. Soru ve fikir
tartışmaları için faydalıdır. Destek rehberi Discussions'a atıf yapar.

## 10. Releases

- Etiket formatı: `v1.0.6`
- Her sürüm için `docs/releases/v1.0.6.md` dosyası bulunmalı.
- Release iş akışı ZIP ve `SHA256SUMS.txt` üretir.
- **Draft** olarak oluşturup kontrol ettikten sonra yayınlamak daha güvenlidir
  (iş akışını buna göre değiştirebilirsiniz).

## 11. İlk yayın kontrol listesi

- [ ] README'deki tüm bağlantılar çalışıyor
- [ ] Lisans GitHub tarafından MIT olarak algılanıyor
- [ ] `Cargo.toml` repository/homepage alanları dolu
- [ ] CI yeşil
- [ ] `docs/releases/v1.0.6.md` mevcut
- [ ] `docs/assets/banner.svg` ve `docs/assets/social-preview.png` mevcut
- [ ] Sosyal önizleme ayarlandı
- [ ] Konular (topics) eklendi
- [ ] Branch protection açık
- [ ] Dependabot aktif
- [ ] Private vulnerability reporting açık
- [ ] İlk release oluşturuldu
- [ ] README release badge'i güncel

## 12. README badge'leri

Badge'ler şu depo adresini kullanır:

```text
emralkac21/AptozaQ3
```

Depo adı/kullanıcı değişirse README, `Cargo.toml`, `CHANGELOG.md` ve
`docs/releases/*.md` içindeki bağlantıları güncelleyin.

## 13. Bakım

- Haftalık: Dependabot PR'larını inceleyin.
- Aylık: FFmpeg uyumluluğunu ve Windows runner imajını kontrol edin.
- Her sürümde: `CHANGELOG.md`, sürüm notları ve README'yi güncelleyin.
- Güvenlik bildirimlerine `SECURITY.md` süreleri içinde yanıt verin.
