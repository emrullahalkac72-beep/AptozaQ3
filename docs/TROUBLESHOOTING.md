# Sorun Giderme

Bu belge en sık karşılaşılan sorunları ve çözümlerini listeler. Sorununuz burada
yoksa [SUPPORT.md](../SUPPORT.md) üzerinden hata bildirin.

## Hızlı kontrol listesi

1. `ffmpeg -version`, `ffprobe -version`, `ffplay -version` çalışıyor mu?
2. AptozaQ3 güncel mi?
3. Çıktı klasörü yazılabilir mi?
4. Kaynak medya dosyası hâlâ aynı yolda mı?
5. Terminali/uygulamayı yeniden başlattınız mı?

## FFmpeg ve medya sorunları

### “FFmpeg bulunamadı” / “ffmpeg çalıştırılamadı”

**Belirti:** Kayıt sonrası birleştirme, dışa aktarma, ses efekti veya dönüştürme
başarısız olur.

**Çözüm:**

```powershell
ffmpeg -version
ffprobe -version
ffplay -version
```

Komut bulunamıyorsa FFmpeg'i kurun ve `PATH`'e ekleyin. Kurulumdan sonra
terminali ve AptozaQ3'ü kapatıp yeniden açın.

```powershell
winget install --id Gyan.FFmpeg -e
```

Kurulumdan sonra hâlâ bulunamıyorsa:

```powershell
where.exe ffmpeg
```

Yol görünmüyorsa FFmpeg `bin` klasörünü kullanıcı `PATH` değişkenine ekleyin.

### “No such filter” / bilinmeyen filtre

**Neden:** FFmpeg sürümü eski veya ilgili codec/filtre olmadan derlenmiş.

**Çözüm:** Güncel bir FFmpeg dağıtımı kullanın. Ses efektleri için `loudnorm`,
`acompressor`, `afftfilt`, `rubberband` gibi filtreler gerekir; her FFmpeg
derlemesi bunları içermeyebilir. Gyan.dev veya BtbN yapılarını deneyin.

### Dışa aktarma çok yavaş

- 4K yerine 1080p/720p deneyin.
- Uzun timeline'larda disk alanı ve CPU kullanımını kontrol edin.
- Antivirüsün `%TEMP%` klasörünü taraması yavaşlatabilir.
- Aynı anda başka ağır video işleme uygulamaları çalıştırmayın.

### Export “ses akışı yok” hatası

Bu sürümde farklı çözünürlük ve eksik ses akışı normalizasyonu düzeltilmiştir.
Hata devam ederse:

- Kaynak dosyada ses olup olmadığını `ffprobe` ile kontrol edin.
- Tüm klipleri ve ses katmanlarını geçici olarak kaldırıp tek tek ekleyin.
- FFmpeg sürümünüzü güncelleyin.
- Hata çıktısının son 20 satırını issue'ya ekleyin.

```powershell
ffprobe -v error -show_streams "C:\Videos\kaynak.mp4"
```

### Video/ses senkronu kayıyor

- Kaynak dosyanın değişken kare hızına (VFR) sahip olup olmadığını kontrol
  edin; mümkünse sabit kare hızına dönüştürün.
- Çok yüksek hız değerleri (4× üzeri) ve çok kısa kliplerde küçük sapmalar
  olabilir.
- Farklı fps'li kaynakları aynı timeline'da kullanırken dışa aktarma fps'ini
  sabit tutun.

## Kayıt sorunları

### Kayıt siyah veya boş geliyor

- Monitör/bölge seçimini kontrol edin.
- Windows gizlilik ayarlarında ekran yakalama iznini kontrol edin.
- Uzak masaüstü (RDP) oturumunda ekran yakalama kısıtlı olabilir.
- Oyunlarda tam ekran yerine kenarlıksız pencere modunu deneyin.
- Grafik sürücüsünü güncelleyin.

### Kayıtta ses yok

1. **Kayıt** sekmesinde doğru ses aygıtını seçin.
2. Windows **Ayarlar → Gizlilik → Mikrofon** iznini açın.
3. Aygıtın başka uygulama tarafından kullanılmadığından emin olun.
4. Sistem sesi için uygun loopback/Stereo Mix aygıtını seçin; her ses
   sürücüsünde bu aygıt görünmez.
5. Kayıt sırasında ses seviyesini kontrol edin.

### Webcam açılmıyor

