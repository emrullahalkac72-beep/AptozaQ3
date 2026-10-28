//! Kayıt, ayarlar ve yardım sekmeleri ile canlı webcam penceresi.
use super::*;

impl App {
    pub fn ui_record(&mut self, ui: &mut egui::Ui) {
        h1(ui, "🎬  Kayıt Kaynakları");
        ui.add_space(2.0);
        muted(ui, "Ekran, webcam ve ses kaynağını seçin. Kayıt sırasında kenar çubuğundaki kontrolü kullanın.");
        ui.add_space(6.0);

        let mut r = self.settings.recorder.clone();
        let mut pick_region = false;
        let mut query_res = false;
        let mut pick_outdir = false;

        ui.columns(3, |cols| {
            // ---------------------------------------------------------- ekran
            card(&mut cols[0], |ui| {
                h2(ui, "🖥  Ekran Kaydı");
                ui.checkbox(&mut r.screen_on, RichText::new("Etkin").size(13.0));
                ui.add_space(4.0);
                if ui
                    .add(
                        secondary_button("📐  Bölge Seç")
                            .min_size(egui::vec2(ui.available_width(), 30.0)),
                    )
                    .clicked()
                {
                    pick_region = true;
                }
                let region_label = r
                    .region
                    .filter(|x| x.valid())
                    .map(|x| x.label())
                    .unwrap_or_else(|| "Tam Ekran".into());
                ui.label(
                    RichText::new(region_label)
                        .size(11.0)
                        .color(theme::accent()),
                );
                if r.region.is_some()
                    && ui
                        .add(
                            secondary_button("Tam ekrana dön")
                                .min_size(egui::vec2(ui.available_width(), 24.0)),
                        )
                        .clicked()
                {
                    r.region = None;
                }
                ui.add_space(6.0);
                ui.label(
                    RichText::new("🎯 FPS")
                        .size(12.0)
                        .strong()
                        .color(theme::txt2()),
                );
                combo_u32(ui, "fps", &mut r.fps, &model::FPS_CHOICES);
                ui.add_space(4.0);
                ui.label(
                    RichText::new("✨ Video Kalitesi")
                        .size(12.0)
                        .strong()
                        .color(theme::txt2()),
                );
                let qtext = model::quality_label(r.quality).to_string();
                egui::ComboBox::from_id_salt("quality")
                    .selected_text(qtext)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for (crf, label) in model::QUALITIES {
                            if ui.selectable_label(r.quality == crf, label).clicked() {
                                r.quality = crf;
                            }
                        }
                    });
            });

            // ---------------------------------------------------------- webcam
            card(&mut cols[1], |ui| {
                h2(ui, "📷  Webcam");
                ui.checkbox(&mut r.webcam_on, RichText::new("Etkin").size(13.0));
                ui.checkbox(
                    &mut r.pip_enabled,
                    RichText::new("PiP (Küçük ekran)").size(12.0),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new("🔢 Kamera İndeksi")
                        .size(12.0)
                        .strong()
                        .color(theme::txt2()),
                );
                let mut cam_i = r.cam_idx as u32;
                egui::ComboBox::from_id_salt("cam_idx")
                    .selected_text(cam_i.to_string())
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        for i in 0..4u32 {
                            ui.selectable_value(&mut cam_i, i, i.to_string());
                        }
                    });
                r.cam_idx = cam_i as usize;

                ui.label(
                    RichText::new("Webcam Çözünürlüğü")
                        .size(12.0)
                        .color(theme::txt2()),
                );
                let cur = format!("{}×{}", r.webcam_res.0, r.webcam_res.1);
                egui::ComboBox::from_id_salt("cam_res")
                    .selected_text(cur)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for (w, h) in self.cam_res_list.clone() {
                            if ui
                                .selectable_label(r.webcam_res == (w, h), format!("{w}×{h}"))
                                .clicked()
                            {
                                r.webcam_res = (w, h);
                            }
                        }
                    });
                if ui
                    .add_enabled(
                        !self.cam_res_busy,
                        secondary_button(if self.cam_res_busy {
                            "⏳ Sorgulanıyor..."
                        } else {
                            "🔍 Desteklenen Çözünürlükleri Bul"
                        })
                        .min_size(egui::vec2(ui.available_width(), 26.0)),
                    )
                    .clicked()
                {
                    query_res = true;
                }
                ui.label(RichText::new("PiP Boyutu").size(12.0).color(theme::txt2()));
                let pip_label = model::PIP_SIZES
                    .iter()
                    .find(|(w, h, _)| (*w, *h) == r.pip_size)
                    .map(|(_, _, l)| l.to_string())
                    .unwrap_or_else(|| format!("{}×{}", r.pip_size.0, r.pip_size.1));
                egui::ComboBox::from_id_salt("pip_size")
                    .selected_text(pip_label)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for (w, h, label) in model::PIP_SIZES {
                            if ui.selectable_label(r.pip_size == (w, h), label).clicked() {
                                r.pip_size = (w, h);
                            }
                        }
                    });
                ui.label(RichText::new("PiP Konumu").size(12.0).color(theme::txt2()));
                egui::ComboBox::from_id_salt("pip_pos")
                    .selected_text(&r.pip_pos)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for p in model::PIP_POSITIONS {
                            ui.selectable_value(&mut r.pip_pos, p.to_string(), p);
                        }
                    });
                ui.checkbox(
                    &mut self.show_cam_preview,
                    RichText::new("Kayıt sırasında önizleme").size(12.0),
                );
            });

            // ---------------------------------------------------------- ses
            card(&mut cols[2], |ui| {
                h2(ui, "🎙  Ses");
                ui.checkbox(&mut r.audio_on, RichText::new("Etkin").size(13.0));
                if r.audio_on {
                    muted(ui, "Mikrofon, ffmpeg dshow ile kaydedilir.");
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Örnekleme hızı")
                        .size(12.0)
                        .color(theme::txt2()),
                );
                egui::ComboBox::from_id_salt("audio_rate")
                    .selected_text(r.audio_rate.to_string())
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        for rate in model::AUDIO_RATES {
                            ui.selectable_value(&mut r.audio_rate, rate, rate.to_string());
                        }
                    });
                ui.add_space(8.0);
                ui.label(
                    RichText::new("🎨 Ekran Efektleri")
                        .size(12.0)
                        .strong()
                        .color(theme::txt2()),
                );
                muted(
                    ui,
                    "İmleç vurgusu, filigran ve zoom Ayarlar sekmesinden yönetilir.",
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "Ses: {}  |  FPS: {}  |  CRF: {}",
                        if r.audio_on { "Açık" } else { "Kapalı" },
                        r.fps,
                        r.quality
                    ))
                    .size(11.0)
                    .color(theme::accent2()),
                );
            });
        });

        ui.add_space(8.0);
        card(ui, |ui| {
            h2(ui, "📁  Çıktı Klasörü");
            ui.horizontal(|ui| {
                ui.label(RichText::new(&r.output_dir).size(12.0).color(theme::txt2()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(secondary_button("Değiştir")).clicked() {
                        pick_outdir = true;
                    }
                });
            });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add(primary_button("⏺  Kaydı Başlat").min_size(egui::vec2(200.0, 40.0)))
                .clicked()
            {
                self.settings.recorder = r.clone();
                self.start_recording();
                return;
            }
            if ui
                .add(secondary_button("🔍 Kaynakları Kontrol Et"))
                .clicked()
            {
                let _ = util::which("ffmpeg");
            }
            if !util::which("ffmpeg") {
                ui.label(RichText::new("⚠ ffmpeg bulunamadı!").color(theme::yellow()));
            }
        });

        self.settings.recorder = r;
        if pick_region {
            let ctx = ui.ctx().clone();
            self.start_region_select(&ctx);
        }
        if pick_outdir {
            if let Some(dir) = rfd::FileDialog::new()
                .set_title("Çıktı Klasörü Seç")
                .set_directory(&self.settings.recorder.output_dir)
                .pick_folder()
            {
                self.settings.recorder.output_dir = dir.to_string_lossy().to_string();
            }
        }
        if query_res {
            self.cam_res_busy = true;
            let idx = self.settings.recorder.cam_idx;
            let (tx, rx) = mpsc::channel();
            self.cam_res_rx = Some(rx);
            std::thread::spawn(move || {
                let list = recorder::query_webcam_resolutions(idx);
                let _ = tx.send(list);
            });
        }
    }

    // ------------------------------------------------------------------ ayarlar
    pub fn ui_settings(&mut self, ui: &mut egui::Ui) {
        h1(ui, "⚙  Ayarlar");
        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut r = self.settings.recorder.clone();

                card(ui, |ui| {
                    h2(ui, "🖱  İmleç Vurgusu");
                    ui.checkbox(&mut r.cursor_hl, RichText::new("İmleci vurgula").size(13.0));
                    ui.horizontal(|ui| {
                        ui.label("Renk:");
                        let (cr, cg, cb) = util::hex_rgb(&r.cursor_color);
                        let mut rgb = [cr, cg, cb];
                        if ui.color_edit_button_srgb(&mut rgb).changed() {
                            r.cursor_color = util::rgb_hex(rgb[0], rgb[1], rgb[2]);
                        }
                        ui.label("Boyut:");
                        ui.add(egui::Slider::new(&mut r.cursor_sz, 10..=50));
                    });
                });
                ui.add_space(6.0);

                card(ui, |ui| {
                    h2(ui, "💧  Filigran (Watermark)");
                    ui.checkbox(
                        &mut r.wm_enabled,
                        RichText::new("Filigran etkin").size(13.0),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Metin:");
                        ui.add(
                            egui::TextEdit::singleline(&mut r.wm_text)
                                .char_limit(200)
                                .hint_text("Aptoza")
                                .desired_width(260.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Görsel:");
                        let name = r
                            .wm_image
                            .as_ref()
                            .map(|p| util::basename(p))
                            .unwrap_or_else(|| "Seçilmedi".into());
                        ui.label(RichText::new(name).size(11.0).color(theme::txt2()));
                        if ui.add(secondary_button("📂 Görsel Seç")).clicked() {
                            if let Some(p) = rfd::FileDialog::new()
                                .add_filter("Görseller", &["png", "jpg", "jpeg", "bmp"])
                                .pick_file()
                            {
                                r.wm_image = Some(p.to_string_lossy().to_string());
                            }
                        }
                        if ui.add(danger_button("✗ Temizle")).clicked() {
                            r.wm_image = None;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Konum:");
                        egui::ComboBox::from_id_salt("wm_pos")
                            .selected_text(&r.wm_pos)
                            .width(140.0)
                            .show_ui(ui, |ui| {
                                for p in model::POSITIONS {
                                    ui.selectable_value(&mut r.wm_pos, p.to_string(), p);
                                }
                            });
                        ui.label("Saydamlık:");
                        ui.add(egui::Slider::new(&mut r.wm_opacity, 0.1..=1.0).fixed_decimals(2));
                    });
                });
                ui.add_space(6.0);

                card(ui, |ui| {
                    h2(ui, "🔍  Canlı Zoom");
                    ui.horizontal(|ui| {
                        ui.label("Zoom oranı:");
                        let mut z = r.zoom;
                        if ui
                            .add(
                                egui::Slider::new(&mut z, 1.0..=4.0)
                                    .fixed_decimals(2)
                                    .suffix("×"),
                            )
                            .changed()
                        {
                            r.zoom = z;
                            self.engine.shared.set_zoom(z);
                        }
                        if ui.add(secondary_button("Sıfırla (1×)")).clicked() {
                            r.zoom = 1.0;
                            self.engine.shared.set_zoom(1.0);
                        }
                    });
                    muted(
                        ui,
                        "Kayıt sırasında imleç odaklı canlı yakınlaştırma yapılır.",
                    );
                });
                ui.add_space(6.0);

                card(ui, |ui| {
                    h2(ui, "⏰  Zamanlı Kayıt");
                    let mut enabled = self.scheduler.is_some();
                    ui.checkbox(&mut enabled, RichText::new("Zamanlayıcı etkin").size(13.0));
                    if !enabled {
                        self.scheduler = None;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Başlangıç (SS:DD):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.sched_start_text)
                                .hint_text("14:30")
                                .desired_width(70.0),
                        );
                        ui.label("Bitiş (SS:DD):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.sched_end_text)
                                .hint_text("15:00")
                                .desired_width(70.0),
                        );
                    });
                    if ui.add(primary_button("⏰  Zamanlayıcıyı Ayarla")).clicked() {
                        self.set_scheduler();
                    }
                    if !self.sched_status.is_empty() {
                        ui.label(
                            RichText::new(&self.sched_status)
                                .size(12.0)
                                .color(theme::yellow()),
                        );
                    }
                });
                ui.add_space(6.0);

                card(ui, |ui| {
                    h2(ui, "⌨  Kısayol Tuşları");
                    if !self.hotkey_error.is_empty() {
                        ui.label(
                            RichText::new(format!("⚠ {}", self.hotkey_error))
                                .size(11.0)
                                .color(theme::yellow()),
                        );
                    }
                    let defaults = model::Shortcuts::default();
                    for action in HotAction::ALL {
                        let current = self.shortcut_text(action).to_string();
                        egui::Frame::new()
                            .fill(theme::card2())
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(10, 5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "{}  {}",
                                            action.icon(),
                                            action.label()
                                        ))
                                        .size(13.0)
                                        .color(theme::txt()),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.add(secondary_button("Sıfırla")).clicked() {
                                                let d = match action {
                                                    HotAction::Start => defaults.start.clone(),
                                                    HotAction::Pause => defaults.pause.clone(),
                                                    HotAction::Stop => defaults.stop.clone(),
                                                    HotAction::Screenshot => {
                                                        defaults.screenshot.clone()
                                                    }
                                                    HotAction::Draw => defaults.draw.clone(),
                                                };
                                                self.set_shortcut(action, d);
                                            }
                                            if ui.add(secondary_button("Değiştir")).clicked() {
                                                self.hotkey_capture = Some(action);
                                                self.hotkey_captured.clear();
                                            }
                                            ui.label(
                                                RichText::new(if current.is_empty() {
                                                    "—"
                                                } else {
                                                    &current
                                                })
                                                .monospace()
                                                .strong()
                                                .color(theme::accent2()),
                                            );
                                        },
                                    );
                                });
                            });
                    }
                    muted(
                        ui,
                        "Global kısayollar uygulama arka planda olsa da çalışır.",
                    );
                });
                ui.add_space(6.0);

                card(ui, |ui| {
                    h2(ui, "⚙  Genel");
                    ui.horizontal(|ui| {
                        if ui.add(primary_button("💾  Ayarları Kaydet")).clicked() {
                            self.settings.recorder = r.clone();
                            match self.settings.save() {
                                Ok(()) => self.info("Ayarlar kaydedildi."),
                                Err(e) => self.error(format!("Ayarlar kaydedilemedi: {e}")),
                            }
                        }
                        if ui.add(secondary_button("📂 Çıktı Klasörünü Aç")).clicked() {
                            let dir = self.settings.recorder.output_dir.clone();
                            std::process::Command::new("explorer").arg(dir).spawn().ok();
                        }
                        let mut dark = self.settings.dark;
                        if ui.checkbox(&mut dark, "Koyu tema").changed() {
                            self.settings.dark = dark;
                            theme::set_dark(dark);
                            theme::apply(ui.ctx());
                        }
                    });
                    muted(
                        ui,
                        format!("Ayar dosyası: {}", util::settings_path().display()),
                    );
                });

                self.settings.recorder = r;
                ui.add_space(10.0);
            });
    }

    // ------------------------------------------------------------------ yardım
    pub fn ui_help(&mut self, ui: &mut egui::Ui) {
        h1(ui, "❓  Yardım ve Hakkında");
        ui.add_space(4.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            card(ui, |ui| {
                h2(ui, "📌  Temel İşlevler Kılavuzu");
                ui.label(
                    RichText::new(
                        "🎥 Ekran Kaydı: Ekranınızı, belirli bir bölgeyi, web kamerasını ve bilgisayar/mikrofon sesini eşzamanlı kaydedebilirsiniz.\n\
                         🖌 Canlı Çizim: Araç paneli veya kısayol ile kayıt sırasında ekran üzerine çizim yapabilirsiniz.\n\
                         🔍 Zoom: İmleç odaklı canlı yakınlaştırma yapabilirsiniz.\n\
                         ✂ Video Düzenleme: Kırpma, birleştirme, format dönüştürme ve görselden video üretimi.\n\
                         🎵 Ses Editörü: Kırpma, bölme, dönüştürme, birleştirme ve 20+ ses efekti.\n\
                         ⏱ Timeline: Çoklu klip, geçişler, metin, şekil/görsel katmanları ve dışa aktarma.\n\
                         ⏰ Zamanlayıcı: Belirtilen saatlerde otomatik kayıt.",
                    )
                    .size(13.5)
                    .color(theme::txt()),
                );
            });
            ui.add_space(6.0);
            card(ui, |ui| {
                h2(ui, "👤  Geliştirici ve Bağlantılar");
                ui.label(RichText::new("Emrullah ALKAÇ").size(16.0).strong().color(theme::accent()));
                if ui.link("📺 YouTube: @yazılımhocası-p9c").clicked() {
                    webbrowser::open("https://www.youtube.com/@yaz%C4%B1l%C4%B1mhocas%C4%B1-p9c").ok();
                }
                if ui.link("💻 GitHub: github.com/emrullahalkac72-beep").clicked() {
                    webbrowser::open("https://github.com/emrullahalkac72-beep").ok();
                }
                if ui.link("🎬 Kullanım Kılavuzu videosu").clicked() {
                    webbrowser::open("https://youtu.be/epveUk2KvoQ").ok();
                }
            });
            ui.add_space(6.0);
            card(ui, |ui| {
                h2(ui, "📖  Ayrıntılı Kılavuz");
                let text = std::fs::read_to_string("KULLANIM_KILAVUZU.md").unwrap_or_else(|_| {
                    include_str!("../../KULLANIM_KILAVUZU.md").to_string()
                });
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(text)
                                .monospace()
                                .size(12.0)
                                .color(theme::txt2()),
                        )
                        .wrap(),
                    );
                });
            });
            ui.add_space(10.0);
        });
    }

    // ------------------------------------------------------------------ webcam penceresi
    pub fn webcam_viewport(&mut self, ctx: &egui::Context) {
        let Some(tex) = self.webcam_tex.clone() else {
            return;
        };
        let size = tex.size_vec2();
        let scale = (320.0 / size.x).min(240.0 / size.y).min(1.0);
        let win = egui::vec2(size.x * scale + 16.0, size.y * scale + 44.0);
        let mon = primary_monitor_rect();
        let ppp = ctx.pixels_per_point();
        let pos = [
            (mon.0 as f32 + mon.2 as f32) / ppp - win.x - 24.0,
            (mon.1 as f32 + mon.3 as f32) / ppp - win.y - 24.0,
        ];
        let mut open = self.show_cam_preview;
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("webcam_preview"),
            egui::ViewportBuilder::default()
                .with_title("Webcam Önizleme")
                .with_inner_size([win.x, win.y])
                .with_position(pos)
                .with_always_on_top()
                .with_resizable(true),
            |ctx, _class| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(Color32::BLACK))
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("📷 Canlı Webcam")
                                    .size(12.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                            let avail = ui.available_size();
                            let sc = (avail.x / size.x).min((avail.y - 8.0) / size.y).max(0.05);
                            let rect = egui::Rect::from_min_size(
                                ui.cursor().min + egui::vec2((avail.x - size.x * sc) * 0.5, 0.0),
                                size * sc,
                            );
                            ui.painter().image(
                                tex.id(),
                                rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                            ui.allocate_space(egui::vec2(avail.x, size.y * sc));
                        });
                    });
                if ctx.input(|i| i.viewport().close_requested()) {
                    open = false;
                }
            },
        );
        self.show_cam_preview = open;
    }
}

pub fn primary_monitor_rect() -> (i32, i32, u32, u32) {
    if let Ok(ms) = xcap::Monitor::all() {
        if let Some(m) = ms
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| ms.first())
        {
            return (
                m.x().unwrap_or(0),
                m.y().unwrap_or(0),
                m.width().unwrap_or(1920),
                m.height().unwrap_or(1080),
            );
        }
    }
    (0, 0, 1920, 1080)
}
