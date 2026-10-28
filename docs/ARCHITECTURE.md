# Mimari

Bu belge AptozaQ3'ün modüllerini, veri akışını, thread modelini ve dışa aktarma
hattını açıklar. Kod değiştirmeden önce ilgili bölümü okumanız önerilir.

## 1. Genel bakış

AptozaQ3, Rust 2021 ile yazılmış bir masaüstü uygulamasıdır. Arayüz
[egui/eframe](https://github.com/emilk/egui) üzerinde çalışır; medya işleme
sistemde kurulu FFmpeg araçlarına (`ffmpeg`, `ffprobe`, `ffplay`) devredilir.

```text
┌──────────────────────────────────────────────────────────────┐
│                         eframe / egui                        │
│  üst araç çubuğu · kenar çubuğu · sekmeler · diyaloglar      │
└───────────────┬───────────────────────────────┬──────────────┘
                │                               │
        ┌───────▼────────┐              ┌───────▼────────┐
        │  Kayıt motoru  │              │ Timeline modeli│
        │  recorder.rs   │              │  timeline.rs   │
        └───────┬────────┘              └───────┬────────┘
                │                               │
     ┌──────────▼──────────┐          ┌─────────▼─────────┐
     │ ekran · webcam · ses│          │ FFmpeg filtre/    │
     │ thread'leri         │          │ concat/xfade hattı│
     └──────────┬──────────┘          └─────────┬─────────┘
                │                               │
                └──────────────┬────────────────┘
                               ▼
                    ┌────────────────────┐
                    │ media.rs / util.rs │
                    │ FFmpeg + dosya IO  │
                    └────────────────────┘
```

## 2. Modül haritası

| Dosya | Sorumluluk |
|---|---|
| `src/main.rs` | `eframe` başlangıcı, viewport boyutu, simge, OpenGL (glow) seçimi |
| `src/app.rs` | `App` durumu, sekmeler, üst araç çubuğu, kısayollar, zamanlayıcı, global hotkey yönetimi |
| `src/app/record.rs` | Kayıt sekmesi, aygıt seçimi, bölge seçimi, filigran ve kayıt kontrolleri |
| `src/app/edit.rs` | Video kırpma/birleştirme/dönüştürme ve görselden video araçları |
| `src/app/overlay.rs` | Saydam çizim katmanı, çizim araç paneli, çizim girdisi ve çizim geçmişi |
| `src/app/timeline_ui.rs` | Timeline çizimi, sürükleme/kırpma, menüler, proje işlemleri, dışa aktarma UI'ı |
| `src/app/timeline_ui/dialogs.rs` | Metin/şekil editörleri, efekt pencereleri, onay diyalogları |
| `src/recorder.rs` | Kayıt motoru, ekran/webcam/ses thread'leri, FFmpeg süreç yönetimi |
| `src/timeline.rs` | `Clip`, `AudioTrack`, `TextOverlay`, `Asset`, `Transition`, `Project`; undo/redo, autosave, export |
| `src/media.rs` | FFmpeg video/ses/görsel sarmalayıcıları, ses efektleri, dalga formu |
| `src/draw.rs` | Çizim op modeli, rasterleştirme, fırça/şekil çizimi |
| `src/model.rs` | `Settings`, `RecorderConfig`, `Shortcuts`, varsayılanlar ve sanitizasyon |
| `src/theme.rs` | Renkler, tipografi ve ortak UI yardımcıları |
| `src/util.rs` | FFmpeg çalıştırma, süre sorgulama, kaçışlama, atomik yazma, geçici adlar |

## 3. Uygulama yaşam döngüsü

1. `main()` viewport'u oluşturur ve `App::default()` ile uygulamayı başlatır.
2. `App::new()` ayarları `Settings::load()` ile okur; bozuk değerler
   `sanitize()` ile güvenli aralıklara çekilir.
3. Global kayıt kısayolları `register_hotkeys()` ile kaydedilir. Kısayollar
   kullanıcı tarafından değiştirilebilir; çakışma/başarısızlık kullanıcıya
   gösterilir.
4. Her `update()` çağrısında sırasıyla üst araç çubuğu, kenar çubuğu, aktif
   sekme, diyaloglar ve housekeeping işleri çalışır.
5. Timeline değişiklikleri debounce edilmiş geçmişe ve 30 saniyelik otomatik
   kurtarma dosyasına yazılır.
6. Kapanış isteğinde kaydedilmemiş timeline varsa onay diyaloğu açılır.

## 4. UI ve durum yönetimi

`App` tek bir mutable durum nesnesidir; egui'nin immediate-mode yaklaşımı
nedeniyle ekran her karede yeniden çizilir. Kalıcı olmayan UI durumu (açık
diyaloglar, seçim, sürükleme) `App` ve `TimelineState` içinde tutulur.

Temel sekmeler:

- **Kayıt** — yakalama ve kayıt kontrolleri.
- **Düzenleme** — video araçları ve görselden video.
- **Timeline** — çok katmanlı düzenleme ve dışa aktarma.
- **Ses Editörü** — ses kırpma/birleştirme/efektler.
- **Ayarlar** — tema, çıktı klasörü, kısayollar.
- **Yardım** — kılavuz ve bağlantılar.

Metin alanlarına yazarken global kısayolların tetiklenmemesi için
`ctx.wants_keyboard_input()` ve `hotkey_capture` kontrolleri kullanılır.

## 5. Kayıt motoru

`recorder::Engine`, kayıt oturumunu yönetir:

- `Shared` durum: iptal bayrağı, pause durumu, çizim op'ları, zoom merkezi ve
  istatistikler.
- Ekran thread'i: `xcap` ile kare yakalar, bölge/zoom uygular, çizim katmanını
  kompozit eder ve ham kareleri FFmpeg'e iletir.
- Webcam thread'i: kameradan kare alır, PiP olarak yerleştirir.
- Ses thread'i: Windows dshow aygıtından PCM toplar; FFmpeg muxer'a verir.
- Kontrol thread'i: FFmpeg süreçlerini başlatır, olayları (`EngineEvent`)
  kanal üzerinden UI'a bildirir, durdururken thread'leri birleştirir.

UI thread'i hiçbir zaman uzun süreli yakalama veya FFmpeg işini doğrudan
çalıştırmaz. Kayıt sırasında çizim op'ları fiziksel kare üretiminden bağımsız
olarak `DrawState` içinde tutulur; hem ekranda hem kayıt karesinde aynı op'lar
kullanılır.

### Çizim katmanı

`DrawState` iki liste tutar:

- `ops`: fiziksel kayıt karelerine uygulanan çizimler.
- `overlay_ops`: kullanıcıya gösterilen mantıksal çizimler.

Bu ayrım, ölçeklenen/bölgeye kırpılan karelerde çizimlerin doğru konumda
kalmasını sağlar. `Kalem` aracı noktaları yumuşatır; çizgi gövdesi tam opaktır,
yalnızca kenarlarda anti-aliasing uygulanır. Böylece çizim arka planla
karışmaz. Çizim geçmişi `Ctrl+Z` / `Ctrl+Y` ile ayrı yönetilir.

## 6. Timeline modeli ve geçmiş

`TimelineState` şunları tutar:

- Video klipleri (`clips`) ve başlangıç zamanları.
- Ses katmanları (`audios`).
- Metin katmanları (`texts`).
- Şekil/görsel katmanları (`assets`).
- Klip çiftleri arasındaki geçişler (`HashMap<(Uid, Uid), Transition>`).

`Project`, diske yazılan saf veri modelidir (`.aptproj`, JSON). `Uid` değerleri
proje içinde benzersizdir; yükleme sırasında `next_uid` mevcut en büyük UID'in
üzerine taşınır.

### Undo / redo

- Her içerik değişikliğinde `history_snapshot()` alınır.
- 450 ms içindeki ardışık değişiklikler tek adımda birleştirilir.
- En fazla 100 adım tutulur.
- Oynatma konumu, zoom ve geçici seçimler snapshot dışında bırakılır.
- Draw modunda geçmiş çizim op'larına, timeline'da proje snapshot'larına
  yönlenir.

### Otomatik kurtarma

- 30 saniyede bir, yalnızca içerik değiştiyse
  `~/.aptoza_timeline_autosave.aptproj` güncellenir.
- Başlangıçta otomatik kayıt dosyası varsa kurtarma istemi gösterilir.
- Başarılı kaydetme veya proje kapatma sonrası kurtarma dosyası silinir.
- `source_path`, otomatik kaydın hangi proje dosyasına ait olduğunu saklar.

## 7. Dışa aktarma hattı

`timeline::export` (ve `export_inner`) şu sırayı izler:

1. Klipleri `timeline_start` ve `uid` sırasına göre toplar; süreyi hesaplar.
2. Hedef çözünürlüğü belirler (`Orijinal`, 4K, 1080p, 720p, 480p, 360p).
3. Her klip için:
   - kırpma (`trim_start`, `trim_end`), hız (`speed`) ve döndürme uygulanır;
   - klip efektleri (`eq`, `hue`, `gblur`, `unsharp`, `vignette`, `negate`,
     `hflip`, `vflip`, `fade`) filtre zincirine eklenir;
   - klip hedef çözünürlüğe `scale` + `pad`/`crop` ile normalize edilir;
   - ses varsa hedef format ve örnekleme oranına dönüştürülür; yoksa sessiz
     ses üretilir.
4. Metin ve şekil/görsel katmanları zaman aralıklarına göre `overlay`/`drawtext`
   ile uygulanır.
5. Geçişler `xfade` ile uygulanır; `cut`/`jumpcut` geçişsiz birleştirme kullanır.
6. Segmentler `concat` ile birleştirilir; ses ve video akışları senkronize
   edilir.
7. Çıktı atomik olarak hedef yola taşınır; geçici klasör temizlenir.

Önemli notlar:

- Segmentler birleştirilmeden önce çözünürlük, fps, piksel formatı ve ses
  düzeni normalize edilir. Farklı kaynakların doğrudan `concat` edilmesi
  bozulmalara yol açacağı için bu adım zorunludur.
- `Orijinal` seçildiğinde 90° döndürülen kliplerin metin ölçeği nihai çıktı
  boyutlarına göre hesaplanır.
- Geçişsiz (cut) durumda bile her segment ayrı işlenir; böylece eksik ses
  akışı olan kaynaklar güvenli biçimde birleştirilir.
- Hatalar `status` üzerinden UI'a taşınır; FFmpeg çıktısının son satırları
  kullanıcıya gösterilir.

## 8. Medya katmanı

`media.rs` FFmpeg komutlarını tipli yardımcılar arkasında toplar:

- `Video::trim`, `Video::merge`, `Video::convert`
- `Audio::trim`, `Audio::split`, `Audio::merge`, `Audio::convert`
- `AudioEffects::build_filters`, `apply`, `preview_file`
- `waveform` — ses dalga formu PNG'si
- Görselden video üretimi ve özel çözünürlük doğrulaması

Tüm FFmpeg çağrıları argüman dizisiyle yapılır; shell kullanılmaz. Filtre
metinleri `util::ffmpeg_escape_*` yardımcılarıyla kaçışlanır.

## 9. Ayarlar ve dosya güvenliği

- Ayarlar `~/.ekran_kayit_pro.json` içinde tutulur.
- `Settings::sanitize()` fps, ses oranı, CRF, çözünürlük, opaklık, zoom,
  konum ve metin alanlarını sınırlar.
- Proje yüklemede `Project::sanitize()` tüm süreleri, hızları, efektleri,
  katman konumlarını, renkleri ve UID'leri doğrular.
- `util::write_atomic` önce benzersiz bir geçici dosyaya yazar, sonra hedefe
  taşır. Geçici adlar süreç kimliği, milisaniyeli zaman damgası ve atomik sayaç
  içerir.
- `util::unique_temp` aynı saniyede üretilen adların çakışmasını engeller.

## 10. Thread kuralları

- UI thread'inde `std::thread::sleep` veya uzun FFmpeg komutu çalıştırmayın.
- Paylaşılan durumu `Arc<Mutex<...>>` ile koruyun; kilitleri UI çizimi sırasında
  uzun süre tutmayın.
- Kayıt thread'lerine iptal sinyali gönderin ve `join` ile bekleyin.
- FFmpeg süreçlerinin `stdout`/`stderr` çıktısını okuyun; aksi halde boru
  dolabilir ve süreç kilitlenebilir.
- Geçici dosyaları benzersiz adlarla oluşturun ve hata durumunda temizleyin.

## 11. Genişletme noktaları

- Yeni bir klip efekti: `ClipEffects`, filtre üretimi, UI kaydırıcısı ve
  `Project::sanitize` aralığı birlikte güncellenmelidir.
- Yeni bir geçiş: `TRANSITIONS`, `xfade` eşlemesi, önizleme ve doğrulama.
- Yeni bir çizim aracı: `draw.rs` op modeli, `overlay.rs` girdisi ve araç
  paneli.
- Yeni bir FFmpeg aracı: `media.rs` içinde tipli fonksiyon + hata yönetimi +
  test.

## 12. Bilinen sınırlar

- Birincil hedef Windows'tur; Linux/macOS'ta ekran ve ses yakalama platforma
  göre sınırlıdır.
- FFmpeg uygulamaya gömülü değildir; sistemde kurulu olmalıdır.
- Proje dosyası medyayı kopyalamaz; taşınan/silinen kaynaklar yeniden
  bağlanmalıdır.
- Donanım kodlaması (NVENC/QSV) otomatik seçilmez; yazılım kodlayıcı kullanılır.
- Çok büyük (saatlerce) timeline'lar disk alanı ve RAM kullanımını artırır.
