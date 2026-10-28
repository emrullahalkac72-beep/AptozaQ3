 📖 APTOZAQ3 — Kullanma Kılavuzu

İçindekiler
1. İlk Kurulum ve Başlatma
2. Ana Ekran ve Genel Arayüz
3. Ekran Kaydı Alma
4. Webcam Kullanımı
5. Ses Kaydı Ayarları
6. Filigran Ekleme
7. Ekran Üzerine Çizim
8. Klavye Kısayolları
9. Ses Efektleri Modülü
10. Görselden Video Oluşturma
11. Video Düzenleme Araçları
12. Zamanlanmış Kayıt
13. Ayarlar ve Yapılandırma
14. Sık Karşılaşılan Sorunlar

 1. İlk Kurulum ve Başlatma
 Gereksinimler
- Rust (güncel stable araç zinciri)
- FFmpeg (sistem PATH'inde kayıtlı olmalıdır)
 Adımlar
1. FFmpeg'i yükleyin:
| Platform | Komut / Adım |
| Windows  | https://ffmpeg.org adresinden indirin, PATH'e ekleyin |
| macOS    | `brew install ffmpeg` |
| Linux    | `sudo apt install ffmpeg` |
2. Uygulamayı derleyip başlatın:
```bash
cargo run --release
```
> ⚠️ FFmpeg bulunamazsa video/ses işleme özellikleri çalışmaz. Uygulama başlarken bu durumu size bildirir.
 
 2. Ana Ekran ve Genel Arayüz
Uygulama başladığında üstte her sekmeden erişilebilen bir araç çubuğu, sol tarafta kontrol paneli, sağ tarafta ise sekme tabanlı içerik alanı görünür.
Sekmeler
| 🖥️ Kayıt            | Ekran kaydı ayarları ve başlatma kontrolleri
| 🎵 Ses Efektleri    | Ses dosyalarına efekt uygulama
| 🖼️ Görsel→Video     | Fotoğraflardan video oluşturma
| ✂️ Video Düzenleme  | Kırp, birleştir, format dönüştür
| 🎞️ Timeline         | Çoklu klip, geçiş, metin, şekil ve ses düzenleme
 Genel Araç Çubuğu
| Düğme | İşlev |
| 🆕 | Yeni proje (Ctrl+N) |
| 📂 | Proje aç (Ctrl+O) |
| 💾 | Projeyi kaydet (Ctrl+S); Shift+Ctrl+S farklı kaydet |
| ↩ | Geri al (Ctrl+Z) |
| ↪ | Yinele (Ctrl+Y veya Ctrl+Shift+Z) |
| 📤 | Timeline'ı dışa aktar (Ctrl+E) |
| 📸 | Ekran görüntüsü al |
| ⏺ / ⏹ | Kaydı başlat / durdur |
| 🎨 | Çizim modunu aç/kapat |
| 🧰 | Çizim araç panelini aç/kapat |
Collapsible (Daraltılabilir) Bölümler
Bazı bölümler başlığa tıklanarak kapatılıp açılabilir. Bu özellik, küçük ekranlarda daha düzenli bir görünüm sağlar. Başlıktaki ▼ / ▶ oku bölümün açık/kapalı olduğunu gösterir.
 
 3. Ekran Kaydı Alma
 Kayıt Bölgesi Seçimi
1. Tam Ekran — Varsayılan ayardır; herhangi bir seçim yapılmadan kayıt başlar.
2. Bölge Seçimi — "Bölge Seç" butonuna tıklayın. Ekranda çıkan seçim aracıyla kayıt almak istediğiniz alanı sürükleyerek belirleyin. Koordinatlar `X,Y  Genişlik×Yükseklik px` olarak gösterilir.
 FPS ve Kalite
- FPS (Kare Hızı): Kaydırıcıyı kullanarak 5–60 arası bir değer seçin. Varsayılan değer 30 FPS'tir.
- Kalite: Düşük, Orta, Yüksek, Çok Yüksek seçeneklerinden birini seçin. Yüksek kalite daha büyük dosya boyutu üretir.
 Kaydı Başlatma, Duraklatma ve Durdurma
| Düğme / Kısayol | İşlev |
| ▶ Kayıt Başlat | Kayıt başlar |
| ⏸ Duraklat | Kayıt geçici olarak duraklar |
| ⏹ Durdur | Kayıt tamamlanır ve dosya kaydedilir |
| 📷 Ekran Görüntüsü | Anlık PNG alır |
Kayıt süresini sol paneldeki sayaçtan takip edebilirsiniz.
Çıkış Klasörü
"Kayıt Klasörü" alanının yanındaki 📂 butonuyla kayıtların nereye kaydedileceğini seçin. Varsayılan olarak kullanıcının `Belgeler` dizini kullanılır.
 
 4. Webcam Kullanımı
1. Webcam Aç/Kapat anahtarını etkinleştirin.
2. Açılır menüden kamera indeksini seçin (0 = birinci kamera).
3. Çözünürlük Sorgula butonuna tıklayarak kameranın desteklediği çözünürlükleri yükleyin.
4. Listeden istediğiniz çözünürlüğü seçin.
5. Kameranın ekrandaki konumunu (sol üst, sağ alt, vb.) ve boyutunu ayarlayın.
> 💡 Webcam görüntüsü kayıt sırasında video üzerine katman olarak eklenir.
 
 5. Ses Kaydı Ayarları
 Mikrofon
- Mikrofon Kaydı anahtarını açın.
- Açılır menüden kullanmak istediğiniz mikrofonu seçin.
- Ses seviyesi göstergesi kayıt sırasında anlık geri bildirim verir.
 Sistem Sesi
- Sistem Sesi Kaydı anahtarını açın (platform desteğine göre değişir).
> ⚠️ Ses kaydı, sistemdeki kayıt aygıtları ve FFmpeg'in dshow desteği üzerinden yapılır.
 
 6. Filigran Ekleme
 Metin Filigranı
1. "Filigran Metni" alanına yazmak istediğiniz metni girin.
2. Konumunu (sol üst, sağ alt, merkez vb.) seçin.
3. Opaklık kaydırıcısıyla saydamlığı ayarlayın.
 Görsel Filigran (Logo)
1. "Görsel Seç" butonuna tıklayın ve PNG/JPG logo dosyanızı seçin.
2. Seçilen dosya adı gösterilir. "Temizle" butonu ile kaldırabilirsiniz.
> Her iki filigran türü aynı anda kullanılabilir.
 
 7. Ekran Üzerine Çizim
Kayıt sırasında ekran üzerine anlık açıklama ve işaret ekleyebilirsiniz.
Araçlar
| 🖱️ | Mouse | Çizim yapmaz, imleç serbestçe hareket eder |
| ▭ | Dikdörtgen | Sürükleyerek dikdörtgen çizer |
| ○ | Daire | Sürükleyerek elips çizer |
| ➜ | Ok | Yönlü ok çizer |
| ╱ | Çizgi | Düz çizgi çizer |
| ✏ | Kalem | Serbest el çizimi; çizgi arka planla karışmaz, tam opak kalır |
| T | Metin | Tıklanan konuma metin ekler |
| ⌫ | Silgi | Üzerine geçilen çizimleri siler |
 Çizim Panelini Kullanmak
1. Sol panelde "Çizim Araçları" bölümünden araç seçin.
2. Renk kutusuna tıklayarak çizim rengi belirleyin.
3. Kalınlık kaydırıcısıyla fırça boyutunu ayarlayın.
4. Kayıt başladıktan sonra çizim ekrana yansır.
5. "Geri Al" / "Yinele" düğmeleri veya Ctrl+Z / Ctrl+Y ile son çizim adımlarını geri alın.
6. "Çizimleri Temizle" butonu tüm açıklamaları kaldırır.
> 💡 Kalem çizgileri yumuşatılır ve çizim katmanına tam opak olarak işlenir. Kayıt sırasında da aynı görünüm korunur.
 
 8. Klavye Kısayolları
 Varsayılan Kayıt Kısayolları
| Eylem | Varsayılan Kısayol |
| Kayıt Başlat | F9 |
| Duraklat / Devam | F10 |
| Durdur | F11 |
| Ekran Görüntüsü | F8 |
| Çizim Modu | F7 |
 Uygulama İçi Kısayollar
| Eylem | Kısayol |
| Yeni proje | Ctrl+N |
| Proje aç | Ctrl+O |
| Projeyi kaydet | Ctrl+S |
| Farklı kaydet | Ctrl+Shift+S |
| Geri al | Ctrl+Z |
| Yinele | Ctrl+Y veya Ctrl+Shift+Z |
| Dışa aktar | Ctrl+E |
| Seçileni sil | Delete veya Backspace |
| Oynat / duraklat | Boşluk |
| Başa dön / sona git | Home / End |
| Timeline yakınlaştır / uzaklaştır | + / - |
| Seçili klibi böl | S |
 > Çizim modu açıkken Ctrl+Z / Ctrl+Y önce çizim geçmişini, Timeline'da ise proje geçmişini geri alır/yineler.
 Kısayol Atama
1. Ayarlar sekmesinde "Klavye Kısayolları" bölümünü açın.
2. İlgili eylemin yanındaki "Değiştir" butonuna tıklayın.
3. Açılan pencerede atamak istediğiniz tuşa basın. `Ctrl`, `Shift`, `Alt` ile birlikte kullanılabilir.
4. "Uygula" butonuyla kaydedin. "Temizle" ile kısayolu kaldırabilirsiniz.
> Kısayollar `~/.ekran_kayit_pro.json` dosyasına otomatik kaydedilir.
 
 9. Ses Efektleri Modülü
"Ses Efektleri" sekmesine geçin.
 Dosya Seçimi
- Sağ panelde 📂 Ses Dosyası Seç butonuna tıklayın.
- Tek dosya veya birden fazla dosya seçebilirsiniz (toplu işlem).
- Desteklenen formatlar: MP3, WAV, OGG, AAC, FLAC, M4A, WMA.
 Dalga Formu
Dosya seçildikten sonra dalga formu görselleştirilir ve sağ panelde gösterilir.
 Efekt Grupları
| Grup | İçerik |
| 🎛️ Profesyonel İyileştirmeler | Normalizasyon, gürültü temizleme, kompresör, sessizlik silme, stereo dönüşüm |
| 📻 Ekolayzer & Filtreler | Bas, tiz, high-pass, low-pass, panning, vokal silici |
| 🎭 Modülasyonlar | Koro, flanger, tremolo, vibrato |
| 🎭 Yaratıcı & Eğlenceli | Pitch (ses tonu), hız, yankı, tersine çevirme |
| ⏱️ Geçişler | Fade in, fade out (saniye cinsinden) |
 Önizleme ve Kaydetme
- ▶️ Önizle — Efektler uygulanmış sesin ilk 45 saniyesini çalar.
- ⏹️ — Önizlemeyi durdurur.
- ✨ Efektleri Uygula & Kaydet — Efektleri seçili dosyaya uygular ve kayıt konumu sorar. Toplu işlemde her dosyaya `_FX` soneki eklenerek kaydedilir.
 
 10. Görselden Video Oluşturma
"GörseldenVideo" sekmesine geçin.
 Görsel Ekleme
1. ➕ Görsel Ekle butonuna tıklayın.
2. PNG, JPG, JPEG, BMP veya WebP formatındaki dosyaları seçin.
3. Eklenen görseller sıralı listede görünür.
 Süre Ayarı
- Her görselin yanındaki süre kutusunu düzenleyerek o görselin videoda kaç saniye görüneceğini belirleyin.
- Tüm görseller için varsayılan süreyi "Varsayılan Süre" alanından ayarlayın.
 Görsel Efektleri
Her görselin yanındaki 🎨 butonuna tıklayarak o görsele özel efekt penceresi açılır.
Renk Sekmesi: Parlaklık, kontrast, doygunluk, gama, ton (hue) ayarları.
FX Sekmesi: Bulanıklık, siyah-beyaz, sepya, kırmızı filtre, mavi filtre, vignette, keskinleştir, renkleri ters çevir.
Dönüşüm Sekmesi: Ayna (yatay çevir), dikey çevir, 90° sağa döndür, fade in/out süresi.
> Efekt penceresinde değişiklikler anlık önizlemede yansır.
 Video Oluşturma
1. Sağ panelden Çözünürlük seçin (1920x1080, 1280x720, 854x480 veya Özel).
2. 📂 Kayıt Yeri alanından çıktı dosyasını belirleyin.
3. 🎬 Video Oluştur butonuna tıklayın.
4. İşlem tamamlandığında bildirim gösterilir.
 
 11. Video Düzenleme Araçları
"Video Düzenleme" sekmesine geçin. Tüm işlemler için sistemde FFmpeg yüklü olması gerekir.
 Video Kırpma (Trim)
1. "Kaynak Video" alanına dosya yolu girin veya 📂 ile seçin.
2. Süreyi Oku butonuyla toplam süreyi görüntüleyin.
3. Başlangıç (sn) ve Bitiş (sn) değerlerini girin.
4. ✂️ Kırp butonuna tıklayın ve çıktı konumunu belirleyin.
 Video Birleştirme (Merge)
1. "Ekle" butonu ile birden fazla video dosyası ekleyin.
2. Sırayı kontrol edin (üstteki dosya videoda önce gelir).
3. 🔗 Birleştir butonuna tıklayın.
 Format Dönüştürme
1. Kaynak videoyu seçin.
2. Hedef formatı açılır menüden seçin (MP4, AVI, MKV, MOV, WebM, vb.).
3. 🔄 Dönüştür butonuna tıklayın.
 
 12. Zamanlanmış Kayıt
1. "Zamanlanmış Kayıt" anahtarını açın.
2. Başlangıç Saati ve Bitiş Saati alanlarını `HH:MM` formatında doldurun.
3. Zamanı Ayarla butonuna basın.
4. Durum çubuğunda bekleme zamanı gösterilir.
5. Belirlenen saatte kayıt otomatik başlar ve bitiş saatinde durur.
> Saatler geçmiş bir değerse bir sonraki gün için ayarlanır.
 
 13. Ayarlar ve Yapılandırma
Ayarlar otomatik olarak `~/.ekran_kayit_pro.json` dosyasına kaydedilir. Uygulama her başladığında bu dosya okunur.
 Kaydedilen Ayarlar
- Çıkış dizini
- FPS ve kalite tercihi
- İmleç vurgulama
- Filigran metni, konumu, opaklığı
- Klavye kısayolları
 Ayarları Sıfırlamak
`~/.ekran_kayit_pro.json` dosyasını silin ve uygulamayı yeniden başlatın.
 
 14. Sık Karşılaşılan Sorunlar
 Uygulama başlamıyor
- `cargo build --release` çıktısındaki hata mesajını kontrol edin.
- Güncel Rust araç zincirini kullandığınızdan emin olun.
 Video kaydedilemiyor / boş dosya
- FFmpeg'in yüklü ve PATH'te tanımlı olup olmadığını kontrol edin:
  ```bash
  ffmpeg -version
  ```
 Ses kaydı çalışmıyor
- Doğru mikrofon cihazının seçili olduğundan emin olun.
- Windows'ta sistem gizlilik ayarlarında uygulamaya mikrofon izni verin.
- Ses aygıtının başka bir uygulama tarafından kullanılmadığını kontrol edin.
 Webcam görüntüsü gelmiyor
- Kamera indeksini değiştirip deneyin (0, 1, 2…).
- Başka bir uygulama kamerayı kullanıyor olabilir; önce o uygulamayı kapatın.
 Klavye kısayolları çalışmıyor
- Bir metin alanına yazarken uygulama içi kısayollar devre dışı kalır; önce alandan çıkın.
- Kısayol atama penceresi açıkken diğer kısayollar dinlenmez.
- Global kayıt kısayolları için uygulamanın arka planda çalıştığından emin olun.
 Ses önizlemesi çalışmıyor
- FFmpeg ile birlikte gelen `ffplay` aracının PATH'te olduğundan emin olun.
- Çıktı aygıtının doğru seçildiğini kontrol edin.
*EkranKayıt Pro — Tüm hakları saklıdır.*
 Timeline Projesi Kaydetme / Açma
- Timeline sekmesinde **💾 Proje Kaydet** (Ctrl+S) ile tüm düzen `.aptproj` dosyasına kaydedilir: klipler, kırpmalar, hız, efektler, geçişler, metinler, şekil/görseller, ses parçaları, dışa aktarma çözünürlüğü ve zoom.
- **Shift+Ctrl+S**: farklı kaydet. **📁 Proje Aç** (Ctrl+O): kaydedilmiş projeyi açar, her şey kaldığı gibi geri gelir.
- Proje dosyası medya dosyalarını kopyalamaz, yollarını saklar. Medya taşınmış/silinmişse proje açılırken uyarı gösterilir.
- **Kaydedilmemiş değişiklik uyarısı:** Başka bir proje açarken ya da uygulamayı kapatırken timeline'da kaydedilmemiş değişiklik varsa "Kaydet / Kaydetmeden devam et / İptal" sorulur.
- **Otomatik kayıt:** Kaydedilmemiş değişiklikler 30 saniyede bir kurtarma dosyasına (`~/.aptoza_timeline_autosave.aptproj`) yazılır. Uygulama beklenmedik şekilde kapanırsa bir sonraki açılışta "Geri Yükle / Sil" sorulur. Kendi proje dosyanız otomatik kayıtla değiştirilmez.
