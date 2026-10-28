//! Timeline pencereleri: geçiş seçici, metin/şekil editörü, klip efektleri,
//! dışa aktarma ayarları.
use super::*;
use crate::app::edit::edit_effects_ui;
use crate::timeline::{Asset, AssetKind, ClipEffects, TextOverlay, Transition, TRANSITIONS};

impl App {
    pub fn timeline_dialogs(&mut self, ctx: &egui::Context) {
        self.transition_dialog(ctx);
        self.text_overlay_dialog(ctx);
        self.asset_dialog(ctx);
        self.clip_effects_window(ctx);
        self.export_dialog(ctx);
    }

    // ------------------------------------------------------------ geçiş seçici
    fn transition_dialog(&mut self, ctx: &egui::Context) {
        let Some((a, b)) = self.timeline.transition_pair else {
            return;
        };
        let name_a = self
            .timeline
            .clips
            .iter()
            .find(|c| c.uid == a)
            .map(|c| util::shorten(&c.name, 24))
            .unwrap_or_else(|| "—".into());
        let name_b = self
            .timeline
            .clips
            .iter()
            .find(|c| c.uid == b)
            .map(|c| util::shorten(&c.name, 24))
            .unwrap_or_else(|| "—".into());

        let mut open = true;
        let mut close = false;
        let mut sel = self.timeline.transition_sel.min(TRANSITIONS.len() - 1);
        let mut dur = self.timeline.transition_dur.clamp(0.1, 2.0);
        let mut apply = false;
        let mut reset = false;

        egui::Window::new("🎬 Geçiş Efekti Seç")
            .collapsible(false)
            .resizable(true)
            .default_width(530.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("{name_a}  →  {name_b}"))
                        .size(13.0)
                        .strong()
                        .color(theme::txt()),
                );
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("trans_list")
                    .max_height(330.0)
                    .show(ui, |ui| {
                        for (i, (name, fx, desc, em)) in TRANSITIONS.iter().enumerate() {
                            let selected = sel == i;
                            let text = RichText::new(format!("{em}  {name}"))
                                .size(13.0)
                                .strong()
                                .color(if selected {
                                    Color32::WHITE
                                } else {
                                    theme::txt()
                                });
                            let resp = ui.add_sized(
                                egui::vec2(ui.available_width(), 28.0),
                                egui::SelectableLabel::new(selected, text),
                            );
                            if resp.clicked() {
                                sel = i;
                            }
                            ui.label(
                                RichText::new(format!("      {desc}   [{fx}]"))
                                    .size(10.5)
                                    .color(theme::txt2()),
                            );
                        }
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⏱ Geçiş Süresi:").size(12.0).strong());
                    ui.add(
                        egui::Slider::new(&mut dur, 0.1..=2.0)
                            .fixed_decimals(1)
                            .suffix(" sn"),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    for (label, v) in [
                        ("Hızlı 0.3s", 0.3f32),
                        ("Normal 0.5s", 0.5),
                        ("Yavaş 1.0s", 1.0),
                        ("Sinematik 1.5s", 1.5),
                    ] {
                        if ui.add(secondary_button(label)).clicked() {
                            dur = v;
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.add(primary_button("✓ Uygula")).clicked() {
                        apply = true;
                    }
                    if ui.add(secondary_button("✂ Sıfırla (Cut)")).clicked() {
                        reset = true;
                    }
                    if ui.add(danger_button("✗ İptal")).clicked() {
                        close = true;
                    }
                });
            });

        if close {
            open = false;
        }
        self.timeline.transition_sel = sel;
        self.timeline.transition_dur = dur;

        if apply {
            let effect = TRANSITIONS[sel].1.to_string();
            self.timeline.set_transition(
                a,
                b,
                Transition {
                    effect,
                    duration: (dur as f64 * 10.0).round() / 10.0,
                },
            );
            self.timeline.transition_pair = None;
            self.tl_set_status("✓ Geçiş efekti uygulandı.");
        } else if reset {
            self.timeline.set_transition(a, b, Transition::cut());
            self.timeline.transition_pair = None;
        } else if !open {
            self.timeline.transition_pair = None;
        }
    }

