# Proje ve Ayar Dosyası Formatı

AptozaQ3 projeleri `.aptproj` uzantılı, UTF-8 kodlanmış JSON dosyalarıdır.
Ayar dosyası da JSON'dur. Bu belge alanları, aralıkları ve uyumluluk kurallarını
açıklar.

## 1. Proje dosyası (`.aptproj`)

### Üst düzey yapı

```json
{
  "version": 1,
  "clips": [],
  "audios": [],
  "texts": [],
  "assets": [],
  "transitions": [],
  "export_res": "Orijinal",
  "px_per_sec": 60.0,
  "playhead": 0.0,
  "transition_sel": 0,
  "transition_dur": 0.5,
  "next_uid": 1,
  "source_path": null
}
```

| Alan | Tür | Açıklama |
|---|---|---|
| `version` | `u32` | Şema sürümü; şu anda `1` |
| `clips` | `Clip[]` | Video klipleri |
| `audios` | `AudioTrack[]` | Ses katmanları |
| `texts` | `TextOverlay[]` | Metin katmanları |
| `assets` | `Asset[]` | Şekil ve görsel katmanları |
| `transitions` | `[uid, uid, Transition][]` | Klip çiftleri arası geçişler |
| `export_res` | `string` | `Orijinal`, `3840x2160`, `1920x1080`, `1280x720`, `854x480`, `640x360` |
| `px_per_sec` | `f32` | Timeline yakınlaştırması (5–300) |
| `playhead` | `f64` | Oynatma konumu (saniye) |
| `transition_sel` | `usize` | Seçili geçiş türü indeksi |
| `transition_dur` | `f32` | Varsayılan geçiş süresi (0.1–2.0) |
| `next_uid` | `u64` | Sonraki benzersiz kimlik |
| `source_path` | `string?` | Otomatik kayıtta özgün proje yolu; normal kayıtta `null` |

> `version` ileride şema değişiklikleri için ayrılmıştır. Bilinmeyen alanlar
> serde tarafından yok sayılır; eksik alanlar varsayılan değerlerle tamamlanır.

### Clip

```json
{
  "uid": 1,
  "path": "C:/Videos/kayit.mp4",
  "name": "kayit.mp4",
  "duration": 12.4,
  "trim_start": 0.0,
  "trim_end": 10.0,
  "speed": 1.0,
  "scale_w": 0,
  "scale_h": 0,
  "timeline_start": 0.0,
  "mute": false,
  "effects": {
    "brightness": 0.0,
    "contrast": 1.0,
    "saturation": 1.0,
    "gamma": 1.0,
    "hue": 0.0,
    "blur": 0.0,
    "bw": false,
    "sepia": false,
    "red_filter": false,
    "blue_filter": false,
    "vignette": false,
    "sharpen": false,
    "invert": false,
    "mirror": false,
    "vflip": false,
    "rotate": false,
    "fade_in": 0.0,
    "fade_out": 0.0
  }
}
```

| Alan | Açıklama |
|---|---|
| `path` | Mutlak medya yolu; proje dosyası medyayı kopyalamaz |
| `duration` | Kaynak süresi (saniye), 0.05–864000 |
| `trim_start` / `trim_end` | Kaynak içindeki kırpma aralığı |
| `speed` | 0.1–16.0; timeline süresi `(trim_end - trim_start) / speed` |
| `scale_w` / `scale_h` | `0,0` = doğal boyut; aksi halde 2–16384 |
| `timeline_start` | Timeline üzerindeki başlangıç (saniye) |
| `mute` | Klip sesini sustur |
| `effects` | Klip başına görsel efektler |

Efekt aralıkları:

- `brightness`: -1.0 – 1.0
- `contrast`: 0.0 – 4.0
- `saturation`: 0.0 – 4.0
- `gamma`: 0.1 – 10.0
- `hue`: -360 – 360
- `blur`: 0 – 50
- `fade_in` / `fade_out`: 0 – 3600 saniye

### AudioTrack

