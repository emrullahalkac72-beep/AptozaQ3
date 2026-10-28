//! Aptoza ana arayüzü: kenar çubuğu, sekmeler, kayıt kontrolü, kısayollar,
//! zamanlayıcı, çizim katmanı ve bölge seçici.
mod edit;
mod overlay;
mod record;
mod timeline_ui;

use crate::draw::DrawState;
use crate::media::{AudioEffectsParams, ImageItem};
use crate::model::{self, RecorderConfig, Region, Settings};
use crate::recorder::{self, Engine, EngineEvent};
use crate::theme;
use crate::timeline::{TimelineDrag, TimelineState, Uid};
use crate::util;
use eframe::egui;
use egui::{Color32, RichText};
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

// ------------------------------------------------------------------ yardımcılar
pub fn h1(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(24.0)
            .strong()
            .color(theme::accent()),
    );
}

pub fn h2(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(17.0).strong().color(theme::txt()));
}

pub fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).size(11.5).color(theme::txt2()));
}

pub fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    theme::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui);
    });
}

pub fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .size(13.0)
            .strong()
            .color(Color32::WHITE),
    )
    .fill(theme::accent())
    .corner_radius(8)
    .min_size(egui::vec2(0.0, 34.0))
}

pub fn secondary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .size(12.0)
            .color(theme::txt()),
    )
    .fill(theme::card2())
    .corner_radius(8)
    .min_size(egui::vec2(0.0, 30.0))
}

pub fn danger_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .size(12.0)
            .strong()
            .color(Color32::WHITE),
    )
    .fill(theme::red())
    .corner_radius(8)
    .min_size(egui::vec2(0.0, 30.0))
}

/// Üst araç çubuğundaki kompakt simge düğmesi.
pub fn top_btn(text: &str, fill: Color32) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .size(14.0)
            .color(Color32::WHITE),
    )
    .fill(fill)
    .corner_radius(6)
    .min_size(egui::vec2(28.0, 26.0))
}

pub fn combo_u32(ui: &mut egui::Ui, id: &str, value: &mut u32, choices: &[u32]) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(value.to_string())
        .width(150.0)
        .show_ui(ui, |ui| {
            for &c in choices {
                ui.selectable_value(value, c, c.to_string());
            }
        });
}

pub fn load_tex(
    ctx: &egui::Context,
    path: &Path,
    name: impl Into<String>,
) -> Option<egui::TextureHandle> {
    let img = image::open(path).ok()?.to_rgba8();
    Some(ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(
            [img.width() as usize, img.height() as usize],
            img.as_raw(),
        ),
        egui::TextureOptions::LINEAR,
    ))
}

pub fn tex_from_rgba(
    ctx: &egui::Context,
    img: &image::RgbaImage,
    name: impl Into<String>,
) -> egui::TextureHandle {
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(
            [img.width() as usize, img.height() as usize],
            img.as_raw(),
        ),
        egui::TextureOptions::LINEAR,
    )
}

// ------------------------------------------------------------------ durum tipleri
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    Record,
    Edit,
    Timeline,
    Audio,
    Settings,
    Help,
}

impl Nav {
    pub const ALL: [Nav; 6] = [
        Nav::Record,
        Nav::Edit,
        Nav::Timeline,
        Nav::Audio,
        Nav::Settings,
        Nav::Help,
    ];
    pub fn icon(self) -> &'static str {
        match self {
            Nav::Record => "🎬",
            Nav::Edit => "📝",
            Nav::Timeline => "⏱",
            Nav::Audio => "🎵",
            Nav::Settings => "⚙",
            Nav::Help => "❓",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Nav::Record => "Kayıt",
            Nav::Edit => "Düzenleme",
            Nav::Timeline => "Timeline",
            Nav::Audio => "Ses Editörü",
            Nav::Settings => "Ayarlar",
            Nav::Help => "Yardım",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotAction {
    Start,
    Pause,
    Stop,
    Screenshot,
    Draw,
}

impl HotAction {
    pub const ALL: [HotAction; 5] = [
        HotAction::Start,
        HotAction::Pause,
        HotAction::Stop,
        HotAction::Screenshot,
        HotAction::Draw,
    ];
    pub fn label(self) -> &'static str {
        match self {
            HotAction::Start => "Kayıt Başlat",
            HotAction::Pause => "Duraklat / Devam",
            HotAction::Stop => "Durdur",
            HotAction::Screenshot => "Ekran Görüntüsü",
            HotAction::Draw => "Çizim Modu Aç/Kapat",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            HotAction::Start => "⏺",
            HotAction::Pause => "⏸",
            HotAction::Stop => "⏹",
            HotAction::Screenshot => "📸",
            HotAction::Draw => "✏",
        }
    }
}

#[derive(Clone, Default)]
pub struct TrimForm {
    pub src: String,
    pub t0: String,
    pub t1: String,
    pub status: String,
    pub dur: String,
}

#[derive(Clone)]
pub struct MergeForm {
    pub list: Vec<String>,
    pub sel: Option<usize>,
    pub status: String,
    pub fmt: String,
}

impl Default for MergeForm {
    fn default() -> Self {
        Self {
            list: Vec::new(),
            sel: None,
            status: String::new(),
            fmt: "mp4".into(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ConvertForm {
    pub src: String,
    pub fmt: String,
    pub status: String,
}

#[derive(Clone)]
pub struct AudioForms {
    pub tab: usize,
    pub trim: TrimForm,
    pub split_src: String,
    pub split_time: String,
    pub split_status: String,
    pub split_dur: String,
    pub conv_src: String,
    pub conv_fmt: String,
    pub conv_br: String,
    pub conv_status: String,
    pub merge: Vec<String>,
    pub merge_sel: Option<usize>,
    pub merge_fmt: String,
    pub merge_status: String,
}

impl Default for AudioForms {
    fn default() -> Self {
        Self {
            tab: 0,
            trim: TrimForm::default(),
            split_src: String::new(),
            split_time: String::new(),
            split_status: String::new(),
            split_dur: String::new(),
            conv_src: String::new(),
            conv_fmt: "mp3".into(),
            conv_br: "192k".into(),
            conv_status: String::new(),
            merge: Vec::new(),
            merge_sel: None,
            merge_fmt: "mp3".into(),
            merge_status: String::new(),
        }
    }
}

pub struct AudioFx {
    pub files: Vec<String>,
    pub src: String,
    pub status: String,
    pub params: AudioEffectsParams,
    pub wave_tex: Option<egui::TextureHandle>,
    pub wave_key: String,
    pub wave_busy: bool,
    pub preview: Option<std::process::Child>,
    pub wave_rx: Option<mpsc::Receiver<anyhow::Result<std::path::PathBuf>>>,
    pub preview_rx: Option<mpsc::Receiver<anyhow::Result<std::path::PathBuf>>>,
    pub preview_busy: bool,
}

impl Default for AudioFx {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            src: String::new(),
            status: String::new(),
            params: AudioEffectsParams::default(),
            wave_tex: None,
            wave_key: String::new(),
            wave_busy: false,
            preview: None,
            wave_rx: None,
            preview_rx: None,
            preview_busy: false,
        }
    }
}

pub struct Img2Vid {
    pub items: Vec<ImageItem>,
    pub res: String,
    pub custom_w: String,
    pub custom_h: String,
    pub def_dur: String,
    pub out: String,
    pub status: String,
    pub effects_idx: Option<usize>,
    pub fx_tex: Option<egui::TextureHandle>,
    pub fx_key: u64,
    pub fx_dirty: Option<Instant>,
}

impl Default for Img2Vid {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            res: "1920x1080".into(),
            custom_w: "1920".into(),
            custom_h: "1080".into(),
            def_dur: "5.0".into(),
            out: String::new(),
            status: String::new(),
            effects_idx: None,
            fx_tex: None,
            fx_key: 0,
            fx_dirty: None,
        }
    }
}

pub struct Scheduler {
    pub start: chrono::DateTime<chrono::Local>,
    pub end: chrono::DateTime<chrono::Local>,
}

// ------------------------------------------------------------------ App
pub struct App {
    pub settings: Settings,
    pub engine: Engine,
    pub draw: DrawState,
    pub nav: Nav,
    pub timeline: TimelineState,
    pub edit_tab: usize,

