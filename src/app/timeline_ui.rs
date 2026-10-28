//! Timeline video editörü: önizleme, araç çubuğu, çok katmanlı zaman çizelgesi,
//! özellik panelleri ve dışa aktarma akışı.
mod dialogs;

use super::*;
use crate::timeline::{
    self, Asset, AssetKind, Selection, TextOverlay, TimelineDrag, Transition, TRANSITIONS,
};
use std::sync::mpsc;

pub const ROW_CLIP_Y: f32 = 30.0;
pub const ROW_CLIP_H: f32 = 54.0;
pub const ROW_AUDIO_Y: f32 = 92.0;
pub const ROW_AUDIO_H: f32 = 24.0;
pub const ROW_TEXT_Y: f32 = 124.0;
pub const ROW_TEXT_H: f32 = 22.0;
pub const ROW_ASSET_Y: f32 = 154.0;
pub const ROW_ASSET_H: f32 = 22.0;
pub const TL_HEIGHT: f32 = 184.0;

impl App {
    pub fn ui_timeline(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.poll_timeline_jobs(ctx);
        self.tl_play_tick(ctx);

        h1(ui, "⏰  Timeline Video Editör");
        ui.add_space(2.0);

        let total_w = ui.available_width();
        let total_h = ui.available_height();
        let gap = ui.spacing().item_spacing.x;
        let bar = ui.spacing().scroll.allocated_width();
        // Sağ paneli ekrana göre daralt; sol + sağ + boşluk + kaydırma çubukları
        // toplamı kullanılabilir genişliği asla aşmasın.
        let right_w = (total_w * 0.30).clamp(250.0, 340.0);
        let reserved = gap * 3.0 + 4.0 + bar * 2.0 + 6.0;
        let left_w = (total_w - right_w - reserved).max(260.0);
        let left_inner = (left_w - bar).max(240.0);
        let right_inner = (right_w - bar).max(220.0);

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_w, total_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_max_width(left_w);
                    egui::ScrollArea::vertical()
                        .id_salt("tl_left_scroll")
                        .max_width(left_w)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(left_inner);
                            self.tl_left(ui, ctx);
                        });
                },
            );
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(right_w, total_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_max_width(right_w);
                    egui::ScrollArea::vertical()
                        .id_salt("tl_props_scroll")
                        .max_width(right_w)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(right_inner);
                            self.tl_props(ui, ctx);
                        });
                },
            );
        });
    }

    // ------------------------------------------------------------ sol taraf
    fn tl_left(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.tl_preview(ui, ctx);
        self.tl_toolbar(ui, ctx);
        self.tl_timeline_canvas(ui, ctx);
        ui.add_space(2.0);
        let st = self.tl_status_text();
        if !st.is_empty() {
            let col = if st.starts_with('✓') {
                theme::accent()
            } else if st.starts_with('✗') || st.starts_with('⚠') {
                theme::red()
            } else {
                theme::yellow()
            };
            ui.label(RichText::new(st).size(11.5).color(col));
        }
        self.show_job_status(ui);
    }

    // ------------------------------------------------------------ önizleme
    fn tl_preview(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let dur = self.timeline.duration().max(1.0);
        let playhead = self.timeline.playhead.clamp(0.0, dur);
        let active = self.timeline.clip_at(playhead).cloned();

        // Efektli kare önizlemesini arka planda hazırla.
        if let Some(c) = &active {
            let rel = ((playhead - c.timeline_start) * c.speed.max(0.01) as f64)
                .clamp(0.0, (c.trim_end - c.trim_start - 0.05).max(0.0));
            let tkey = (rel * 4.0).round() as i64;
            let key = (c.uid, tkey);
            if self.timeline.preview_key != Some(key)
                && !self.tl_preview_busy
                && self.tl_preview_rx.is_none()
            {
                let path = c.path.clone();
                let t = (c.trim_start + rel).min((c.duration - 0.05).max(0.0));
                let eff = c.effects.clone();
                let out = util::unique_temp("jpg");
                let (tx, rx) = mpsc::channel();
                self.tl_preview_rx = Some(rx);
                self.tl_preview_busy = true;
                self.timeline.preview_key = Some(key);
                std::thread::spawn(move || {
                    let res = timeline::make_preview(&path, t, &eff, &out).map(|_| out);
                    let _ = tx.send((key.0, key.1, res));
                });
            }
        }

        // Metin ve şekiller önizlemeye doğrudan çizilir.
        let texts: Vec<TextOverlay> = self
            .timeline
            .texts
            .iter()
            .filter(|o| o.start_time <= playhead && playhead <= o.end_time)
            .cloned()
            .collect();
        let assets: Vec<Asset> = self
            .timeline
            .assets
            .iter()
            .filter(|a| a.start_time <= playhead && playhead <= a.end_time)
            .cloned()
            .collect();

        // Tek görsel önbelleği (aktif görsel katmanı için).
        let active_img = assets.iter().find(|a| a.kind == AssetKind::Image).cloned();
        if let Some(a) = &active_img {
            let need = self
                .tl_asset_tex
                .as_ref()
                .map(|(p, _)| p != &a.path)
                .unwrap_or(true);
            if need {
                if let Some(tex) = load_tex(ctx, Path::new(&a.path), format!("tlasset{}", a.uid)) {
                    self.tl_asset_tex = Some((a.path.clone(), tex));
                }
            }
        }
        let asset_tex = self.tl_asset_tex.clone();
        let preview_tex = self.timeline.preview_tex.clone();

        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let w = ui.available_width().max(200.0);
            let h = (w * 0.42).clamp(150.0, 240.0);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
            let p = ui.painter_at(rect);
            p.rect_filled(rect, 8.0, Color32::from_rgb(4, 6, 9));

            if active.is_none() {
                p.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "📽  Video Önizleme\nTimeline'a video ekleyip oynatın",
                    egui::FontId::proportional(14.0),
                    theme::txt2(),
                );
            } else if let Some(tex) = &preview_tex {
                let s = tex.size_vec2();
                let sc = (rect.width() / s.x).min(rect.height() / s.y);
                let img = egui::Rect::from_center_size(rect.center(), s * sc);
                p.image(
                    tex.id(),
                    img,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                draw_preview_layers(&p, img, &texts, &assets, &asset_tex);
            } else {
                p.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "⏳  Önizleme hazırlanıyor...",
                    egui::FontId::proportional(14.0),
                    theme::txt2(),
                );
                ui.ctx().request_repaint_after(Duration::from_millis(120));
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let play_txt = if self.timeline.playing {
                    "⏸  Duraklat"
                } else {
                    "▶  Oynat"
                };
                if ui.add(primary_button(play_txt)).clicked() {
                    self.tl_toggle_play(ctx);
                }
                if ui.add(secondary_button("⏹")).clicked() {
                    self.tl_stop_play();
                }
                if ui.add(secondary_button("⏮")).clicked() {
                    self.tl_seek(0.0);
                }
                ui.label(
                    RichText::new(format!("{} / {}", util::mmss(playhead), util::mmss(dur)))
                        .monospace()
                        .size(12.0)
                        .color(theme::txt2()),
                );
                let mut audio_on = self.tl_audio_on;
                if ui.checkbox(&mut audio_on, "🔊 Ses").changed() {
                    self.tl_audio_on = audio_on;
                    if !audio_on {
                        self.stop_preview_audio();
                        self.tl_playing_clip = None;
                    }
                }
            });

            let mut pos = playhead;
            let slider = ui.add(
                egui::Slider::new(&mut pos, 0.0..=dur)
                    .show_value(false)
                    .trailing_fill(true),
            );
            if slider.changed() {
                self.tl_seek(pos);
            }

            if let Some(c) = &active {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "🎬 {}   ⏱ {:.1}s   {}×",
                            util::shorten(&c.name, 26),
                            c.clip_duration(),
                            c.speed
                        ))
                        .size(11.0)
                        .color(theme::txt2()),
                    );
                    if c.mute {
                        ui.label(RichText::new("🔇").size(11.0).color(theme::red()));
                    }
                });
            }
        });
    }

    // ------------------------------------------------------------ araç çubuğu
    fn tl_toolbar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        theme::dark_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add(tool_btn("🆕 Yeni", Color32::from_rgb(0x38, 0x62, 0x93)))
                    .on_hover_text("Yeni boş proje (Ctrl+N)")
                    .clicked()
                {
                    self.tl_new_project();
                }
                if ui
                    .add_enabled(
                        self.timeline.can_undo(),
                        tool_btn("↩ Geri", Color32::from_rgb(0x5a, 0x5a, 0x66)),
                    )
                    .on_hover_text("Geri al (Ctrl+Z)")
                    .clicked()
                {
                    self.tl_undo();
                }
                if ui
                    .add_enabled(
                        self.timeline.can_redo(),
                        tool_btn("↪ Yinele", Color32::from_rgb(0x5a, 0x5a, 0x66)),
                    )
                    .on_hover_text("Yinele (Ctrl+Y / Ctrl+Shift+Z)")
                    .clicked()
                {
                    self.tl_redo();
                }
                ui.separator();
                if ui
                    .add(tool_btn("📁 Proje Aç", Color32::from_rgb(0x2f, 0x81, 0xf7)))
                    .on_hover_text("Kaydedilmiş timeline projesini aç (Ctrl+O)")
                    .clicked()
                {
                    self.tl_open_project(ctx);
                }
                if ui
                    .add(tool_btn(
                        "💾 Proje Kaydet",
                        Color32::from_rgb(0x23, 0x86, 0x36),
                    ))
                    .on_hover_text(
                        "Timeline'ı proje olarak kaydet (Ctrl+S, Shift ile farklı kaydet)",
                    )
                    .clicked()
                {
                    self.tl_save_project(false);
                }
                ui.separator();
                if ui
                    .add(tool_btn("📂 Video Ekle", theme::accent2()))
                    .clicked()
                {
                    self.tl_import_videos(ctx);
                }
                if ui.add(tool_btn("🎵 Ses Ekle", theme::green())).clicked() {
                    self.tl_import_audio();
                }
                if ui
                    .add(tool_btn(
                        "📝 Metin Ekle",
                        Color32::from_rgb(0xff, 0x95, 0x00),
                    ))
                    .clicked()
                {
                    self.tl_new_text();
                }
                if ui
                    .add(tool_btn("🎬 Geçiş", Color32::from_rgb(0x7c, 0x3a, 0xed)))
                    .clicked()
                {
                    self.open_transition_for_selected();
                }
                if ui.add(tool_btn("🔪 Böl", theme::yellow())).clicked() {
                    self.tl_split();
                }
                if ui
                    .add(tool_btn(
                        "🎵 Sesi Ayır",
                        Color32::from_rgb(0xff, 0x00, 0x5d),
                    ))
                    .clicked()
                {
                    self.tl_extract_audio();
                }
                if ui
                    .add(tool_btn("🔇 Sesi Sil", Color32::from_rgb(0x38, 0x62, 0x93)))
                    .clicked()
                {
                    self.tl_toggle_mute();
                }
                if ui.add(tool_btn("🚮 Sil", theme::red())).clicked() {
                    self.tl_delete_selected();
                }
                if ui
                    .add(tool_btn(
                        "💾 Dışa Aktar",
                        Color32::from_rgb(0xd2, 0x99, 0x22),
                    ))
                    .clicked()
                {
                    self.tl_open_export();
                }
                if ui
                    .add(tool_btn(
                        "🎨 Şekil/Görsel",
                        Color32::from_rgb(0x0d, 0x6e, 0xfd),
                    ))
                    .clicked()
                {
                    self.tl_new_asset();
                }
                ui.separator();
                if ui
                    .add(secondary_button("➕"))
                    .on_hover_text("Yakınlaştır")
                    .clicked()
                {
                    self.timeline.px_per_sec = (self.timeline.px_per_sec * 1.5).min(300.0);
                }
                if ui
                    .add(secondary_button("➖"))
                    .on_hover_text("Uzaklaştır")
                    .clicked()
                {
                    self.timeline.px_per_sec = (self.timeline.px_per_sec / 1.5).max(8.0);
                }
            });
        });
    }

    pub fn tl_status_text(&self) -> String {
        self.timeline
            .status
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    fn tl_set_status(&self, text: impl Into<String>) {
        if let Ok(mut s) = self.timeline.status.lock() {
            *s = text.into();
        }
    }

    // ------------------------------------------------------------ oynatma
    pub fn tl_toggle_play(&mut self, ctx: &egui::Context) {
        if self.timeline.playing {
            self.timeline.playing = false;
            self.timeline.last_tick = None;
            self.stop_preview_audio();
            self.tl_playing_clip = None;
            return;
        }
        if self.timeline.clips.is_empty() {
            self.info("Önce timeline'a bir video ekleyin.");
            return;
        }
        let max = self.timeline.duration();
        if self.timeline.playhead >= max - 0.05 {
            self.timeline.playhead = 0.0;
        }
        self.timeline.playing = true;
        self.timeline.last_tick = Some(Instant::now());
        self.tl_playing_clip = None;
        self.stop_preview_audio();
        ctx.request_repaint();
    }

    pub fn tl_stop_play(&mut self) {
        self.timeline.playing = false;
        self.timeline.last_tick = None;
        self.timeline.playhead = 0.0;
        self.stop_preview_audio();
        self.tl_playing_clip = None;
    }

    pub fn tl_undo(&mut self) {
        self.stop_preview_audio();
        self.tl_playing_clip = None;
        if self.timeline.undo() {
            self.tl_asset_tex = None;
            self.tl_set_status("↩ Geri alındı.");
        }
    }

    pub fn tl_redo(&mut self) {
        self.stop_preview_audio();
        self.tl_playing_clip = None;
        if self.timeline.redo() {
            self.tl_asset_tex = None;
            self.tl_set_status("↪ Yinelendi.");
        }
    }

    pub fn tl_seek(&mut self, pos: f64) {
        let dur = self.timeline.duration();
        self.timeline.playhead = pos.clamp(0.0, dur);
        self.timeline.last_tick = Some(Instant::now());
        self.stop_preview_audio();
        self.tl_playing_clip = None;
    }

    fn tl_play_tick(&mut self, ctx: &egui::Context) {
        if !self.timeline.playing {
            return;
        }
        let now = Instant::now();
        if let Some(last) = self.timeline.last_tick {
            let dt = now.duration_since(last).as_secs_f64().min(0.25);
            self.timeline.playhead += dt;
        }
        self.timeline.last_tick = Some(now);

        let max = self
            .timeline
            .clips
            .iter()
            .map(|c| c.timeline_end())
            .fold(0.0, f64::max);
        if max <= 0.0 || self.timeline.playhead >= max {
            self.timeline.playhead = max.max(0.0);
            self.timeline.playing = false;
            self.timeline.last_tick = None;
            self.stop_preview_audio();
            self.tl_playing_clip = None;
            ctx.request_repaint();
            return;
        }

        let active = self.timeline.clip_at(self.timeline.playhead).cloned();
        match active {
            Some(c) if !c.mute && self.tl_audio_on => {
                if self.tl_playing_clip != Some(c.uid) {
                    let rel =
                        ((self.timeline.playhead - c.timeline_start) * c.speed as f64).max(0.0);
                    self.start_preview_audio(&c, rel);
                    self.tl_playing_clip = Some(c.uid);
                }
            }
            _ => {
                if self.tl_playing_clip.is_some() {
                    self.stop_preview_audio();
                    self.tl_playing_clip = None;
                }
            }
        }
        ctx.request_repaint_after(Duration::from_millis(33));
    }

    // ------------------------------------------------------------ arka plan işleri
    pub fn poll_timeline_jobs(&mut self, ctx: &egui::Context) {
        if let Some(rx) = self.tl_preview_rx.take() {
            match rx.try_recv() {
                Ok((_uid, _tkey, res)) => {
                    self.tl_preview_busy = false;
                    if let Ok(path) = res {
                        let name = format!("tlprev_{}", util::stamp());
                        if let Some(tex) = load_tex(ctx, &path, name) {
                            self.timeline.preview_tex = Some(tex);
                        }
                        let _ = std::fs::remove_file(path);
                    }
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.tl_preview_rx = Some(rx);
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.tl_preview_busy = false;
                }
            }
        }

        if let Some(rx) = self.tl_fx_rx.take() {
            match rx.try_recv() {
                Ok((hash, res)) => {
                    self.tl_fx_busy = false;
                    let current = self.timeline.effects_key.map(|(_, h)| h);
                    if current == Some(hash) {
                        if let Ok(path) = res {
                            let name = format!("tlfx_{}", util::stamp());
                            if let Some(tex) = load_tex(ctx, &path, name) {
                                self.timeline.effects_tex = Some(tex);
                            }
                            let _ = std::fs::remove_file(path);
                        }
                    }
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.tl_fx_rx = Some(rx);
                    ctx.request_repaint_after(Duration::from_millis(120));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.tl_fx_busy = false;
                }
            }
        }
    }

    // ------------------------------------------------------------ proje kaydet / aç
    /// Timeline'ı (klipler, kırpmalar, geçişler, efektler, metin, şekil, ses...) .aptproj olarak kaydeder.
    pub fn tl_save_project(&mut self, save_as: bool) {
        let path = match (&self.timeline.project_path, save_as) {
            (Some(p), false) => p.clone(),
            _ => {
                let default_name = self
                    .timeline
                    .project_path
                    .as_deref()
                    .map(|p| util::basename(p))
                    .unwrap_or_else(|| {
                        format!(
                            "timeline_{}.{}",
                            chrono::Local::now().format("%Y%m%d_%H%M%S"),
                            crate::timeline::PROJECT_EXT
                        )
                    });
                let Some(f) = rfd::FileDialog::new()
                    .set_title("Timeline Projesini Kaydet")
                    .set_directory(&self.settings.recorder.output_dir)
                    .set_file_name(default_name)
                    .add_filter("Aptoza Projesi", &[crate::timeline::PROJECT_EXT])
                    .save_file()
                else {
                    return;
                };
                let mut p = f.to_string_lossy().to_string();
                if !p
                    .to_lowercase()
                    .ends_with(&format!(".{}", crate::timeline::PROJECT_EXT))
                {
                    p.push('.');
                    p.push_str(crate::timeline::PROJECT_EXT);
                }
                p
            }
        };
        match self.timeline.save_project(&path) {
            Ok(()) => self.tl_set_status(format!("✓ Proje kaydedildi: {}", util::basename(&path))),
            Err(e) => self.error(format!("Proje kaydedilemedi:\n{e}")),
        }
    }

    /// Yeni boş proje; kaydedilmemiş değişiklik varsa önce sorar.
    pub fn tl_new_project(&mut self) {
        if self.timeline.is_dirty() {
            self.tl_pending = Some(TlPending::NewProject);
            return;
        }
        self.tl_new_project_force();
    }

    pub fn tl_new_project_force(&mut self) {
        self.timeline.new_project();
        self.stop_preview_audio();
        self.tl_playing_clip = None;
        self.tl_asset_tex = None;
        self.nav = Nav::Timeline;
        self.tl_set_status("🆕 Yeni proje oluşturuldu.");
    }

    /// Kaydedilmiş projeyi açar; kaydedilmemiş değişiklik varsa önce sorar.
    pub fn tl_open_project(&mut self, ctx: &egui::Context) {
        if self.timeline.is_dirty() {
            self.tl_pending = Some(TlPending::OpenProject);
            return;
        }
        self.tl_open_project_force(ctx);
    }

    /// Dosya seçtirip projeyi açar; tüm timeline durumu geri yüklenir.
    pub fn tl_open_project_force(&mut self, ctx: &egui::Context) {
        self.stop_preview_audio();
        self.timeline.playing = false;
        self.timeline.last_tick = None;
        self.tl_playing_clip = None;
        let Some(f) = rfd::FileDialog::new()
            .set_title("Timeline Projesi Aç")
            .set_directory(&self.settings.recorder.output_dir)
            .add_filter("Aptoza Projesi", &[crate::timeline::PROJECT_EXT])
            .pick_file()
        else {
            return;
        };
        let path = f.to_string_lossy().to_string();
        match self.timeline.load_project(&path) {
            Ok(missing) => {
                self.nav = Nav::Timeline;
                self.tl_asset_tex = None;
                self.timeline.regenerate_thumbs(ctx);
                if missing.is_empty() {
                    self.tl_set_status(format!("✓ Proje açıldı: {}", util::basename(&path)));
                } else {
                    let list = missing
                        .iter()
                        .take(8)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.tl_set_status(format!(
                        "⚠ Proje açıldı, {} dosya bulunamadı.",
                        missing.len()
                    ));
                    self.error(format!(
                        "Proje açıldı ancak aşağıdaki dosyalar bulunamadı (taşınmış veya silinmiş olabilir):\n{list}"
                    ));
                }
                ctx.request_repaint();
            }
            Err(e) => self.error(format!("Proje açılamadı:\n{e}")),
        }
    }

    // ------------------------------------------------------------ otomatik kayıt / uyarılar
    /// Kapatma isteğini denetler ve kaydedilmemiş timeline'ı periyodik olarak otomatik kaydeder.
    pub fn tl_housekeeping(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.tl_allow_close {
            if self.timeline.is_dirty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.tl_pending = Some(TlPending::Close);
            } else if !self.recover_prompt {
                self.timeline.remove_autosave();
            }
        }
        if self.recover_prompt {
            return;
        }
        let due = self
            .tl_last_autosave
            .map(|t| t.elapsed() >= Duration::from_secs(30))
            .unwrap_or(true);
        if due {
            self.tl_last_autosave = Some(Instant::now());
            if self.timeline.is_dirty() {
                let _ = self.timeline.autosave();
            } else if self.timeline.autosaved_snapshot.is_empty() {
                // temiz durumda eski kurtarma dosyası kalmasın
                if util::autosave_path().exists() {
                    self.timeline.remove_autosave();
                }
            } else {
                self.timeline.remove_autosave();
            }
        }
        ctx.request_repaint_after(Duration::from_secs(30));
    }

    /// Kurtarma ve "kaydedilmemiş değişiklikler" pencereleri.
    pub fn tl_unsaved_dialogs(&mut self, ctx: &egui::Context) {
        if self.recover_prompt {
            let mut act = 0;
            egui::Window::new("♻ Kurtarılan Timeline")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(
                        "Önceki oturumdan kaydedilmemiş bir timeline otomatik kaydı bulundu.\nGeri yüklemek ister misiniz?",
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.add(primary_button("↩  Geri Yükle")).clicked() {
                            act = 1;
                        }
                        if ui.add(danger_button("🗑  Sil")).clicked() {
                            act = 2;
                        }
                    });
                });
            match act {
                1 => {
                    let path = util::autosave_path().to_string_lossy().to_string();
                    match self.timeline.load_project(&path) {
                        Ok(missing) => {
                            // kullanıcı projesine bağla; içerik kaydedilmemiş sayılır
                            self.timeline.project_path = self.timeline.recovered_source.take();
                            self.timeline.saved_snapshot.clear();
                            self.timeline.regenerate_thumbs(ctx);
                            self.nav = Nav::Timeline;
                            if missing.is_empty() {
                                self.tl_set_status("✓ Otomatik kayıt geri yüklendi.");
                            } else {
                                self.error(format!(
                                    "Geri yüklendi ancak {} dosya bulunamadı.",
                                    missing.len()
                                ));
                            }
                        }
                        Err(e) => self.error(format!("Otomatik kayıt okunamadı:\n{e}")),
                    }
                    self.recover_prompt = false;
                }
                2 => {
                    self.timeline.remove_autosave();
                    self.recover_prompt = false;
                }
                _ => {}
            }
            return;
        }

        let Some(pending) = self.tl_pending else {
            return;
        };
        let (title, text, save_label, discard_label) = match pending {
            TlPending::OpenProject => (
                "Kaydedilmemiş Değişiklikler",
                "Timeline'da kaydedilmemiş değişiklikler var.\nBaşka bir proje açmadan önce kaydetmek ister misiniz?",
                "💾  Kaydet ve Aç",
                "Kaydetmeden Aç",
            ),
            TlPending::Close => (
                "Kaydedilmemiş Değişiklikler",
                "Timeline'da kaydedilmemiş değişiklikler var.\nÇıkmadan önce kaydetmek ister misiniz?",
                "💾  Kaydet ve Çık",
                "Kaydetmeden Çık",
            ),
            TlPending::NewProject => (
                "Kaydedilmemiş Değişiklikler",
                "Timeline'da kaydedilmemiş değişiklikler var.\nYeni proje açmadan önce kaydetmek ister misiniz?",
                "💾  Kaydet ve Yeni",
                "Kaydetmeden Yeni",
            ),
        };
        let mut act = 0;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(text);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(primary_button(save_label)).clicked() {
                        act = 1;
                    }
                    if ui.add(danger_button(discard_label)).clicked() {
                        act = 2;
                    }
                    if ui.add(secondary_button("İptal")).clicked() {
                        act = 3;
                    }
                });
            });
        match act {
            1 => {
                self.tl_save_project(false);
                if !self.timeline.is_dirty() {
                    self.tl_pending = None;
                    self.tl_finish_pending(pending, ctx);
                }
            }
            2 => {
                self.tl_pending = None;
                self.tl_finish_pending(pending, ctx);
            }
            3 => self.tl_pending = None,
            _ => {}
        }
    }

    fn tl_finish_pending(&mut self, pending: TlPending, ctx: &egui::Context) {
        match pending {
            TlPending::OpenProject => self.tl_open_project_force(ctx),
            TlPending::NewProject => self.tl_new_project_force(),
            TlPending::Close => {
                self.timeline.remove_autosave();
                self.tl_allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    // ------------------------------------------------------------ içe aktarma / eylemler
    pub fn tl_import_videos(&mut self, ctx: &egui::Context) {
        let files = rfd::FileDialog::new()
            .set_title("Video Seç")
            .add_filter(
                "Video Dosyaları",
                &["mp4", "avi", "mkv", "mov", "webm", "flv", "m4v", "wmv"],
            )
            .pick_files()
            .unwrap_or_default();
        if files.is_empty() {
            return;
        }
        let mut added = 0usize;
        for f in files {
            let path = f.to_string_lossy().to_string();
            if util::duration(&path) <= 0.05 {
                self.error(format!("Süre okunamadı:\n{path}"));
                continue;
            }
            self.timeline.add_clip(path);
            added += 1;
        }
        if added > 0 {
            self.timeline.regenerate_thumbs(ctx);
            self.tl_set_status(format!("✓ {added} video timeline'a eklendi."));
        }
    }

    pub fn tl_import_audio(&mut self) {
        if let Some(f) = rfd::FileDialog::new()
            .set_title("Ses Dosyası Seç")
            .add_filter(
                "Ses Dosyaları",
                &["mp3", "wav", "aac", "ogg", "flac", "m4a", "opus", "wma"],
            )
            .pick_file()
        {
            let path = f.to_string_lossy().to_string();
            let d = util::duration(&path);
            if d <= 0.05 {
                self.error("Ses dosyasının süresi okunamadı.");
                return;
            }
            let start = self
                .timeline
                .audios
                .iter()
                .map(|a| a.start_time + a.clip_duration())
                .fold(0.0, f64::max);
            let uid = self.timeline.add_audio(path);
            if let Some(a) = self.timeline.audios.iter_mut().find(|a| a.uid == uid) {
                a.start_time = start;
            }
            self.tl_set_status("✓ Ses dosyası timeline'a eklendi.");
        }
    }

    pub fn tl_new_text(&mut self) {
        let mut ed = crate::timeline::TextEditorState::new();
        ed.start = self.timeline.playhead as f32;
        ed.end = (self.timeline.playhead + 5.0) as f32;
        self.timeline.text_editor = Some(ed);
    }

    pub fn tl_new_asset(&mut self) {
        self.timeline.asset_editor = Some(crate::timeline::AssetEditorState::new());
    }

    pub fn tl_split(&mut self) {
        match self.timeline.selected {
            Selection::Clip(_) | Selection::Audio(_) => {
                self.timeline.split_selected();
                self.tl_set_status("✓ Seçili öğe oynatma kafasından bölündü.");
            }
            _ => self.info("Önce timeline'dan bir video veya ses seçin."),
        }
    }

    pub fn tl_delete_selected(&mut self) {
        if self.timeline.selected == Selection::None {
            self.info("Önce timeline'dan bir öğe seçin.");
            return;
        }
        self.timeline.delete_selected();
        self.tl_playing_clip = None;
        self.stop_preview_audio();
        self.tl_set_status("🗑 Seçili öğe silindi.");
    }

    pub fn tl_toggle_mute(&mut self) {
        let Some(c) = self.timeline.selected_clip_mut() else {
            self.info("Önce timeline'dan bir video klip seçin.");
            return;
        };
        c.mute = !c.mute;
        let muted = c.mute;
        self.tl_set_status(if muted {
            "🔇 Klip sesi dışa aktarmada kapatıldı."
        } else {
            "🔊 Klip sesi yeniden açıldı."
        });
    }

    pub fn tl_extract_audio(&mut self) {
        let Some(c) = self.timeline.selected_clip().cloned() else {
            self.info("Önce timeline'dan bir video klip seçin.");
            return;
        };
        let Some(out) = rfd::FileDialog::new()
            .set_title("Sesi Kaydet")
            .set_directory(&self.settings.recorder.output_dir)
            .set_file_name(format!(
                "{}_ses.mp3",
                Path::new(&c.path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "ses".into())
            ))
            .add_filter("MP3", &["mp3"])
            .add_filter("WAV", &["wav"])
            .add_filter("AAC", &["aac"])
            .save_file()
        else {
            return;
        };
        let out = out.to_string_lossy().to_string();
        let codec = if out.to_lowercase().ends_with(".mp3") {
            "libmp3lame"
        } else if out.to_lowercase().ends_with(".wav") {
            "pcm_s16le"
        } else {
            "aac"
        };
        let start = c.trim_start;
        let dur = (c.trim_end - c.trim_start).max(0.01);
        self.run_job(move || {
            let args = vec![
                "-y".into(),
                "-ss".into(),
                format!("{start:.3}"),
                "-t".into(),
                format!("{dur:.3}"),
                "-i".into(),
                c.path.clone(),
                "-vn".into(),
                "-c:a".into(),
                codec.into(),
                out.clone(),
            ];
            util::ffmpeg_ok(&args).map_err(|e| anyhow::anyhow!(e))?;
            Ok(format!("✓ Ses ayrıldı: {}", util::basename(&out)))
        });
    }

    pub fn tl_open_export(&mut self) {
        if self.timeline.clips.is_empty() {
            self.info("Timeline'a en az bir video ekleyin.");
            return;
        }
        if self.tl_export_out.trim().is_empty() {
            let dir = self
                .settings
                .recorder
                .output_dir
                .trim_end_matches(['/', '\\']);
            self.tl_export_out = format!("{dir}/timeline_{}.mp4", util::stamp());
        }
        self.tl_export_open = true;
    }

    pub fn open_transition_for_selected(&mut self) {
        let pairs = self.timeline.sorted_pairs();
        if pairs.is_empty() {
            self.info("Geçiş eklemek için timeline'da en az iki video olmalı.");
            return;
        }
        let mut pair = pairs[0];
        if let Selection::Clip(uid) = self.timeline.selected {
            if let Some(p) = pairs.iter().find(|(a, b)| *a == uid || *b == uid) {
                pair = *p;
            }
        }
        self.timeline.transition_pair = Some(pair);
        let t = self.timeline.transition(pair.0, pair.1);
        self.timeline.transition_dur = t.duration as f32;
        self.timeline.transition_sel = TRANSITIONS
            .iter()
            .position(|(_, f, _, _)| *f == t.effect)
            .unwrap_or(0);
    }
}

fn tool_btn(text: &str, fill: Color32) -> egui::Button<'static> {
    egui::Button::new(
        RichText::new(text.to_string())
            .size(12.0)
            .strong()
            .color(Color32::WHITE),
    )
    .fill(fill)
    .corner_radius(7)
    .min_size(egui::vec2(0.0, 30.0))
}

impl App {
    // ------------------------------------------------------------ zaman çizelgesi
    fn tl_timeline_canvas(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let dur = self.timeline.duration().max(10.0);
        let pps = self.timeline.px_per_sec.max(4.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("⏰ Timeline")
                    .size(13.0)
                    .strong()
                    .color(theme::txt()),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(secondary_button("➕"))
                    .on_hover_text("Yakınlaştır")
                    .clicked()
                {
                    self.timeline.px_per_sec = (pps * 1.5).min(300.0);
                }
                if ui
                    .add(secondary_button("➖"))
                    .on_hover_text("Uzaklaştır")
                    .clicked()
                {
                    self.timeline.px_per_sec = (pps / 1.5).max(8.0);
                }
                ui.label(RichText::new("Zoom:").size(11.0).color(theme::txt2()));
            });
        });

        let clips = self.timeline.sorted_clips();
        let audios = self.timeline.audios.clone();
        let texts = self.timeline.texts.clone();
        let assets = self.timeline.assets.clone();
        let transitions = self.timeline.transitions.clone();
        let thumbs: Vec<(Uid, egui::TextureHandle)> = self
            .timeline
            .thumbs
            .iter()
            .map(|(u, t)| (*u, t.clone()))
            .collect();
        let selected = self.timeline.selected;
        let playhead = self.timeline.playhead;

        theme::card_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::horizontal()
                .id_salt("tl_canvas_scroll")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    let total_w = (dur as f32 * pps + 160.0).max(ui.available_width());
                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(total_w, TL_HEIGHT),
                        egui::Sense::click_and_drag(),
                    );
                    let p = ui.painter_at(rect);
                    p.rect_filled(rect, 3.0, Color32::from_rgb(0x0d, 0x11, 0x17));

                    let row_bg = |p: &egui::Painter, y: f32, h: f32, c: Color32| {
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(rect.left(), rect.top() + y),
                                egui::vec2(rect.width(), h),
                            ),
                            0.0,
                            c,
                        );
                    };
                    row_bg(&p, 0.0, 22.0, Color32::from_rgb(0x1c, 0x21, 0x28));
                    row_bg(
                        &p,
                        ROW_CLIP_Y,
                        ROW_CLIP_H,
                        Color32::from_rgb(0x0d, 0x11, 0x17),
                    );
                    row_bg(
                        &p,
                        ROW_AUDIO_Y,
                        ROW_AUDIO_H,
                        Color32::from_rgb(0x0a, 0x1a, 0x15),
                    );
                    row_bg(
                        &p,
                        ROW_TEXT_Y,
                        ROW_TEXT_H,
                        Color32::from_rgb(0x1a, 0x15, 0x0a),
                    );
                    row_bg(
                        &p,
                        ROW_ASSET_Y,
                        ROW_ASSET_H,
                        Color32::from_rgb(0x0a, 0x0a, 0x1a),
                    );

                    for y in [
                        ROW_CLIP_Y,
                        ROW_AUDIO_Y,
                        ROW_TEXT_Y,
                        ROW_ASSET_Y,
                        ROW_ASSET_Y + ROW_ASSET_H,
                    ] {
                        p.line_segment(
                            [
                                egui::pos2(rect.left(), rect.top() + y),
                                egui::pos2(rect.right(), rect.top() + y),
                            ],
                            egui::Stroke::new(1.0_f32, theme::border()),
                        );
                    }

                    // Cetvel
                    let step = [0.5f64, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0]
                        .iter()
                        .copied()
                        .find(|s| *s as f32 * pps >= 58.0)
                        .unwrap_or(600.0);
                    let mut s = 0.0f64;
                    while s <= dur + step {
                        let x = rect.left() + s as f32 * pps;
                        p.line_segment(
                            [
                                egui::pos2(x, rect.top() + 15.0),
                                egui::pos2(x, rect.top() + 22.0),
                            ],
                            egui::Stroke::new(1.0_f32, theme::border()),
                        );
                        p.text(
                            egui::pos2(x + 2.0, rect.top() + 2.0),
                            egui::Align2::LEFT_TOP,
                            util::mmss(s),
                            egui::FontId::proportional(9.0),
                            theme::txt2(),
                        );
                        s += step;
                    }

                    // Video klipleri
                    let palette = [
                        theme::accent2(),
                        theme::purple(),
                        Color32::from_rgb(0x2d, 0x6a, 0x4f),
                        Color32::from_rgb(0x6a, 0x2d, 0x4f),
                        Color32::from_rgb(0x4f, 0x6a, 0x2d),
                        Color32::from_rgb(0x6a, 0x4f, 0x2d),
                    ];
                    for (i, c) in clips.iter().enumerate() {
                        let x0 = rect.left() + c.timeline_start as f32 * pps;
                        let x1 = rect.left() + c.timeline_end() as f32 * pps;
                        let r = egui::Rect::from_min_size(
                            egui::pos2(x0, rect.top() + ROW_CLIP_Y),
                            egui::vec2((x1 - x0).max(2.0), ROW_CLIP_H),
                        );
                        let is_sel = selected == Selection::Clip(c.uid);
                        p.rect_filled(r, 3.0, palette[i % palette.len()]);
                        p.rect_stroke(
                            r,
                            3.0,
                            egui::Stroke::new(
                                if is_sel { 2.0 } else { 1.0_f32 },
                                if is_sel {
                                    Color32::WHITE
                                } else {
                                    theme::border()
                                },
                            ),
                            egui::StrokeKind::Inside,
                        );
                        let mut tx = x0 + 5.0;
                        if let Some((_, tex)) = thumbs.iter().find(|(u, _)| *u == c.uid) {
                            let tw = 36.0f32.min((x1 - x0 - 10.0).max(0.0));
                            if tw > 6.0 {
                                let tr = egui::Rect::from_min_size(
                                    egui::pos2(x0 + 4.0, rect.top() + ROW_CLIP_Y + 9.0),
                                    egui::vec2(tw, tw * 0.75),
                                );
                                p.image(
                                    tex.id(),
                                    tr,
                                    egui::Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    Color32::WHITE,
                                );
                                tx = x0 + 4.0 + tw + 4.0;
                            }
                        }
                        let mut label = util::shorten(&c.name, 18);
                        if c.mute {
                            label = format!("🔇 {label}");
                        }
                        p.text(
                            egui::pos2(tx, rect.top() + ROW_CLIP_Y + 3.0),
                            egui::Align2::LEFT_TOP,
                            label,
                            egui::FontId::proportional(9.5),
                            Color32::WHITE,
                        );
                        p.text(
                            egui::pos2(tx, rect.top() + ROW_CLIP_Y + 18.0),
                            egui::Align2::LEFT_TOP,
                            format!(
                                "{}{}",
                                util::mmss(c.clip_duration()),
                                if (c.speed - 1.0).abs() > 0.001 {
                                    format!("   {}×", c.speed)
                                } else {
                                    String::new()
                                }
                            ),
                            egui::FontId::proportional(9.0),
                            Color32::from_rgb(0xc8, 0xd0, 0xda),
                        );
                        if c.scale_w > 0 && c.scale_h > 0 {
                            p.text(
                                egui::pos2(tx, rect.top() + ROW_CLIP_Y + 32.0),
                                egui::Align2::LEFT_TOP,
                                format!("↔ {}×{}", c.scale_w, c.scale_h),
                                egui::FontId::proportional(8.0),
                                Color32::from_rgb(0xc8, 0xd0, 0xda),
                            );
                        }
                        let hc = if is_sel {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(0x88, 0x88, 0x88)
                        };
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(x0, rect.top() + ROW_CLIP_Y),
                                egui::vec2(6.0, ROW_CLIP_H),
                            ),
                            0.0,
                            hc,
                        );
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2((x1 - 6.0).max(x0), rect.top() + ROW_CLIP_Y),
                                egui::vec2(6.0, ROW_CLIP_H),
                            ),
                            0.0,
                            hc,
                        );
                    }

                    // Geçiş işaretleri
                    for w in clips.windows(2) {
                        let a = &w[0];
                        let b = &w[1];
                        let jx = rect.left() + a.timeline_end() as f32 * pps;
                        let t = transitions
                            .get(&(a.uid, b.uid))
                            .cloned()
                            .unwrap_or_else(Transition::cut);
                        let mid = rect.top() + ROW_CLIP_Y + ROW_CLIP_H * 0.5;
                        if t.is_cut() {
                            let mut yy = rect.top() + ROW_CLIP_Y + 4.0;
                            while yy < rect.top() + ROW_CLIP_Y + ROW_CLIP_H - 4.0 {
                                p.line_segment(
                                    [
                                        egui::pos2(jx, yy),
                                        egui::pos2(
                                            jx,
                                            (yy + 4.0)
                                                .min(rect.top() + ROW_CLIP_Y + ROW_CLIP_H - 4.0),
                                        ),
                                    ],
                                    egui::Stroke::new(1.5_f32, Color32::from_white_alpha(160)),
                                );
                                yy += 8.0;
                            }
                        } else {
                            let r = 9.0;
                            let pts = vec![
                                egui::pos2(jx, mid - r),
                                egui::pos2(jx + r, mid),
                                egui::pos2(jx, mid + r),
                                egui::pos2(jx - r, mid),
                            ];
                            p.add(egui::Shape::convex_polygon(
                                pts,
                                Color32::from_rgb(0xf5, 0x9e, 0x0b),
                                egui::Stroke::new(1.0_f32, Color32::WHITE),
                            ));
                            p.text(
                                egui::pos2(jx, mid - 2.0),
                                egui::Align2::CENTER_CENTER,
                                "",
                                egui::FontId::proportional(8.0),
                                Color32::BLACK,
                            );
                            p.text(
                                egui::pos2(jx, rect.top() + ROW_CLIP_Y + ROW_CLIP_H + 1.0),
                                egui::Align2::CENTER_TOP,
                                format!("{:.1}s", t.duration),
                                egui::FontId::proportional(8.0),
                                Color32::from_rgb(0xf5, 0x9e, 0x0b),
                            );
                        }
                    }

                    // Ses katmanları
                    for a in &audios {
                        let x0 = rect.left() + a.start_time as f32 * pps;
                        let x1 = x0 + a.clip_duration() as f32 * pps;
                        let r = egui::Rect::from_min_size(
                            egui::pos2(x0, rect.top() + ROW_AUDIO_Y),
                            egui::vec2((x1 - x0).max(2.0), ROW_AUDIO_H),
                        );
                        let is_sel = selected == Selection::Audio(a.uid);
                        p.rect_filled(r, 2.0, Color32::from_rgb(0x2d, 0x6a, 0x4f));
                        p.rect_stroke(
                            r,
                            2.0,
                            egui::Stroke::new(
                                if is_sel { 2.0 } else { 1.0_f32 },
                                if is_sel {
                                    Color32::WHITE
                                } else {
                                    theme::border()
                                },
                            ),
                            egui::StrokeKind::Inside,
                        );
                        p.text(
                            egui::pos2(x0 + 4.0, rect.top() + ROW_AUDIO_Y + 4.0),
                            egui::Align2::LEFT_TOP,
                            format!(
                                "🎵 {}  {:.1}→{:.1}s",
                                util::shorten(&a.name, 18),
                                a.trim_start,
                                a.trim_end
                            ),
                            egui::FontId::proportional(8.5),
                            Color32::WHITE,
                        );
                        let hc = if is_sel {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(0x55, 0x55, 0x55)
                        };
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(x0, rect.top() + ROW_AUDIO_Y),
                                egui::vec2(5.0, ROW_AUDIO_H),
                            ),
                            0.0,
                            hc,
                        );
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2((x1 - 5.0).max(x0), rect.top() + ROW_AUDIO_Y),
                                egui::vec2(5.0, ROW_AUDIO_H),
                            ),
                            0.0,
                            hc,
                        );
                    }

                    // Metin katmanları
                    for t in &texts {
                        let x0 = rect.left() + t.start_time as f32 * pps;
                        let x1 = rect.left() + t.end_time as f32 * pps;
                        let r = egui::Rect::from_min_size(
                            egui::pos2(x0, rect.top() + ROW_TEXT_Y),
                            egui::vec2((x1 - x0).max(2.0), ROW_TEXT_H),
                        );
                        let is_sel = selected == Selection::Text(t.uid);
                        p.rect_filled(
                            r,
                            2.0,
                            if is_sel {
                                Color32::from_rgb(0xf5, 0x9e, 0x0b)
                            } else {
                                Color32::from_rgb(0x92, 0x40, 0x0e)
                            },
                        );
                        p.rect_stroke(
                            r,
                            2.0,
                            egui::Stroke::new(
                                if is_sel { 2.0 } else { 1.0_f32 },
                                if is_sel {
                                    Color32::from_rgb(0xfb, 0xbf, 0x24)
                                } else {
                                    Color32::from_rgb(0x78, 0x35, 0x0f)
                                },
                            ),
                            egui::StrokeKind::Inside,
                        );
                        p.text(
                            egui::pos2(x0 + 4.0, rect.top() + ROW_TEXT_Y + 3.0),
                            egui::Align2::LEFT_TOP,
                            format!("T  {}", util::shorten(&t.text, 14)),
                            egui::FontId::proportional(8.5),
                            Color32::from_rgb(0xfe, 0xf3, 0xc7),
                        );
                        let hc = if is_sel {
                            Color32::from_rgb(0xfb, 0xbf, 0x24)
                        } else {
                            Color32::from_rgb(0x78, 0x35, 0x0f)
                        };
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(x0, rect.top() + ROW_TEXT_Y),
                                egui::vec2(4.0, ROW_TEXT_H),
                            ),
                            0.0,
                            hc,
                        );
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2((x1 - 4.0).max(x0), rect.top() + ROW_TEXT_Y),
                                egui::vec2(4.0, ROW_TEXT_H),
                            ),
                            0.0,
                            hc,
                        );
                    }

                    // Şekil / görsel katmanları
                    for a in &assets {
                        let x0 = rect.left() + a.start_time as f32 * pps;
                        let x1 = rect.left() + a.end_time as f32 * pps;
                        let r = egui::Rect::from_min_size(
                            egui::pos2(x0, rect.top() + ROW_ASSET_Y),
                            egui::vec2((x1 - x0).max(2.0), ROW_ASSET_H),
                        );
                        let is_sel = selected == Selection::Asset(a.uid);
                        let base = match a.kind {
                            AssetKind::Rect => Color32::from_rgb(0x1d, 0x4e, 0xd8),
                            AssetKind::Circle => Color32::from_rgb(0x6d, 0x28, 0xd9),
                            AssetKind::Line => Color32::from_rgb(0x0e, 0x74, 0x90),
                            AssetKind::Image => Color32::from_rgb(0x06, 0x5f, 0x46),
                        };
                        p.rect_filled(
                            r,
                            2.0,
                            if is_sel {
                                Color32::from_rgb(0x3b, 0x82, 0xf6)
                            } else {
                                base
                            },
                        );
                        p.rect_stroke(
                            r,
                            2.0,
                            egui::Stroke::new(
                                if is_sel { 2.0 } else { 1.0_f32 },
                                if is_sel {
                                    Color32::from_rgb(0x93, 0xc5, 0xfd)
                                } else {
                                    Color32::from_rgb(0x1e, 0x3a, 0x8a)
                                },
                            ),
                            egui::StrokeKind::Inside,
                        );
                        p.text(
                            egui::pos2(x0 + 4.0, rect.top() + ROW_ASSET_Y + 3.0),
                            egui::Align2::LEFT_TOP,
                            util::shorten(&a.name(), 18),
                            egui::FontId::proportional(8.5),
                            Color32::from_rgb(0xdb, 0xea, 0xfe),
                        );
                        let hc = if is_sel {
                            Color32::from_rgb(0x93, 0xc5, 0xfd)
                        } else {
                            Color32::from_rgb(0x1e, 0x3a, 0x8a)
                        };
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(x0, rect.top() + ROW_ASSET_Y),
                                egui::vec2(4.0, ROW_ASSET_H),
                            ),
                            0.0,
                            hc,
                        );
                        p.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2((x1 - 4.0).max(x0), rect.top() + ROW_ASSET_Y),
                                egui::vec2(4.0, ROW_ASSET_H),
                            ),
                            0.0,
                            hc,
                        );
                    }

                    // Playhead
                    let px = rect.left() + playhead as f32 * pps;
                    p.line_segment(
                        [egui::pos2(px, rect.top()), egui::pos2(px, rect.bottom())],
                        egui::Stroke::new(2.0_f32, theme::red()),
                    );
                    p.add(egui::Shape::convex_polygon(
                        vec![
                            egui::pos2(px - 7.0, rect.top()),
                            egui::pos2(px + 7.0, rect.top()),
                            egui::pos2(px, rect.top() + 10.0),
                        ],
                        theme::red(),
                        egui::Stroke::NONE,
                    ));

                    let mut dbl = false;
                    if resp.double_clicked() {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            let x = pos.x - rect.left();
                            let mut hit_pair = None;
                            for w in clips.windows(2) {
                                let jx = w[0].timeline_end() as f32 * pps;
                                if (x - jx).abs() <= 14.0 {
                                    hit_pair = Some((w[0].uid, w[1].uid));
                                    break;
                                }
                            }
                            if let Some(pair) = hit_pair {
                                self.timeline.transition_pair = Some(pair);
                                let tr = self.timeline.transition(pair.0, pair.1);
                                self.timeline.transition_dur = tr.duration as f32;
                                self.timeline.transition_sel = TRANSITIONS
                                    .iter()
                                    .position(|(_, f, _, _)| *f == tr.effect)
                                    .unwrap_or(0);
                                dbl = true;
                            }
                        }
                    }

                    if resp.drag_started() {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            self.tl_drag_begin(pos, rect);
                        }
                    }
                    if resp.dragged() {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            self.tl_drag_update(pos, rect);
                        }
                    }
                    if resp.drag_stopped() {
                        if self.tl_overlap_warn {
                            self.tl_set_status(
                                "⚠ Klip üst üste binemez! En yakın geçerli konuma yapıştırıldı.",
                            );
                            self.tl_overlap_warn = false;
                        }
                        self.tl_drag = None;
                        self.timeline.regenerate_thumbs(ctx);
                    }
                    if resp.clicked() && !dbl {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            self.tl_canvas_click(pos, rect);
                        }
                    }
                });
        });
    }

    fn tl_drag_begin(&mut self, pos: egui::Pos2, rect: egui::Rect) {
        let x = pos.x - rect.left();
        let y = pos.y - rect.top();
        let pps = self.timeline.px_per_sec.max(4.0);
        let t = (x / pps).max(0.0) as f64;

        if y >= ROW_CLIP_Y && y <= ROW_CLIP_Y + ROW_CLIP_H {
            let hit = self.timeline.clips.iter().rev().find(|c| {
                let x0 = c.timeline_start as f32 * pps;
                let x1 = c.timeline_end() as f32 * pps;
                x0 - 2.0 <= x && x <= x1 + 2.0
            });
            if let Some(c) = hit {
                let x0 = c.timeline_start as f32 * pps;
                let x1 = c.timeline_end() as f32 * pps;
                let mode = if x <= x0 + 7.0 {
                    timeline::DragMode::TrimL
                } else if x >= x1 - 7.0 {
                    timeline::DragMode::TrimR
                } else {
                    timeline::DragMode::Move
                };
                let (uid, offset) = (c.uid, x - x0);
                self.timeline.selected = Selection::Clip(uid);
                self.tl_drag = Some(TimelineDrag {
                    kind: 0,
                    uid,
                    mode,
                    offset,
                });
                return;
            }
        } else if y >= ROW_AUDIO_Y && y <= ROW_AUDIO_Y + ROW_AUDIO_H {
            let hit = self.timeline.audios.iter().rev().find(|a| {
                let x0 = a.start_time as f32 * pps;
                let x1 = x0 + a.clip_duration() as f32 * pps;
                x0 - 2.0 <= x && x <= x1 + 2.0
            });
            if let Some(a) = hit {
                let x0 = a.start_time as f32 * pps;
                let x1 = x0 + a.clip_duration() as f32 * pps;
                let mode = if x <= x0 + 6.0 {
                    timeline::DragMode::TrimL
                } else if x >= x1 - 6.0 {
                    timeline::DragMode::TrimR
                } else {
                    timeline::DragMode::Move
                };
                let (uid, offset) = (a.uid, x - x0);
                self.timeline.selected = Selection::Audio(uid);
                self.tl_drag = Some(TimelineDrag {
                    kind: 1,
                    uid,
                    mode,
                    offset,
                });
                return;
            }
        } else if y >= ROW_TEXT_Y && y <= ROW_TEXT_Y + ROW_TEXT_H {
            let hit = self.timeline.texts.iter().rev().find(|o| {
                let x0 = o.start_time as f32 * pps;
                let x1 = o.end_time as f32 * pps;
                x0 - 2.0 <= x && x <= x1 + 2.0
            });
            if let Some(o) = hit {
                let x0 = o.start_time as f32 * pps;
                let x1 = o.end_time as f32 * pps;
                let mode = if x <= x0 + 5.0 {
                    timeline::DragMode::TrimL
                } else if x >= x1 - 5.0 {
                    timeline::DragMode::TrimR
                } else {
                    timeline::DragMode::Move
                };
                let (uid, offset) = (o.uid, x - x0);
                self.timeline.selected = Selection::Text(uid);
                self.tl_drag = Some(TimelineDrag {
                    kind: 2,
                    uid,
                    mode,
                    offset,
                });
                return;
            }
        } else if y >= ROW_ASSET_Y && y <= ROW_ASSET_Y + ROW_ASSET_H {
            let hit = self.timeline.assets.iter().rev().find(|a| {
                let x0 = a.start_time as f32 * pps;
                let x1 = a.end_time as f32 * pps;
                x0 - 2.0 <= x && x <= x1 + 2.0
            });
            if let Some(a) = hit {
                let x0 = a.start_time as f32 * pps;
                let x1 = a.end_time as f32 * pps;
                let mode = if x <= x0 + 5.0 {
                    timeline::DragMode::TrimL
                } else if x >= x1 - 5.0 {
                    timeline::DragMode::TrimR
                } else {
                    timeline::DragMode::Move
                };
                let (uid, offset) = (a.uid, x - x0);
                self.timeline.selected = Selection::Asset(uid);
                self.tl_drag = Some(TimelineDrag {
                    kind: 3,
                    uid,
                    mode,
                    offset,
                });
                return;
            }
        }
        self.tl_seek(t);
    }

    fn tl_drag_update(&mut self, pos: egui::Pos2, rect: egui::Rect) {
        let Some(drag) = self.tl_drag else { return };
        let pps = self.timeline.px_per_sec.max(4.0);
        let x = (pos.x - rect.left()).max(0.0);
        let t = (x / pps) as f64;
        let mut warn = false;

        match drag.kind {
            0 => {
                let others: Vec<(Uid, f64, f64)> = self
                    .timeline
                    .clips
                    .iter()
                    .filter(|c| c.uid != drag.uid)
                    .map(|c| (c.uid, c.timeline_start, c.timeline_end()))
                    .collect();
                if let Some(c) = self.timeline.clips.iter_mut().find(|c| c.uid == drag.uid) {
                    match drag.mode {
                        timeline::DragMode::Move => {
                            let dur = c.clip_duration();
                            let prop = (t - drag.offset as f64 / pps as f64).max(0.0);
                            let (s, w) = resolve_placement(&others, prop, dur, pps);
                            c.timeline_start = s;
                            warn = w;
                        }
                        timeline::DragMode::TrimL => {
                            let speed = c.speed.max(0.01) as f64;
                            let min_start = (c.timeline_start - c.trim_start / speed).max(0.0);
                            let max_start = (c.timeline_end() - 0.1).max(min_start);
                            let nt = t.clamp(min_start, max_start);
                            let diff = nt - c.timeline_start;
                            c.trim_start = (c.trim_start + diff * speed)
                                .clamp(0.0, (c.trim_end - 0.1).max(0.0));
                            c.timeline_start = nt;
                        }
                        timeline::DragMode::TrimR => {
                            let speed = c.speed.max(0.01) as f64;
                            let min_end = c.timeline_start + 0.1;
                            let max_end =
                                c.timeline_start + ((c.duration - c.trim_start).max(0.1) / speed);
                            let ne = t.clamp(min_end, max_end.max(min_end));
                            let diff = ne - c.timeline_end();
                            c.trim_end = (c.trim_end + diff * speed)
                                .clamp(c.trim_start + 0.1, c.duration.max(c.trim_start + 0.1));
                        }
                    }
                }
            }
            1 => {
                let others: Vec<(Uid, f64, f64)> = self
                    .timeline
                    .audios
                    .iter()
                    .filter(|a| a.uid != drag.uid)
                    .map(|a| (a.uid, a.start_time, a.start_time + a.clip_duration()))
                    .collect();
                if let Some(a) = self.timeline.audios.iter_mut().find(|a| a.uid == drag.uid) {
                    match drag.mode {
                        timeline::DragMode::Move => {
                            let dur = a.clip_duration();
                            let prop = (t - drag.offset as f64 / pps as f64).max(0.0);
                            let (s, w) = resolve_placement(&others, prop, dur, pps);
                            a.start_time = s;
                            warn = w;
                        }
                        timeline::DragMode::TrimL => {
                            let min_start = (a.start_time - a.trim_start).max(0.0);
                            let max_start = (a.start_time + a.clip_duration() - 0.1).max(min_start);
                            let nt = t.clamp(min_start, max_start);
                            let diff = nt - a.start_time;
                            a.trim_start =
                                (a.trim_start + diff).clamp(0.0, (a.trim_end - 0.1).max(0.0));
                            a.start_time = nt;
                        }
                        timeline::DragMode::TrimR => {
                            let min_end = a.start_time + 0.1;
                            let max_end = a.start_time + (a.duration - a.trim_start).max(0.1);
                            let ne = t.clamp(min_end, max_end.max(min_end));
                            let diff = ne - (a.start_time + a.clip_duration());
                            a.trim_end = (a.trim_end + diff)
                                .clamp(a.trim_start + 0.1, a.duration.max(a.trim_start + 0.1));
                        }
                    }
                }
            }
            2 => {
                if let Some(o) = self.timeline.texts.iter_mut().find(|o| o.uid == drag.uid) {
                    match drag.mode {
                        timeline::DragMode::Move => {
                            let dur = o.end_time - o.start_time;
                            o.start_time = (t - drag.offset as f64 / pps as f64).max(0.0);
                            o.end_time = o.start_time + dur;
                        }
                        timeline::DragMode::TrimL => {
                            o.start_time = t.min(o.end_time - 0.1).max(0.0);
                        }
                        timeline::DragMode::TrimR => {
                            o.end_time = t.max(o.start_time + 0.1);
                        }
                    }
                }
            }
            _ => {
                if let Some(a) = self.timeline.assets.iter_mut().find(|a| a.uid == drag.uid) {
                    match drag.mode {
                        timeline::DragMode::Move => {
                            let dur = a.end_time - a.start_time;
                            a.start_time = (t - drag.offset as f64 / pps as f64).max(0.0);
                            a.end_time = a.start_time + dur;
                        }
                        timeline::DragMode::TrimL => {
                            a.start_time = t.min(a.end_time - 0.1).max(0.0);
                        }
                        timeline::DragMode::TrimR => {
                            a.end_time = t.max(a.start_time + 0.1);
                        }
                    }
                }
            }
        }
        if warn {
            self.tl_overlap_warn = true;
        }
    }

    fn tl_canvas_click(&mut self, pos: egui::Pos2, rect: egui::Rect) {
        let x = (pos.x - rect.left()).max(0.0);
        let y = pos.y - rect.top();
        let pps = self.timeline.px_per_sec.max(4.0);
        let t = (x / pps) as f64;

        if y < 22.0 {
            self.tl_seek(t);
            return;
        }
        if y >= ROW_CLIP_Y && y <= ROW_CLIP_Y + ROW_CLIP_H {
            if let Some(c) = self.timeline.clips.iter().rev().find(|c| {
                let x0 = c.timeline_start as f32 * pps;
                let x1 = c.timeline_end() as f32 * pps;
                x0 <= x && x <= x1
            }) {
                self.timeline.selected = Selection::Clip(c.uid);
                self.tl_tab = 0;
                return;
            }
        } else if y >= ROW_AUDIO_Y && y <= ROW_AUDIO_Y + ROW_AUDIO_H {
            if let Some(a) = self.timeline.audios.iter().rev().find(|a| {
                let x0 = a.start_time as f32 * pps;
                let x1 = x0 + a.clip_duration() as f32 * pps;
                x0 <= x && x <= x1
            }) {
                self.timeline.selected = Selection::Audio(a.uid);
                self.tl_tab = 2;
                return;
            }
        } else if y >= ROW_TEXT_Y && y <= ROW_TEXT_Y + ROW_TEXT_H {
            if let Some(o) = self.timeline.texts.iter().rev().find(|o| {
                let x0 = o.start_time as f32 * pps;
                let x1 = o.end_time as f32 * pps;
                x0 <= x && x <= x1
            }) {
                self.timeline.selected = Selection::Text(o.uid);
                self.tl_tab = 1;
                return;
            }
        } else if y >= ROW_ASSET_Y && y <= ROW_ASSET_Y + ROW_ASSET_H {
            if let Some(a) = self.timeline.assets.iter().rev().find(|a| {
                let x0 = a.start_time as f32 * pps;
                let x1 = a.end_time as f32 * pps;
                x0 <= x && x <= x1
            }) {
                self.timeline.selected = Selection::Asset(a.uid);
                self.tl_tab = 4;
                return;
            }
        }
        self.timeline.selected = Selection::None;
        self.tl_seek(t);
    }
}

