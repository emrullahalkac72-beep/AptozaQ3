# Sürüm ve Yayın Süreci

Bu belge AptozaQ3 için sürüm numarası, etiket, GitHub Release ve paketleme
adımlarını açıklar.

## 1. Sürümleme

Proje [Semantic Versioning](https://semver.org/lang/tr/) kullanır:

- **MAJOR** — geriye dönük uyumsuz değişiklik
- **MINOR** — geriye dönük uyumlu yeni özellik
- **PATCH** — geriye dönük uyumlu hata düzeltmesi

Sürümün tek kaynağı `Cargo.toml` içindeki `version` alanıdır. Etiketler `v`
önekiyle oluşturulur: `v1.0.6`.

## 2. Sürüm hazırlığı

1. `main` dalının güncel ve yeşil olduğundan emin olun.
2. `Cargo.toml` sürümünü artırın.
3. `CHANGELOG.md` içine yeni sürüm bölümünü ekleyin.
4. `docs/releases/vX.Y.Z.md` sürüm notlarını yazın.
5. Gerekirse `README.md` ve `KULLANIM_KILAVUZU.md`'yi güncelleyin.
6. Yerel kontrolleri çalıştırın:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets
cargo check --all-targets
$env:APTOZA_SKIP_SCREEN_TEST = "1"
cargo test --all-targets
cargo build --release --locked
```

7. Değişiklikleri commit'leyin:

```powershell
git add Cargo.toml Cargo.lock CHANGELOG.md docs/releases/v1.0.6.md
git commit -m "chore(release): v1.0.6"
```

8. Etiketi oluşturup gönderin:

```powershell
git tag -a v1.0.6 -m "AptozaQ3 v1.0.6"
git push origin main
git push origin v1.0.6
```

> Etiket gönderildiğinde `.github/workflows/release.yml` çalışır. Etiketi
> göndermeden önce CI'ın yeşil olduğundan emin olun.

## 3. Otomatik release iş akışı

`.github/workflows/release.yml` şunları yapar:

1. Etiket ile `Cargo.toml` sürümünü ve `docs/releases/<etiket>.md`
   dosyasını doğrular.
2. Windows runner üzerinde Rust stable kurar ve FFmpeg'i hazırlar.
3. `APTOZA_SKIP_SCREEN_TEST=1` ile testleri çalıştırır.
4. `cargo build --release --locked` çalıştırır.
5. `AptozaQ3-<sürüm>-windows-x64.zip` paketini üretir.
6. `SHA256SUMS.txt` oluşturur.
7. GitHub Release'i `docs/releases/<etiket>.md` gövdesiyle oluşturur.
8. ZIP ve checksum dosyasını release'e ekler.

Örnek varlıklar:

- `AptozaQ3-1.0.6-windows-x64.zip`
- `SHA256SUMS.txt`

## 4. Sürüm notları

Her sürüm için `docs/releases/vX.Y.Z.md` dosyası oluşturun. Dosya adı etiketle
aynı olmalıdır (örn. `v1.0.6.md`).

Önerilen yapı:

```markdown
# AptozaQ3 vX.Y.Z

Kısa özet.

## Öne çıkanlar
- ...

## Eklenen
- ...

## Düzeltilen
- ...

## Kurulum
...

## Bilinen sorunlar
...
```

Sürüm notları kullanıcıya dönük olmalı; commit listesi değil, neyin değiştiği
ve neyin önemli olduğu açıklanmalıdır.

## 5. Release sonrası kontrol

- [ ] GitHub Actions yeşil
- [ ] Release oluştu ve taslak değil
- [ ] ZIP indirilebiliyor
- [ ] ZIP içinde `AptozaQ3.exe`, `README.md`, `LICENSE`, `KULLANIM_KILAVUZU.md`
      ve `docs/` klasörü var
- [ ] README'deki göreli bağlantılar ZIP içinde çalışıyor
- [ ] `SHA256SUMS.txt` doğru
- [ ] Sürüm notları görünüyor
- [ ] README'deki release bağlantısı çalışıyor
- [ ] Etiket doğru commit'i gösteriyor

## 6. Manuel paketleme

CI kullanmadan:

```powershell
.\scripts\package.ps1 -Version 1.0.6
```

Ardından GitHub'da **Releases → Draft a new release** ile etiketi seçip ZIP'i
yükleyin. Sürüm notlarını `docs/releases/v1.0.6.md` içinden kopyalayın.

## 7. Geri alma

Hatalı bir sürüm yayınlandıysa:

1. Release'i **draft** durumuna alın veya silin.
2. Gerekirse etiketi silin:

```powershell
git tag -d v1.0.6
git push origin :refs/tags/v1.0.6
```

3. Düzeltilmiş sürümü `v1.0.7` olarak yayınlayın. Yayınlanmış bir etiketi
   mümkünse yeniden kullanmayın.

## 8. Kod imzalama

Mevcut otomatik yapı kod imzasızdır. Windows SmartScreen uyarısı gösterebilir.
Kurumsal dağıtım için kendi code-signing sertifikanızla imzalayın ve bu adımı
release iş akışına ekleyin.

## 9. FFmpeg

FFmpeg telif/lisans nedenleriyle ZIP'e dahil edilmez. Kullanıcılar FFmpeg'i
kendisi kurmalıdır. README ve sürüm notlarında bu belirtilir.