    pub tl_tab: usize,
    pub tl_drag: Option<TimelineDrag>,
    pub tl_preview_rx: Option<mpsc::Receiver<(Uid, i64, anyhow::Result<std::path::PathBuf>)>>,
    pub tl_preview_busy: bool,
    pub tl_fx_rx: Option<mpsc::Receiver<(u64, anyhow::Result<std::path::PathBuf>)>>,
    pub tl_fx_busy: bool,
    pub tl_fx_dirty: Option<Instant>,
    pub tl_export_open: bool,
    pub tl_export_out: String,
    pub tl_audio_on: bool,
    pub tl_asset_tex: Option<(String, egui::TextureHandle)>,
    pub tl_overlap_warn: bool,
    pub tl_playing_clip: Option<Uid>,
    pub tl_pending: Option<TlPending>,
    pub tl_allow_close: bool,
    pub tl_last_autosave: Option<Instant>,
    pub recover_prompt: bool,

    pub notice: Option<(bool, String)>,
    pub recording_stopping: bool,
    pub draw_mode: bool,
    pub tool_open: bool,
    pub overlay_ops: Vec<crate::draw::DrawOp>,
    pub overlay_text: Option<(egui::Pos2, String)>,
    pub overlay_last_pt: Option<egui::Pos2>,
    pub overlay_freehand: Vec<(f32, f32)>,
    /// Çizim geri alma / yineleme yığınları.
    /// Her giriş: (kayıt katmanı anlık görüntüsü, görünür katman anlık görüntüsü).
    pub draw_undo: Vec<(Vec<crate::draw::DrawOp>, Vec<crate::draw::DrawOp>)>,
    pub draw_redo: Vec<(Vec<crate::draw::DrawOp>, Vec<crate::draw::DrawOp>)>,

    pub webcam_tex: Option<egui::TextureHandle>,
    pub webcam_seq: u64,
    pub show_cam_preview: bool,
    pub cam_res_list: Vec<(u32, u32)>,
    pub cam_res_busy: bool,
    pub cam_res_rx: Option<mpsc::Receiver<Vec<(u32, u32)>>>,

    pub trim: TrimForm,
    pub merge: MergeForm,
    pub convert: ConvertForm,
    pub audio: AudioForms,
    pub afx: AudioFx,
    pub img2vid: Img2Vid,

    pub sched_start_text: String,
    pub sched_end_text: String,
    pub sched_status: String,
    pub scheduler: Option<Scheduler>,
    pub hotkey_capture: Option<HotAction>,
    pub hotkey_captured: String,
    pub hotkey_mgr: Option<GlobalHotKeyManager>,
    pub hotkey_ids: Vec<(u32, HotAction)>,
    pub hotkey_error: String,

    pub region_select: Option<image::RgbaImage>,
    pub region_tex: Option<egui::TextureHandle>,
    pub region_drag: Option<(egui::Pos2, egui::Pos2)>,
    pub region_mon: (i32, i32),

