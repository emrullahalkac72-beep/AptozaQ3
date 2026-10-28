# Sürüm Notları

Her GitHub Release için bu klasörde `vX.Y.Z.md` dosyası tutulur. Dosya adı,
GitHub etiketiyle birebir eşleşmelidir.

## Mevcut sürümler

| Sürüm | Dosya |
|---|---|
| v1.0.0 | [v1.0.0.md](v1.0.0.md) |

## Yeni sürüm ekleme

1. `Cargo.toml` sürümünü artırın.
2. `docs/releases/vX.Y.Z.md` dosyasını oluşturun.
3. `CHANGELOG.md` içine aynı sürümü ekleyin.
4. `vX.Y.Z` etiketini gönderin.

Release iş akışı, gövde metnini doğrudan bu dosyadan okur:

```yaml
body_path: docs/releases/${{ github.ref_name }}.md
```

Dosya yoksa release iş akışı başarısız olur. Bu bilinçli bir kontroldür; sürüm
notu olmadan yayın yapılmasını engeller.

## İçerik önerisi

- Kısa özet
- Öne çıkanlar
- Eklenenler
- Değiştirilenler
- Düzeltilenler
- Güvenlik
- İndirme/kurulum
- Yükseltme notları
- Bilinen sorunlar
- Doğrulama

Ayrıntılı süreç: [../RELEASE.md](../RELEASE.md)
