//! Timeline veri modeli, klip efektleri, geçişler, varlıklar ve dışa aktarma.
use crate::util;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

pub const REF_W: u32 = 1920;
pub const REF_H: u32 = 1080;

pub type Uid = u64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipEffects {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub gamma: f32,
    pub hue: f32,
    pub blur: f32,
    pub bw: bool,
    pub sepia: bool,
    pub red_filter: bool,
    pub blue_filter: bool,
    pub vignette: bool,
    pub sharpen: bool,
    pub invert: bool,
    pub mirror: bool,
    pub vflip: bool,
    pub rotate: bool,
    pub fade_in: f32,
    pub fade_out: f32,
}

impl Default for ClipEffects {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            hue: 0.0,
            blur: 0.0,
            bw: false,
            sepia: false,
            red_filter: false,
            blue_filter: false,
            vignette: false,
            sharpen: false,
            invert: false,
            mirror: false,
            vflip: false,
            rotate: false,
            fade_in: 0.0,
            fade_out: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub uid: Uid,
    pub path: String,
    pub name: String,
    pub duration: f64,
    pub trim_start: f64,
    pub trim_end: f64,
    pub speed: f32,
    pub scale_w: u32,
    pub scale_h: u32,
    pub timeline_start: f64,
    pub mute: bool,
    pub effects: ClipEffects,
}