```json
{
  "uid": 2,
  "path": "C:/Audio/muzik.mp3",
  "name": "muzik.mp3",
  "duration": 180.0,
  "trim_start": 0.0,
  "trim_end": 30.0,
  "start_time": 2.0,
  "volume": 1.0
}
```

`volume` 0.0–4.0 aralığındadır. `start_time` timeline üzerindeki başlangıçtır.

### TextOverlay

```json
{
  "uid": 3,
  "text": "Merhaba AptozaQ3",
  "x": 100,
  "y": 100,
  "size": 32,
  "color": "#ffffff",
  "start_time": 0.0,
  "end_time": 5.0
}
```

- `x`, `y`: 1920×1080 referans koordinat sisteminde piksel.
- `size`: 8–200.
- `color`: `#rrggbb` biçimi.
- `text`: en fazla 500 karakter; FFmpeg için kaçışlanır.
- `start_time` / `end_time`: 0–36000 saniye; `end_time > start_time`.

### Asset

```json
{
  "uid": 4,
  "kind": "Rect",
  "path": "",
  "x_px": 100,
  "y_px": 100,
  "w_px": 400,
  "h_px": 200,
  "start_time": 0.0,
  "end_time": 5.0,
  "color": "#ff0000",
  "thickness": 3,
  "opacity": 1.0
}
```

`kind` değerleri: `Rect`, `Circle`, `Line`, `Image`.

- `Image` için `path` dolu olmalıdır.
- `x_px` / `y_px`: 0–1920 / 0–1080.
- `w_px` / `h_px`: 2–1920 / 2–1080.
- `thickness`: 1–100.
- `opacity`: 0.0–1.0.

### Transition

```json
{
  "effect": "fade",
  "duration": 0.5
}
```

Geçerli `effect` değerleri:

`cut`, `jumpcut`, `fade`, `dissolve`, `wipeleft`, `zoomin`, `fadewhite`,
`pixelize`, `diagtl`, `radial`, `hlslice`, `fadeblack`, `smoothleft`,
`circlecrop`, `slideleft`.

`transitions` dizisindeki her öğe `[önceki_klip_uid, sonraki_klip_uid, geçiş]`
biçimindedir. Geçişin uygulanması için iki klip timeline'da ardışık olmalıdır.

### Tam örnek

```json
{
  "version": 1,
  "clips": [
    {
      "uid": 1,
      "path": "C:/Videos/sahne1.mp4",
      "name": "sahne1.mp4",
      "duration": 12.4,
      "trim_start": 0.0,
      "trim_end": 10.0,
      "speed": 1.0,
      "scale_w": 0,
      "scale_h": 0,
      "timeline_start": 0.0,
      "mute": false,
      "effects": {
        "brightness": 0.02,
        "contrast": 1.05,
        "saturation": 1.1,
        "gamma": 1.0,
        "hue": 0.0,
        "blur": 0.0,
        "bw": false,
        "sepia": false,
        "red_filter": false,
        "blue_filter": false,
        "vignette": true,
        "sharpen": true,
        "invert": false,
        "mirror": false,
        "vflip": false,
        "rotate": false,
        "fade_in": 0.5,
        "fade_out": 0.5
      }
    },
    {
      "uid": 2,
      "path": "C:/Videos/sahne2.mp4",
      "name": "sahne2.mp4",
      "duration": 8.0,
      "trim_start": 1.0,
      "trim_end": 7.0,
      "speed": 1.25,
      "scale_w": 0,
      "scale_h": 0,
      "timeline_start": 10.0,
      "mute": false,
      "effects": {
        "brightness": 0.0,
        "contrast": 1.0,
        "saturation": 1.0,
        "gamma": 1.0,
        "hue": 0.0,
        "blur": 0.0,
        "bw": false,
        "sepia": false,
        "red_filter": false,
        "blue_filter": false,
        "vignette": false,
        "sharpen": false,
        "invert": false,
        "mirror": false,
        "vflip": false,
        "rotate": false,
        "fade_in": 0.0,
        "fade_out": 0.0
      }
    }
  ],
  "audios": [
    {
      "uid": 3,
      "path": "C:/Audio/muzik.mp3",
      "name": "muzik.mp3",
      "duration": 180.0,
      "trim_start": 0.0,
      "trim_end": 30.0,
      "start_time": 0.0,
      "volume": 0.8
    }
  ],
  "texts": [
    {
      "uid": 4,
      "text": "AptozaQ3",
      "x": 80,
      "y": 900,
      "size": 48,
      "color": "#ffffff",
      "start_time": 0.5,
      "end_time": 5.0
    }
  ],
  "assets": [],
  "transitions": [
    [1, 2, { "effect": "fade", "duration": 0.6 }]
  ],
  "export_res": "1920x1080",
  "px_per_sec": 60.0,
  "playhead": 0.0,
  "transition_sel": 1,
  "transition_dur": 0.5,
  "next_uid": 5,
  "source_path": null
}
```