    pub fx_dirty: Option<Instant>,
    pub job_status: SharedStatus,
    pub show_webcam_window: bool,
    pub preview_audio: Option<std::process::Child>,
    pub last_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::load();
        theme::set_dark(settings.dark);
        let draw = DrawState::default();
        let engine = Engine::new(draw.ops.clone());
        let mut app = Self {
            settings,
            engine,
            draw,
            nav: Nav::Record,
            timeline: TimelineState::default(),
            edit_tab: 0,
            tl_tab: 0,
            tl_drag: None,
            tl_preview_rx: None,
            tl_preview_busy: false,
            tl_fx_rx: None,
            tl_fx_busy: false,
            tl_fx_dirty: None,
            tl_export_open: false,
            tl_export_out: String::new(),
            tl_audio_on: true,
            tl_asset_tex: None,
            tl_overlap_warn: false,
            tl_playing_clip: None,
            tl_pending: None,
            tl_allow_close: false,
            tl_last_autosave: None,
            recover_prompt: false,
            notice: None,
            recording_stopping: false,
            draw_mode: false,
            tool_open: false,
            overlay_ops: Vec::new(),
            overlay_text: None,
            overlay_last_pt: None,
            overlay_freehand: Vec::new(),
            draw_undo: Vec::new(),
            draw_redo: Vec::new(),
            webcam_tex: None,
            webcam_seq: 0,
            show_cam_preview: true,
            cam_res_list: vec![(1280, 720)],
            cam_res_busy: false,
            cam_res_rx: None,
            trim: TrimForm::default(),
            merge: MergeForm::default(),
            convert: ConvertForm {
                fmt: "mp4".into(),
                ..Default::default()
            },
            audio: AudioForms::default(),
            afx: AudioFx::default(),
            img2vid: Img2Vid::default(),
            sched_start_text: String::new(),
            sched_end_text: String::new(),
            sched_status: String::new(),
            scheduler: None,
            hotkey_capture: None,
            hotkey_captured: String::new(),
            hotkey_mgr: None,
            hotkey_ids: Vec::new(),
            hotkey_error: String::new(),
            region_select: None,
            region_tex: None,
            region_drag: None,
            region_mon: (0, 0),
            fx_dirty: None,
            job_status: Arc::new(Mutex::new(String::new())),
            show_webcam_window: true,
            preview_audio: None,
            last_tick: Instant::now(),
        };
        app.img2vid.out = Path::new(&app.settings.recorder.output_dir)
            .join("gorsel_video.mp4")
            .to_string_lossy()
            .to_string();
        app.register_hotkeys();
        app.recover_prompt = util::autosave_path().exists();
        app
    }

    // -------------------------------------------------------------- bildirim
    pub fn info(&mut self, text: impl Into<String>) {
        self.notice = Some((false, text.into()));
    }

    pub fn error(&mut self, text: impl Into<String>) {
        self.notice = Some((true, text.into()));
    }