impl App {
    // ------------------------------------------------------------ özellik paneli
    fn tl_props(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.label(
            RichText::new("⚙  Özellikler")
                .size(15.0)
                .strong()
                .color(theme::txt()),
        );
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            let tabs = ["🎬 Klip", "📝 Metin", "🎵 Ses", "🎬 Geçiş", "🔷 Şekiller"];
            for (i, t) in tabs.iter().enumerate() {
                if ui
                    .selectable_label(self.tl_tab == i, RichText::new(*t).size(11.5).strong())
                    .clicked()
                {
                    self.tl_tab = i;
                }
            }
        });
        ui.separator();

        match self.tl_tab {
            0 => self.tl_props_clip(ui),
            1 => self.tl_props_text(ui),
            2 => self.tl_props_audio(ui),
            3 => self.tl_props_transition(ui),
            _ => self.tl_props_asset(ui),
        }

        ui.add_space(6.0);
        ui.separator();
        ui.label(
            RichText::new(format!(
                "⏱ Süre: {}   •   🎬 {} klip   •   🎵 {} ses   •   📝 {} metin   •   🔷 {} şekil",
                util::mmss(self.timeline.duration()),
                self.timeline.clips.len(),
                self.timeline.audios.len(),
                self.timeline.texts.len(),
                self.timeline.assets.len()
            ))
            .size(10.5)
            .color(theme::txt2()),
        );
        self.show_job_status(ui);
    }

    fn tl_props_clip(&mut self, ui: &mut egui::Ui) {
        let uid = match self.timeline.selected {
            Selection::Clip(u) => Some(u),
            _ => None,
        };
        let Some(uid) = uid else {
            muted(
                ui,
                "Bir video klip seçmek için zaman çizelgesinde klibe tıklayın.",
            );
            if ui.add(primary_button("📂 Video Ekle")).clicked() {
                self.tl_import_videos(ui.ctx());
            }
            return;
        };
        let Some(c) = self.timeline.clips.iter_mut().find(|c| c.uid == uid) else {
            self.timeline.selected = Selection::None;
            return;
        };

        ui.label(
            RichText::new(format!("🎬 {}", util::shorten(&c.name, 28)))
                .size(13.0)
                .strong(),
        );
        muted(
            ui,
            format!(
                "Kaynak süresi: {:.2} sn   •   Timeline: {:.2} – {:.2}",
                c.duration,
                c.timeline_start,
                c.timeline_end()
            ),
        );

        let duration = c.duration;
        let clip_dur = c.clip_duration();
        let mut changed = false;
        card(ui, |ui| {
            ui.label(
                RichText::new("✂ Video Kırpma")
                    .size(12.0)
                    .strong()
                    .color(theme::accent()),
            );
            ui.horizontal(|ui| {
                ui.label("Başlangıç (sn):");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut c.trim_start)
                            .range(0.0..=duration)
                            .speed(0.01)
                            .fixed_decimals(2),
                    )
                    .changed();
                ui.label("Bitiş (sn):");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut c.trim_end)
                            .range(0.0..=duration)
                            .speed(0.01)
                            .fixed_decimals(2),
                    )
                    .changed();
            });
            if c.trim_end <= c.trim_start {
                ui.label(
                    RichText::new("⚠ Bitiş, başlangıçtan büyük olmalı.")
                        .size(11.0)
                        .color(theme::red()),
                );
            }
            muted(ui, format!("Klibin timeline süresi: {clip_dur:.2} sn"));
            ui.add_space(4.0);

            ui.label(
                RichText::new("⚡ Hız")
                    .size(12.0)
                    .strong()
                    .color(theme::accent()),
            );
            changed |= ui
                .add(
                    egui::Slider::new(&mut c.speed, 0.25..=4.0)
                        .fixed_decimals(2)
                        .suffix("×"),
                )
                .changed();
            ui.horizontal(|ui| {
                for s in [0.5f32, 1.0, 1.5, 2.0] {
                    if ui.add(secondary_button(&format!("{s}×"))).clicked() {
                        c.speed = s;
                        changed = true;
                    }
                }
            });
            ui.add_space(4.0);

            ui.label(
                RichText::new("📐 Boyut / Ölçek")
                    .size(12.0)
                    .strong()
                    .color(theme::accent()),
            );
            ui.horizontal(|ui| {
                ui.label("Genişlik:");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut c.scale_w)
                            .range(0..=7680)
                            .speed(1.0),
                    )
                    .changed();
                ui.label("Yükseklik:");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut c.scale_h)
                            .range(0..=4320)
                            .speed(1.0),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                if ui.add(secondary_button("1080p")).clicked() {
                    c.scale_w = 1920;
                    c.scale_h = 1080;
                    changed = true;
                }
                if ui.add(secondary_button("720p")).clicked() {
                    c.scale_w = 1280;
                    c.scale_h = 720;
                    changed = true;
                }
                if ui.add(secondary_button("480p")).clicked() {
                    c.scale_w = 854;
                    c.scale_h = 480;
                    changed = true;
                }
                if ui.add(secondary_button("Orijinal")).clicked() {
                    c.scale_w = 0;
                    c.scale_h = 0;
                    changed = true;
                }
            });
            muted(ui, "(0 = Orijinal boyutu koru)");

            let muted_clip = c.mute;
            if ui
                .add(if muted_clip {
                    danger_button("🔊 Sesi Aç")
                } else {
                    secondary_button("🔇 Sesi Sil")
                })
                .clicked()
            {
                c.mute = !c.mute;
                changed = true;
            }
        });
        if changed {
            if let Some(cc) = self.timeline.clips.iter_mut().find(|x| x.uid == uid) {
                if cc.trim_end <= cc.trim_start + 0.05 {
                    cc.trim_end = (cc.trim_start + 0.1).min(cc.duration.max(cc.trim_start + 0.1));
                }
                if cc.trim_start < 0.0 {
                    cc.trim_start = 0.0;
                }
                if cc.speed < 0.25 {
                    cc.speed = 0.25;
                }
            }
            self.tl_seek(self.timeline.playhead);
        }

        ui.add_space(6.0);
        if ui
            .add(
                primary_button("🎨  Efekt & Filtre Aç")
                    .min_size(egui::vec2(ui.available_width(), 34.0)),
            )
            .clicked()
        {
            self.timeline.effects_open = true;
            self.timeline.effects_uid = Some(uid);
            self.timeline.effects_tex = None;
            self.timeline.effects_key = None;
            self.tl_fx_dirty = Some(Instant::now());
        }
        ui.horizontal(|ui| {
            if ui.add(secondary_button("🔪 Böl")).clicked() {
                self.timeline.split_selected();
            }
            if ui.add(secondary_button("🎵 Sesi Ayır")).clicked() {
                self.tl_extract_audio();
            }
            if ui.add(danger_button("🚮 Sil")).clicked() {
                self.timeline.delete_selected();
                self.tl_playing_clip = None;
                self.stop_preview_audio();
            }
        });

        if let Some(c) = self.timeline.clips.iter().find(|c| c.uid == uid) {
            let mut active = Vec::new();
            if c.effects.brightness != 0.0 {
                active.push("parlaklık");
            }
            if c.effects.contrast != 1.0 {
                active.push("kontrast");
            }
            if c.effects.saturation != 1.0 {
                active.push("doygunluk");
            }
            if c.effects.blur > 0.0 {
                active.push("blur");
            }
            if c.effects.bw {
                active.push("S/B");
            }
            if c.effects.sepia {
                active.push("sepya");
            }
            if c.effects.mirror {
                active.push("ayna");
            }
            if c.effects.rotate {
                active.push("döndür");
            }
            if c.effects.vignette {
                active.push("vignette");
            }
            if active.is_empty() {
                muted(ui, "Aktif efekt yok.");
            } else {
                muted(ui, format!("Aktif: {}", active.join(", ")));
            }
        }
    }

    fn tl_props_text(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(
                primary_button("➕  Yeni Metin Ekle")
                    .min_size(egui::vec2(ui.available_width(), 32.0)),
            )
            .clicked()
        {
            self.tl_new_text();
        }
        ui.add_space(4.0);

        let list: Vec<(Uid, String, f64, f64, i32, i32)> = self
            .timeline
            .texts
            .iter()
            .map(|o| (o.uid, o.text.clone(), o.start_time, o.end_time, o.x, o.y))
            .collect();
        if list.is_empty() {
            muted(ui, "Henüz metin katmanı eklenmedi.");
        }
        for (uid, text, s, e, x, y) in list {
            let sel = self.timeline.selected == Selection::Text(uid);
            theme::dark_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("📝 {}", util::shorten(&text, 20)))
                                .size(11.5)
                                .color(if sel { theme::accent() } else { theme::txt() }),
                        );
                        muted(ui, format!("{s:.1}→{e:.1}s   [{x},{y}]"));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(secondary_button("🗑")).clicked() {
                            self.timeline.texts.retain(|o| o.uid != uid);
                            if self.timeline.selected == Selection::Text(uid) {
                                self.timeline.selected = Selection::None;
                            }
                        }
                        if ui.add(secondary_button("✏")).clicked() {
                            if let Some(o) = self.timeline.texts.iter().find(|o| o.uid == uid) {
                                self.timeline.text_editor =
                                    Some(crate::timeline::TextEditorState::from_overlay(o));
                            }
                        }
                        if ui.add(secondary_button("👁")).clicked() {
                            self.timeline.selected = Selection::Text(uid);
                            self.tl_tab = 1;
                        }
                    });
                });
            });
        }

        if let Selection::Text(uid) = self.timeline.selected {
            if let Some(o) = self.timeline.texts.iter_mut().find(|o| o.uid == uid) {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("🗒 Seçili Metin")
                        .size(12.0)
                        .strong()
                        .color(theme::accent()),
                );
                card(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut o.text)
                            .char_limit(500)
                            .desired_width(f32::INFINITY),
                    );
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut o.x).range(0..=1920));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut o.y).range(0..=1080));
                    });
                    ui.add(egui::Slider::new(&mut o.size, 12..=120).text("Yazı boyutu"));
                    let mut col = preview_color(&o.color);
                    if ui.color_edit_button_srgba(&mut col).changed() {
                        o.color = util::rgb_hex(col.r(), col.g(), col.b());
                    }
                    ui.horizontal(|ui| {
                        ui.label("Başlangıç:");
                        ui.add(
                            egui::DragValue::new(&mut o.start_time)
                                .range(0.0..=36000.0)
                                .speed(0.05)
                                .fixed_decimals(2),
                        );
                        ui.label("Bitiş:");
                        ui.add(
                            egui::DragValue::new(&mut o.end_time)
                                .range(0.0..=36000.0)
                                .speed(0.05)
                                .fixed_decimals(2),
                        );
                    });
                });
                if ui.add(secondary_button("✏  Ayrıntılı Düzenle")).clicked() {
                    if let Some(ov) = self.timeline.texts.iter().find(|x| x.uid == uid) {
                        self.timeline.text_editor =
                            Some(crate::timeline::TextEditorState::from_overlay(ov));
                    }
                }
            }
        }
    }

    fn tl_props_audio(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(primary_button("➕  Ses Ekle").min_size(egui::vec2(ui.available_width(), 32.0)))
            .clicked()
        {
            self.tl_import_audio();
        }
        ui.add_space(4.0);

        let list: Vec<(Uid, String, f64, f64)> = self
            .timeline
            .audios
            .iter()
            .map(|a| (a.uid, a.name.clone(), a.start_time, a.clip_duration()))
            .collect();
        if list.is_empty() {
            muted(ui, "Henüz ses parçası eklenmedi.");
        }
        for (uid, name, start, dur) in list {
            let sel = self.timeline.selected == Selection::Audio(uid);
            theme::dark_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("🎵 {}", util::shorten(&name, 20)))
                                .size(11.5)
                                .color(if sel { theme::accent() } else { theme::txt() }),
                        );
                        muted(ui, format!("@{start:.1}s  •  {dur:.1}s"));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(secondary_button("🗑")).clicked() {
                            self.timeline.audios.retain(|a| a.uid != uid);
                            if self.timeline.selected == Selection::Audio(uid) {
                                self.timeline.selected = Selection::None;
                            }
                        }
                        if ui.add(secondary_button("📍")).clicked() {
                            self.timeline.selected = Selection::Audio(uid);
                            self.tl_tab = 2;
                        }
                    });
                });
            });
        }

        if let Selection::Audio(uid) = self.timeline.selected {
            if let Some(a) = self.timeline.audios.iter_mut().find(|a| a.uid == uid) {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("🎚 Ses Özellikleri")
                        .size(12.0)
                        .strong()
                        .color(theme::accent()),
                );
                let duration = a.duration;
                card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Başlangıç (sn):");
                        ui.add(
                            egui::DragValue::new(&mut a.start_time)
                                .range(0.0..=36000.0)
                                .speed(0.05)
                                .fixed_decimals(2),
                        );
                        if ui.add(secondary_button("▶ Kafaya")).clicked() {
                            a.start_time = self.timeline.playhead;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Kırpma başlangıcı:");
                        ui.add(
                            egui::DragValue::new(&mut a.trim_start)
                                .range(0.0..=duration)
                                .speed(0.01)
                                .fixed_decimals(2),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Kırpma bitişi:");
                        ui.add(
                            egui::DragValue::new(&mut a.trim_end)
                                .range(0.0..=duration)
                                .speed(0.01)
                                .fixed_decimals(2),
                        );
                    });
                    ui.add(egui::Slider::new(&mut a.volume, 0.0..=2.0).text("Ses seviyesi"));
                    muted(ui, format!("Ses süresi: {:.2} sn", a.clip_duration()));
                });
            }
            ui.horizontal(|ui| {
                if ui.add(secondary_button("🔪 Böl")).clicked() {
                    self.timeline.split_selected();
                }
                if ui.add(danger_button("🚮 Sil")).clicked() {
                    self.timeline.delete_selected();
                }
            });
        }
    }

    fn tl_props_transition(&mut self, ui: &mut egui::Ui) {
        muted(
            ui,
            "Seçili klipten sonraki klibe veya iki klip sınırına çift tıklayın.",
        );
        if ui
            .add(
                primary_button("🎬  Geçiş Efekti Seç")
                    .min_size(egui::vec2(ui.available_width(), 34.0)),
            )
            .clicked()
        {
            self.open_transition_for_selected();
        }
        ui.add_space(4.0);

        let pairs = self.timeline.sorted_pairs();
        if pairs.is_empty() {
            muted(
                ui,
                "Henüz geçiş eklenmedi. Timeline'a en az 2 video ekleyin.",
            );
            return;
        }
        let clips = self.timeline.sorted_clips();
        for (a, b) in pairs {
            let na = clips
                .iter()
                .find(|c| c.uid == a)
                .map(|c| util::shorten(&c.name, 12))
                .unwrap_or_default();
            let nb = clips
                .iter()
                .find(|c| c.uid == b)
                .map(|c| util::shorten(&c.name, 12))
                .unwrap_or_default();
            let t = self.timeline.transition(a, b);
            theme::dark_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        muted(ui, format!("{na}  →  {nb}"));
                        ui.label(
                            RichText::new(format!("{}  {:.1}s", t.display(), t.duration))
                                .size(11.5)
                                .strong()
                                .color(if t.is_cut() {
                                    theme::txt2()
                                } else {
                                    Color32::from_rgb(0xf5, 0x9e, 0x0b)
                                }),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(secondary_button("✏")).clicked() {
                            self.timeline.transition_pair = Some((a, b));
                            self.timeline.transition_dur = t.duration as f32;
                            self.timeline.transition_sel = TRANSITIONS
                                .iter()
                                .position(|(_, f, _, _)| *f == t.effect)
                                .unwrap_or(0);
                        }
                        if ui
                            .add(secondary_button("✂"))
                            .on_hover_text("Cut yap")
                            .clicked()
                        {
                            self.timeline.set_transition(a, b, Transition::cut());
                        }
                    });
                });
            });
        }
    }

    fn tl_props_asset(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(
                primary_button("➕  Yeni Şekil / Görsel Ekle")
                    .min_size(egui::vec2(ui.available_width(), 32.0)),
            )
            .clicked()
        {
            self.tl_new_asset();
        }
        ui.add_space(4.0);

        let list: Vec<(Uid, String, f64, f64)> = self
            .timeline
            .assets
            .iter()
            .map(|a| (a.uid, a.name(), a.start_time, a.end_time))
            .collect();
        if list.is_empty() {
            muted(ui, "Henüz şekil eklenmedi.");
        }
        for (uid, name, s, e) in list {
            let sel = self.timeline.selected == Selection::Asset(uid);
            theme::dark_frame().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(util::shorten(&name, 22))
                                .size(11.5)
                                .color(if sel { theme::accent() } else { theme::txt() }),
                        );
                        muted(ui, format!("{s:.1}s - {e:.1}s"));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(secondary_button("🗑")).clicked() {
                            self.timeline.assets.retain(|a| a.uid != uid);
                            if self.timeline.selected == Selection::Asset(uid) {
                                self.timeline.selected = Selection::None;
                            }
                        }
                        if ui.add(secondary_button("✏")).clicked() {
                            if let Some(a) = self.timeline.assets.iter().find(|a| a.uid == uid) {
                                self.timeline.asset_editor =
                                    Some(crate::timeline::AssetEditorState::from_asset(a));
                            }
                        }
                    });
                });
            });
        }

        if let Selection::Asset(uid) = self.timeline.selected {
            if let Some(a) = self.timeline.assets.iter_mut().find(|a| a.uid == uid) {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("🔷 Seçili Şekil")
                        .size(12.0)
                        .strong()
                        .color(theme::accent()),
                );
                card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut a.x_px).range(0..=1920));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut a.y_px).range(0..=1080));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Genişlik:");
                        ui.add(egui::DragValue::new(&mut a.w_px).range(1..=1920));
                        ui.label("Yükseklik:");
                        ui.add(egui::DragValue::new(&mut a.h_px).range(1..=1080));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Başlangıç:");
                        ui.add(
                            egui::DragValue::new(&mut a.start_time)
                                .range(0.0..=36000.0)
                                .speed(0.05)
                                .fixed_decimals(2),
                        );
                        ui.label("Bitiş:");
                        ui.add(
                            egui::DragValue::new(&mut a.end_time)
                                .range(0.0..=36000.0)
                                .speed(0.05)
                                .fixed_decimals(2),
                        );
                    });
                    if a.kind != AssetKind::Image {
                        let mut col = preview_color(&a.color);
                        if ui.color_edit_button_srgba(&mut col).changed() {
                            a.color = util::rgb_hex(col.r(), col.g(), col.b());
                        }
                        ui.add(egui::Slider::new(&mut a.thickness, 1..=40).text("Kalınlık"));
                    } else {
                        ui.add(egui::Slider::new(&mut a.opacity, 0.0..=1.0).text("Opaklık"));
                    }
                });
                if ui.add(secondary_button("✏  Ayrıntılı Düzenle")).clicked() {
                    if let Some(asset) = self.timeline.assets.iter().find(|x| x.uid == uid) {
                        self.timeline.asset_editor =
                            Some(crate::timeline::AssetEditorState::from_asset(asset));
                    }
                }
            }
        }
    }
}

