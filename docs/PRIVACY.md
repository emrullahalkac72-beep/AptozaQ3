# Gizlilik

AptozaQ3 yerel öncelikli bir masaüstü uygulamasıdır. Bu belge hangi verilerin
işlendiğini ve nereye gittiğini açıklar.

## Kısa özet

- Hesap, abonelik, reklam veya telemetri **yoktur**.
- Kayıtlar, projeler, ayarlar ve geçici dosyalar bilgisayarınızda kalır.
- Uygulama medya işleme için yerel FFmpeg süreçlerini çalıştırır.
- İnternet yalnızca siz bir bağlantıya tıkladığınızda kullanılır (GitHub,
  YouTube vb.).
- Ekran, mikrofon ve webcam yalnızca siz kayıt başlattığınızda kullanılır.

## İşlenen veriler

### Ekran ve webcam

Kayıt başlattığınızda seçilen monitör/bölge ve webcam görüntüsü yakalanır.
Görüntüler yerel olarak işlenir ve seçtiğiniz çıktı klasörüne yazılır.

### Ses

Seçtiğiniz mikrofon veya ses aygıtından gelen ses, kayıt dosyasına eklenir.
Ses efekti araçlarında seçtiğiniz ses dosyaları yerel FFmpeg ile işlenir.

### Ayarlar

`~/.ekran_kayit_pro.json` içinde kayıt tercihleri, kısayollar, çıktı klasörü ve
tema tercihi tutulur. Bu dosya sizin bilgisayarınızdadır.

### Projeler

`.aptproj` dosyaları medya yollarını, kırpma bilgilerini, efektleri ve katman
konumlarını içerir. Proje dosyaları otomatik olarak hiçbir yere gönderilmez.

### Otomatik kurtarma

`~/.aptoza_timeline_autosave.aptproj` yalnızca kaydedilmemiş timeline
değişikliklerini kurtarmak için kullanılır. Başarılı kaydetme veya proje
kapatma sonrası silinir.

### Geçici dosyalar

FFmpeg işlemleri sırasında `%TEMP%` altında `aptoza_*` dosyaları oluşabilir.
Bunlar işlem sonunda temizlenir; beklenmedik kapanmada kalabilir ve elle
silinebilir.

## Ağ erişimi

Uygulama kendi başına ağ isteği göndermez. Ancak:

- **Yardım** sekmesindeki GitHub/YouTube bağlantıları varsayılan tarayıcıda
  açılır.
- Gelecekte bir güncelleme denetimi eklenirse bu belge güncellenir.

Kayıt, düzenleme ve dışa aktarma işlemleri çevrimdışı çalışır. FFmpeg
kurulumunuz kendi başına ağ erişimi yapabilir; bu AptozaQ3'ün kontrolü dışındadır.

## Üçüncü taraf bileşenler

- **FFmpeg** yerel bir süreç olarak çalışır ve medyanızı kendi belleğinde işler.
- **Rust crate'leri** uygulamaya derlenir; çoğu ağ erişimi yapmaz.
- **webbrowser** yalnızca açık bağlantıları tarayıcıya iletir.

## Çocukların gizliliği

Uygulama kişisel veri toplamaz. Ebeveynlerin, çocukların ekran/ses kaydı
yaparken gizlilik ve izin konusunda bilinçli olmasını öneririz.

## Haklarınız

Yerel verileriniz üzerinde tam kontrole sahipsiniz:

- Ayar dosyasını silerek tercihleri sıfırlayabilirsiniz.
- Proje ve kayıt dosyalarını istediğiniz zaman silebilirsiniz.
- `%TEMP%` altındaki `aptoza_*` dosyalarını temizleyebilirsiniz.

## Yasal uyarı

Başkalarının ekranını, sesini, toplantısını veya özel bilgilerini kaydetmek
yasalara ve kurum politikalarına tabi olabilir. Kayıt öncesi gerekli izinleri
almak sizin sorumluluğunuzdadır.

## İletişim

Gizlilikle ilgili sorular için GitHub üzerinden iletişime geçin:

**https://github.com/emrullahalkac72-beep**

Güvenlik açıkları için [SECURITY.md](../SECURITY.md) kullanın.

## Değişiklikler

Bu belge önemli ölçüde değişirse `CHANGELOG.md` ve sürüm notlarında belirtilir.

_Son güncelleme: 2026-10-28_
