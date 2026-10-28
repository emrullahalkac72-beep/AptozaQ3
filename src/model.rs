//! Ayarlar ve kayıt yapılandırması.
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::util;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Region {
    pub fn valid(&self) -> bool {
        self.w > 20 && self.h > 20
    }
    pub fn label(&self) -> String {
        format!("{},{}  {}×{} px", self.x, self.y, self.w, self.h)
    }
    pub fn tuple(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.w, self.h)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RecorderConfig {
    pub screen_on: bool,
    pub webcam_on: bool,
    pub audio_on: bool,
    pub audio_rate: u32,
    pub fps: u32,
    pub quality: u32,
    pub pip_enabled: bool,
    pub pip_pos: String,
    pub pip_size: (u32, u32),
    pub cam_idx: usize,
    pub cam_name: String,
    pub webcam_res: (u32, u32),
    pub audio_device: String,
    pub cursor_hl: bool,
    pub cursor_color: String,
    pub cursor_sz: i32,
    pub wm_enabled: bool,
    pub wm_text: String,
    pub wm_image: Option<String>,
    pub wm_pos: String,
    pub wm_opacity: f32,
    pub region: Option<Region>,
    pub output_dir: String,
    pub zoom: f32,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            screen_on: true,
            webcam_on: false,
            audio_on: true,
            audio_rate: 44100,
            fps: 30,
            quality: 18,
            pip_enabled: true,
            pip_pos: "bottom-right".into(),
            pip_size: (240, 135),
            cam_idx: 0,
            cam_name: String::new(),
            webcam_res: (1280, 720),
            audio_device: String::new(),
            cursor_hl: false,
            cursor_color: "#ff3333".into(),
            cursor_sz: 22,
            wm_enabled: false,
            wm_text: String::new(),
            wm_image: None,
            wm_pos: "bottom-right".into(),
            wm_opacity: 0.6,
            region: None,
            output_dir: util::default_output_dir().to_string_lossy().to_string(),
            zoom: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Shortcuts {
    pub start: String,
    pub pause: String,
    pub stop: String,
    pub screenshot: String,
    pub draw: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            start: "F9".into(),
            pause: "F10".into(),
            stop: "F11".into(),
            screenshot: "F8".into(),
            draw: "F7".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub recorder: RecorderConfig,
    pub shortcuts: Shortcuts,
    pub screenshot_format: String,
    pub dark: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            recorder: RecorderConfig::default(),
            shortcuts: Shortcuts::default(),
            screenshot_format: "png".into(),
            dark: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = util::settings_path();
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(mut s) = serde_json::from_str::<Settings>(&text) {
                s.sanitize();
                return s;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        util::write_atomic(&util::settings_path(), text.as_bytes())
    }

    /// Bozuk/elle düzenlenmiş ayar dosyalarının uygulamayı kilitlemesini önler.
    pub fn sanitize(&mut self) {
        let r = &mut self.recorder;
        r.fps = r.fps.clamp(1, 240);
        r.audio_rate = r.audio_rate.clamp(8_000, 192_000);
        r.quality = r.quality.min(51);
        r.webcam_res.0 = r.webcam_res.0.clamp(2, 16_384);
        r.webcam_res.1 = r.webcam_res.1.clamp(2, 16_384);
        r.pip_size.0 = r.pip_size.0.clamp(16, 16_384);
        r.pip_size.1 = r.pip_size.1.clamp(16, 16_384);
        r.cursor_sz = r.cursor_sz.clamp(1, 1_024);
        r.wm_opacity = if r.wm_opacity.is_finite() {
            r.wm_opacity.clamp(0.0, 1.0)
        } else {
            0.6
        };
        r.zoom = if r.zoom.is_finite() {
            r.zoom.clamp(1.0, 4.0)
        } else {
            1.0
        };
        r.wm_text = util::clean_text(&r.wm_text, 200);
        if !POSITIONS.contains(&r.wm_pos.as_str()) {
            r.wm_pos = "bottom-right".into();
        }
        if !PIP_POSITIONS.contains(&r.pip_pos.as_str()) {
            r.pip_pos = "bottom-right".into();
        }
        if let Some(path) = &r.wm_image {
            if path.trim().is_empty() {
                r.wm_image = None;
            }
        }
        if r.region
            .map(|reg| !reg.valid() || reg.w > 16_384 || reg.h > 16_384)
            .unwrap_or(false)
        {
            r.region = None;
        }
        if r.output_dir.trim().is_empty() || !Path::new(&r.output_dir).exists() {
            r.output_dir = util::default_output_dir().to_string_lossy().to_string();
        }
        if !matches!(
            self.screenshot_format.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "bmp" | "webp" | "tiff" | "tif"
        ) {
            self.screenshot_format = "png".into();
        } else {
            self.screenshot_format = self.screenshot_format.to_ascii_lowercase();
        }
    }
}

pub fn quality_label(crf: u32) -> &'static str {
    match crf {
        16 => "Çok Yüksek (CRF 16)",
        18 => "Yüksek (CRF 18)",
        23 => "Orta (CRF 23)",
        28 => "Düşük (CRF 28)",
        _ => "Yüksek (CRF 18)",
    }
}

pub const QUALITIES: [(u32, &str); 4] = [
    (16, "Çok Yüksek (CRF 16)"),
    (18, "Yüksek (CRF 18)"),
    (23, "Orta (CRF 23)"),
    (28, "Düşük (CRF 28)"),
];

pub const FPS_CHOICES: [u32; 5] = [10, 15, 24, 30, 60];

pub const PIP_SIZES: [(u32, u32, &str); 4] = [
    (160, 90, "Küçük (160×90)"),
    (240, 135, "Orta (240×135)"),
    (320, 180, "Büyük (320×180)"),
    (480, 270, "Tam (480×270)"),
];

pub const AUDIO_RATES: [u32; 3] = [22050, 44100, 48000];

pub const POSITIONS: [&str; 5] = [
    "top-left",
    "top-right",
    "bottom-left",
    "bottom-right",
    "center",
];
pub const PIP_POSITIONS: [&str; 4] = ["bottom-right", "bottom-left", "top-right", "top-left"];