fn resolve_placement(others: &[(Uid, f64, f64)], start: f64, dur: f64, pps: f32) -> (f64, bool) {
    let snap = 12.0 / pps.max(1.0) as f64;
    let start = start.max(0.0);
    let dur = dur.max(0.01);
    let overlaps = |s: f64| {
        others
            .iter()
            .any(|(_, o_s, o_e)| s < *o_e - 1e-9 && s + dur > *o_s + 1e-9)
    };

    // Yakın kenarlara yapıştırma adayları (başlangıç ve bitiş).
    let mut candidates: Vec<f64> = Vec::new();
    for (_, o_s, o_e) in others {
        for c in [*o_e, o_s - dur, *o_s, o_e - dur] {
            if c >= 0.0 && (c - start).abs() <= snap {
                candidates.push(c);
            }
        }
    }
    candidates.sort_by(|a, b| {
        (a - start)
            .abs()
            .partial_cmp(&(b - start).abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    if let Some(cand) = candidates.into_iter().find(|c| !overlaps(*c)) {
        return (cand, false);
    }

    // Boş yer yok: çakışan kliplerin sağına doğru ilerlet.
    let mut s = start;
    for _ in 0..=others.len() {
        let hit = others
            .iter()
            .filter(|(_, o_s, o_e)| s < *o_e - 1e-9 && s + dur > *o_s + 1e-9)
            .min_by(|x, y| x.2.partial_cmp(&y.2).unwrap_or(std::cmp::Ordering::Equal));
        match hit {
            Some((_, _, o_e)) => s = *o_e,
            None => return (s, false),
        }
    }
    (s.max(0.0), true)
}

fn preview_color(hex: &str) -> Color32 {
    let (r, g, b) = util::hex_rgb(hex);
    Color32::from_rgb(r, g, b)
}

fn draw_preview_layers(
    p: &egui::Painter,
    img: egui::Rect,
    texts: &[TextOverlay],
    assets: &[Asset],
    asset_tex: &Option<(String, egui::TextureHandle)>,
) {
    let sx = img.width() / timeline::REF_W as f32;
    let sy = img.height() / timeline::REF_H as f32;
    for t in texts {
        let pos = egui::pos2(img.left() + t.x as f32 * sx, img.top() + t.y as f32 * sy);
        let size = (t.size as f32 * sy).clamp(6.0, 180.0);
        p.text(
            pos,
            egui::Align2::LEFT_TOP,
            &t.text,
            egui::FontId::proportional(size),
            preview_color(&t.color),
        );
    }
    for a in assets {
        let x = img.left() + a.x_px as f32 * sx;
        let y = img.top() + a.y_px as f32 * sy;
        let w = (a.w_px as f32 * sx).max(1.0);
        let h = (a.h_px as f32 * sy).max(1.0);
        let color = preview_color(&a.color);
        let stroke = egui::Stroke::new((a.thickness as f32).max(1.0), color);
        match a.kind {
            AssetKind::Rect => {
                p.rect_stroke(
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
                    0.0,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
            AssetKind::Circle => {
                p.circle_stroke(egui::pos2(x + w * 0.5, y + h * 0.5), w.min(h) * 0.5, stroke);
            }
            AssetKind::Line => {
                p.line_segment([egui::pos2(x, y), egui::pos2(x + w, y + 2.0)], stroke);
            }
            AssetKind::Image => {
                if let Some((path, tex)) = asset_tex {
                    if path == &a.path {
                        p.image(
                            tex.id(),
                            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::from_white_alpha((a.opacity.clamp(0.0, 1.0) * 255.0) as u8),
                        );
                    }
                }
            }
        }
    }
}