    pub fn job_text(&self) -> String {
        self.job_status
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn run_job<F>(&mut self, job: F)
    where
        F: FnOnce() -> anyhow::Result<String> + Send + 'static,
    {
        util::spawn_job(self.job_status.clone(), job);
    }

    pub fn show_job_status(&self, ui: &mut egui::Ui) {
        let s = self.job_text();
        if s.is_empty() {
            return;
        }
        let col = if s.starts_with('✓') {
            theme::accent()
        } else if s.starts_with('✗') {
            theme::red()
        } else {
            theme::yellow()
        };
        ui.label(RichText::new(s).size(12.0).color(col));
    }

    pub fn show_notice(&mut self, ctx: &egui::Context) {
        let Some((is_err, text)) = self.notice.clone() else {
            return;
        };
        let mut open = true;
        egui::Window::new(if is_err { "⚠ Hata" } else { "ℹ Bilgi" })
            .collapsible(false)
            .resizable(true)
            .default_width(460.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(&text)
                        .color(if is_err { theme::red() } else { theme::txt() })
                        .size(13.0),
                );
                ui.add_space(8.0);
                if ui.add(primary_button("Tamam")).clicked() {
                    self.notice = None;
                }
            });
        if !open {
            self.notice = None;
        }
    }

    // -------------------------------------------------------------- kayıt
    pub fn apply_engine_settings(&mut self) {
        if self.settings.recorder.webcam_res.0 < 2 || self.settings.recorder.webcam_res.1 < 2 {
            self.settings.recorder.webcam_res = (1280, 720);
        }
    }

    pub fn start_recording(&mut self) {
        if self.engine.is_active() || self.recording_stopping {
            return;
        }
        self.apply_engine_settings();
        let cfg: RecorderConfig = self.settings.recorder.clone();
        if !cfg.screen_on && !cfg.webcam_on {
            self.error("Lütfen en az bir kayıt kaynağı etkinleştirin (Ekran Kaydı veya Webcam).");
            return;
        }
        match self.engine.start(cfg) {
            Ok(()) => {
                self.show_webcam_window = true;
                if self.draw_mode {
                    self.draw.clear();
                }
                self.overlay_ops.clear();
                self.clear_draw_history();
                self.draw_mode = self.settings.recorder.screen_on && self.draw_mode;
                self.tool_open = self.draw_mode;
                self.settings.save().ok();
            }
            Err(e) => self.error(format!("Kayıt başlatılamadı: {e}")),
        }
    }

    pub fn stop_recording(&mut self) {
        if !self.engine.is_active() || self.recording_stopping {
            return;
        }
        self.recording_stopping = true;
        self.engine.stop();
        self.draw_mode = false;
        self.tool_open = false;
        self.stop_preview_audio();
    }

    pub fn pause_resume(&mut self) {
        if !self.engine.is_active() {
            return;
        }
        if self.engine.is_paused() {
            self.engine.resume();
        } else {
            self.engine.pause();
        }
    }

    pub fn process_engine(&mut self) {
        while let Some(ev) = self.engine.poll() {
            let EngineEvent::Done { ok, path, message } = ev;
            self.recording_stopping = false;
            self.show_webcam_window = false;
            self.webcam_tex = None;
            if ok {
                let p = path.unwrap_or_default();
                self.notice = Some((false, format!("🎉 Video kaydedildi:\n{}", p)));
            } else {
                self.notice = Some((
                    true,
                    message.unwrap_or_else(|| {
                        "Video kaydedilemedi. FFmpeg kurulumunu, çıktı klasörü izinlerini ve disk alanını kontrol edin.".into()
                    }),
                ));
            }
        }
    }

    pub fn update_webcam(&mut self, ctx: &egui::Context) {
        if !self.engine.is_active() || !self.settings.recorder.webcam_on {
            return;
        }
        let Some(fr) = self.engine.shared.webcam_frame() else {
            return;
        };
        if fr.seq == self.webcam_seq {
            return;
        }
        self.webcam_seq = fr.seq;
        let expected = (fr.w as usize).checked_mul(fr.h as usize).unwrap_or(0);
        if fr.w < 2 || fr.h < 2 || expected == 0 || fr.bgr.len() < expected * 3 {
            return;
        }
        let mut rgba = Vec::with_capacity(expected * 4);
        for px in fr.bgr.chunks_exact(3).take(expected) {
            rgba.extend_from_slice(&[px[2], px[1], px[0], 255]);
        }
        self.webcam_tex = Some(ctx.load_texture(
            "webcam_live",
            egui::ColorImage::from_rgba_unmultiplied([fr.w as usize, fr.h as usize], &rgba),
            egui::TextureOptions::LINEAR,
        ));
    }

    pub fn take_screenshot(&mut self) {
        let fmt = self.settings.screenshot_format.clone();
        let name = format!("screenshot_{}.{}", util::stamp(), fmt);
        let start_dir = Path::new(&self.settings.recorder.output_dir);
        let Some(path) = rfd::FileDialog::new()
            .set_title("Ekran Görüntüsünü Kaydet")
            .set_directory(start_dir)
            .set_file_name(&name)
            .add_filter(fmt.to_uppercase(), &[fmt.as_str()])
            .save_file()
        else {
            return;
        };
        let region = self.settings.recorder.region.map(|r| r.tuple());
        match util::capture_rgba(region) {
            Ok(img) => match util::save_rgba(&path, &img, &fmt) {
                Ok(()) => self.info(format!(
                    "📸 Ekran görüntüsü kaydedildi:\n{}",
                    path.display()
                )),
                Err(e) => self.error(format!("Görsel kaydedilemedi: {e}")),
            },
            Err(e) => self.error(format!("Ekran görüntüsü alınamadı: {e}")),
        }
    }

    // -------------------------------------------------------------- bölge
    pub fn start_region_select(&mut self, ctx: &egui::Context) {
        let monitors = match xcap::Monitor::all() {
            Ok(m) => m,
            Err(e) => {
                self.error(format!("Monitörler listelenemedi: {e}"));
                return;
            }
        };
        let mon = monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| monitors.first())
            .cloned();
        let Some(mon) = mon else {
            self.error("Monitör bulunamadı.");
            return;
        };
        let (mx, my) = (mon.x().unwrap_or(0), mon.y().unwrap_or(0));
        match mon.capture_image() {
            Ok(img) => {
                let tex = tex_from_rgba(ctx, &img, "region_shot");
                self.region_mon = (mx, my);
                self.region_tex = Some(tex);
                self.region_select = Some(img);
                self.region_drag = None;
            }
            Err(e) => self.error(format!("Ekran görüntüsü alınamadı: {e}")),
        }
    }

    pub fn set_region(&mut self, r: Option<Region>) {
        self.settings.recorder.region = r;
        self.settings.save().ok();
    }

    // -------------------------------------------------------------- çizim
    pub fn toggle_draw(&mut self, ctx: &egui::Context) {
        self.draw_mode = !self.draw_mode;
        self.tool_open = self.draw_mode;
        if !self.draw_mode {
            self.overlay_text = None;
            self.draw.clear();
            self.overlay_ops.clear();
            self.clear_draw_history();
        }
        ctx.request_repaint();
    }

    pub fn clear_overlay(&mut self) {
        self.overlay_ops.clear();
        self.draw.clear();
        self.draw.drag_start = None;
        self.overlay_freehand.clear();
        self.overlay_last_pt = None;
        self.clear_draw_history();
    }

    // -------------------------------------------------------------- çizim geçmişi
    pub fn clear_draw_history(&mut self) {
        self.draw_undo.clear();
        self.draw_redo.clear();
    }

    /// Yeni bir çizim işlemi tamamlandığında çağrılır.
    pub fn draw_history_push(
        &mut self,
        rec_before: Vec<crate::draw::DrawOp>,
        visible_before: Vec<crate::draw::DrawOp>,
    ) {
        self.draw_undo.push((rec_before, visible_before));
        if self.draw_undo.len() > 100 {
            self.draw_undo.remove(0);
        }
        self.draw_redo.clear();
    }

    pub fn drawing_undo(&mut self) -> bool {
        let Some((rec, visible)) = self.draw_undo.pop() else {
            return false;
        };
        self.draw_redo
            .push((self.draw.snapshot(), self.overlay_ops.clone()));
        self.draw.restore(rec);
        self.overlay_ops = visible;
        self.draw.drag_start = None;
        self.overlay_freehand.clear();
        self.overlay_last_pt = None;
        true
    }

    pub fn drawing_redo(&mut self) -> bool {
        let Some((rec, visible)) = self.draw_redo.pop() else {
            return false;
        };
        self.draw_undo
            .push((self.draw.snapshot(), self.overlay_ops.clone()));
        self.draw.restore(rec);
        self.overlay_ops = visible;
        self.draw.drag_start = None;
        self.overlay_freehand.clear();
        self.overlay_last_pt = None;
        true
    }

    pub fn set_zoom(&mut self, z: f32) {
        self.settings.recorder.zoom = z.clamp(1.0, 4.0);
        self.engine.shared.set_zoom(self.settings.recorder.zoom);
    }

    // -------------------------------------------------------------- kısayollar
    pub fn register_hotkeys(&mut self) {
        self.hotkey_mgr = None;
        self.hotkey_ids.clear();
        self.hotkey_error.clear();
        let mgr = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                self.hotkey_error = format!("Global kısayollar başlatılamadı: {e}");
                return;
            }
        };
        let list: Vec<(HotAction, String)> = vec![
            (HotAction::Start, self.settings.shortcuts.start.clone()),
            (HotAction::Pause, self.settings.shortcuts.pause.clone()),
            (HotAction::Stop, self.settings.shortcuts.stop.clone()),
            (
                HotAction::Screenshot,
                self.settings.shortcuts.screenshot.clone(),
            ),
            (HotAction::Draw, self.settings.shortcuts.draw.clone()),
        ];
        let mut errors = Vec::new();
        for (action, text) in list {
            let t = text.trim();
            if t.is_empty() || t == "—" || t == "-" {
                continue;
            }
            match t.parse::<HotKey>() {
                Ok(hk) => match mgr.register(hk) {
                    Ok(()) => self.hotkey_ids.push((hk.id(), action)),
                    Err(e) => errors.push(format!("{}: {e}", t)),
                },
                Err(e) => errors.push(format!("{}: geçersiz ({e})", t)),
            }
        }
        if !errors.is_empty() {
            self.hotkey_error = errors.join("  |  ");
        }
        self.hotkey_mgr = Some(mgr);
    }

    pub fn shortcut_text(&self, a: HotAction) -> &str {
        match a {
            HotAction::Start => &self.settings.shortcuts.start,
            HotAction::Pause => &self.settings.shortcuts.pause,
            HotAction::Stop => &self.settings.shortcuts.stop,
            HotAction::Screenshot => &self.settings.shortcuts.screenshot,
            HotAction::Draw => &self.settings.shortcuts.draw,
        }
    }

    pub fn set_shortcut(&mut self, a: HotAction, text: String) {
        match a {
            HotAction::Start => self.settings.shortcuts.start = text,
            HotAction::Pause => self.settings.shortcuts.pause = text,
            HotAction::Stop => self.settings.shortcuts.stop = text,
            HotAction::Screenshot => self.settings.shortcuts.screenshot = text,
            HotAction::Draw => self.settings.shortcuts.draw = text,
        }
        self.register_hotkeys();
    }

    fn poll_hotkeys(&mut self, ctx: &egui::Context) {
        if self.hotkey_ids.is_empty() {
            return;
        }
        let mut action = None;
        for ev in GlobalHotKeyEvent::receiver().try_iter() {
            if ev.state != HotKeyState::Pressed {
                continue;
            }
            if let Some((_, a)) = self.hotkey_ids.iter().find(|(id, _)| *id == ev.id) {
                action = Some(*a);
                break;
            }
        }
        if let Some(a) = action {
            self.run_hot_action(a, ctx);
        }
    }

    pub fn run_hot_action(&mut self, a: HotAction, ctx: &egui::Context) {
        match a {
            HotAction::Start => self.start_recording(),
            HotAction::Pause => self.pause_resume(),
            HotAction::Stop => self.stop_recording(),
            HotAction::Screenshot => self.take_screenshot(),
            HotAction::Draw => self.toggle_draw(ctx),
        }
    }

    pub fn capture_hotkey(&mut self, ctx: &egui::Context) {
        if self.hotkey_capture.is_none() {
            return;
        }
        let mut pressed: Option<String> = None;
        let mut cancel = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                cancel = true;
            }
            for ev in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = ev
                {
                    if let Some(name) = egui_key_name(*key) {
                        let mut s = String::new();
                        if modifiers.ctrl {
                            s.push_str("ctrl+");
                        }
                        if modifiers.shift {
                            s.push_str("shift+");
                        }
                        if modifiers.alt {
                            s.push_str("alt+");
                        }
                        if modifiers.command && !modifiers.ctrl {
                            s.push_str("super+");
                        }
                        s.push_str(&name);
                        pressed = Some(s);
                    }
                }
            }
        });
        if let Some(p) = pressed {
            self.hotkey_captured = p;
        }
        if cancel {
            self.hotkey_capture = None;
            self.hotkey_captured.clear();
        }
    }

    // -------------------------------------------------------------- zamanlayıcı
    pub fn set_scheduler(&mut self) {
        use chrono::{Local, Timelike};
        let parse = |s: &str| -> Option<(u32, u32)> {
            let (h, m) = s.split_once(':')?;
            Some((h.trim().parse().ok()?, m.trim().parse().ok()?))
        };
        let Some((sh, sm)) = parse(&self.sched_start_text) else {
            self.error("Başlangıç saati SS:DD biçiminde olmalıdır (örn. 14:30).");
            return;
        };
        let Some((eh, em)) = parse(&self.sched_end_text) else {
            self.error("Bitiş saati SS:DD biçiminde olmalıdır (örn. 15:00).");
            return;
        };
        if sh > 23 || eh > 23 || sm > 59 || em > 59 {
            self.error("Saat değerleri geçersiz.");
            return;
        }
        if sh == eh && sm == em {
            self.error("Bitiş saati başlangıçtan farklı olmalıdır.");
            return;
        }
        let now = Local::now();
        let mut start = now
            .with_hour(sh)
            .and_then(|d| d.with_minute(sm))
            .and_then(|d| d.with_second(0))
            .unwrap_or(now);
        let mut end = now
            .with_hour(eh)
            .and_then(|d| d.with_minute(em))
            .and_then(|d| d.with_second(0))
            .unwrap_or(now);
        if start < now {
            start += chrono::Duration::days(1);
        }
        if end < start {
            end += chrono::Duration::days(1);
        }
        if end <= start {
            self.error("Bitiş saati başlangıçtan sonra olmalıdır.");
            return;
        }
        self.sched_status = format!(
            "⏰ Bekleniyor... Başlangıç: {}  |  Bitiş: {}",
            start.format("%H:%M"),
            end.format("%H:%M")
        );
        self.scheduler = Some(Scheduler { start, end });
    }

    pub fn start_preview_audio(&mut self, clip: &crate::timeline::Clip, rel: f64) {
        self.stop_preview_audio();
        if clip.mute {
            return;
        }
        let args = vec![
            "-nodisp".into(),
            "-autoexit".into(),
            "-loglevel".into(),
            "quiet".into(),
            "-ss".into(),
            format!("{:.3}", clip.trim_start + rel),
            "-t".into(),
            format!("{:.3}", (clip.clip_duration() - rel).clamp(0.2, 60.0)),
            "-i".into(),
            clip.path.clone(),
        ];
        if let Ok(child) = util::ffplay()
            .args(&args)
            .stdin(std::process::Stdio::null())
            .spawn()
        {
            self.preview_audio = Some(child);
        }
    }

    pub fn stop_preview_audio(&mut self) {
        if let Some(mut c) = self.preview_audio.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    // -------------------------------------------------------------- kenar çubuğu
    fn sidebar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("✨ AptozaQ3")
                            .size(26.0)
                            .strong()
                            .color(theme::accent()),
                    );
                    muted(ui, "Modern Ekran Kaydedici");
                });
                ui.add_space(6.0);
                ui.separator();
                let mut switch_to = None;
                for nav in Nav::ALL {
                    let selected = self.nav == nav;
                    let text = RichText::new(format!("{}  {}", nav.icon(), nav.label()))
                        .size(13.0)
                        .strong()
                        .color(if selected {
                            Color32::WHITE
                        } else {
                            theme::txt()
                        });
                    let btn = egui::Button::new(text)
                        .fill(if selected {
                            theme::accent2()
                        } else {
                            theme::card2()
                        })
                        .corner_radius(8)
                        .min_size(egui::vec2(ui.available_width(), 38.0));
                    if ui.add(btn).clicked() {
                        switch_to = Some(nav);
                    }
                }
                if let Some(n) = switch_to {
                    self.nav = n;
                }
                ui.add_space(4.0);
                if ui
                    .add(
                        primary_button("🎨  Araç Paneli")
                            .min_size(egui::vec2(ui.available_width(), 36.0)),
                    )
                    .clicked()
                {
                    self.tool_open = !self.tool_open;
                }
                ui.separator();

                ui.label(
                    RichText::new("📸 Ekran Görüntüsü")
                        .size(12.0)
                        .strong()
                        .color(theme::accent2()),
                );
                let mut fmt = self.settings.screenshot_format.clone();
                egui::ComboBox::from_id_salt("ss_fmt")
                    .selected_text(fmt.to_uppercase())
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for f in ["png", "jpg", "bmp", "webp", "tiff"] {
                            ui.selectable_value(&mut fmt, f.to_string(), f.to_uppercase());
                        }
                    });
                self.settings.screenshot_format = fmt;
                if ui
                    .add(
                        primary_button("📷  Ekran Görüntüsü Al")
                            .min_size(egui::vec2(ui.available_width(), 34.0)),
                    )
                    .clicked()
                {
                    self.take_screenshot();
                }
                ui.separator();

                let active = self.engine.is_active();
                let paused = self.engine.is_paused();
                let (status, status_col) = if self.recording_stopping {
                    ("⏳ Kaydediliyor...".to_string(), theme::yellow())
                } else if active && paused {
                    ("⏸ Duraklatıldı".to_string(), theme::yellow())
                } else if active {
                    ("● Kayıt Yapılıyor".to_string(), theme::red())
                } else {
                    ("● Hazır".to_string(), theme::green())
                };
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(status).size(12.0).strong().color(status_col));
                    let t = if active {
                        self.engine.shared.elapsed_str()
                    } else {
                        "00:00:00".into()
                    };
                    ui.label(
                        RichText::new(t)
                            .size(28.0)
                            .strong()
                            .monospace()
                            .color(theme::accent()),
                    );
                });

                let start_enabled = !active && !self.recording_stopping;
                if ui
                    .add_enabled(
                        start_enabled,
                        primary_button("⏺ Başlat").min_size(egui::vec2(ui.available_width(), 38.0)),
                    )
                    .clicked()
                {
                    self.start_recording();
                }
                let pause_text = if paused {
                    "▶ Devam Et"
                } else {
                    "⏸ Duraklat"
                };
                if ui
                    .add_enabled(
                        active && !self.recording_stopping,
                        secondary_button(pause_text)
                            .min_size(egui::vec2(ui.available_width(), 34.0)),
                    )
                    .clicked()
                {
                    self.pause_resume();
                }
                if ui
                    .add_enabled(
                        active && !self.recording_stopping,
                        danger_button("⏹ Durdur").min_size(egui::vec2(ui.available_width(), 34.0)),
                    )
                    .clicked()
                {
                    self.stop_recording();
                }
                let muted = self.engine.is_muted();
                let mic_text = if muted {
                    "🔇 Mikrofon Kapalı"
                } else {
                    "🎙 Mikrofon Açık"
                };
                if ui
                    .add_enabled(
                        active && self.settings.recorder.audio_on,
                        secondary_button(mic_text).min_size(egui::vec2(ui.available_width(), 32.0)),
                    )
                    .clicked()
                {
                    self.engine.toggle_mute();
                }
                ui.separator();
                ui.checkbox(
                    &mut self.draw_mode,
                    RichText::new("Ekranda Çizim Modu").size(13.0).strong(),
                );
                if self.draw_mode {
                    self.tool_open = true;
                }
                ui.separator();
                ui.label(
                    RichText::new("🎨 Tema")
                        .size(12.0)
                        .strong()
                        .color(theme::txt2()),
                );
                let mut dark = self.settings.dark;
                ui.horizontal(|ui| {
                    if ui.selectable_label(dark, "Koyu").clicked() {
                        dark = true;
                    }
                    if ui.selectable_label(!dark, "Açık").clicked() {
                        dark = false;
                    }
                });
                if dark != self.settings.dark {
                    self.settings.dark = dark;
                    theme::set_dark(dark);
                    theme::apply(ctx);
                }
                ui.add_space(8.0);
            });
    }

    // -------------------------------------------------------------- üst araç çubuğu
    /// Her sekmeden erişilebilen global araç çubuğu.
    fn top_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("global_toolbar")
            .exact_height(40.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::card())
                    .inner_margin(egui::Margin::symmetric(8, 5)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if ui
                        .add(top_btn("🆕", theme::accent2()))
                        .on_hover_text("Yeni proje (Ctrl+N)")
                        .clicked()
                    {
                        self.tl_new_project();
                    }
                    if ui
                        .add(top_btn("📂", Color32::from_rgb(0x2f, 0x81, 0xf7)))
                        .on_hover_text("Proje aç (Ctrl+O)")
                        .clicked()
                    {
                        self.tl_open_project(ctx);
                    }
                    if ui
                        .add(top_btn("💾", Color32::from_rgb(0x23, 0x86, 0x36)))
                        .on_hover_text("Projeyi kaydet (Ctrl+S, Shift ile farklı kaydet)")
                        .clicked()
                    {
                        self.tl_save_project(false);
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.timeline.can_undo(),
                            top_btn("↩", Color32::from_rgb(0x5a, 0x5a, 0x66)),
                        )
                        .on_hover_text("Geri al (Ctrl+Z)")
                        .clicked()
                    {
                        self.tl_undo();
                    }
                    if ui
                        .add_enabled(
                            self.timeline.can_redo(),
                            top_btn("↪", Color32::from_rgb(0x5a, 0x5a, 0x66)),
                        )
                        .on_hover_text("Yinele (Ctrl+Y / Ctrl+Shift+Z)")
                        .clicked()
                    {
                        self.tl_redo();
                    }
                    ui.separator();
                    if ui
                        .add(top_btn("📤", Color32::from_rgb(0xd2, 0x99, 0x22)))
                        .on_hover_text("Timeline'ı dışa aktar (Ctrl+E)")
                        .clicked()
                    {
                        self.nav = Nav::Timeline;
                        self.tl_open_export();
                    }
                    if ui
                        .add(top_btn("📸", theme::accent()))
                        .on_hover_text("Ekran görüntüsü al")
                        .clicked()
                    {
                        self.take_screenshot();
                    }
                    if self.engine.is_active() {
                        if ui
                            .add(top_btn("⏹", theme::red()))
                            .on_hover_text("Kaydı durdur")
                            .clicked()
                        {
                            self.stop_recording();
                        }
                    } else if ui
                        .add(top_btn("⏺", theme::red()))
                        .on_hover_text("Kayıt başlat")
                        .clicked()
                    {
                        self.start_recording();
                    }
                    let draw_fill = if self.draw_mode {
                        theme::accent()
                    } else {
                        Color32::from_rgb(0x5a, 0x5a, 0x66)
                    };
                    if ui
                        .add(top_btn("🎨", draw_fill))
                        .on_hover_text("Çizim modunu aç/kapat")
                        .clicked()
                    {
                        self.toggle_draw(ctx);
                    }
                    if ui
                        .add(top_btn("🧰", Color32::from_rgb(0x7c, 0x3a, 0xed)))
                        .on_hover_text("Araç panelini aç/kapat")
                        .clicked()
                    {
                        self.tool_open = !self.tool_open;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("AptozaQ3")
                                .size(12.0)
                                .strong()
                                .color(theme::txt2()),
                        );
                    });
                });
            });
    }

    // -------------------------------------------------------------- kısayollar (uygulama içi)
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Metin alanına yazarken veya kısayol atama penceresi açıkken karışma.
        if self.hotkey_capture.is_some() || ctx.wants_keyboard_input() {
            return;
        }
        let (cmd, shift, alt, z, y, s, o, n, e, del, space, home, end, zoom_in, zoom_out, split) =
            ctx.input(|i| {
                (
                    i.modifiers.command,
                    i.modifiers.shift,
                    i.modifiers.alt,
                    i.key_pressed(egui::Key::Z),
                    i.key_pressed(egui::Key::Y),
                    i.key_pressed(egui::Key::S),
                    i.key_pressed(egui::Key::O),
                    i.key_pressed(egui::Key::N),
                    i.key_pressed(egui::Key::E),
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                    i.key_pressed(egui::Key::Space),
                    i.key_pressed(egui::Key::Home),
                    i.key_pressed(egui::Key::End),
                    i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals),
                    i.key_pressed(egui::Key::Minus),
                    i.key_pressed(egui::Key::S),
                )
            });
        if alt {
            return;
        }
        let draw_context = self.draw_mode && self.nav != Nav::Timeline;
        if cmd && z {
            if shift {
                if draw_context {
                    self.drawing_redo();
                } else {
                    self.tl_redo();
                }
            } else if draw_context {
                self.drawing_undo();
            } else {
                self.tl_undo();
            }
        } else if cmd && y {
            if draw_context {
                self.drawing_redo();
            } else {
                self.tl_redo();
            }
        }
        if cmd && s {
            self.tl_save_project(shift);
        }
        if cmd && o {
            self.tl_open_project(ctx);
        }
        if cmd && n {
            self.tl_new_project();
        }
        if cmd && e {
            self.nav = Nav::Timeline;
            self.tl_open_export();
        }
        if self.nav == Nav::Timeline {
            if del {
                self.tl_delete_selected();
            }
            if space {
                self.tl_toggle_play(ctx);
            }
            if home {
                self.tl_seek(0.0);
            }
            if end {
                let d = self.timeline.duration();
                self.tl_seek(d);
            }
            if zoom_in {
                self.timeline.px_per_sec = (self.timeline.px_per_sec * 1.25).min(300.0);
            }
            if zoom_out {
                self.timeline.px_per_sec = (self.timeline.px_per_sec / 1.25).max(8.0);
            }
            if split && !cmd && !shift {
                self.tl_split();
            }
        }
    }

    // -------------------------------------------------------------- güncelleme
    fn tick(&mut self, ctx: &egui::Context) {
        if self.engine.is_active() {
            if self.settings.recorder.zoom > 1.001 {
                if let Some((x, y)) = util::cursor_pos() {
                    let (ox, oy) = self
                        .settings
                        .recorder
                        .region
                        .map(|r| (r.x, r.y))
                        .unwrap_or((0, 0));
                    let cz = self.engine.shared.zoom_center.lock();
                    if let Ok(mut g) = cz {
                        *g = Some(((x - ox) as f32, (y - oy) as f32));
                    }
                }
            }
            let fps = self.settings.recorder.fps.max(5) as u64;
            ctx.request_repaint_after(Duration::from_millis((1000 / fps).max(15)));
        } else if self.draw_mode {
            ctx.request_repaint_after(Duration::from_millis(33));
        }

        if let Some(sched) = self.scheduler.take() {
            use chrono::Local;
            let now = Local::now();
            if now >= sched.start && !self.engine.is_active() && !self.recording_stopping {
                self.start_recording();
            } else if now >= sched.end {
                if self.engine.is_active() {
                    self.stop_recording();
                } else {
                    self.sched_status = "✓ Zamanlanmış kayıt tamamlandı.".into();
                    ctx.request_repaint();
                    return;
                }
            }
            self.scheduler = Some(sched);
        }

        if let Some(rx) = self.cam_res_rx.take() {
            match rx.try_recv() {
                Ok(list) => {
                    self.cam_res_busy = false;
                    if !list.is_empty() {
                        self.cam_res_list = list;
                        self.settings.recorder.webcam_res = self.cam_res_list[0];
                    }
                }
                Err(mpsc::TryRecvError::Empty) => self.cam_res_rx = Some(rx),
                Err(mpsc::TryRecvError::Disconnected) => self.cam_res_busy = false,
            }
        }

        if self.settings.recorder.zoom < 0.99 {
            self.settings.recorder.zoom = 1.0;
        }

        if Instant::now().duration_since(self.last_tick) > Duration::from_millis(750) {
            self.last_tick = Instant::now();
            if let Some(mut child) = self.preview_audio.take() {
                match child.try_wait() {
                    Ok(Some(_)) => {}
                    _ => self.preview_audio = Some(child),
                }
            }
        }
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.hotkey_capture.is_some() {
            self.capture_hotkey(ctx);
            let action = self.hotkey_capture.unwrap();
            let mut close = false;
            let mut clear = false;
            egui::Window::new("Kısayol Ata")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new(format!("« {} »\niçin bir tuşa basın:", action.label()))
                            .size(14.0),
                    );
                    ui.add_space(6.0);
                    let shown = if self.hotkey_captured.is_empty() {
                        "..."
                    } else {
                        self.hotkey_captured.as_str()
                    };
                    ui.label(
                        RichText::new(shown)
                            .monospace()
                            .size(18.0)
                            .strong()
                            .color(theme::accent2()),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(primary_button("✓  Uygula")).clicked()
                            && !self.hotkey_captured.is_empty()
                        {
                            self.set_shortcut(action, self.hotkey_captured.clone());
                            close = true;
                        }
                        if ui.add(danger_button("✗  Temizle")).clicked() {
                            clear = true;
                        }
                        if ui.add(secondary_button("İptal")).clicked() {
                            close = true;
                        }
                    });
                });
            if clear {
                self.set_shortcut(action, "—".into());
                close = true;
            }
            if close {
                self.hotkey_capture = None;
                self.hotkey_captured.clear();
            }
        }
    }
}