impl Clip {
    pub fn new(uid: Uid, path: String) -> Self {
        let duration = util::duration(&path);
        let name = util::basename(&path);
        Self {
            uid,
            path,
            name,
            duration,
            trim_start: 0.0,
            trim_end: if duration > 0.05 { duration } else { 0.05 },
            speed: 1.0,
            scale_w: 0,
            scale_h: 0,
            timeline_start: 0.0,
            mute: false,
            effects: ClipEffects::default(),
        }
    }
    pub fn clip_duration(&self) -> f64 {
        ((self.trim_end - self.trim_start) / self.speed.max(0.01) as f64).max(0.01)
    }
    pub fn timeline_end(&self) -> f64 {
        self.timeline_start + self.clip_duration()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextOverlay {
    pub uid: Uid,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub size: u32,
    pub color: String,
    pub start_time: f64,
    pub end_time: f64,
}

impl TextOverlay {
    pub fn new(uid: Uid) -> Self {
        Self {
            uid,
            text: "Metin".into(),
            x: 100,
            y: 100,
            size: 32,
            color: "#ffffff".into(),
            start_time: 0.0,
            end_time: 5.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioTrack {
    pub uid: Uid,
    pub path: String,
    pub name: String,
    pub duration: f64,
    pub trim_start: f64,
    pub trim_end: f64,
    pub start_time: f64,
    pub volume: f32,
}

impl AudioTrack {
    pub fn new(uid: Uid, path: String) -> Self {
        let duration = util::duration(&path);
        let name = util::basename(&path);
        Self {
            uid,
            path,
            name,
            duration,
            trim_start: 0.0,
            trim_end: if duration > 0.05 { duration } else { 0.05 },
            start_time: 0.0,
            volume: 1.0,
        }
    }
    pub fn clip_duration(&self) -> f64 {
        (self.trim_end - self.trim_start).max(0.01)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetKind {
    Rect,
    Circle,
    Line,
    Image,
}

impl AssetKind {
    pub fn label(self) -> &'static str {
        match self {
            AssetKind::Rect => "⬜ Dikdörtgen",
            AssetKind::Circle => "⭕ Daire",
            AssetKind::Line => "📏 Çizgi",
            AssetKind::Image => "🎨 Görsel",
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            AssetKind::Rect => "rect",
            AssetKind::Circle => "circle",
            AssetKind::Line => "line",
            AssetKind::Image => "image",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub uid: Uid,
    pub kind: AssetKind,
    pub path: String,
    pub x_px: i32,
    pub y_px: i32,
    pub w_px: i32,
    pub h_px: i32,
    pub start_time: f64,
    pub end_time: f64,
    pub color: String,
    pub thickness: i32,
    pub opacity: f32,
}

impl Asset {
    pub fn name(&self) -> String {
        match self.kind {
            AssetKind::Image => format!("🎨 {}", util::shorten(&util::basename(&self.path), 18)),
            AssetKind::Rect => format!("⬜ RECT {},{}", self.x_px, self.y_px),
            AssetKind::Circle => format!("⭕ CIRCLE {},{}", self.x_px, self.y_px),
            AssetKind::Line => format!("📏 LINE {},{}", self.x_px, self.y_px),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub effect: String,
    pub duration: f64,
}

impl Transition {
    pub fn cut() -> Self {
        Self {
            effect: "cut".into(),
            duration: 0.5,
        }
    }
    pub fn is_cut(&self) -> bool {
        self.effect == "cut" || self.effect == "jumpcut"
    }
    pub fn display(&self) -> String {
        let (name, em) = display_for(&self.effect);
        format!("{em} {name}")
    }
}

pub const TRANSITIONS: [(&str, &str, &str, &str); 15] = [
    ("Cut", "cut", "Doğrudan kesim – geçiş yok", "✂"),
    ("Fade", "fade", "Yavaş karart / aç", "🌅"),
    ("Dissolve", "dissolve", "Çözülerek geçiş", "💧"),
    ("Wipe", "wipeleft", "Soldaki ekrandan sil", "🌊"),
    ("Zoom", "zoomin", "Yakınlaşarak geçiş", "🔍"),
    ("Jump Cut", "jumpcut", "Anlık zıplama (cut özel)", "⚡"),
    ("Light Leak", "fadewhite", "Parlak ışık sızması", "💡"),
    ("Glitch", "pixelize", "Piksel bozulma efekti", "📺"),
    ("Paper Tear", "diagtl", "Köşeden yırtılma", "📄"),
    ("Wormhole", "radial", "Dairesel girdap", "🌀"),
    ("Warp Zoom", "hlslice", "Yatay dilim / büküm", "🌪"),
    ("Flash", "fadeblack", "Siyah flaş", "⚫"),
    ("Cross Dissolve", "smoothleft", "Yumuşak çapraz geçiş", "🔄"),
    ("Match Cut", "circlecrop", "Daire içi eşleşme", "🎯"),
    ("Push", "slideleft", "Yeni sahneyi itiyor", "➡"),
];

pub fn display_for(fx: &str) -> (&'static str, &'static str) {
    for (name, f, _d, em) in TRANSITIONS {
        if f == fx {
            return (name, em);
        }
    }
    ("Cut", "✂")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    None,
    Clip(Uid),
    Audio(Uid),
    Text(Uid),
    Asset(Uid),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragMode {
    Move,
    TrimL,
    TrimR,
}

#[derive(Clone, Copy, Debug)]
pub struct TimelineDrag {
    pub kind: u8, // 0 video, 1 ses, 2 metin, 3 şekil
    pub uid: Uid,
    pub mode: DragMode,
    pub offset: f32,
}

pub struct AssetEditorState {
    pub editing: Option<Uid>,
    pub kind: AssetKind,
    pub path: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub start: f32,
    pub end: f32,
    pub color: String,
    pub thickness: i32,
    pub opacity: f32,
    pub error: String,
}

impl AssetEditorState {
    pub fn new() -> Self {
        Self {
            editing: None,
            kind: AssetKind::Rect,
            path: String::new(),
            x: 100,
            y: 100,
            w: 400,
            h: 200,
            start: 0.0,
            end: 5.0,
            color: "#ff0000".into(),
            thickness: 3,
            opacity: 1.0,
            error: String::new(),
        }
    }
    pub fn from_asset(a: &Asset) -> Self {
        Self {
            editing: Some(a.uid),
            kind: a.kind,
            path: a.path.clone(),
            x: a.x_px,
            y: a.y_px,
            w: a.w_px,
            h: a.h_px,
            start: a.start_time as f32,
            end: a.end_time as f32,
            color: a.color.clone(),
            thickness: a.thickness,
            opacity: a.opacity,
            error: String::new(),
        }
    }
}

pub struct TextEditorState {
    pub editing: Option<Uid>,
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub size: u32,
    pub color: String,
    pub start: f32,
    pub end: f32,
}

impl TextEditorState {
    pub fn new() -> Self {
        Self {
            editing: None,
            text: "Metin".into(),
            x: 100,
            y: 100,
            size: 32,
            color: "#ffffff".into(),
            start: 0.0,
            end: 5.0,
        }
    }
    pub fn from_overlay(o: &TextOverlay) -> Self {
        Self {
            editing: Some(o.uid),
            text: o.text.clone(),
            x: o.x,
            y: o.y,
            size: o.size,
            color: o.color.clone(),
            start: o.start_time as f32,
            end: o.end_time as f32,
        }
    }
}

pub struct TimelineState {
    pub clips: Vec<Clip>,
    pub audios: Vec<AudioTrack>,
    pub texts: Vec<TextOverlay>,
    pub assets: Vec<Asset>,
    pub transitions: HashMap<(Uid, Uid), Transition>,
    pub thumbs: HashMap<Uid, egui::TextureHandle>,
    pub selected: Selection,
    pub px_per_sec: f32,
    pub scroll: f32,
    pub playhead: f64,
    pub playing: bool,
    pub last_tick: Option<std::time::Instant>,
    pub status: Arc<Mutex<String>>,
    pub exporting: bool,
    pub preview_tex: Option<egui::TextureHandle>,
    pub preview_key: Option<(Uid, i64)>,
    pub effects_open: bool,
    pub effects_uid: Option<Uid>,
    pub effects_tex: Option<egui::TextureHandle>,
    pub effects_key: Option<(Uid, u64)>,
    pub asset_editor: Option<AssetEditorState>,
    pub text_editor: Option<TextEditorState>,
    pub transition_pair: Option<(Uid, Uid)>,
    pub transition_sel: usize,
    pub transition_dur: f32,
    pub export_res: String,
    /// Açık proje dosyası (kaydet için).
    pub project_path: Option<String>,
    /// Son kaydedilen/açılan durumun özeti (kaydedilmemiş değişiklik tespiti için).
    pub saved_snapshot: String,
    /// En son otomatik kaydedilen durumun özeti.
    pub autosaved_snapshot: String,
    /// Kurtarma dosyasından okunan özgün proje yolu.
    pub recovered_source: Option<String>,
    /// İçerik değişikliklerinin geri alma / yineleme yığınları.
    pub undo_stack: Vec<Project>,
    pub redo_stack: Vec<Project>,
    history_last: Option<Project>,
    /// Değişiklik başlamadan önceki durum; kısa aralıklı düzenlemeler
    /// tek bir geri alma adımında birleştirilir.
    history_pending: Option<Project>,
    history_last_at: Option<std::time::Instant>,
    next_uid: Uid,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            clips: Vec::new(),
            audios: Vec::new(),
            texts: Vec::new(),
            assets: Vec::new(),
            transitions: HashMap::new(),
            thumbs: HashMap::new(),
            selected: Selection::None,
            px_per_sec: 60.0,
            scroll: 0.0,
            playhead: 0.0,
            playing: false,
            last_tick: None,
            status: Arc::new(Mutex::new(String::new())),
            exporting: false,
            preview_tex: None,
            preview_key: None,
            effects_open: false,
            effects_uid: None,
            effects_tex: None,
            effects_key: None,
            asset_editor: None,
            text_editor: None,
            transition_pair: None,
            transition_sel: 0,
            transition_dur: 0.5,
            export_res: "Orijinal".into(),
            project_path: None,
            saved_snapshot: String::new(),
            autosaved_snapshot: String::new(),
            recovered_source: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            history_last: None,
            history_pending: None,
            history_last_at: None,
            next_uid: 1,
        }
    }
}

/// Diske kaydedilen timeline projesi (.aptproj, JSON).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub version: u32,
    pub clips: Vec<Clip>,
    pub audios: Vec<AudioTrack>,
    pub texts: Vec<TextOverlay>,
    pub assets: Vec<Asset>,
    /// (önceki klip, sonraki klip, geçiş)
    pub transitions: Vec<(Uid, Uid, Transition)>,
    pub export_res: String,
    pub px_per_sec: f32,
    pub playhead: f64,
    pub transition_sel: usize,
    pub transition_dur: f32,
    pub next_uid: Uid,
    /// Otomatik kayıtta, kaydın ait olduğu proje dosyası (kurtarmada geri verilir).
    #[serde(default)]
    pub source_path: Option<String>,
}

pub const PROJECT_EXT: &str = "aptproj";

fn finite32(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

fn finite64(v: f64, fallback: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

impl Project {
    /// Proje dosyasından gelen değerleri güvenli aralıklara çeker.
    pub fn sanitize(&mut self) {
        for c in &mut self.clips {
            c.duration = finite64(c.duration, 0.05).clamp(0.05, 864_000.0);
            c.trim_start = finite64(c.trim_start, 0.0).clamp(0.0, c.duration - 0.01);
            c.trim_end = finite64(c.trim_end, c.duration).clamp(c.trim_start + 0.01, c.duration);
            c.speed = finite32(c.speed, 1.0).clamp(0.1, 16.0);
            c.timeline_start = finite64(c.timeline_start, 0.0).clamp(0.0, 864_000.0);
            if c.scale_w == 0 || c.scale_h == 0 {
                c.scale_w = 0;
                c.scale_h = 0;
            } else {
                c.scale_w = c.scale_w.clamp(2, 16_384);
                c.scale_h = c.scale_h.clamp(2, 16_384);
            }
            let e = &mut c.effects;
            e.brightness = finite32(e.brightness, 0.0).clamp(-1.0, 1.0);
            e.contrast = finite32(e.contrast, 1.0).clamp(0.0, 4.0);
            e.saturation = finite32(e.saturation, 1.0).clamp(0.0, 4.0);
            e.gamma = finite32(e.gamma, 1.0).clamp(0.1, 10.0);
            e.hue = finite32(e.hue, 0.0).clamp(-360.0, 360.0);
            e.blur = finite32(e.blur, 0.0).clamp(0.0, 50.0);
            e.fade_in = finite32(e.fade_in, 0.0).clamp(0.0, 3_600.0);
            e.fade_out = finite32(e.fade_out, 0.0).clamp(0.0, 3_600.0);
        }
        for a in &mut self.audios {
            a.duration = finite64(a.duration, 0.05).clamp(0.05, 864_000.0);
            a.trim_start = finite64(a.trim_start, 0.0).clamp(0.0, a.duration - 0.01);
            a.trim_end = finite64(a.trim_end, a.duration).clamp(a.trim_start + 0.01, a.duration);
            a.start_time = finite64(a.start_time, 0.0).clamp(0.0, 864_000.0);
            a.volume = finite32(a.volume, 1.0).clamp(0.0, 4.0);
        }
        for t in &mut self.texts {
            t.text = util::clean_text(&t.text, 500);
            t.x = t.x.clamp(0, REF_W as i32);
            t.y = t.y.clamp(0, REF_H as i32);
            t.size = t.size.clamp(8, 200);
            t.start_time = finite64(t.start_time, 0.0).clamp(0.0, 35_999.0);
            let lo = (t.start_time + 0.01).min(36_000.0);
            t.end_time = finite64(t.end_time, t.start_time + 5.0).clamp(lo, 36_000.0);
        }
        for a in &mut self.assets {
            a.x_px = a.x_px.clamp(0, REF_W as i32);
            a.y_px = a.y_px.clamp(0, REF_H as i32);
            a.w_px = a.w_px.clamp(2, REF_W as i32);
            a.h_px = a.h_px.clamp(2, REF_H as i32);
            a.start_time = finite64(a.start_time, 0.0).clamp(0.0, 35_999.0);
            let lo = (a.start_time + 0.01).min(36_000.0);
            a.end_time = finite64(a.end_time, a.start_time + 5.0).clamp(lo, 36_000.0);
            a.thickness = a.thickness.clamp(1, 100);
            a.opacity = finite32(a.opacity, 1.0).clamp(0.0, 1.0);
        }
        self.transitions.retain(|(_, _, t)| {
            let known = t.effect == "cut"
                || t.effect == "jumpcut"
                || TRANSITIONS.iter().any(|(_, f, _, _)| *f == t.effect);
            known && t.duration.is_finite()
        });
        for (_, _, t) in &mut self.transitions {
            t.duration = finite64(t.duration, 0.5).clamp(0.0, 10.0);
        }
        if !matches!(
            self.export_res.as_str(),
            "Orijinal" | "3840x2160" | "1920x1080" | "1280x720" | "854x480" | "640x360"
        ) {
            self.export_res = "Orijinal".into();
        }
        self.px_per_sec = finite32(self.px_per_sec, 60.0).clamp(5.0, 300.0);
        self.playhead = finite64(self.playhead, 0.0).max(0.0);
        self.transition_dur = finite32(self.transition_dur, 0.5).clamp(0.1, 2.0);
        self.transition_sel = self.transition_sel.min(TRANSITIONS.len() - 1);
        let max_uid = self
            .clips
            .iter()
            .map(|c| c.uid)
            .chain(self.audios.iter().map(|a| a.uid))
            .chain(self.texts.iter().map(|t| t.uid))
            .chain(self.assets.iter().map(|a| a.uid))
            .max()
            .unwrap_or(0);
        self.next_uid = self.next_uid.max(max_uid.saturating_add(1)).max(1);
    }
}

impl TimelineState {
    /// Yalnızca içeriği temsil eden özet (zoom, imleç vb. arayüz durumu hariç).
    pub fn snapshot(&self) -> String {
        let mut p = self.to_project();
        p.playhead = 0.0;
        p.px_per_sec = 0.0;
        p.transition_sel = 0;
        p.transition_dur = 0.0;
        p.next_uid = 0;
        p.source_path = None;
        serde_json::to_string(&p).unwrap_or_default()
    }

    /// Undo/redo karşılaştırmasında oynatma konumu ve zoom gibi geçici
    /// arayüz durumlarını dışarıda bırakan içerik anlık görüntüsü.
    pub fn history_snapshot(&self) -> Project {
        let mut p = self.to_project();
        p.playhead = 0.0;
        p.px_per_sec = 0.0;
        p.transition_sel = 0;
        p.transition_dur = 0.0;
        p.source_path = None;
        p
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// İçerik değişikliklerini kısa aralıklarla tek bir adımda toplar.
    /// UI çizildikten sonra her karede çağrılır.
    pub fn history_tick(&mut self) {
        let current = self.history_snapshot();
        let now = std::time::Instant::now();
        let Some(last) = self.history_last.replace(current.clone()) else {
            self.history_last = Some(current);
            self.history_last_at = Some(now);
            return;
        };
        if last == current {
            // Değişiklik durdu; birleştirilmiş adımı yığına aktar.
            if self.history_pending.is_some()
                && self
                    .history_last_at
                    .map(|t| now.duration_since(t) >= std::time::Duration::from_millis(450))
                    .unwrap_or(true)
            {
                self.commit_pending();
            }
            return;
        }
        // Değişiklik başladı: değişiklik öncesi durumu sakla.
        if self.history_pending.is_none() {
            self.history_pending = Some(last);
        }
        self.history_last_at = Some(now);
    }

    /// Bekleyen (henüz yığına aktarılmamış) değişikliği geri alma yığınına ekler.
    pub fn commit_pending(&mut self) {
        let Some(pending) = self.history_pending.take() else {
            return;
        };
        if self.history_last.as_ref() == Some(&pending) {
            return;
        }
        self.undo_stack.push(pending);
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn reset_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.history_pending = None;
        self.history_last = Some(self.history_snapshot());
        self.history_last_at = Some(std::time::Instant::now());
    }

    fn apply_project_state(&mut self, p: &Project) {
        let max_uid = p
            .clips
            .iter()
            .map(|c| c.uid)
            .chain(p.audios.iter().map(|a| a.uid))
            .chain(p.texts.iter().map(|t| t.uid))
            .chain(p.assets.iter().map(|a| a.uid))
            .max()
            .unwrap_or(0);
        self.clips = p.clips.clone();
        self.audios = p.audios.clone();
        self.texts = p.texts.clone();
        self.assets = p.assets.clone();
        self.transitions = p
            .transitions
            .iter()
            .map(|(a, b, t)| ((*a, *b), t.clone()))
            .collect();
        self.export_res = p.export_res.clone();
        self.next_uid = p.next_uid.max(max_uid.saturating_add(1)).max(1);
        self.selected = Selection::None;
        self.playing = false;
        self.last_tick = None;
        self.preview_tex = None;
        self.preview_key = None;
        self.effects_tex = None;
        self.effects_key = None;
        self.effects_open = false;
        self.effects_uid = None;
        self.asset_editor = None;
        self.text_editor = None;
        self.transition_pair = None;
        self.playhead = self.playhead.clamp(0.0, self.duration());
    }

    pub fn undo(&mut self) -> bool {
        // Henüz birleştirme süresi dolmamış değişikliği de geri alınabilir yap.
        self.commit_pending();
        let Some(prev) = self.undo_stack.pop() else {
            return false;
        };
        self.redo_stack.push(self.history_snapshot());
        if self.redo_stack.len() > 100 {
            self.redo_stack.remove(0);
        }
        self.apply_project_state(&prev);
        self.history_last = Some(self.history_snapshot());
        self.history_last_at = Some(std::time::Instant::now());
        self.history_pending = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        self.commit_pending();
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };
        self.undo_stack.push(self.history_snapshot());
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.apply_project_state(&next);
        self.history_last = Some(self.history_snapshot());
        self.history_last_at = Some(std::time::Instant::now());
        self.history_pending = None;
        true
    }

    pub fn new_project(&mut self) {
        self.clips.clear();
        self.audios.clear();
        self.texts.clear();
        self.assets.clear();
        self.transitions.clear();
        self.thumbs.clear();
        self.selected = Selection::None;
        self.playhead = 0.0;
        self.scroll = 0.0;
        self.playing = false;
        self.last_tick = None;
        self.preview_tex = None;
        self.preview_key = None;
        self.effects_open = false;
        self.effects_uid = None;
        self.effects_tex = None;
        self.effects_key = None;
        self.asset_editor = None;
        self.text_editor = None;
        self.transition_pair = None;
        self.project_path = None;
        self.recovered_source = None;
        self.next_uid = 1;
        self.mark_saved();
        self.reset_history();
    }

    pub fn has_content(&self) -> bool {
        !(self.clips.is_empty()
            && self.audios.is_empty()
            && self.texts.is_empty()
            && self.assets.is_empty())
    }

    /// Kaydedilmemiş değişiklik var mı?
    pub fn is_dirty(&self) -> bool {
        if !self.has_content() && self.saved_snapshot.is_empty() {
            return false;
        }
        self.snapshot() != self.saved_snapshot
    }

    pub fn mark_saved(&mut self) {
        self.saved_snapshot = self.snapshot();
    }

    /// Kurtarma için otomatik kayıt (kullanıcının proje dosyasına dokunmaz).
    pub fn autosave(&mut self) -> Result<()> {
        let snap = self.snapshot();
        if snap == self.autosaved_snapshot {
            return Ok(());
        }
        let json = serde_json::to_string(&self.to_project())?;
        util::write_atomic(&util::autosave_path(), json.as_bytes())?;
        self.autosaved_snapshot = snap;
        Ok(())
    }

    pub fn remove_autosave(&mut self) {
        let _ = fs::remove_file(util::autosave_path());
        self.autosaved_snapshot.clear();
    }

    pub fn to_project(&self) -> Project {
        let mut transitions: Vec<(Uid, Uid, Transition)> = self
            .transitions
            .iter()
            .map(|((a, b), t)| (*a, *b, t.clone()))
            .collect();
        transitions.sort_by_key(|(a, b, _)| (*a, *b));
        Project {
            version: 1,
            clips: self.clips.clone(),
            audios: self.audios.clone(),
            texts: self.texts.clone(),
            assets: self.assets.clone(),
            transitions,
            export_res: self.export_res.clone(),
            px_per_sec: self.px_per_sec,
            playhead: self.playhead,
            transition_sel: self.transition_sel,
            transition_dur: self.transition_dur,
            next_uid: self.next_uid,
            source_path: self.project_path.clone(),
        }
    }

    /// Projeyi JSON olarak yazar (önce geçici dosyaya, sonra yeniden adlandırarak).
    pub fn save_project(&mut self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.to_project())?;
        util::write_atomic(std::path::Path::new(path), json.as_bytes())?;
        self.project_path = Some(path.to_string());
        self.mark_saved();
        Ok(())
    }

    /// Proje dosyasını okur ve timeline durumunu tamamen onunla değiştirir.
    /// Dönüş: eksik (diskte bulunamayan) medya dosyalarının listesi.
    pub fn load_project(&mut self, path: &str) -> Result<Vec<String>> {
        let text = fs::read_to_string(path)?;
        let mut p: Project =
            serde_json::from_str(&text).map_err(|e| anyhow!("Proje dosyası okunamadı: {e}"))?;
        p.sanitize();
        let mut missing: Vec<String> = Vec::new();
        for c in &p.clips {
            if !std::path::Path::new(&c.path).exists() {
                missing.push(c.path.clone());
            }
        }
        for a in &p.audios {
            if !std::path::Path::new(&a.path).exists() {
                missing.push(a.path.clone());
            }
        }
        for a in &p.assets {
            if a.kind == AssetKind::Image
                && !a.path.is_empty()
                && !std::path::Path::new(&a.path).exists()
            {
                missing.push(a.path.clone());
            }
        }
        missing.sort();
        missing.dedup();

        self.apply_project_state(&p);
        self.px_per_sec = p.px_per_sec;
        self.playhead = p.playhead.min(self.duration());
        self.transition_sel = p.transition_sel;
        self.transition_dur = p.transition_dur;
        // geçici arayüz durumunu sıfırla
        self.thumbs.clear();
        self.scroll = 0.0;
        self.recovered_source = p.source_path;
        self.project_path = Some(path.to_string());
        self.mark_saved();
        self.reset_history();
        Ok(missing)
    }
}

impl TimelineState {
    fn uid(&mut self) -> Uid {
        let max_used = self
            .clips
            .iter()
            .map(|c| c.uid)
            .chain(self.audios.iter().map(|a| a.uid))
            .chain(self.texts.iter().map(|t| t.uid))
            .chain(self.assets.iter().map(|a| a.uid))
            .max()
            .unwrap_or(0);
        let uid = self.next_uid.max(max_used.saturating_add(1)).max(1);
        self.next_uid = uid.saturating_add(1);
        uid
    }

    pub fn add_clip(&mut self, path: String) -> Uid {
        let uid = self.uid();
        let mut c = Clip::new(uid, path);
        c.timeline_start = self
            .clips
            .iter()
            .map(|c| c.timeline_end())
            .fold(0.0, f64::max);
        self.clips.push(c);
        self.selected = Selection::Clip(uid);
        uid
    }

    pub fn add_audio(&mut self, path: String) -> Uid {
        let uid = self.uid();
        let a = AudioTrack::new(uid, path);
        self.audios.push(a);
        self.selected = Selection::Audio(uid);
        uid
    }

    pub fn add_asset(&mut self, a: Asset) {
        let uid = self.uid();
        let mut a = a;
        a.uid = uid;
        self.assets.push(a);
        self.selected = Selection::Asset(uid);
    }

    pub fn add_text(&mut self, mut ov: TextOverlay) -> Uid {
        let uid = self.uid();
        ov.uid = uid;
        self.texts.push(ov);
        self.selected = Selection::Text(uid);
        uid
    }

    pub fn save_asset(&mut self, mut a: Asset) {
        let old_uid = a.uid;
        if let Some(pos) = self.assets.iter().position(|x| x.uid == old_uid) {
            self.assets[pos] = a;
            self.selected = Selection::Asset(old_uid);
        } else {
            let uid = self.uid();
            a.uid = uid;
            self.assets.push(a);
            self.selected = Selection::Asset(uid);
        }
    }

    pub fn save_text(&mut self, mut ov: TextOverlay) {
        let old_uid = ov.uid;
        if let Some(pos) = self.texts.iter().position(|x| x.uid == old_uid) {
            self.texts[pos] = ov;
            self.selected = Selection::Text(old_uid);
        } else {
            let uid = self.uid();
            ov.uid = uid;
            self.texts.push(ov);
            self.selected = Selection::Text(uid);
        }
    }

    pub fn selected_clip(&self) -> Option<&Clip> {
        match self.selected {
            Selection::Clip(uid) => self.clips.iter().find(|c| c.uid == uid),
            _ => None,
        }
    }
    pub fn selected_clip_mut(&mut self) -> Option<&mut Clip> {
        match self.selected {
            Selection::Clip(uid) => self.clips.iter_mut().find(|c| c.uid == uid),
            _ => None,
        }
    }
    pub fn selected_audio_mut(&mut self) -> Option<&mut AudioTrack> {
        match self.selected {
            Selection::Audio(uid) => self.audios.iter_mut().find(|a| a.uid == uid),
            _ => None,
        }
    }

    pub fn duration(&self) -> f64 {
        let mut d: f64 = self
            .clips
            .iter()
            .map(|c| c.timeline_end())
            .fold(0.0, f64::max);
        d = d.max(
            self.audios
                .iter()
                .map(|a| a.start_time + a.clip_duration())
                .fold(0.0, f64::max),
        );
        d = d.max(self.texts.iter().map(|t| t.end_time).fold(0.0, f64::max));
        d = d.max(self.assets.iter().map(|a| a.end_time).fold(0.0, f64::max));
        d.max(1.0)
    }

    pub fn sorted_clips(&self) -> Vec<Clip> {
        let mut cs = self.clips.clone();
        cs.sort_by(|a, b| {
            a.timeline_start
                .partial_cmp(&b.timeline_start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        cs
    }

    pub fn sorted_pairs(&self) -> Vec<(Uid, Uid)> {
        let cs = self.sorted_clips();
        cs.windows(2).map(|w| (w[0].uid, w[1].uid)).collect()
    }

    pub fn transition(&self, a: Uid, b: Uid) -> Transition {
        self.transitions
            .get(&(a, b))
            .cloned()
            .unwrap_or_else(Transition::cut)
    }

    pub fn set_transition(&mut self, a: Uid, b: Uid, t: Transition) {
        self.transitions.insert((a, b), t);
    }

    pub fn clip_at(&self, t: f64) -> Option<&Clip> {
        self.clips
            .iter()
            .filter(|c| t >= c.timeline_start && t <= c.timeline_end())
            .min_by(|a, b| {
                a.timeline_start
                    .partial_cmp(&b.timeline_start)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn delete_selected(&mut self) {
        match self.selected {
            Selection::None => {}
            Selection::Clip(uid) => {
                self.clips.retain(|c| c.uid != uid);
                self.transitions.retain(|(a, b), _| *a != uid && *b != uid);
                self.thumbs.remove(&uid);
            }
            Selection::Audio(uid) => self.audios.retain(|a| a.uid != uid),
            Selection::Text(uid) => self.texts.retain(|t| t.uid != uid),
            Selection::Asset(uid) => self.assets.retain(|a| a.uid != uid),
        }
        self.selected = Selection::None;
    }

    pub fn split_selected(&mut self) {
        if let Selection::Clip(uid) = self.selected {
            let Some(pos) = self.clips.iter().position(|c| c.uid == uid) else {
                return;
            };
            let t = self.playhead;
            let (start, end, speed, trim_start) = {
                let c = &self.clips[pos];
                (c.timeline_start, c.timeline_end(), c.speed, c.trim_start)
            };
            if t <= start + 0.01 || t >= end - 0.01 {
                return;
            }
            let offset = (t - start) * speed as f64;
            let mut right = self.clips[pos].clone();
            let new_uid = self.uid();
            right.uid = new_uid;
            right.timeline_start = t;
            right.trim_start = trim_start + offset;
            self.clips[pos].trim_end = trim_start + offset;
            self.clips.push(right);
            self.selected = Selection::Clip(new_uid);
        } else if let Selection::Audio(uid) = self.selected {
            let Some(pos) = self.audios.iter().position(|a| a.uid == uid) else {
                return;
            };
            let t = self.playhead;
            let a = self.audios[pos].clone();
            if t <= a.start_time + 0.01 || t >= a.start_time + a.clip_duration() - 0.01 {
                return;
            }
            let offset = t - a.start_time;
            let mut right = a.clone();
            let new_uid = self.uid();
            right.uid = new_uid;
            right.start_time = t;
            right.trim_start = a.trim_start + offset;
            self.audios[pos].trim_end = a.trim_start + offset;
            self.audios.push(right);
            self.selected = Selection::Audio(new_uid);
        }
    }

    pub fn extract_audio(&mut self, out: &str) -> Result<()> {
        let Some(c) = self.selected_clip() else {
            return Err(anyhow!("Önce bir video seçin."));
        };
        let args = vec![
            "-y".into(),
            "-ss".into(),
            format!("{:.3}", c.trim_start),
            "-t".into(),
            format!("{:.3}", (c.trim_end - c.trim_start).max(0.01)),
            "-i".into(),
            c.path.clone(),
            "-vn".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "192k".into(),
            out.into(),
        ];
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
    }

    pub fn regenerate_thumbs(&mut self, ctx: &egui::Context) {
        let list: Vec<(Uid, String, f64)> = self
            .clips
            .iter()
            .filter(|c| !self.thumbs.contains_key(&c.uid))
            .map(|c| {
                (
                    c.uid,
                    c.path.clone(),
                    ((c.trim_start + c.trim_end) * 0.5)
                        .min((c.duration - 0.05).max(0.0))
                        .max(0.0),
                )
            })
            .collect();
        for (uid, path, t) in list {
            if let Ok(p) = make_thumb(&path, t) {
                if let Ok(img) = image::open(&p) {
                    let img = img.to_rgba8();
                    let ci = egui::ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
                        img.as_raw(),
                    );
                    let tex =
                        ctx.load_texture(format!("tlthumb{uid}"), ci, egui::TextureOptions::LINEAR);
                    self.thumbs.insert(uid, tex);
                }
                let _ = fs::remove_file(p);
            }
        }
    }
}

pub fn make_thumb(path: &str, t: f64) -> Result<std::path::PathBuf> {
    let out = util::unique_temp("jpg");
    let args = vec![
        "-y".into(),
        "-ss".into(),
        format!("{t:.3}"),
        "-i".into(),
        path.into(),
        "-vframes".into(),
        "1".into(),
        "-vf".into(),
        "scale=320:-2".into(),
        out.to_string_lossy().to_string(),
    ];
    util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))?;
    Ok(out)
}

/// Tek kare önizleme (klip efektleri uygulanmış) üretir.
pub fn make_preview(path: &str, t: f64, eff: &ClipEffects, out: &std::path::Path) -> Result<()> {
    let (vf, _, _) = build_clip_vf(eff, None);
    let mut args: Vec<String> = vec![
        "-y".into(),
        "-ss".into(),
        format!("{t:.3}"),
        "-i".into(),
        path.into(),
        "-vframes".into(),
        "1".into(),
    ];
    if !vf.is_empty() {
        args.push("-vf".into());
        args.push(vf.join(","));
    }
    args.push(out.to_string_lossy().to_string());
    util::ffmpeg_ok(&args).map_err(|e| anyhow::anyhow!(e))
}

// ------------------------------------------------ filtre üreticileri
pub fn build_clip_vf(eff: &ClipEffects, dur: Option<f64>) -> (Vec<String>, f32, f32) {
    let mut vf = Vec::new();
    if eff.rotate {
        vf.push("transpose=1".to_string());
    }
    if eff.mirror {
        vf.push("hflip".to_string());
    }
    if eff.vflip {
        vf.push("vflip".to_string());
    }
    if eff.invert {
        vf.push("negate".to_string());
    }
    if eff.brightness != 0.0 || eff.contrast != 1.0 || eff.saturation != 1.0 || eff.gamma != 1.0 {
        vf.push(format!(
            "eq=brightness={}:contrast={}:saturation={}:gamma={}",
            eff.brightness, eff.contrast, eff.saturation, eff.gamma
        ));
    }
    if eff.hue != 0.0 {
        vf.push(format!("hue=h={}", eff.hue));
    }
    if eff.blur > 0.0 {
        vf.push(format!("boxblur={}:1", eff.blur));
    }
    if eff.sepia {
        vf.push(
            "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131:0:0:0:0:1".into(),
        );
    } else if eff.bw {
        vf.push("hue=s=0".into());
    } else if eff.red_filter {
        vf.push("colorchannelmixer=1:0:0:0:0:0:0:0:0:0:0:0:0:0:0:1".into());
    } else if eff.blue_filter {
        vf.push("colorchannelmixer=0:0:0:0:0:0:0:0:0:0:1:0:0:0:0:1".into());
    }
    if eff.vignette {
        vf.push("vignette".into());
    }
    if eff.sharpen {
        vf.push("unsharp=5:5:1.0:5:5:0.0".into());
    }
    let mut fi = 0.0;
    let mut fo = 0.0;
    if let Some(d) = dur {
        if eff.fade_in > 0.0 {
            vf.push(format!("fade=t=in:st=0:d={}", eff.fade_in));
            fi = eff.fade_in;
        }
        if eff.fade_out > 0.0 {
            vf.push(format!(
                "fade=t=out:st={:.3}:d={}",
                (d - eff.fade_out as f64).max(0.0),
                eff.fade_out
            ));
            fo = eff.fade_out;
        }
    }
    (vf, fi, fo)
}

pub fn build_asset_vf(
    assets: &[&Asset],
    clip_start: f64,
    clip_dur: f64,
    vid_w: u32,
    vid_h: u32,
    speed: f64,
) -> Vec<String> {
    let (vid_w, vid_h) = if vid_w == 0 || vid_h == 0 {
        (REF_W, REF_H)
    } else {
        (vid_w, vid_h)
    };
    let speed = speed.max(0.01);
    let src_dur = clip_dur.max(0.01) * speed;
    let mut simple = Vec::new();
    let mut img_data: Vec<(&Asset, f64, f64)> = Vec::new();
    for a in assets {
        // Filtre zinciri hız değişiminden (setpts) önce çalıştığı için
        // zamanlar kaynak saniyelerine çevrilir.
        let s = ((a.start_time - clip_start) * speed).max(0.0);
        let e = ((a.end_time - clip_start) * speed).min(src_dur);
        if e <= s {
            continue;
        }
        let enable = format!("enable='between(t,{s:.3},{e:.3})'");
        let px_x = (a.x_px as f64 * vid_w as f64 / REF_W as f64).round() as i32;
        let px_y = (a.y_px as f64 * vid_h as f64 / REF_H as f64).round() as i32;
        let px_w = ((a.w_px as f64 * vid_w as f64 / REF_W as f64).round() as i32).max(2);
        let px_h = ((a.h_px as f64 * vid_h as f64 / REF_H as f64).round() as i32).max(2);
        match a.kind {
            AssetKind::Rect => {
                let hx = util::ff_hex(&a.color);
                simple.push(format!(
                    "drawbox=x={px_x}:y={px_y}:w={px_w}:h={px_h}:color=0x{hx}:thickness={}:{enable}",
                    a.thickness
                ));
            }
            AssetKind::Circle => {
                let cx = px_x as f64 + px_w as f64 / 2.0;
                let cy = px_y as f64 + px_h as f64 / 2.0;
                let rx = (px_w as f64 / 2.0).max(1.0);
                let ry = (px_h as f64 / 2.0).max(1.0);
                let th = a.thickness.max(2) as f64;
                let (r_c, g_c, b_c) = util::hex_rgb(&a.color);
                let cx_n = cx / vid_w as f64;
                let cy_n = cy / vid_h as f64;
                let rx_n = rx / vid_w as f64;
                let ry_n = ry / vid_h as f64;
                let rx_in = (rx - th).max(0.001) / vid_w as f64;
                let ry_in = (ry - th).max(0.001) / vid_h as f64;
                let dist = format!(
                    "pow((X/W-{cx_n:.6})/{rx_n:.6}\\,2)+pow((Y/H-{cy_n:.6})/{ry_n:.6}\\,2)"
                );
                let dist_in = format!(
                    "pow((X/W-{cx_n:.6})/{rx_in:.6}\\,2)+pow((Y/H-{cy_n:.6})/{ry_in:.6}\\,2)"
                );
                let cond = format!("lte({dist}\\,1.0)*gte({dist_in}\\,1.0)");
                simple.push(format!(
                    "geq=r='if({cond}\\,{r_c}\\,r(X\\,Y))':g='if({cond}\\,{g_c}\\,g(X\\,Y))':b='if({cond}\\,{b_c}\\,b(X\\,Y))':{enable}"
                ));
            }
            AssetKind::Line => {
                let h = ((a.thickness as f64 * vid_h as f64 / REF_H as f64).round() as i32).max(2);
                let hx = util::ff_hex(&a.color);
                simple.push(format!(
                    "drawbox=x={px_x}:y={px_y}:w={px_w}:h={h}:color=0x{hx}:thickness=-1:{enable}"
                ));
            }
            AssetKind::Image => {
                if std::path::Path::new(&a.path).exists() {
                    img_data.push((a, s, e));
                }
            }
        }
    }
    let mut result: Vec<String> = Vec::new();
    if !img_data.is_empty() {
        let mut movie = Vec::new();
        let mut overlay = Vec::new();
        let mut prev = "0:v".to_string();
        for (idx, (a, s, e)) in img_data.iter().enumerate() {
            let safe = util::escape_filter_path(&a.path.replace('\\', "/"));
            let enable = format!("enable='between(t,{s:.3},{e:.3})'");
            let im = format!("im{idx}");
            let last = idx == img_data.len() - 1;
            let out = if last {
                "[vout]".to_string()
            } else {
                format!("[ov{idx}]")
            };
            let ox = (a.x_px as f64 * vid_w as f64 / REF_W as f64).round() as i32;
            let oy = (a.y_px as f64 * vid_h as f64 / REF_H as f64).round() as i32;
            let ow = ((a.w_px as f64 * vid_w as f64 / REF_W as f64).round() as i32).max(2);
            let oh = ((a.h_px as f64 * vid_h as f64 / REF_H as f64).round() as i32).max(2);
            if a.opacity < 0.99 {
                movie.push(format!(
                    "movie='{safe}':loop=0[{im}_raw];[{im}_raw]scale={ow}:{oh},colorchannelmixer=aa={:.3}[{im}]",
                    a.opacity
                ));
            } else {
                movie.push(format!(
                    "movie='{safe}':loop=0[{im}_raw];[{im}_raw]scale={ow}:{oh}[{im}]"
                ));
            }
            overlay.push(format!(
                "[{prev}][{im}]overlay=x={ox}:y={oy}:shortest=0:eof_action=repeat:{enable}{out}"
            ));
            if !last {
                prev = format!("ov{idx}");
            }
        }
        result.push(format!("{}", [movie, overlay].concat().join(";")));
    }
    result.extend(simple);
    result
}

// ------------------------------------------------ dışa aktarma
#[derive(Clone)]
pub struct ExportInput {
    pub clips: Vec<Clip>,
    pub audios: Vec<AudioTrack>,
    pub texts: Vec<TextOverlay>,
    pub assets: Vec<Asset>,
    pub transitions: HashMap<(Uid, Uid), Transition>,
    pub resolution: String,
    pub out: String,
}

impl TimelineState {
    pub fn export_input(&self, resolution: String, out: String) -> ExportInput {
        ExportInput {
            clips: self.clips.clone(),
            audios: self.audios.clone(),
            texts: self.texts.clone(),
            assets: self.assets.clone(),
            transitions: self.transitions.clone(),
            resolution,
            out,
        }
    }
}

pub fn export(inp: ExportInput, status: Arc<Mutex<String>>) {
    let res = export_inner(&inp, &status);
    if let Ok(mut s) = status.lock() {
        match res {
            Ok(path) => *s = format!("✓ Dışa aktarma tamamlandı: {}", util::basename(&path)),
            Err(e) => *s = format!("✗ Export hatası: {}", util::shorten(&e.to_string(), 450)),
        }
    }
}

fn set_status(st: &Arc<Mutex<String>>, s: impl Into<String>) {
    if let Ok(mut g) = st.lock() {
        *g = s.into();
    }
}

fn export_inner(inp: &ExportInput, status: &Arc<Mutex<String>>) -> Result<String> {
    if inp.clips.is_empty() {
        return Err(anyhow!("Timeline'a en az bir video ekleyin."));
    }
    let out = &inp.out;
    let parent = std::path::Path::new(out)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    // Eşzamanlı dışa aktarmalar birbirinin geçici dosyalarını ezmesin.
    let tmp_guard = tempfile::Builder::new()
        .prefix("_aptoza_export_")
        .tempdir_in(&parent)?;
    let tmp_dir = tmp_guard.path().to_path_buf();

    let mut sorted: Vec<Clip> = inp.clips.clone();
    sorted.sort_by(|a, b| {
        a.timeline_start
            .partial_cmp(&b.timeline_start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut processed: Vec<String> = Vec::new();

    for (i, clip) in sorted.iter().enumerate() {
        set_status(
            status,
            format!("⏳ Segment {}/{} hazırlanıyor...", i + 1, sorted.len()),
        );
        let tmp_out = tmp_dir.join(format!("seg_{i:04}.mp4"));
        let mut v_filters: Vec<String> = Vec::new();
        let mut a_filters: Vec<String> = Vec::new();
        let clip_has_audio = util::has_audio(&clip.path);
        let seg_start = clip.timeline_start;
        let seg_end = clip.timeline_end();
        let active_assets: Vec<&Asset> = inp
            .assets
            .iter()
            .filter(|a| a.start_time < seg_end && a.end_time > seg_start)
            .collect();
        if !active_assets.is_empty() {
            // Şekiller klibin doğal çözünürlüğünde çizilir; ardından gelen
            // ölçekleme/pad ile birlikte hedef çözünürlüğe taşınır.
            let (vw, vh) = util::dimensions(&clip.path);
            let asset_vf = build_asset_vf(
                &active_assets,
                seg_start,
                clip.clip_duration(),
                vw,
                vh,
                clip.speed.max(0.01) as f64,
            );
            for part in asset_vf {
                if part.contains(';') || part.contains("movie=") {
                    v_filters.insert(0, format!("__FC__:{part}"));
                } else {
                    v_filters.push(part);
                }
            }
        }
        if (clip.speed - 1.0).abs() > 0.001 {
            v_filters.push(format!(
                "setpts={:.4}*(PTS-STARTPTS)",
                1.0 / clip.speed as f64
            ));
            let spd = clip.speed as f64;
            if (0.5..=2.0).contains(&spd) {
                a_filters.push(format!("atempo={spd:.4}"));
            } else if spd > 2.0 {
                a_filters.push("atempo=2.0".into());
                a_filters.push(format!("atempo={:.4}", spd / 2.0));
            } else {
                a_filters.push("atempo=0.5".into());
                a_filters.push(format!("atempo={:.4}", spd * 2.0));
            }
        } else {
            v_filters.push("setpts=PTS-STARTPTS".into());
        }
        if clip.mute {
            a_filters.push("volume=0".into());
        }
        if inp.resolution != "Orijinal" {
            if let Some((rw, rh)) = inp.resolution.split_once('x') {
                v_filters.push(format!(
                    "scale={rw}:{rh}:force_original_aspect_ratio=decrease:force_divisible_by=2"
                ));
                v_filters.push(format!("pad={rw}:{rh}:(ow-iw)/2:(oh-ih)/2:color=black"));
            }
        } else if clip.scale_w > 0 && clip.scale_h > 0 {
            let sw = (clip.scale_w - (clip.scale_w % 2)).max(2);
            let sh = (clip.scale_h - (clip.scale_h % 2)).max(2);
            v_filters.push(format!("scale={sw}:{sh}"));
        }
        let real_out_dur = clip.clip_duration();
        let (eff_vf, fi_af, fo_af) = build_clip_vf(&clip.effects, Some(real_out_dur));
        v_filters.extend(eff_vf);
        if fi_af > 0.0 {
            a_filters.push(format!("afade=t=in:st=0:d={fi_af}"));
        }
        if fo_af > 0.0 {
            a_filters.push(format!(
                "afade=t=out:st={:.3}:d={fo_af}",
                (real_out_dur - fo_af as f64).max(0.0)
            ));
        }
        let ff_font = util::windows_font();
        let (out_w, out_h) = if inp.resolution != "Orijinal" {
            if let Some((w, h)) = inp.resolution.split_once('x') {
                (
                    w.parse::<u32>().unwrap_or(REF_W),
                    h.parse::<u32>().unwrap_or(REF_H),
                )
            } else {
                (REF_W, REF_H)
            }
        } else if clip.scale_w > 0 && clip.scale_h > 0 {
            (clip.scale_w, clip.scale_h)
        } else {
            util::dimensions(&clip.path)
        };
        // Orijinal çözünürlükte 90° döndürme çıktı boyutlarını yer değiştirir.
        let (out_w, out_h) = if inp.resolution == "Orijinal"
            && clip.scale_w == 0
            && clip.scale_h == 0
            && clip.effects.rotate
        {
            (out_h, out_w)
        } else {
            (out_w, out_h)
        };
        let tsx = if out_w > 0 {
            out_w as f64 / REF_W as f64
        } else {
            1.0
        };
        let tsy = if out_h > 0 {
            out_h as f64 / REF_H as f64
        } else {
            1.0
        };
        for ov in &inp.texts {
            let rel_s = ov.start_time - clip.timeline_start;
            let rel_e = ov.end_time - clip.timeline_start;
            if rel_e > 0.0 && rel_s < real_out_dur {
                let rs = rel_s.max(0.0);
                let re = rel_e.min(real_out_dur);
                let safe = util::escape_drawtext(&ov.text);
                let hx = util::ff_hex(&ov.color);
                let tx = (ov.x as f64 * tsx).round() as i64;
                let ty = (ov.y as f64 * tsy).round() as i64;
                let tsize = ((ov.size as f64 * tsy).round() as i64).max(6);
                v_filters.push(format!(
                    "drawtext=fontfile='{ff_font}':text='{safe}':fontsize={tsize}:fontcolor=0x{hx}:x={tx}:y={ty}:enable='between(t,{rs:.3},{re:.3})'"
                ));
            }
        }

        let mut fc_chains = Vec::new();
        let mut timing_vf = Vec::new();
        let mut geom_vf = Vec::new();
        let mut scale_vf = Vec::new();
        let mut overlay_vf = Vec::new();
        for part in v_filters {
            if let Some(rest) = part.strip_prefix("__FC__:") {
                fc_chains.push(rest.to_string());
            } else if part.starts_with("setpts") {
                timing_vf.push(part);
            } else if part.starts_with("transpose")
                || part.starts_with("hflip")
                || part.starts_with("vflip")
            {
                geom_vf.push(part);
            } else if part.starts_with("scale") || part.starts_with("pad") {
                scale_vf.push(part);
            } else {
                overlay_vf.push(part);
            }
        }
        let mut simple_vf = timing_vf;
        // Döndürme/aynalama, hedef çözünürlüğe ölçeklemeden önce uygulanmalı;
        // aksi halde genişlik/yükseklik yer değiştirir ve birleştirme bozulur.
        simple_vf.extend(geom_vf);
        simple_vf.extend(scale_vf);
        simple_vf.push("format=yuv420p".into());
        simple_vf.extend(overlay_vf);
        let seg_dur = clip.trim_end - clip.trim_start;

        let mut cmd: Vec<String> = vec![
            "-y".into(),
            "-ss".into(),
            format!("{:.3}", clip.trim_start),
            "-t".into(),
            format!("{:.3}", seg_dur),
            "-i".into(),
            clip.path.clone(),
        ];
        if !fc_chains.is_empty() {
            let mut fc = fc_chains.join(";");
            if !simple_vf.is_empty() {
                fc.push_str(&format!(";[vout]{}[vout2]", simple_vf.join(",")));
                cmd.extend([
                    "-filter_complex".into(),
                    fc,
                    "-map".into(),
                    "[vout2]".into(),
                ]);
            } else {
                cmd.extend(["-filter_complex".into(), fc, "-map".into(), "[vout]".into()]);
            }
        } else if !simple_vf.is_empty() {
            cmd.extend(["-vf".into(), simple_vf.join(",")]);
        }
        if clip_has_audio {
            if !a_filters.is_empty() {
                cmd.extend(["-af".into(), a_filters.join(",")]);
            }
            cmd.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "192k".into()]);
        } else {
            cmd.push("-an".into());
        }
        cmd.extend([
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            "fast".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-r".into(),
            "30".into(),
            "-avoid_negative_ts".into(),
            "make_zero".into(),
            "-movflags".into(),
            "+faststart".into(),
            tmp_out.to_string_lossy().to_string(),
        ]);
        let o = util::run("ffmpeg", &cmd).map_err(|e| anyhow!(e))?;
        if !o.status.success() {
            let err = String::from_utf8_lossy(&o.stderr);
            let interesting: Vec<&str> = err
                .lines()
                .filter(|l| {
                    l.contains("Error")
                        || l.contains("error")
                        || l.contains("Invalid")
                        || l.contains("No such")
                        || l.contains("failed")
                })
                .collect();
            let msg = if interesting.is_empty() {
                util::last_error_lines(&err, 8)
            } else {
                util::last_error_lines(&interesting.join("\n"), 6)
            };
            return Err(anyhow!("Segment {} hatası: {}", i + 1, msg));
        }
        processed.push(tmp_out.to_string_lossy().to_string());
    }

    set_status(status, "⚙ Segmentler birleştiriliyor...");
    let mut final_vid = processed[0].clone();
    if processed.len() > 1 {
        let pairs = sorted
            .windows(2)
            .map(|w| (w[0].uid, w[1].uid))
            .collect::<Vec<_>>();
        let mut has_fx = false;
        for (a, b) in &pairs {
            let t = inp
                .transitions
                .get(&(*a, *b))
                .cloned()
                .unwrap_or_else(Transition::cut);
            if !t.is_cut() {
                has_fx = true;
                break;
            }
        }

        // Tüm segmentleri ortak çözünürlük ve ses düzenine getir. Aksi halde
        // concat/copy farklı çözünürlüklerde veya eksik ses akışlarında bozulur.
        let mut segments = processed.clone();
        let (ref_w, ref_h) = util::dimensions(&segments[0]);
        for i in 1..segments.len() {
            let (pw, ph) = util::dimensions(&segments[i]);
            if pw == ref_w && ph == ref_h {
                continue;
            }
            let norm = tmp_dir.join(format!("norm_{i:04}.mp4"));
            let has_audio = util::has_audio(&segments[i]);
            let mut args: Vec<String> = vec![
                "-y".into(),
                "-i".into(),
                segments[i].clone(),
                "-vf".into(),
                format!(
                    "scale={ref_w}:{ref_h}:force_original_aspect_ratio=decrease,pad={ref_w}:{ref_h}:(ow-iw)/2:(oh-ih)/2:color=black"
                ),
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "fast".into(),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-r".into(),
                "30".into(),
            ];
            if has_audio {
                args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "192k".into()]);
            } else {
                args.push("-an".into());
            }
            args.push(norm.to_string_lossy().to_string());
            util::ffmpeg_ok(&args)
                .map_err(|e| anyhow!("Segment {} normalizasyon hatası: {}", i + 1, e))?;
            segments[i] = norm.to_string_lossy().to_string();
        }

        // Geçişler ses miksleme kullanır; sesi olmayan segmentlere sessiz
        // parça eklenir. Düz birleştirmede ise yalnızca bazı segmentlerde ses
        // varsa aynı akış düzenini korumak için eksikler tamamlanır.
        let any_audio = segments.iter().any(|p| util::has_audio(p));
        if has_fx || any_audio {
            for i in 0..segments.len() {
                if util::has_audio(&segments[i]) {
                    continue;
                }
                let sil = tmp_dir.join(format!("sil_{i:04}.mp4"));
                let dur = util::duration(&segments[i]).max(0.1);
                let args: Vec<String> = vec![
                    "-y".into(),
                    "-i".into(),
                    segments[i].clone(),
                    "-f".into(),
                    "lavfi".into(),
                    "-i".into(),
                    format!("aevalsrc=0:c=stereo:s=44100:d={dur:.3}"),
                    "-map".into(),
                    "0:v".into(),
                    "-map".into(),
                    "1:a".into(),
                    "-c:v".into(),
                    "copy".into(),
                    "-c:a".into(),
                    "aac".into(),
                    "-b:a".into(),
                    "128k".into(),
                    "-shortest".into(),
                    sil.to_string_lossy().to_string(),
                ];
                util::ffmpeg_ok(&args)
                    .map_err(|e| anyhow!("Segment {} ses tamamlama hatası: {}", i + 1, e))?;
                segments[i] = sil.to_string_lossy().to_string();
            }
        }

        if !has_fx {
            let lst = tmp_dir.join("list.txt");
            let mut text = String::new();
            for p in &segments {
                text.push_str(&format!(
                    "file '{}'\n",
                    p.replace('\\', "/").replace('\'', "'\\''")
                ));
            }
            fs::write(&lst, text)?;
            let merged = tmp_dir.join("merged.mp4");
            let args: Vec<String> = vec![
                "-y".into(),
                "-f".into(),
                "concat".into(),
                "-safe".into(),
                "0".into(),
                "-i".into(),
                lst.to_string_lossy().to_string(),
                "-c".into(),
                "copy".into(),
                merged.to_string_lossy().to_string(),
            ];
            util::ffmpeg_ok(&args).map_err(|e| anyhow!("Birleştirme hatası: {}", e))?;
            final_vid = merged.to_string_lossy().to_string();
        } else {
            set_status(status, "⚙ Geçiş efektleri uygulanıyor...");
            let seg_durs: Vec<f64> = segments
                .iter()
                .map(|p| util::duration(p).max(0.1))
                .collect();
            let audio_ready = segments;
            let mut offsets = vec![0.0f64];
            let mut cum = seg_durs[0];
            for (i, (a, b)) in pairs.iter().enumerate() {
                let t = inp
                    .transitions
                    .get(&(*a, *b))
                    .cloned()
                    .unwrap_or_else(Transition::cut);
                let mut tdur = if t.is_cut() { 0.0 } else { t.duration };
                tdur = tdur
                    .min(seg_durs[i] * 0.85)
                    .min(seg_durs[i + 1] * 0.85)
                    .max(0.0);
                let off = (cum - tdur).max(0.0);
                offsets.push(off);
                cum = off + seg_durs[i + 1];
            }
            let mut v_parts = Vec::new();
            let mut a_parts = Vec::new();
            let mut prev = "[0:v]".to_string();
            let mut cum2 = seg_durs[0];
            for (i, (a, b)) in pairs.iter().enumerate() {
                let t = inp
                    .transitions
                    .get(&(*a, *b))
                    .cloned()
                    .unwrap_or_else(Transition::cut);
                let mut tdur = if t.is_cut() { 0.0 } else { t.duration };
                tdur = tdur
                    .min(seg_durs[i] * 0.85)
                    .min(seg_durs[i + 1] * 0.85)
                    .max(0.0);
                let off = (cum2 - tdur).max(0.0);
                let is_last = i == pairs.len() - 1;
                let v_out = if is_last {
                    "[vout]".to_string()
                } else {
                    format!("[xv{i}]")
                };
                let next = format!("[{}:v]", i + 1);
                if tdur > 0.0 {
                    v_parts.push(format!(
                        "{prev}{next}xfade=transition={}:duration={tdur:.3}:offset={off:.3}{v_out}",
                        t.effect
                    ));
                } else {
                    v_parts.push(format!(
                        "{prev}{next}xfade=transition=fade:duration=0.001:offset={off:.3}{v_out}"
                    ));
                }
                prev = v_out;
                cum2 = off + seg_durs[i + 1];
            }
            for si in 0..audio_ready.len() {
                let delay = (offsets[si] * 1000.0) as i64;
                a_parts.push(format!(
                    "[{si}:a]asetpts=PTS-STARTPTS,adelay={delay}|{delay}[da{si}]"
                ));
            }
            let mix_ins: String = (0..audio_ready.len()).map(|i| format!("[da{i}]")).collect();
            a_parts.push(format!(
                "{mix_ins}amix=inputs={}:duration=longest:normalize=0[aout]",
                audio_ready.len()
            ));
            let fc = format!("{};{}", v_parts.join(";"), a_parts.join(";"));
            let merged = tmp_dir.join("merged.mp4");
            let mut args: Vec<String> = vec!["-y".into()];
            for p in &audio_ready {
                args.extend(["-i".into(), p.clone()]);
            }
            args.extend([
                "-filter_complex".into(),
                fc,
                "-map".into(),
                "[vout]".into(),
                "-map".into(),
                "[aout]".into(),
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "fast".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "192k".into(),
                "-movflags".into(),
                "+faststart".into(),
                merged.to_string_lossy().to_string(),
            ]);
            if util::ffmpeg_ok(&args).is_ok() {
                final_vid = merged.to_string_lossy().to_string();
            } else {
                set_status(status, "⚠ xfade başarısız, concat ile deneniyor...");
                let lst = tmp_dir.join("list2.txt");
                let mut text = String::new();
                for p in &audio_ready {
                    text.push_str(&format!(
                        "file '{}'\n",
                        p.replace('\\', "/").replace('\'', "'\\''")
                    ));
                }
                fs::write(&lst, text)?;
                let args2: Vec<String> = vec![
                    "-y".into(),
                    "-f".into(),
                    "concat".into(),
                    "-safe".into(),
                    "0".into(),
                    "-i".into(),
                    lst.to_string_lossy().to_string(),
                    "-c".into(),
                    "copy".into(),
                    merged.to_string_lossy().to_string(),
                ];
                util::ffmpeg_ok(&args2).map_err(|e| anyhow!("Birleştirme hatası: {}", e))?;
                final_vid = merged.to_string_lossy().to_string();
            }
        }
    }

    // bağımsız ses katmanları
    if !inp.audios.is_empty() && std::path::Path::new(&final_vid).exists() {
        set_status(status, "🎧 Ses katmanları karıştırılıyor...");
        let main_has_audio = util::has_audio(&final_vid);
        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), final_vid.clone()];
        for at in &inp.audios {
            args.extend(["-i".into(), at.path.clone()]);
        }
        let mut filters = Vec::new();
        let mut mix_inputs = Vec::new();
        if main_has_audio {
            mix_inputs.push("[0:a]".to_string());
        }
        for (j, at) in inp.audios.iter().enumerate() {
            let delay = (at.start_time * 1000.0) as i64;
            let mut trim_part = String::new();
            if at.trim_start > 0.0 || at.trim_end < at.duration {
                trim_part = format!(
                    "atrim={:.3}:{:.3},asetpts=PTS-STARTPTS,",
                    at.trim_start, at.trim_end
                );
            }
            let vol = finite32(at.volume, 1.0).clamp(0.0, 4.0);
            filters.push(format!(
                "[{}:a]{trim_part}volume={vol:.3},adelay={delay}|{delay}[a{j}]",
                j + 1
            ));
            mix_inputs.push(format!("[a{j}]"));
        }
        let mix_count = mix_inputs.len();
        let duration = if main_has_audio { "first" } else { "longest" };
        filters.push(format!(
            "{}amix=inputs={mix_count}:duration={duration}:dropout_transition=0:normalize=0[aout]",
            mix_inputs.concat()
        ));
        let fc = filters.join(";");
        args.extend([
            "-filter_complex".into(),
            fc,
            "-map".into(),
            "0:v".into(),
            "-map".into(),
            "[aout]".into(),
            "-c:v".into(),
            "copy".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "192k".into(),
            "-shortest".into(),
            "-movflags".into(),
            "+faststart".into(),
            out.clone(),
        ]);
        util::ffmpeg_ok(&args).map_err(|e| anyhow!("Ses katmanları karıştırılamadı: {e}"))?;
    } else {
        fs::copy(&final_vid, out)?;
    }

    let _ = fs::remove_dir_all(&tmp_dir);
    if !std::path::Path::new(out).exists() {
        return Err(anyhow!("Çıktı dosyası oluşturulamadı."));
    }
    Ok(out.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn ff(args: &[&str]) {
        let a: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        util::ffmpeg_ok(&a).expect("ffmpeg komutu başarısız");
    }

    #[test]
    fn timeline_export_with_transition_and_text() {
        let dir = std::env::temp_dir().join("aptoza_export_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.mp4");
        let b = dir.join("b.mp4");
        ff(&[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=320x240:rate=30",
            "-t",
            "1",
            "-pix_fmt",
            "yuv420p",
            a.to_str().unwrap(),
        ]);
        ff(&[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "smptebars=size=320x240:rate=30",
            "-t",
            "1",
            "-pix_fmt",
            "yuv420p",
            b.to_str().unwrap(),
        ]);

        let mut st = TimelineState::default();
        let ua = st.add_clip(a.to_string_lossy().to_string());
        let ub = st.add_clip(b.to_string_lossy().to_string());
        st.set_transition(
            ua,
            ub,
            Transition {
                effect: "fade".into(),
                duration: 0.4,
            },
        );
        st.add_text(TextOverlay {
            uid: 0,
            text: "Merhaba".into(),
            x: 20,
            y: 20,
            size: 36,
            color: "#ffffff".into(),
            start_time: 0.2,
            end_time: 1.4,
        });

        let out = dir.join("out.mp4");
        let inp = st.export_input("640x360".into(), out.to_string_lossy().to_string());
        let status = Arc::new(Mutex::new(String::new()));
        export(inp, status.clone());
        let s = status.lock().unwrap().clone();
        assert!(s.starts_with('✓'), "export durumu: {s}");
        let meta = std::fs::metadata(&out).expect("çıktı dosyası yok");
        assert!(meta.len() > 10_000, "çıktı çok küçük: {}", meta.len());
        let dur = util::duration(&out.to_string_lossy());
        assert!(dur > 1.4 && dur < 2.8, "beklenmeyen süre: {dur}");
    }

    #[test]
    fn timeline_export_mixed_resolution_and_audio_without_transition() {
        let dir = std::env::temp_dir().join("aptoza_export_mixed_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a_audio.mp4");
        let b = dir.join("b_silent.mp4");
        ff(&[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=320x240:rate=30",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-t",
            "1",
            "-pix_fmt",
            "yuv420p",
            "-shortest",
            a.to_str().unwrap(),
        ]);
        ff(&[
            "-y",
            "-f",
            "lavfi",
            "-i",
            "smptebars=size=640x360:rate=30",
            "-t",
            "1",
            "-pix_fmt",
            "yuv420p",
            b.to_str().unwrap(),
        ]);

        let mut st = TimelineState::default();
        st.add_clip(a.to_string_lossy().to_string());
        st.add_clip(b.to_string_lossy().to_string());
        let out = dir.join("out_mixed.mp4");
        let inp = st.export_input("640x360".into(), out.to_string_lossy().to_string());
        let status = Arc::new(Mutex::new(String::new()));
        export(inp, status.clone());
        let s = status.lock().unwrap().clone();
        assert!(s.starts_with('✓'), "export durumu: {s}");
        assert_eq!(util::dimensions(&out.to_string_lossy()), (640, 360));
        assert!(util::has_audio(&out.to_string_lossy()));
        let dur = util::duration(&out.to_string_lossy());
        assert!(dur > 1.6 && dur < 2.8, "beklenmeyen süre: {dur}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
