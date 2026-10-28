# Katkı Sağlama Rehberi

AptozaQ3'e ilgi gösterdiğiniz için teşekkürler. Bu rehber; hata bildirimi,
özellik önerisi, kod katkısı ve dokümantasyon düzeltmelerinin nasıl yapılacağını
açıklar.

Bu proje [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) içindeki topluluk kurallarına
tabidir. Katkı sağlayarak bu kurallara uymayı kabul etmiş olursunuz.

---

## 1. Başlamadan önce

- Aynı hatayı veya öneriyi bildiren mevcut bir issue olup olmadığını arayın.
- Güvenlik açıklarını **herkese açık issue olarak açmayın**; [SECURITY.md](SECURITY.md)
  dosyasındaki özel bildirim yöntemini kullanın.
- Büyük özellikler veya mimari değişiklikler için önce bir issue açıp yaklaşımı
  tartışın. Böylece boşa emek harcanmasının önüne geçilir.
- UI metinleri Türkçe, kod ve teknik dokümantasyon İngilizce/Türkçe karışık
  olabilir. Mevcut dosyanın dilini koruyun.

## 2. Geliştirme ortamı

### Gereksinimler

- Windows 10/11 (birincil hedef)
- Rust stable
- Visual Studio Build Tools + Windows SDK
- FFmpeg, ffprobe ve ffplay (`PATH`'te)
- Git

### Kurulum

```powershell
git clone https://github.com/emralkac21/AptozaQ3.git
cd AptozaQ3
cargo build
cargo test --all-targets
```

FFmpeg doğrulaması:

```powershell
ffmpeg -version
ffprobe -version
ffplay -version
```

Ayrıntılı platform notları: [docs/BUILDING.md](docs/BUILDING.md)

## 3. Dal ve commit düzeni

### Dal adları

| Tür | Örnek |
|---|---|
| Özellik | `feat/timeline-snap` |
| Hata düzeltmesi | `fix/export-audio-sync` |
| Dokümantasyon | `docs/kurulum-notlari` |
| Bakım | `chore/dependency-update` |
| Performans | `perf/record-frame-copy` |

### Commit mesajları

[Conventional Commits](https://www.conventionalcommits.org/tr/) biçimi tercih
edilir:

```text
feat(timeline): klibe mıknatısla hizalama ekle
fix(export): sesi olmayan segmentlerde birleştirme hatasını düzelt
docs(readme): FFmpeg kurulum adımlarını güncelle
test(media): dalga formu testini genişlet
```

Kurallar:

- Bir commit tek bir mantıksal değişikliği ele alsın.
- Kullanıcıya görünen değişiklikleri açıklayın.
- Anlamsız mesajlardan (`update`, `fix`, `changes`) kaçının.
- Otomatik biçimlendirme commit'lerini ayrı tutun.

## 4. Kod standartları

### Rust

- `cargo fmt --all` zorunludur.
- `cargo clippy --all-targets` çalıştırın; yeni uyarı eklemeyin.
- `#![allow(dead_code)]` mevcut olsa da ölü kod eklemekten kaçının.
- Hata yönetiminde `anyhow::Result` ve kullanıcıya anlamlı Türkçe mesajlar
  kullanın.
- UI thread'inde uzun süren FFmpeg işlemleri çalıştırmayın; arka plan işi ve
  `Arc<Mutex<String>>` durum desenini kullanın.
- Kullanıcı girdilerini FFmpeg filtrelerine veya dosya yollarına aktarmadan
  önce doğrulayın/kaçışlayın.
- Sayısal değerlerde `NaN`, sonsuzluk ve alt/üst sınırları kontrol edin.
- Dosya yazımında `util::write_atomic` kullanın.

### UI / UX

- Mevcut egui stilini, tema renklerini ve `card`, `muted`, `primary_button`
  yardımcılarını koruyun.
- Türkçe etiketlerde tutarlılık sağlayın.
- Klavye erişimi ve odak davranışını bozmayın.
- Metin alanlarına yazarken global kısayolların devre dışı kalması gerektiğini
  unutmayın.
- Hata durumlarında kullanıcıya ne yapacağını söyleyin; yalnızca teknik hata
  kodu göstermeyin.

### Güvenlik

- `Command` argümanlarını shell'e vermeyin; mevcut `util::run`/`ffmpeg_ok`
  yapısını kullanın.
- Proje/ayar dosyalarını okurken `sanitize()` çağrılmadan kullanmayın.
- Geçici klasörleri `tempfile` veya süreç/ sayaç içeren benzersiz adlarla
  oluşturun.
- Kullanıcı metinlerini sınırlayın ve FFmpeg filtrelerinde kaçışlayın.

## 5. Testler

Değişiklik türüne göre:

```powershell
cargo test --all-targets
cargo test media::tests -- --nocapture
cargo test timeline::tests -- --nocapture
```

Ekran kaydı testi etkileşimli masaüstü gerektirir. CI veya uzak oturumda:

```powershell
$env:APTOZA_SKIP_SCREEN_TEST = "1"
cargo test --all-targets
```

Yeni özellik ekliyorsanız:

- Saf hesaplama/filtre üretimi için birim testi ekleyin.
- FFmpeg hattı değişiyorsa kısa süreli, düşük çözünürlüklü bir entegrasyon testi
  ekleyin.
- Testler geçici klasörlerini temizlesin.
- Testler birbirine veya sabit dosya adlarına bağımlı olmasın.

## 6. Pull request süreci

PR açmadan önce:

1. Dalınızı güncel `main` üzerine rebase edin.
2. `cargo fmt --all -- --check` başarılı olsun.
3. `cargo clippy --all-targets` yeni hata üretmesin.
4. `cargo test --all-targets` geçsin.
5. Kullanıcıya görünen değişiklikler için ekran görüntüsü veya kısa video
   ekleyin.
6. `CHANGELOG.md` içindeki `[Yayınlanmamış]` bölümünü güncelleyin.

PR açıklamasında şunlar bulunsun:

- Amaç ve kapsam
- Nasıl test edildiği
- Ekran görüntüsü/video (UI değişikliği varsa)
- Geriye dönük uyumluluk notu
- İlgili issue numarası (`Closes #123`)

PR şablonu otomatik olarak açılır: [.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md)

## 7. Sürümleme

- Sürüm numarası `Cargo.toml` içindeki `version` alanından yönetilir.
- SemVer kullanılır: `MAJOR.MINOR.PATCH`.
- Sürüm notları `docs/releases/` altında `vX.Y.Z.md` olarak tutulur.
- Release süreci: [docs/RELEASE.md](docs/RELEASE.md)

## 8. Lisans

Katkılarınız MIT Lisansı altında sunulur. Bunu kabul etmiyorsanız lütfen kod
göndermeyin.

## 9. Yardım

- Soru: [SUPPORT.md](SUPPORT.md)
- Hata: GitHub issue formu
- Güvenlik: [SECURITY.md](SECURITY.md)
- Davranış: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