fn egui_key_name(key: egui::Key) -> Option<String> {
    use egui::Key as K;
    let s = match key {
        K::F1 => "f1",
        K::F2 => "f2",
        K::F3 => "f3",
        K::F4 => "f4",
        K::F5 => "f5",
        K::F6 => "f6",
        K::F7 => "f7",
        K::F8 => "f8",
        K::F9 => "f9",
        K::F10 => "f10",
        K::F11 => "f11",
        K::F12 => "f12",
        K::F13 => "f13",
        K::F14 => "f14",
        K::F15 => "f15",
        K::F16 => "f16",
        K::F17 => "f17",
        K::F18 => "f18",
        K::F19 => "f19",
        K::F20 => "f20",
        K::A => "a",
        K::B => "b",
        K::C => "c",
        K::D => "d",
        K::E => "e",
        K::F => "f",
        K::G => "g",
        K::H => "h",
        K::I => "i",
        K::J => "j",
        K::K => "k",
        K::L => "l",
        K::M => "m",
        K::N => "n",
        K::O => "o",
        K::P => "p",
        K::Q => "q",
        K::R => "r",
        K::S => "s",
        K::T => "t",
        K::U => "u",
        K::V => "v",
        K::W => "w",
        K::X => "x",
        K::Y => "y",
        K::Z => "z",
        K::Num0 => "0",
        K::Num1 => "1",
        K::Num2 => "2",
        K::Num3 => "3",
        K::Num4 => "4",
        K::Num5 => "5",
        K::Num6 => "6",
        K::Num7 => "7",
        K::Num8 => "8",
        K::Num9 => "9",
        K::Space => "space",
        K::Enter => "enter",
        K::Tab => "tab",
        K::Backspace => "backspace",
        K::Delete => "delete",
        K::Home => "home",
        K::End => "end",
        K::PageUp => "pageup",
        K::PageDown => "pagedown",
        K::Insert => "insert",
        K::ArrowUp => "up",
        K::ArrowDown => "down",
        K::ArrowLeft => "left",
        K::ArrowRight => "right",
        K::Minus => "-",
        K::Plus => "=",
        K::Comma => ",",
        K::Period => ".",
        K::Slash => "/",
        K::Backslash => "\\",
        K::Semicolon => ";",
        K::Quote => "'",
        K::OpenBracket => "[",
        K::CloseBracket => "]",
        K::Backtick => "`",
        _ => return None,
    };
    Some(s.to_string())
}