## 2. Ayar dosyası

Konum: `~/.ekran_kayit_pro.json`

```json
{
  "recorder": {
    "screen_on": true,
    "webcam_on": false,
    "audio_on": true,
    "audio_rate": 44100,
    "fps": 30,
    "quality": 18,
    "pip_enabled": true,
    "pip_pos": "bottom-right",
    "pip_size": [240, 135],
    "cam_idx": 0,
    "cam_name": "",
    "webcam_res": [1280, 720],
    "audio_device": "",
    "cursor_hl": false,
    "cursor_color": "#ff3333",
    "cursor_sz": 22,
    "wm_enabled": false,
    "wm_text": "",
    "wm_image": null,
    "wm_pos": "bottom-right",
    "wm_opacity": 0.6,
    "region": null,
    "output_dir": "C:/Users/Kullanici/Videos",
    "zoom": 1.0
  },
  "shortcuts": {
    "start": "F9",
    "pause": "F10",
    "stop": "F11",
    "screenshot": "F8",
    "draw": "F7"
  },
  "screenshot_format": "png",
  "dark": true
}
```

### Ayar doğrulama sınırları

| Alan | Aralık / kural |
|---|---|
| `fps` | 1–240 |
| `audio_rate` | 8000–192000 |
| `quality` | ≤ 51 (CRF) |
| `webcam_res` | 2–16384 |
| `pip_size` | 16–16384 |
| `cursor_sz` | 1–1024 |
| `wm_opacity` | 0.0–1.0 |
| `zoom` | 1.0–4.0 |
| `wm_pos` / `pip_pos` | `top-left`, `top-right`, `bottom-left`, `bottom-right`, `center` |
| `region` | genişlik/yükseklik > 20 ve ≤ 16384 |
| `screenshot_format` | `png`, `jpg`, `jpeg`, `bmp`, `webp`, `tiff`, `tif` |
| `output_dir` | Boş veya mevcut olmayan yol varsayılana döner |

Elle düzenlenen dosyalarda bilinmeyen alanlar yok sayılır; geçersiz değerler
`sanitize()` ile düzeltilir.

## 3. Otomatik kurtarma dosyası

Konum: `~/.aptoza_timeline_autosave.aptproj`

Bu dosya normal proje formatındadır. Ek olarak `source_path` alanı, otomatik
kaydın hangi `.aptproj` dosyasına ait olduğunu tutar. Uygulama açılışta bu
dosyayı algılarsa kurtarma istemi gösterir.

## 4. Uyumluluk notları

- Medya yolları mutlaktır; proje dosyasını başka bilgisayara taşırken medyayı
  da taşımanız veya yolları düzenlemeniz gerekir.
- `version` değeri ileride artırılırsa eski sürümler yeni dosyaları
  açamayabilir.
- UID'ler proje içinde benzersiz olmalıdır. Yinelenen UID'ler seçim ve dışa
  aktarma hatalarına yol açabilir.
- Proje dosyasını elle düzenlerken JSON sözdizimini ve aralıkları koruyun.
- Bilinmeyen geçiş efektleri yükleme sırasında atılır.