    // ------------------------------------------------------------ metin editörü
    fn text_overlay_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut ed) = self.timeline.text_editor.take() else {
            return;
        };
        let mut open = true;
        let mut close = false;
        let mut apply = false;
        let title = if ed.editing.is_some() {
            "✏ Metin Katmanını Düzenle"
        } else {
            "📝 Metin Katmanı Ekle"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .default_width(390.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Metin:");
                    ui.add(
                        egui::TextEdit::singleline(&mut ed.text)
                            .char_limit(500)
                            .desired_width(f32::INFINITY),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("X konumu:");
                    ui.add(egui::DragValue::new(&mut ed.x).range(0..=1920));
                    ui.label("Y konumu:");
                    ui.add(egui::DragValue::new(&mut ed.y).range(0..=1080));
                });
                ui.horizontal(|ui| {
                    ui.label("Başlangıç (sn):");
                    ui.add(
                        egui::DragValue::new(&mut ed.start)
                            .range(0.0..=36000.0)
                            .speed(0.05)
                            .fixed_decimals(2),
                    );
                    ui.label("Bitiş (sn):");
                    ui.add(
                        egui::DragValue::new(&mut ed.end)
                            .range(0.0..=36000.0)
                            .speed(0.05)
                            .fixed_decimals(2),
                    );
                });
                ui.add(egui::Slider::new(&mut ed.size, 12..=120).text("Yazı boyutu"));
                ui.horizontal(|ui| {
                    ui.label("Renk:");
                    let mut col = preview_color(&ed.color);
                    if ui.color_edit_button_srgba(&mut col).changed() {
                        ed.color = util::rgb_hex(col.r(), col.g(), col.b());
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut ed.color)
                            .desired_width(80.0)
                            .hint_text("#ffffff"),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    for c in [
                        "#ffffff", "#ffff00", "#ff5555", "#55ff55", "#55aaff", "#ff8800",
                    ] {
                        let col = preview_color(c);
                        if ui.color_edit_button_srgba(&mut col.clone()).clicked() {
                            ed.color = c.into();
                        }
                    }
                });
                if ed.end <= ed.start {
                    ui.label(
                        RichText::new("⚠ Bitiş zamanı başlangıçtan büyük olmalı.")
                            .size(11.0)
                            .color(theme::red()),
                    );
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(primary_button(if ed.editing.is_some() {
                            "💾 Değişiklikleri Kaydet"
                        } else {
                            "✓  Ekle"
                        }))
                        .clicked()
                        && ed.end > ed.start
                        && !ed.text.trim().is_empty()
                    {
                        apply = true;
                    }
                    if ui.add(secondary_button("İptal")).clicked() {
                        close = true;
                    }
                });
            });

        if close {
            open = false;
        }
        if apply {
            let ov = TextOverlay {
                uid: ed.editing.unwrap_or(0),
                text: ed.text.clone(),
                x: ed.x.clamp(0, timeline::REF_W as i32),
                y: ed.y.clamp(0, timeline::REF_H as i32),
                size: ed.size.clamp(8, 200),
                color: ed.color.clone(),
                start_time: ed.start.max(0.0) as f64,
                end_time: ed.end.max(ed.start + 0.1) as f64,
            };
            self.timeline.save_text(ov);
            self.tl_set_status("✓ Metin katmanı kaydedildi.");
            open = false;
        }
        if !open {
            self.timeline.text_editor = None;
        } else {
            self.timeline.text_editor = Some(ed);
        }
    }

    // ------------------------------------------------------------ şekil / görsel editörü
    fn asset_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut ed) = self.timeline.asset_editor.take() else {
            return;
        };
        let mut open = true;
        let mut apply = false;
        let title = if ed.editing.is_some() {
            "✏ Varlık Düzenle"
        } else {
            "🎨 Varlık Ekle"
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .default_width(560.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Tür:").size(12.0).color(theme::txt2()));
                    ui.selectable_value(&mut ed.kind, AssetKind::Rect, "⬜ Dikdörtgen");
                    ui.selectable_value(&mut ed.kind, AssetKind::Circle, "⭕ Daire");
                    ui.selectable_value(&mut ed.kind, AssetKind::Line, "📏 Çizgi");
                    ui.selectable_value(&mut ed.kind, AssetKind::Image, "🎨 Görsel");
                });

                if ed.kind == AssetKind::Image {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut ed.path)
                                .hint_text("Görsel yolu...")
                                .desired_width(ui.available_width() - 110.0),
                        );
                        if ui.add(secondary_button("📂 Görsel Seç")).clicked() {
                            if let Some(p) = rfd::FileDialog::new()
                                .set_title("Görsel Seç")
                                .add_filter("Görseller", &["png", "jpg", "jpeg", "bmp", "webp"])
                                .pick_file()
                            {
                                ed.path = p.to_string_lossy().to_string();
                            }
                        }
                    });
                }

                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "Referans çözünürlük: {}×{} piksel",
                        timeline::REF_W,
                        timeline::REF_H
                    ))
                    .size(10.5)
                    .color(theme::txt2()),
                );
                egui::Grid::new("asset_grid")
                    .num_columns(4)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("X (sol):");
                        ui.add(
                            egui::DragValue::new(&mut ed.x)
                                .range(0..=timeline::REF_W as i32)
                                .speed(2.0),
                        );
                        ui.label("Y (üst):");
                        ui.add(
                            egui::DragValue::new(&mut ed.y)
                                .range(0..=timeline::REF_H as i32)
                                .speed(2.0),
                        );
                        ui.end_row();
                        ui.label("Genişlik:");
                        ui.add(
                            egui::DragValue::new(&mut ed.w)
                                .range(1..=timeline::REF_W as i32)
                                .speed(2.0),
                        );
                        ui.label("Yükseklik:");
                        ui.add(
                            egui::DragValue::new(&mut ed.h)
                                .range(1..=timeline::REF_H as i32)
                                .speed(2.0),
                        );
                        ui.end_row();
                    });

                ui.horizontal(|ui| {
                    ui.label("Başlangıç:");
                    ui.add(
                        egui::DragValue::new(&mut ed.start)
                            .range(0.0..=36000.0)
                            .speed(0.05)
                            .fixed_decimals(2),
                    );
                    ui.label("sn    Bitiş:");
                    ui.add(
                        egui::DragValue::new(&mut ed.end)
                            .range(0.0..=36000.0)
                            .speed(0.05)
                            .fixed_decimals(2),
                    );
                    ui.label("sn");
                });

                ui.horizontal(|ui| {
                    if ed.kind != AssetKind::Image {
                        ui.label("Renk:");
                        let mut col = preview_color(&ed.color);
                        if ui.color_edit_button_srgba(&mut col).changed() {
                            ed.color = util::rgb_hex(col.r(), col.g(), col.b());
                        }
                        ui.label("Kalınlık:");
                        ui.add(egui::DragValue::new(&mut ed.thickness).range(1..=40));
                        ui.label("px");
                    } else {
                        ui.label("Opaklık:");
                        ui.add(egui::Slider::new(&mut ed.opacity, 0.0..=1.0).fixed_decimals(2));
                    }
                });

                // Mini önizleme
                ui.separator();
                ui.label(
                    RichText::new("Önizleme  (1:5)")
                        .size(11.0)
                        .color(theme::txt2()),
                );
                let pw = ui.available_width().min(384.0);
                let ph = pw * 9.0 / 16.0;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(pw, ph), egui::Sense::hover());
                let p = ui.painter_at(rect);
                p.rect_filled(rect, 4.0, Color32::from_rgb(0x1a, 0x1a, 0x2e));
                for i in 1..3 {
                    p.line_segment(
                        [
                            egui::pos2(rect.left() + pw * i as f32 / 3.0, rect.top()),
                            egui::pos2(rect.left() + pw * i as f32 / 3.0, rect.bottom()),
                        ],
                        egui::Stroke::new(1.0_f32, Color32::from_rgb(0x22, 0x22, 0x33)),
                    );
                    p.line_segment(
                        [
                            egui::pos2(rect.left(), rect.top() + ph * i as f32 / 3.0),
                            egui::pos2(rect.right(), rect.top() + ph * i as f32 / 3.0),
                        ],
                        egui::Stroke::new(1.0_f32, Color32::from_rgb(0x22, 0x22, 0x33)),
                    );
                }
                let sx = pw / timeline::REF_W as f32;
                let sy = ph / timeline::REF_H as f32;
                let x = rect.left() + ed.x.max(0) as f32 * sx;
                let y = rect.top() + ed.y.max(0) as f32 * sy;
                let w = (ed.w.max(1) as f32 * sx).max(2.0);
                let h = (ed.h.max(1) as f32 * sy).max(2.0);
                let col = preview_color(&ed.color);
                match ed.kind {
                    AssetKind::Rect => {
                        p.rect_stroke(
                            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
                            0.0,
                            egui::Stroke::new(1.5_f32, col),
                            egui::StrokeKind::Inside,
                        );
                    }
                    AssetKind::Circle => {
                        p.circle_stroke(
                            egui::pos2(x + w * 0.5, y + h * 0.5),
                            w.min(h) * 0.5,
                            egui::Stroke::new(1.5_f32, col),
                        );
                    }
                    AssetKind::Line => {
                        p.line_segment(
                            [egui::pos2(x, y), egui::pos2(x + w, y + 2.0)],
                            egui::Stroke::new((ed.thickness as f32).max(1.0), col),
                        );
                    }
                    AssetKind::Image => {
                        p.rect_stroke(
                            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
                            0.0,
                            egui::Stroke::new(1.0_f32, col),
                            egui::StrokeKind::Inside,
                        );
                        p.text(
                            egui::pos2(x + w * 0.5, y + h * 0.5),
                            egui::Align2::CENTER_CENTER,
                            "IMG",
                            egui::FontId::proportional((h * 0.25).clamp(8.0, 22.0)),
                            col,
                        );
                    }
                }

                if ed.end <= ed.start {
                    ui.label(
                        RichText::new("⚠ Bitiş > Başlangıç olmalı.")
                            .size(11.0)
                            .color(theme::red()),
                    );
                }
                if ed.kind == AssetKind::Image && ed.path.trim().is_empty() {
                    ui.label(
                        RichText::new("⚠ Görsel seçilmedi.")
                            .size(11.0)
                            .color(theme::red()),
                    );
                }
                if !ed.error.is_empty() {
                    ui.label(RichText::new(&ed.error).size(11.0).color(theme::red()));
                }
                ui.add_space(4.0);
                if ui
                    .add(primary_button(if ed.editing.is_some() {
                        "💾  Güncelle"
                    } else {
                        "💾  Timeline'a Ekle"
                    }))
                    .clicked()
                {
                    if ed.end > ed.start
                        && !(ed.kind == AssetKind::Image && ed.path.trim().is_empty())
                    {
                        apply = true;
                    } else if ed.end <= ed.start {
                        ed.error = "Bitiş zamanı başlangıçtan büyük olmalı.".into();
                    } else {
                        ed.error = "Lütfen bir görsel dosyası seçin.".into();
                    }
                }
            });

        if apply {
            let asset = Asset {
                uid: ed.editing.unwrap_or(0),
                kind: ed.kind,
                path: ed.path.clone(),
                x_px: ed.x.clamp(0, timeline::REF_W as i32),
                y_px: ed.y.clamp(0, timeline::REF_H as i32),
                w_px: ed.w.clamp(1, timeline::REF_W as i32),
                h_px: ed.h.clamp(1, timeline::REF_H as i32),
                start_time: ed.start.max(0.0) as f64,
                end_time: ed.end.max(ed.start + 0.1) as f64,
                color: ed.color.clone(),
                thickness: ed.thickness.max(1),
                opacity: ed.opacity.clamp(0.0, 1.0),
            };
            self.timeline.save_asset(asset);
            self.tl_set_status("✓ Şekil / görsel kaydedildi.");
            self.tl_asset_tex = None;
            open = false;
        }
        if !open {
            self.timeline.asset_editor = None;
        } else {
            self.timeline.asset_editor = Some(ed);
        }
    }

    // ------------------------------------------------------------ klip efekt penceresi
    fn clip_effects_window(&mut self, ctx: &egui::Context) {
        if !self.timeline.effects_open {
            return;
        }
        let Some(uid) = self.timeline.effects_uid else {
            self.timeline.effects_open = false;
            return;
        };
        let Some(clip) = self.timeline.clips.iter().find(|c| c.uid == uid) else {
            self.timeline.effects_open = false;
            self.timeline.effects_uid = None;
            return;
        };
        let mut eff = clip.effects.clone();
        let path = clip.path.clone();
        let mid = (clip.trim_start + clip.trim_end) * 0.5;
        let mut open = true;
        let mut changed = false;

        egui::Window::new("🎨 Efekt ve Filtre Ayarları")
            .collapsible(false)
            .resizable(true)
            .default_width(820.0)
            .default_height(540.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.columns(2, |cols| {
                    egui::ScrollArea::vertical()
                        .id_salt("clip_effects_scroll")
                        .max_height(470.0)
                        .show(&mut cols[0], |ui| {
                            changed = edit_effects_ui(ui, &mut eff);
                            ui.add_space(6.0);
                            if ui.add(secondary_button("🔄 Önizlemeyi Yenile")).clicked() {
                                self.timeline.effects_key = None;
                                self.tl_fx_dirty = Some(Instant::now() - Duration::from_secs(1));
                            }
                        });
                    cols[1].vertical_centered(|ui| {
                        ui.label(
                            RichText::new("📸 Canlı Önizleme")
                                .size(13.0)
                                .strong()
                                .color(theme::txt()),
                        );
                        ui.add_space(4.0);
                        if let Some(tex) = self.timeline.effects_tex.clone() {
                            let avail = ui.available_size();
                            let s = tex.size_vec2();
                            let sc = (avail.x / s.x).min((avail.y - 10.0) / s.y).max(0.05);
                            let size = s * sc;
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(avail.x, size.y.max(40.0)),
                                egui::Sense::hover(),
                            );
                            let img = egui::Rect::from_center_size(
                                egui::pos2(rect.center().x, rect.top() + size.y * 0.5),
                                size,
                            );
                            ui.painter().image(
                                tex.id(),
                                img,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                        } else {
                            ui.label(
                                RichText::new("⏳ Önizleme hazırlanıyor...").color(theme::txt2()),
                            );
                            ui.allocate_space(egui::vec2(ui.available_width(), 160.0));
                        }
                    });
                });
            });

        if changed {
            if let Some(c) = self.timeline.clips.iter_mut().find(|c| c.uid == uid) {
                c.effects = eff.clone();
            }
            self.tl_fx_dirty = Some(Instant::now());
        }
        if !open {
            self.timeline.effects_open = false;
            self.timeline.effects_uid = None;
            self.timeline.effects_tex = None;
            self.timeline.effects_key = None;
            self.tl_fx_dirty = None;
            return;
        }

        let hash = effects_hash(&eff);
        let current = self.timeline.effects_key.map(|(_, h)| h);
        let ready = self
            .tl_fx_dirty
            .map(|t| t.elapsed() > Duration::from_millis(420))
            .unwrap_or(false);
        if current != Some(hash) && ready && !self.tl_fx_busy && self.tl_fx_rx.is_none() {
            let out = util::unique_temp("jpg");
            let (tx, rx) = mpsc::channel();
            self.tl_fx_rx = Some(rx);
            self.tl_fx_busy = true;
            self.timeline.effects_key = Some((uid, hash));
            let eff2 = eff.clone();
            std::thread::spawn(move || {
                let res = timeline::make_preview(&path, mid, &eff2, &out).map(|_| out);
                let _ = tx.send((hash, res));
            });
        } else if !ready {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    // ------------------------------------------------------------ dışa aktarma
    fn export_dialog(&mut self, ctx: &egui::Context) {
        if !self.tl_export_open {
            return;
        }
        if self.timeline.clips.is_empty() {
            self.tl_export_open = false;
            self.info("Timeline'a en az bir video ekleyin.");
            return;
        }
        let mut open = true;
        let mut close = false;
        let mut start = false;
        let mut browse = false;
        let resolutions = [
            ("Orijinal (kaynak boyutu)", "Orijinal"),
            ("3840 × 2160  (4K UHD)", "3840x2160"),
            ("1920 × 1080  (Full HD)", "1920x1080"),
            ("1280 × 720   (HD)", "1280x720"),
            ("854 × 480    (SD)", "854x480"),
            ("640 × 360    (360p)", "640x360"),
        ];

        egui::Window::new("📐 Dışa Aktarma Ayarları")
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Çözünürlük Seç")
                        .size(14.0)
                        .strong()
                        .color(theme::txt()),
                );
                for (label, val) in resolutions {
                    ui.radio_value(&mut self.timeline.export_res, val.to_string(), label);
                }
                ui.separator();
                ui.label(
                    RichText::new("Çıktı dosyası:")
                        .size(11.5)
                        .color(theme::txt2()),
                );
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.tl_export_out)
                            .desired_width(ui.available_width() - 50.0),
                    );
                    if ui.add(secondary_button("📂")).clicked() {
                        browse = true;
                    }
                });
                if let Some(c) = self.timeline.clips.first() {
                    let (w, h) = util::dimensions(&c.path);
                    muted(ui, format!("İlk klip kaynağı: {w}×{h}"));
                }
                muted(
                    ui,
                    format!(
                        "{} klip • {} ses • toplam {}",
                        self.timeline.clips.len(),
                        self.timeline.audios.len(),
                        util::mmss(self.timeline.duration())
                    ),
                );
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.add(primary_button("✓  Dışa Aktar")).clicked() {
                        start = true;
                    }
                    if ui.add(danger_button("✗  İptal")).clicked() {
                        close = true;
                    }
                });
            });

        if close {
            open = false;
        }
        if browse {
            if let Some(p) = rfd::FileDialog::new()
                .set_title("Videoyu Kaydet")
                .set_directory(&self.settings.recorder.output_dir)
                .set_file_name(format!("timeline_{}.mp4", util::stamp()))
                .add_filter("MP4", &["mp4"])
                .add_filter("MKV", &["mkv"])
                .save_file()
            {
                self.tl_export_out = p.to_string_lossy().to_string();
            }
        }

        if start {
            let mut out = self.tl_export_out.trim().to_string();
            if out.is_empty() {
                out = format!(
                    "{}/timeline_{}.mp4",
                    self.settings
                        .recorder
                        .output_dir
                        .trim_end_matches(['/', '\\']),
                    util::stamp()
                );
            }
            if !out.to_lowercase().ends_with(".mp4") && !out.to_lowercase().ends_with(".mkv") {
                out.push_str(".mp4");
            }
            if let Some(parent) = Path::new(&out).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let inp = self
                .timeline
                .export_input(self.timeline.export_res.clone(), out.clone());
            self.timeline.exporting = true;
            self.tl_set_status("⏳ Dışa aktarılıyor...");
            let status = self.timeline.status.clone();
            std::thread::spawn(move || {
                timeline::export(inp, status);
            });
            self.tl_export_open = false;
            self.tl_set_status("⏳ Dışa aktarma başlatıldı...");
        } else if !open {
            self.tl_export_open = false;
        }
    }
}

fn effects_hash(eff: &ClipEffects) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{eff:?}").hash(&mut h);
    h.finish()
}