impl eframe::App for App {
    /// Saydam çizim penceresinin arkasının görünmesi için tamamen şeffaf temizleme.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::set_dark(self.settings.dark);
        theme::apply(ctx);

        self.process_engine();
        self.tl_housekeeping(ctx);
        self.poll_hotkeys(ctx);
        self.handle_shortcuts(ctx);
        self.update_webcam(ctx);
        self.tick(ctx);

        // timeline dışa aktarma durumunu izle
        if self.timeline.exporting {
            let st = self
                .timeline
                .status
                .lock()
                .map(|s| s.clone())
                .unwrap_or_default();
            if !st.starts_with('⏳') && !st.is_empty() {
                self.timeline.exporting = false;
                ctx.request_repaint();
            }
        }

        self.top_toolbar(ctx);

        egui::SidePanel::left("sidebar")
            .exact_width(222.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(theme::card())
                    .inner_margin(egui::Margin::symmetric(8, 6)),
            )
            .show(ctx, |ui| self.sidebar(ui, ctx));

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::bg())
                    .inner_margin(egui::Margin::symmetric(4, 4)),
            )
            .show(ctx, |ui| match self.nav {
                Nav::Record => self.ui_record(ui),
                Nav::Edit => self.ui_edit(ui),
                Nav::Timeline => self.ui_timeline(ui, ctx),
                Nav::Audio => self.ui_audio(ui),
                Nav::Settings => self.ui_settings(ui),
                Nav::Help => self.ui_help(ui),
            });

        if self.draw_mode {
            self.draw_overlay_viewport(ctx);
        } else {
            #[cfg(windows)]
            overlay::destroy_display();
        }
        if self.tool_open {
            self.tools_viewport(ctx);
        }
        if self.show_webcam_window
            && self.engine.is_active()
            && self.settings.recorder.webcam_on
            && self.show_cam_preview
        {
            self.webcam_viewport(ctx);
        }
        if self.region_select.is_some() {
            self.region_viewport(ctx);
        }

        self.img_effects_window(ctx);
        self.timeline_dialogs(ctx);
        self.tl_unsaved_dialogs(ctx);
        if self.nav == Nav::Timeline {
            self.timeline.history_tick();
        }
        self.show_notice(ctx);
        self.dialogs(ctx);
    }
}

/// Kaydedilmemiş timeline değişikliği varken bekleyen eylem.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TlPending {
    OpenProject,
    NewProject,
    Close,
}

// yeniden kullanılan tipler
pub type SharedStatus = Arc<Mutex<String>>;
