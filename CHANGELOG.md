# Değişiklik Günlüğü

Bu proje [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) biçimini ve
[Semantic Versioning](https://semver.org/lang/tr/) kurallarını izler.

## [Yayınlanmamış]

### Eklendi

- GitHub deposu altyapısı: CI, release iş akışı, issue/PR şablonları, Dependabot,
  güvenlik politikası, katkı rehberi ve ayrıntılı dokümantasyon.
- Windows geliştirme ve paketleme betikleri.

### Değiştirildi

- README ve kullanım kılavuzu güncel Rust/egui sürümüne göre yenilendi.

## [1.0.6] - 2026-10-28

### Eklendi

- Her sekmeden erişilebilen global üst araç çubuğu: yeni proje, aç, kaydet,
  geri al, yinele, dışa aktar, ekran görüntüsü, kayıt, çizim ve araç paneli.
- Timeline için 100 adımlık geri alma/yineleme geçmişi.
- Çizim katmanı için ayrı geri alma/yineleme geçmişi.
- `Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S`, `Ctrl+Z`, `Ctrl+Y`,
  `Ctrl+Shift+Z`, `Ctrl+E`, `Delete`, `Backspace`, `Space`, `Home`, `End`,
  `+`, `-` ve `S` kısayolları.
- Serbest çizim aracı **Kalem**; yumuşatılmış ve tam opak çizim.
- Timeline dışa aktarma için farklı çözünürlük ve ses düzeni normalizasyonu.
- Karışık çözünürlüklü, geçişsiz timeline dışa aktarma testi.
- Proje ve ayar dosyaları için kapsamlı doğrulama/sanitizasyon.

### Değiştirildi

- Çizim rasterleştirmesi tam opak çizgi gövdesi ve yalnızca kenarlarda
  anti-aliasing kullanacak şekilde güncellendi.
- Proje otomatik kaydı ve atomik dosya yazımı iyileştirildi.
- Timeline kırpma, hız, metin konumu ve dışa aktarma ölçekleme matematiği
  düzeltildi.
- `Orijinal` çözünürlükte 90° döndürülen kliplerde metin ölçeği çıktı
  boyutlarına göre hesaplanıyor.

### Düzeltildi

- `colorchannelmixer` filtrelerinde alfa kanalının kaybolması.
- Aynı saniyede üretilen geçici dosya adlarının çakışması.
- `write_atomic` geçici dosya adlarında eşzamanlı çakışma riski.
- Filigran ve metin katmanlarında sınırsız metin uzunluğu.
- `next_uid` taşma riski.
- Negatif/sonsuz kırpma ve bölme sürelerinin kabul edilmesi.
- Görselden video özel çözünürlüğünde üst sınır eksikliği.
- Kontrast kaydırıcısı ile proje doğrulama aralığının uyuşmaması.
- Geçişsiz export sırasında farklı çözünürlük veya eksik ses akışı.
- Webcam BGR tampon uzunluğu doğrulaması.
- Zamanlanmış kayıtta eşit/ters saat aralığı doğrulaması.

### Güvenlik

- FFmpeg filtre yolları ve `drawtext` metinleri kaçış karakterleriyle
  güçlendirildi.
- Ayarlar ve proje dosyaları yüklenirken sayısal aralıklar, renkler, konumlar
  ve metinler doğrulanıyor.
- Ekran görüntüsü bölgesi monitör sınırlarına karşı kontrol ediliyor.
- Dışa aktarma geçici klasörleri çakışmayacak şekilde oluşturuluyor.

## Sürüm geçmişi hakkında

1.0.6 öncesine ait ayrıntılı sürüm notları bu depoda tutulmamıştır. Eski
sürümler için GitHub Releases sayfasına bakın.

[Yayınlanmamış]: https://github.com/emralkac21/AptozaQ3/compare/v1.0.6...HEAD
[1.0.6]: https://github.com/emralkac21/AptozaQ3/releases/tag/v1.0.6