- Kamerayı kullanan diğer uygulamaları kapatın.
- **Yenile** ile kamera listesini güncelleyin.
- Windows kamera iznini kontrol edin.
- Desteklenen çözünürlük listesini sorgulayın; kamera yalnızca belirli
  modları destekleyebilir.
- USB kamerayı farklı bir porta takın.

### Kayıt dosyası oluşmuyor

- Çıktı klasörünün var ve yazılabilir olduğundan emin olun.
- Disk alanını kontrol edin.
- Antivirüsün dosyayı engellemediğinden emin olun.
- Türkçe/boşluklu yollar desteklenir; yine de sorun yaşıyorsanız
  `C:\AptozaTest` gibi basit bir klasör deneyin.

### Global kısayol çalışmıyor

- Başka bir uygulama aynı tuşu kullanıyor olabilir.
- AptozaQ3 arka planda çalışıyor mu?
- Yönetici olarak çalışan oyun/uygulamalar global kısayolları engelleyebilir.
- Ayarlardan farklı bir kombinasyon atayın.
- Kısayolları kaydettikten sonra uygulamayı yeniden başlatın.

## Çizim ve overlay sorunları

### Çizim arka planla karışıyor

Bu sürümde `Kalem` tam opak çizim kullanır. Eski sürümdeyseniz güncelleyin.
Hâlâ sorun varsa:

- Çizgi kalınlığını artırın.
- Çizim modunu kapatıp yeniden açın.
- Ekran ölçekleme oranını (DPI) kontrol edin.

### Overlay siyah görünüyor

Uygulama Windows'ta `glow` (OpenGL) renderer kullanır. Grafik sürücüsünü
güncelleyin. Uzak masaüstünde donanım hızlandırma kısıtlı olabilir.

### Çizim kayda yansımıyor

- Kayıt başlamadan önce çizim modunun açık olduğundan emin olun.
- Çizimleri kayıt sırasında yapın; kayıt öncesi çizimler kalıcı değildir.
- Çizim katmanının görünür olduğunu kontrol edin.

## Timeline sorunları

### Proje açıldı ama medya “bulunamadı” diyor

Proje dosyası medyayı kopyalamaz. Kaynak dosyaları taşıdıysanız:

- Dosyaları eski konumlarına geri taşıyın veya
- Yeni yoldan yeniden ekleyin.

### Geri al çalışmıyor

- Timeline sekmesinde misiniz? Çizim modunda geri al çizim geçmişine gider.
- Metin alanına odaklanmış olabilirsiniz; alandan çıkın.
- Çok hızlı ardışık değişiklikler tek adımda birleştirilir (450 ms).

### Kaydetme uyarısı sürekli çıkıyor

- Projeyi `Ctrl+S` ile kaydedin.
- Otomatik kurtarma dosyasını kabul edip kaydedin.
- Proje klasörünün yazılabilir olduğundan emin olun.

### Timeline'da ses/video hizası bozuk

- Klipleri kırptıktan sonra `timeline_start` değerlerini kontrol edin.
- Geçiş süreleri toplam süreyi aşıyorsa kısaltın.
- Farklı fps'li kaynakları normalize edin.

## Uygulama sorunları

### Uygulama açılmıyor

- Windows 10/11 64-bit kullandığınızdan emin olun.
- Antivirüs karantinasını kontrol edin.
- `target/release/aptoza.exe` dosyasını doğrudan çalıştırın.
- Debug derlemesinde konsol çıktısını inceleyin.

### Arayüz donuyor

- Çok büyük video dosyalarıyla çalışırken bekleyin; dışa aktarma arka planda
  sürüyor olabilir.
- Aynı anda birden fazla export başlatmayın.
- Uygulamayı yeniden başlatın; otomatik kurtarma dosyası varsa kurtarma istemi
  görünür.

### Türkçe karakterler bozuk

Dosyalar UTF-8'dir. Terminal/IDE kodlamasını UTF-8 yapın. Windows'ta eski
konsol kod sayfası sorun çıkarabilir:

```powershell
chcp 65001
```

## Hata raporu ekleri

Issue açarken şunları ekleyin:

- AptozaQ3 sürümü (`Cargo.toml` veya uygulama başlığı)
- Windows sürümü (`winver`)
- `ffmpeg -version` çıktısının ilk 3 satırı
- Hata adımları
- Ekran görüntüsü/kısa video
- Varsa FFmpeg stderr'inin son 20 satırı

Güvenlik açıkları için [SECURITY.md](../SECURITY.md) kullanın.
