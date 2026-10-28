# Güvenlik Politikası

## Desteklenen sürümler

Güvenlik düzeltmeleri öncelikli olarak en son kararlı sürüm için yayınlanır.

| Sürüm | Destek |
|---|---|
| 1.0.x (en son) | ✅ Aktif |
| 1.0.6 öncesi | ❌ Desteklenmez |
| Geliştirme (`main`) | ✅ Düzeltme kabul edilir, garanti verilmez |

## Güvenlik açığı bildirimi

**Lütfen güvenlik açıklarını herkese açık issue olarak bildirmeyin.**

Tercih edilen yöntem:

1. Deponun **Security** sekmesine gidin.
2. **Report a vulnerability** / **Advisories → New advisory** seçeneğini açın.
3. Özel bildirim formunu doldurun.

Doğrudan bağlantı:

**https://github.com/emralkac21/AptozaQ3/security/advisories/new**

Alternatif olarak GitHub profilinden özel iletişim talep edebilirsiniz:

**https://github.com/emralkac21**

### Bildirimde bulunması gerekenler

- Etkilenen sürüm ve işletim sistemi
- Açığın türü (komut enjeksiyonu, yol aşımı, dosya üzerine yazma, DoS, XSS vb.)
- Yeniden üretme adımları
- Varsa kavram kanıtı (PoC) veya hata kaydı
- Olası etki ve saldırganın gerektirdiği erişim düzeyi
- Önerilen düzeltme (varsa)

### Yanıt süreci

Bu proje gönüllü olarak geliştirilir; aşağıdaki süreler hedef niteliğindedir:

| Aşama | Hedef |
|---|---|
| İlk yanıt | 7 gün içinde |
| Değerlendirme | 14 gün içinde |
| Düzeltme planı | 30 gün içinde |
| Düzeltme sürümü | Ciddiyete göre en kısa sürede |

Düzeltme yayınlanana kadar ayrıntıları gizli tutmanızı rica ederiz. Katkınız
uygun görülürse sürüm notlarında teşekkür edilir.

## Kapsam

Aşağıdaki alanlar güvenlik kapsamındadır:

- FFmpeg/ffprobe/ffplay komut satırı oluşturma
- Proje (`.aptproj`), ayar ve medya dosyası yükleme
- FFmpeg filtre grafiği oluşturma (`drawtext`, `movie`, `colorchannelmixer`,
  `concat`, `xfade`)
- Dosya yolu işleme ve geçici dosya oluşturma
- Atomik dosya yazma ve otomatik kayıt
- Ekran görüntüsü / kayıt bölgesi sınırları
- Webcam ve ses tamponları
- Dışa aktarma sırasında temp klasör güvenliği
- Bağımlılık zinciri güvenlik açıkları

## Kapsam dışı

- Kullanıcının kendi kaydettiği içeriklerin telif/gizlilik ihlalleri
- Kullanıcının sisteminde zaten var olan zararlı yazılımlar
- FFmpeg'in kendi güvenlik açıkları (ilgili projeye bildirilmelidir)
- Windows işletim sistemi veya sürücü kaynaklı sorunlar
- Sosyal mühendislik
- Kullanıcı tarafından seçilen çıktı klasöründeki mevcut dosyaların üzerine
  yazılması (uygulama `-y` kullanır; kullanıcı çıktı yolunu kendisi seçer)

## Güvenlik tasarımı

- Dış komutlar `std::process::Command` ile argüman dizisi olarak çalıştırılır;
  shell kullanılmaz.
- Kullanıcı metinleri `drawtext` ve filtre yolları için kaçışlanır.
- Proje ve ayarlar yüklenirken sayısal aralıklar, konumlar, renkler ve metin
  uzunlukları doğrulanır.
- Dosya yazımı geçici dosya + yeniden adlandırma ile yapılır.
- Geçici adlar süreç kimliği, zaman damgası ve atomik sayaç içerir.
- Ekran görüntüsü bölgesi monitör sınırlarına karşı doğrulanır.
- Dışa aktarma işlemleri çakışmayan özel geçici klasörlerde çalışır.

## Teşekkür

Sorumlu bildirimde bulunan herkese teşekkür ederiz. İsim belirtmek
istemiyorsanız anonim bildirim de kabul edilir.
