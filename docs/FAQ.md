# Sık Sorulan Sorular

## Genel

### AptozaQ3 ücretsiz mi?

Evet. Proje MIT Lisansı ile açık kaynaktır. Ücretli sürüm, abonelik, reklam
veya özellik kilidi yoktur.

### Hangi platformlarda çalışır?

Birincil hedef **Windows 10/11 (64-bit)**'tir. Linux ve macOS'ta derlenebilir
ancak ekran/ses yakalama desteği deneyseldir.

### Kurulum paketi var mı?

Şu anda taşınabilir ZIP dağıtımı vardır. ZIP'i çıkarıp `AptozaQ3.exe` dosyasını
çalıştırın. MSI/NSIS kurulum paketi gelecekte eklenebilir.

### FFmpeg gerekli mi?

Evet. FFmpeg, ffprobe ve ffplay sistem `PATH`'inde olmalıdır. Uygulama FFmpeg'i
paket içinde dağıtmaz.

### İnternet bağlantısı gerekiyor mu?

Hayır. Kayıt, düzenleme ve dışa aktarma çevrimdışı çalışır. Yalnızca Yardım
sekmesindeki bağlantılara tıkladığınızda tarayıcı açılır.

### Verilerim bir yere gönderiliyor mu?

Hayır. Telemetri, hesap veya bulut yükleme yoktur. Ayrıntılar:
[docs/PRIVACY.md](PRIVACY.md).

## Kayıt

### Sistem sesini kaydedebilir miyim?

Windows'ta seçili ses aygıtı kullanılır. Sistem sesi için ses sürücünüzün
sunduğu **Stereo Mix** veya loopback aygıtını seçmeniz gerekir. Her sürücüde bu
aygıt görünmez. Mikrofon ve sistem sesini aynı anda kaydetmek, aygıt/sürücü
desteğine bağlıdır.

### Aynı anda birden fazla monitörü kaydedebilir miyim?

Hayır. Bir monitör veya seçili bir bölge kaydedilir. Çoklu monitör için ayrı
kayıtlar alıp timeline'da birleştirebilirsiniz.

### Azami kayıt süresi nedir?

Sabit bir süre sınırı yoktur; disk alanı, RAM ve CPU belirler. Uzun kayıtlarda
çıktı klasöründe yeterli alan olduğundan emin olun.

### Kayıt sırasında çizim yapabilir miyim?

Evet. `F7` ile çizim modunu açın. Çizimler hem ekranda hem kayıt karesinde
görünür. `Kalem` tam opak çizim kullanır.

### DRM korumalı içeriği kaydedebilir miyim?

Hayır. Korumalı içerikler siyah veya boş gelebilir. Bu, işletim sistemi ve
içerik koruma mekanizmalarının bir sonucudur.

### Kayıt sırasında bilgisayar yavaşlıyor

- FPS'i düşürün (60 → 30).
- 4K yerine 1080p kaydedin.
- Webcam/filigran gibi ek katmanları kapatın.
- Çıktı klasörünü hızlı bir diske taşıyın.
- Antivirüs taramalarını geçici olarak durdurun.

## Düzenleme

### Kaydı timeline'a nasıl eklerim?

Timeline sekmesinde **Video Ekle** ile kayıt dosyasını seçin. Klibi
sürükleyerek konumlandırabilir, kenarlardan kırpabilirsiniz.

### Kaç geçiş efekti var?

15 geçiş türü vardır: Cut, Fade, Dissolve, Wipe, Zoom, Jump Cut, Light Leak,
Glitch, Paper Tear, Wormhole, Warp Zoom, Flash, Cross Dissolve, Match Cut ve
Push.

### Geri alma kaç adım?

Timeline için 100 adım, çizim için ayrı bir geçmiş tutulur. Geçmiş uygulama
kapatıldığında sıfırlanır; proje dosyasına yazılmaz.

### Projeyi başka bilgisayarda açabilir miyim?

Evet, ancak `.aptproj` medyayı kopyalamaz. Medya dosyalarını da taşıyın veya
yolları aynı tutun.

### Metin/şekil katmanları neden görünmüyor?

- Katmanın zaman aralığı oynatma konusunu kapsıyor mu?
- Opaklık sıfır mı?
- Katman başka bir katmanın arkasında mı?
- Metin rengi arka planla aynı mı?

### Dışa aktarma hangi çözünürlükleri destekler?

`Orijinal`, 4K (3840×2160), 1080p, 720p, 480p ve 360p. `Orijinal`, ilk
klibin/çıktının doğal boyutunu kullanır.

### 4K export neden yavaş?

Yazılım kodlayıcı (libx264) ve filtre grafiği CPU yoğundur. 1080p deneyin,
geçiş/efekt sayısını azaltın ve arka plan uygulamalarını kapatın.

## Ses

### Ses efektleri hangi dosyalarda çalışır?

FFmpeg'in desteklediği yaygın formatlarda (MP3, WAV, M4A, AAC, OGG, FLAC vb.).
Bazı egzotik codec'ler için FFmpeg derlemenizde ilgili kodlayıcı olmayabilir.

### Vokal kaldırma nasıl çalışır?

Stereo kanallar arasındaki farkı kullanarak merkezdeki vokali azaltır. Her
şarkıda mükemmel sonuç vermez; mono kayıtlarda etkisi yoktur.

### Efekt önizlemesi neden 45 saniye?

Hızlı geri bildirim için önizleme ilk 45 saniyeyle sınırlıdır. Nihai uygulama
tüm dosyaya uygulanır.

## Teknik

### Hangi renderer kullanılıyor?

Windows'ta saydam çizim katmanının doğru çalışması için `eframe` **glow**
(OpenGL) renderer kullanır.

### GPU ile video kodlama var mı?

Şu anda otomatik NVENC/QSV/AMF seçimi yoktur; yazılım kodlayıcı kullanılır.
FFmpeg komutlarını özelleştirerek donanım kodlaması eklenebilir.

### Komut satırı arayüzü var mı?

Hayır. Uygulama masaüstü GUI olarak tasarlanmıştır. FFmpeg araçlarını doğrudan
kullanabilirsiniz.

### Ayarları nasıl sıfırlarım?

`~/.ekran_kayit_pro.json` dosyasını silin ve uygulamayı yeniden başlatın.
Otomatik kurtarma dosyasını da silmek için
`~/.aptoza_timeline_autosave.aptproj` dosyasını kaldırın.

### Güncelleme nasıl yapılır?

Yeni ZIP'i indirin, eski `AptozaQ3.exe` dosyasını değiştirin. Ayarlar ve
projeler korunur. Yedek almak için `.aptproj` dosyalarınızı saklayın.

### Windows SmartScreen uyarısı veriyor

Yayınlanan EXE kod imzalı değildir. “Ek bilgi → Yine de çalıştır” ile
ilerleyebilirsiniz. Kurumsal ortamda imzalı dağıtım için kendi sertifikanızla
imzalayın.

### Katkı sağlayabilir miyim?

Evet. [CONTRIBUTING.md](../CONTRIBUTING.md) ve
[CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) dosyalarını okuyun.

### Hata nasıl bildiririm?

GitHub issue formunu kullanın:
https://github.com/emrullahalkac72-beep/AptozaQ3/issues/new/choose

Güvenlik açıkları için [SECURITY.md](../SECURITY.md).
