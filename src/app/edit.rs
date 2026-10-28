//! Video düzenleme, görsel→video ve ses editörü sekmeleri.
use super::*;
use crate::media;
use crate::timeline::ClipEffects;

pub fn pick_video(title: &str) -> Option<String> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(
            "Video",
            &["mp4", "avi", "mkv", "mov", "webm", "m4v", "flv", "wmv"],
        )
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
}

pub fn pick_audio(title: &str) -> Option<String> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(
            "Ses",
            &["mp3", "wav", "ogg", "aac", "flac", "m4a", "opus", "wma"],
        )
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
}

fn pick_videos(title: &str) -> Vec<String> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(
            "Video",
            &["mp4", "avi", "mkv", "mov", "webm", "m4v", "flv", "wmv"],
        )
        .pick_files()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

fn pick_audios(title: &str) -> Vec<String> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter(
            "Ses",
            &["mp3", "wav", "ogg", "aac", "flac", "m4a", "opus", "wma"],
        )
        .pick_files()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

fn append_unique_paths(target: &mut Vec<String>, paths: Vec<String>) {
    for path in paths {
        if !target
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&path))
        {
            target.push(path);
        }
    }
}

fn pick_save(title: &str, default_ext: &str, default_name: &str, dir: &str) -> Option<String> {
    rfd::FileDialog::new()
        .set_title(title)
        .set_directory(dir)
        .set_file_name(default_name)
        .add_filter(default_ext.to_uppercase(), &[default_ext])
        .save_file()
        .map(|p| p.to_string_lossy().to_string())
}

fn duration_label(path: &str) -> String {
    let d = util::duration(path);
    format!(
        "⏱  Süre: {d:.2} sn  ({:02}:{:02})",
        (d / 60.0) as i64,
        (d % 60.0) as i64
    )
}

fn file_row(ui: &mut egui::Ui, value: &mut String, title: &str, video: bool) {
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(if video {
                    "Video dosyası seçin..."
                } else {
                    "Ses dosyası seçin..."
                })
                .desired_width(ui.available_width() - 46.0),
        );
        if ui.add(secondary_button("📂")).clicked() {
            if let Some(p) = if video {
                pick_video(title)
            } else {
                pick_audio(title)
            } {
                *value = p;
            }
        }
    });
}

fn fx_slider(
    ui: &mut egui::Ui,
    label: &str,
    v: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(12.0).color(theme::txt2()));
        ui.add(egui::Slider::new(v, range).fixed_decimals(2).suffix(suffix));
    });
}

fn fx_switch(ui: &mut egui::Ui, label: &str, v: &mut bool) {
    ui.checkbox(v, RichText::new(label).size(12.0));
}

pub fn edit_effects_ui(ui: &mut egui::Ui, eff: &mut ClipEffects) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("🎨 Renk & Işık")
                .size(13.0)
                .strong()
                .color(theme::accent()),
        );
    });
    changed |= ui
        .add(
            egui::Slider::new(&mut eff.brightness, -1.0..=1.0)
                .text("Parlaklık")
                .fixed_decimals(2),
        )
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut eff.contrast, 0.0..=4.0)
                .text("Kontrast")
                .fixed_decimals(2),
        )
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut eff.saturation, 0.0..=3.0)
                .text("Doygunluk")
                .fixed_decimals(2),
        )
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut eff.gamma, 0.1..=3.0)
                .text("Gama")
                .fixed_decimals(2),
        )
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut eff.hue, 0.0..=360.0)
                .text("Ton (Hue)")
                .fixed_decimals(0),
        )
        .changed();
    ui.separator();
    ui.label(
        RichText::new("🎛 Filtreler")
            .size(13.0)
            .strong()
            .color(theme::accent()),
    );
    changed |= ui
        .add(
            egui::Slider::new(&mut eff.blur, 0.0..=20.0)
                .text("Bulanıklık")
                .fixed_decimals(1),
        )
        .changed();
    ui.horizontal_wrapped(|ui| {
        changed |= ui.checkbox(&mut eff.bw, "Siyah Beyaz").changed();
        changed |= ui.checkbox(&mut eff.sepia, "Sepya").changed();
        changed |= ui.checkbox(&mut eff.red_filter, "Kırmızı Filtre").changed();
        changed |= ui.checkbox(&mut eff.blue_filter, "Mavi Filtre").changed();
        changed |= ui.checkbox(&mut eff.vignette, "Vignette").changed();
        changed |= ui.checkbox(&mut eff.sharpen, "Keskinleştir").changed();
        changed |= ui
            .checkbox(&mut eff.invert, "Renkleri Ters Çevir")
            .changed();
    });
    ui.separator();
    ui.label(
        RichText::new("🔁 Dönüşüm")
            .size(13.0)
            .strong()
            .color(theme::accent()),
    );
    ui.horizontal_wrapped(|ui| {
        changed |= ui.checkbox(&mut eff.mirror, "Ayna (Yatay)").changed();
        changed |= ui.checkbox(&mut eff.vflip, "Dikey Çevir").changed();
        changed |= ui.checkbox(&mut eff.rotate, "Döndür (90°)").changed();
    });
    ui.horizontal(|ui| {
        ui.label("Fade In (sn):");
        changed |= ui
            .add(
                egui::DragValue::new(&mut eff.fade_in)
                    .range(0.0..=10.0)
                    .speed(0.05)
                    .fixed_decimals(2),
            )
            .changed();
        ui.label("Fade Out (sn):");
        changed |= ui
            .add(
                egui::DragValue::new(&mut eff.fade_out)
                    .range(0.0..=10.0)
                    .speed(0.05)
                    .fixed_decimals(2),
            )
            .changed();
    });
    changed
}

impl App {
    // ================================================================ video düzenleme
    pub fn ui_edit(&mut self, ui: &mut egui::Ui) {
        h1(ui, "✏  Video Düzenleme");
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let tabs = [
                "✏  Kırp",
                "🔗  Birleştir",
                "🔄  Dönüştür",
                "🎬 GörseldenVideo",
            ];
            for (i, t) in tabs.iter().enumerate() {
                if ui
                    .selectable_label(self.edit_tab == i, RichText::new(*t).size(13.0).strong())
                    .clicked()
                {
                    self.edit_tab = i;
                }
            }
        });
        ui.separator();
        match self.edit_tab {
            0 => self.edit_trim(ui),
            1 => self.edit_merge(ui),
            2 => self.edit_convert(ui),
            _ => self.image_to_video(ui),
        }
        if !util::which("ffmpeg") {
            ui.label(RichText::new("⚠  Bu modül ffmpeg gerektirir.").color(theme::yellow()));
        }
        self.show_job_status(ui);
    }

    fn edit_trim(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Kaynak Video:")
                        .size(13.0)
                        .color(theme::txt2()),
                );
                file_row(ui, &mut self.trim.src, "Video Seç", true);
                if !self.trim.src.is_empty() && ui.add(secondary_button("⏱  Süreyi Oku")).clicked()
                {
                    self.trim.dur = duration_label(&self.trim.src);
                }
                if !self.trim.dur.is_empty() {
                    ui.label(
                        RichText::new(&self.trim.dur)
                            .size(11.0)
                            .color(theme::accent()),
                    );
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Başlangıç (sn):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.trim.t0)
                            .hint_text("0")
                            .desired_width(80.0),
                    );
                    ui.label("Bitiş (sn):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.trim.t1)
                            .hint_text("60")
                            .desired_width(80.0),
                    );
                });
                ui.add_space(6.0);
                if ui.add(primary_button("✏  Kırp ve Kaydet")).clicked() {
                    let src = self.trim.src.clone();
                    let t0: f64 = self.trim.t0.trim().parse().unwrap_or(0.0);
                    let t1: f64 = self.trim.t1.trim().parse().unwrap_or(0.0);
                    if src.is_empty() || !Path::new(&src).exists() {
                        self.error("Lütfen geçerli bir video dosyası seçin.");
                    } else if !t0.is_finite() || !t1.is_finite() || t0 < 0.0 || t1 <= t0 {
                        self.error(
                            "Başlangıç 0 veya daha büyük, bitiş başlangıçtan büyük olmalıdır.",
                        );
                    } else {
                        let dir = self.settings.recorder.output_dir.clone();
                        if let Some(out) =
                            pick_save("Kırpılmış Videoyu Kaydet", "mp4", "kirpilmis.mp4", &dir)
                        {
                            self.run_job(move || {
                                media::VideoEditor::trim(&src, &out, t0, t1)?;
                                Ok(format!("✓ Tamamlandı: {}", util::basename(&out)))
                            });
                        }
                    }
                }
            });
    }

    fn edit_merge(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(
            RichText::new("Birleştirilecek Videolar (sırayla ekleyin):")
                .size(13.0)
                .color(theme::txt2()),
        );
        egui::ScrollArea::vertical()
            .id_salt("merge_list")
            .max_height(190.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, p) in self.merge.list.iter().enumerate() {
                    let selected = self.merge.sel == Some(i);
                    let text = format!("{}. {}", i + 1, util::shorten(&util::basename(p), 60));
                    if ui.selectable_label(selected, text).clicked() {
                        self.merge.sel = Some(i);
                    }
                }
            });
        let mut act_up = false;
        let mut act_down = false;
        let mut act_remove = false;
        ui.horizontal_wrapped(|ui| {
            if ui.add(secondary_button("➕ Çoklu Dosya Ekle")).clicked() {
                let paths = pick_videos("Birleştirilecek Videoları Seç (çoklu seçim)");
                append_unique_paths(&mut self.merge.list, paths);
            }
            if ui.add(danger_button("🗑 Listeyi Temizle")).clicked() {
                self.merge.list.clear();
                self.merge.sel = None;
            }
            if ui.add(danger_button("➖ Seçileni Kaldır")).clicked() {
                act_remove = true;
            }
            if ui.add(secondary_button("⬆ Yukarı")).clicked() {
                act_up = true;
            }
            if ui.add(secondary_button("⬇ Aşağı")).clicked() {
                act_down = true;
            }
        });
        if let Some(i) = self.merge.sel {
            if act_remove && i < self.merge.list.len() {
                self.merge.list.remove(i);
                self.merge.sel = None;
            }
            if act_up && i > 0 {
                self.merge.list.swap(i, i - 1);
                self.merge.sel = Some(i - 1);
            }
            if act_down && i + 1 < self.merge.list.len() {
                self.merge.list.swap(i, i + 1);
                self.merge.sel = Some(i + 1);
            }
        }
        ui.add_space(6.0);
        if ui.add(primary_button("🔗  Birleştir ve Kaydet")).clicked() {
            if self.merge.list.len() < 2 {
                self.error("En az 2 video eklemeniz gerekiyor.");
            } else {
                let dir = self.settings.recorder.output_dir.clone();
                if let Some(out) = pick_save(
                    "Birleştirilmiş Videoyu Kaydet",
                    "mp4",
                    "birlestirilmis.mp4",
                    &dir,
                ) {
                    let paths = self.merge.list.clone();
                    self.run_job(move || {
                        media::VideoEditor::merge(&paths, &out)?;
                        Ok(format!("✓ Tamamlandı: {}", util::basename(&out)))
                    });
                }
            }
        }
    }

    fn edit_convert(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(
            RichText::new("Kaynak Video:")
                .size(13.0)
                .color(theme::txt2()),
        );
        file_row(ui, &mut self.convert.src, "Video Seç", true);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Hedef Format:")
                .size(13.0)
                .color(theme::txt2()),
        );
        let formats = ["mp4", "avi", "mkv", "mov", "gif", "webm", "mp3", "wav"];
        ui.horizontal_wrapped(|ui| {
            for f in formats {
                ui.selectable_value(&mut self.convert.fmt, f.to_string(), f.to_uppercase());
            }
        });
        ui.add_space(8.0);
        if ui.add(primary_button("🔄  Dönüştür ve Kaydet")).clicked() {
            let src = self.convert.src.clone();
            let fmt = self.convert.fmt.clone();
            if src.is_empty() || !Path::new(&src).exists() {
                self.error("Lütfen geçerli bir video dosyası seçin.");
            } else {
                let dir = self.settings.recorder.output_dir.clone();
                let default_name = format!("donusturulmus.{fmt}");
                if let Some(out) =
                    pick_save("Dönüştürülmüş Dosyayı Kaydet", &fmt, &default_name, &dir)
                {
                    self.run_job(move || {
                        media::VideoEditor::convert(&src, &out, &fmt)?;
                        Ok(format!("✓ Tamamlandı: {}", util::basename(&out)))
                    });
                }
            }
        }
    }

    // ================================================================ görsel → video
    fn image_to_video(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.add(primary_button("➕ Görsel Ekle")).clicked() {
                if let Some(paths) = rfd::FileDialog::new()
                    .set_title("Görselleri Seç")
                    .add_filter("Görseller", &["png", "jpg", "jpeg", "bmp", "webp"])
                    .pick_files()
                {
                    let dur = self
                        .img2vid
                        .def_dur
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(5.0)
                        .max(0.1);
                    for p in paths {
                        self.img2vid
                            .items
                            .push(ImageItem::new(p.to_string_lossy().to_string(), dur));
                    }
                }
            }
            if ui.add(danger_button("🗑 Listeyi Temizle")).clicked() {
                self.img2vid.items.clear();
                self.img2vid.effects_idx = None;
                self.img2vid.fx_tex = None;
            }
            if ui.add(secondary_button("➕ Tümünü Ekle")).clicked() {
                if let Some(paths) = rfd::FileDialog::new()
                    .add_filter("Görseller", &["png", "jpg", "jpeg", "bmp", "webp"])
                    .pick_files()
                {
                    let dur = self
                        .img2vid
                        .def_dur
                        .trim()
                        .parse::<f32>()
                        .unwrap_or(5.0)
                        .max(0.1);
                    for p in paths {
                        self.img2vid
                            .items
                            .push(ImageItem::new(p.to_string_lossy().to_string(), dur));
                    }
                }
            }
        });
        ui.separator();

        ui.columns(2, |cols| {
            let ui = &mut cols[0];
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new("📁 Görsel Sırası & Süreler")
                    .size(13.0)
                    .strong(),
            );
            egui::ScrollArea::vertical()
                .id_salt("img_list")
                .max_height(330.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut remove = None;
                    let mut open_fx = None;
                    let mut move_item: Option<(usize, isize)> = None;
                    for (i, item) in self.img2vid.items.iter_mut().enumerate() {
                        theme::card_frame().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "🎨 {}",
                                        util::shorten(&item.name(), 28)
                                    ))
                                    .size(12.0)
                                    .color(theme::txt()),
                                );
                                ui.label("Süre:");
                                ui.add(
                                    egui::DragValue::new(&mut item.duration)
                                        .range(0.1..=120.0)
                                        .speed(0.1)
                                        .fixed_decimals(1),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("✕").clicked() {
                                            remove = Some(i);
                                        }
                                        if ui.small_button("🎨").clicked() {
                                            open_fx = Some(i);
                                        }
                                        if ui.small_button("⬆").clicked() {
                                            move_item = Some((i, -1));
                                        }
                                        if ui.small_button("⬇").clicked() {
                                            move_item = Some((i, 1));
                                        }
                                    },
                                );
                            });
                        });
                    }
                    if let Some(i) = remove {
                        self.img2vid.items.remove(i);
                        self.img2vid.effects_idx = None;
                    }
                    if let Some((i, d)) = move_item {
                        let j = (i as isize + d) as usize;
                        if j < self.img2vid.items.len() {
                            self.img2vid.items.swap(i, j);
                        }
                    }
                    if let Some(i) = open_fx {
                        self.img2vid.effects_idx = Some(i);
                        self.img2vid.fx_key = 0;
                        self.img2vid.fx_dirty = Some(Instant::now());
                    }
                });
            cols[1].vertical(|ui| {
                ui.label(RichText::new("⚙ Çıktı Ayarları").size(13.0).strong());
                ui.label("Video Çözünürlüğü:");
                egui::ComboBox::from_id_salt("img_res")
                    .selected_text(&self.img2vid.res)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for r in ["1920x1080", "1280x720", "854x480", "Özel"] {
                            ui.selectable_value(&mut self.img2vid.res, r.to_string(), r);
                        }
                    });
                if self.img2vid.res == "Özel" {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.img2vid.custom_w)
                                .hint_text("Genişlik")
                                .desired_width(80.0),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.img2vid.custom_h)
                                .hint_text("Yükseklik")
                                .desired_width(80.0),
                        );
                    });
                }
                ui.label("Varsayılan Süre (sn):");
                ui.add(egui::TextEdit::singleline(&mut self.img2vid.def_dur).desired_width(80.0));
                ui.label("Kayıt Yeri & İsim:");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.img2vid.out)
                            .hint_text("gorsel_video.mp4")
                            .desired_width(ui.available_width() - 46.0),
                    );
                    if ui.add(secondary_button("📂")).clicked() {
                        let dir = self.settings.recorder.output_dir.clone();
                        if let Some(p) =
                            pick_save("Görsel Videosunu Kaydet", "mp4", "gorsel_video.mp4", &dir)
                        {
                            self.img2vid.out = p;
                        }
                    }
                });
                ui.add_space(6.0);
                if ui.add(primary_button("🎬 Video Oluştur")).clicked() {
                    let (w, h) = if self.img2vid.res == "Özel" {
                        (
                            self.img2vid
                                .custom_w
                                .trim()
                                .parse::<u32>()
                                .unwrap_or(1920)
                                .clamp(16, 16_384),
                            self.img2vid
                                .custom_h
                                .trim()
                                .parse::<u32>()
                                .unwrap_or(1080)
                                .clamp(16, 16_384),
                        )
                    } else {
                        let (a, b) = self.img2vid.res.split_once('x').unwrap_or(("1920", "1080"));
                        (a.parse().unwrap_or(1920), b.parse().unwrap_or(1080))
                    };
                    let mut out = self.img2vid.out.clone();
                    if out.trim().is_empty() {
                        out = std::path::Path::new(&self.settings.recorder.output_dir)
                            .join("gorsel_video.mp4")
                            .to_string_lossy()
                            .to_string();
                    } else if std::path::Path::new(&out).is_dir() || out.ends_with(['\\', '/']) {
                        out = std::path::Path::new(&out)
                            .join("gorsel_video.mp4")
                            .to_string_lossy()
                            .to_string();
                        self.img2vid.out = out.clone();
                    }
                    if self.img2vid.items.is_empty() {
                        self.error("Lütfen en az bir görsel ekleyin.");
                    } else {
                        let items = self.img2vid.items.clone();
                        let status = self.job_status.clone();
                        self.run_job(move || {
                            media::generate_images(&items, &out, w, h, status)?;
                            Ok(format!("✓ Video oluşturuldu: {}", util::basename(&out)))
                        });
                    }
                }
                ui.add_space(4.0);
                muted(
                    ui,
                    format!(
                        "Toplam süre: {:.1} sn",
                        self.img2vid.items.iter().map(|i| i.duration).sum::<f32>()
                    ),
                );
            });
        });
    }

    pub fn img_effects_window(&mut self, ctx: &egui::Context) {
        let Some(idx) = self.img2vid.effects_idx else {
            return;
        };
        if idx >= self.img2vid.items.len() {
            self.img2vid.effects_idx = None;
            return;
        }
        let mut open = true;
        let mut eff = self.img2vid.items[idx].effects.clone();
        let path = self.img2vid.items[idx].path.clone();
        let mut changed = false;
        egui::Window::new("🎨 Efekt ve Filtre Ayarları")
            .resizable(true)
            .default_width(720.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.columns(2, |cols| {
                    egui::ScrollArea::vertical()
                        .max_height(430.0)
                        .show(&mut cols[0], |ui| {
                            changed = edit_effects_ui(ui, &mut eff);
                            ui.add_space(6.0);
                            if ui.add(secondary_button("🔄 Önizlemeyi Yenile")).clicked() {
                                self.img2vid.fx_key = 0;
                                self.img2vid.fx_dirty =
                                    Some(Instant::now() - Duration::from_secs(1));
                            }
                        });
                    cols[1].vertical_centered(|ui| {
                        ui.label(RichText::new("📸 Canlı Önizleme").size(13.0).strong());
                        let avail = ui.available_size();
                        if let Some(tex) = self.img2vid.fx_tex.clone() {
                            let s = tex.size_vec2();
                            let sc = (avail.x / s.x).min((avail.y - 12.0) / s.y).max(0.05);
                            let rect = egui::Rect::from_min_size(
                                ui.cursor().min + egui::vec2((avail.x - s.x * sc) * 0.5, 0.0),
                                s * sc,
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
                            ui.allocate_space(egui::vec2(avail.x, s.y * sc));
                        } else {
                            ui.label(
                                RichText::new("⏳ Önizleme hazırlanıyor...").color(theme::txt2()),
                            );
                            ui.allocate_space(egui::vec2(avail.x, 120.0));
                        }
                    });
                });
            });
        if changed {
            self.img2vid.items[idx].effects = eff.clone();
            self.img2vid.fx_dirty = Some(Instant::now());
        }
        if !open {
            self.img2vid.effects_idx = None;
            self.img2vid.fx_tex = None;
            return;
        }
        let hkey = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            format!("{:?}", eff).hash(&mut h);
            h.finish()
        };
        let should = hkey != self.img2vid.fx_key
            && self
                .img2vid
                .fx_dirty
                .map(|t| t.elapsed() > Duration::from_millis(450))
                .unwrap_or(false);
        if should {
            let out = util::unique_temp("jpg");
            if media::preview_image(&path, &eff, &out).is_ok() {
                self.img2vid.fx_tex = load_tex(ctx, &out, format!("imgfx{idx}"));
                let _ = std::fs::remove_file(out);
            }
            self.img2vid.fx_key = hkey;
        }
    }

    // ================================================================ ses editörü
    pub fn ui_audio(&mut self, ui: &mut egui::Ui) {
        h1(ui, "🎵  Ses Editörü");
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            let tabs = [
                "✂  Kırp",
                "🔀  Böl",
                "🔄  Dönüştür",
                "🔗  Birleştir",
                "✨  Efektler",
            ];
            for (i, t) in tabs.iter().enumerate() {
                if ui
                    .selectable_label(self.audio.tab == i, RichText::new(*t).size(13.0).strong())
                    .clicked()
                {
                    self.audio.tab = i;
                }
            }
        });
        ui.separator();
        match self.audio.tab {
            0 => self.audio_trim(ui),
            1 => self.audio_split(ui),
            2 => self.audio_convert(ui),
            3 => self.audio_merge(ui),
            _ => self.audio_effects(ui, ui.ctx().clone()),
        }
        self.show_job_status(ui);
    }

    fn audio_trim(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Kaynak Ses Dosyası:")
                        .size(13.0)
                        .color(theme::txt2()),
                );
                file_row(ui, &mut self.audio.trim.src, "Ses Seç", false);
                if ui.add(secondary_button("⏱  Süreyi Oku")).clicked() {
                    self.audio.trim.dur = duration_label(&self.audio.trim.src);
                }
                ui.label(
                    RichText::new(&self.audio.trim.dur)
                        .size(11.0)
                        .color(theme::accent()),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Başlangıç (sn):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.audio.trim.t0)
                            .hint_text("0")
                            .desired_width(80.0),
                    );
                    ui.label("Bitiş (sn):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.audio.trim.t1)
                            .hint_text("60")
                            .desired_width(80.0),
                    );
                });
                ui.add_space(6.0);
                if ui.add(primary_button("✂  Kırp ve Kaydet")).clicked() {
                    let src = self.audio.trim.src.clone();
                    let t0: f64 = self.audio.trim.t0.trim().parse().unwrap_or(0.0);
                    let t1: f64 = self.audio.trim.t1.trim().parse().unwrap_or(0.0);
                    if src.is_empty() || !Path::new(&src).exists() {
                        self.error("Lütfen geçerli bir ses dosyası seçin.");
                    } else if !t0.is_finite() || !t1.is_finite() || t0 < 0.0 || t1 <= t0 {
                        self.error(
                            "Başlangıç 0 veya daha büyük, bitiş başlangıçtan büyük olmalıdır.",
                        );
                    } else {
                        let ext = Path::new(&src)
                            .extension()
                            .map(|e| e.to_string_lossy().to_string())
                            .unwrap_or_else(|| "mp3".into());
                        let dir = self.settings.recorder.output_dir.clone();
                        if let Some(out) = pick_save(
                            "Kırpılmış Dosyayı Kaydet",
                            &ext,
                            &format!("kirpilmis.{ext}"),
                            &dir,
                        ) {
                            self.run_job(move || {
                                media::AudioEditor::trim(&src, &out, t0, t1)?;
                                Ok(format!("✓ Tamamlandı: {}", util::basename(&out)))
                            });
                        }
                    }
                }
            });
    }

    fn audio_split(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(6.0);
            ui.label(RichText::new("Kaynak Ses Dosyası:").size(13.0).color(theme::txt2()));
            file_row(ui, &mut self.audio.split_src, "Ses Seç", false);
            if ui.add(secondary_button("⏱  Süreyi Oku")).clicked() {
                self.audio.split_dur = duration_label(&self.audio.split_src);
            }
            ui.label(RichText::new(&self.audio.split_dur).size(11.0).color(theme::accent()));
            ui.horizontal(|ui| {
                ui.label("Bölme Noktası (sn):");
                ui.add(egui::TextEdit::singleline(&mut self.audio.split_time).hint_text("30").desired_width(80.0));
            });
            muted(ui, "Dosya belirtilen saniyede ikiye bölünür. Parça 1: 0 → N, Parça 2: N → son.\nİkinci parça, birinci dosya adının sonuna _2 eklenerek kaydedilir.");
            ui.add_space(6.0);
            if ui.add(primary_button("🔀  Böl ve Kaydet")).clicked() {
                let src = self.audio.split_src.clone();
                let split: f64 = self.audio.split_time.trim().parse().unwrap_or(0.0);
                if src.is_empty() || !Path::new(&src).exists() {
                    self.error("Lütfen geçerli bir ses dosyası seçin.");
                } else if split <= 0.0 {
                    self.error("Bölme noktası 0'dan büyük olmalıdır.");
                } else if split >= util::duration(&src) - 0.01 {
                    self.error("Bölme noktası ses süresinden küçük olmalıdır.");
                } else {
                    let ext = Path::new(&src).extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_else(|| "mp3".into());
                    let dir = self.settings.recorder.output_dir.clone();
                    if let Some(out1) = pick_save("1. Parçanın Adını ve Yerini Seçin", &ext, &format!("parca1.{ext}"), &dir) {
                        let p = Path::new(&out1);
                        let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "parca1".into());
                        let out2 = p.with_file_name(format!("{stem}_2.{ext}")).to_string_lossy().to_string();
                        self.run_job(move || {
                            media::AudioEditor::split(&src, &out1, &out2, split)?;
                            Ok(format!("✓ Tamamlandı: {} + {}", util::basename(&out1), util::basename(&out2)))
                        });
                    }
                }
            }
        });
    }

    fn audio_convert(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Kaynak Ses Dosyası:")
                        .size(13.0)
                        .color(theme::txt2()),
                );
                file_row(ui, &mut self.audio.conv_src, "Ses Seç", false);
                ui.horizontal(|ui| {
                    ui.label("Çıktı Formatı:");
                    egui::ComboBox::from_id_salt("aconv_fmt")
                        .selected_text(&self.audio.conv_fmt)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            for f in media::AudioEditor::FORMATS {
                                ui.selectable_value(&mut self.audio.conv_fmt, f.to_string(), f);
                            }
                        });
                    ui.label("Bit Hızı:");
                    egui::ComboBox::from_id_salt("aconv_br")
                        .selected_text(&self.audio.conv_br)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            for b in ["64k", "96k", "128k", "160k", "192k", "256k", "320k"] {
                                ui.selectable_value(&mut self.audio.conv_br, b.to_string(), b);
                            }
                        });
                    muted(ui, "(WAV / FLAC için geçersiz)");
                });
                ui.add_space(6.0);
                if ui.add(primary_button("🔄  Dönüştür ve Kaydet")).clicked() {
                    let src = self.audio.conv_src.clone();
                    let fmt = self.audio.conv_fmt.clone();
                    let br = self.audio.conv_br.clone();
                    if src.is_empty() || !Path::new(&src).exists() {
                        self.error("Lütfen geçerli bir ses dosyası seçin.");
                    } else {
                        let dir = self.settings.recorder.output_dir.clone();
                        if let Some(out) = pick_save(
                            "Dönüştürülmüş Dosyayı Kaydet",
                            &fmt,
                            &format!("donusturulmus.{fmt}"),
                            &dir,
                        ) {
                            self.run_job(move || {
                                media::AudioEditor::convert(&src, &out, &fmt, &br)?;
                                Ok(format!("✓ Tamamlandı: {}", util::basename(&out)))
                            });
                        }
                    }
                }
            });
    }

    fn audio_merge(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(
            RichText::new("Birleştirilecek Ses Dosyaları (sırayla ekleyin):")
                .size(13.0)
                .color(theme::txt2()),
        );
        egui::ScrollArea::vertical()
            .id_salt("amerge_list")
            .max_height(180.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, p) in self.audio.merge.iter().enumerate() {
                    if ui
                        .selectable_label(
                            self.audio.merge_sel == Some(i),
                            format!("{}. {}", i + 1, util::shorten(&util::basename(p), 64)),
                        )
                        .clicked()
                    {
                        self.audio.merge_sel = Some(i);
                    }
                }
            });
        let mut up = false;
        let mut down = false;
        let mut rm = false;
        ui.horizontal_wrapped(|ui| {
            if ui.add(secondary_button("➕ Çoklu Dosya Ekle")).clicked() {
                let paths = pick_audios("Birleştirilecek Ses Dosyalarını Seç (çoklu seçim)");
                append_unique_paths(&mut self.audio.merge, paths);
            }
            if ui.add(danger_button("🗑 Listeyi Temizle")).clicked() {
                self.audio.merge.clear();
                self.audio.merge_sel = None;
            }
            if ui.add(danger_button("➖ Seçileni Kaldır")).clicked() {
                rm = true;
            }
            if ui.add(secondary_button("⬆ Yukarı")).clicked() {
                up = true;
            }
            if ui.add(secondary_button("⬇ Aşağı")).clicked() {
                down = true;
            }
            ui.label("Çıktı Formatı:");
            egui::ComboBox::from_id_salt("amerge_fmt")
                .selected_text(&self.audio.merge_fmt)
                .width(90.0)
                .show_ui(ui, |ui| {
                    for f in media::AudioEditor::FORMATS {
                        ui.selectable_value(&mut self.audio.merge_fmt, f.to_string(), f);
                    }
                });
        });
        if let Some(i) = self.audio.merge_sel {
            if rm && i < self.audio.merge.len() {
                self.audio.merge.remove(i);
                self.audio.merge_sel = None;
            }
            if up && i > 0 {
                self.audio.merge.swap(i, i - 1);
                self.audio.merge_sel = Some(i - 1);
            }
            if down && i + 1 < self.audio.merge.len() {
                self.audio.merge.swap(i, i + 1);
                self.audio.merge_sel = Some(i + 1);
            }
        }
        ui.add_space(6.0);
        if ui.add(primary_button("🔗  Birleştir ve Kaydet")).clicked() {
            if self.audio.merge.len() < 2 {
                self.error("En az 2 ses dosyası eklemeniz gerekiyor.");
            } else {
                let fmt = self.audio.merge_fmt.clone();
                let dir = self.settings.recorder.output_dir.clone();
                if let Some(out) = pick_save(
                    "Birleştirilmiş Dosyayı Kaydet",
                    &fmt,
                    &format!("birlestirilmis.{fmt}"),
                    &dir,
                ) {
                    let paths = self.audio.merge.clone();
                    self.run_job(move || {
                        media::AudioEditor::merge(&paths, &out)?;
                        Ok(format!("✓ Tamamlandı: {}", util::basename(&out)))
                    });
                }
            }
        }
    }

    fn audio_effects(&mut self, ui: &mut egui::Ui, ctx: egui::Context) {
        // arka plan işlerini topla
        if let Some(rx) = self.afx.wave_rx.take() {
            match rx.try_recv() {
                Ok(Ok(p)) => {
                    self.afx.wave_busy = false;
                    self.afx.wave_tex = load_tex(&ctx, &p, "waveform");
                    let _ = std::fs::remove_file(p);
                }
                Ok(Err(e)) => {
                    self.afx.wave_busy = false;
                    self.afx.status = format!("✗ Dalga formu hatası: {e}");
                }
                Err(mpsc::TryRecvError::Empty) => self.afx.wave_rx = Some(rx),
                Err(mpsc::TryRecvError::Disconnected) => self.afx.wave_busy = false,
            }
        }
        if let Some(rx) = self.afx.preview_rx.take() {
            match rx.try_recv() {
                Ok(Ok(p)) => {
                    self.afx.preview_busy = false;
                    if let Some(mut c) = self.afx.preview.take() {
                        let _ = c.kill();
                        let _ = c.wait();
                    }
                    let pstr = p.to_string_lossy().to_string();
                    match util::ffplay()
                        .args(["-nodisp", "-autoexit", "-loglevel", "quiet", pstr.as_str()])
                        .stdin(std::process::Stdio::null())
                        .spawn()
                    {
                        Ok(c) => {
                            self.afx.preview = Some(c);
                            self.afx.status = "▶ Önizleme çalıyor...".into();
                        }
                        Err(e) => self.afx.status = format!("✗ Oynatıcı hatası: {e}"),
                    }
                }
                Ok(Err(e)) => {
                    self.afx.preview_busy = false;
                    self.afx.status = format!("✗ Önizleme hatası: {e}");
                }
                Err(mpsc::TryRecvError::Empty) => self.afx.preview_rx = Some(rx),
                Err(mpsc::TryRecvError::Disconnected) => self.afx.preview_busy = false,
            }
        }

        ui.columns(2, |cols| {
            // sol: ayarlar
            let ui = &mut cols[0];
            egui::ScrollArea::vertical()
                .id_salt("afx_scroll")
                .max_height(430.0)
                .show(ui, |ui| {
                    theme::card_frame().show(ui, |ui| {
                        ui.label(
                            RichText::new("🎛 Profesyonel İyileştirmeler")
                                .size(13.0)
                                .strong()
                                .color(theme::accent()),
                        );
                        fx_switch(
                            ui,
                            "Otomatik Normalizasyon (Loudnorm)",
                            &mut self.afx.params.loudnorm,
                        );
                        fx_switch(
                            ui,
                            "Gürültü Temizleme (Denoise)",
                            &mut self.afx.params.noise,
                        );
                        fx_switch(
                            ui,
                            "Ses Patlaması Önleyici (Compressor)",
                            &mut self.afx.params.compressor,
                        );
                        fx_switch(
                            ui,
                            "Boşlukları/Sessizliği Sil",
                            &mut self.afx.params.silence,
                        );
                        fx_switch(ui, "Mono'yu Stereo'ya Çevir", &mut self.afx.params.stereo);
                        fx_slider(
                            ui,
                            "Ana Ses Seviyesi:",
                            &mut self.afx.params.volume,
                            0.0..=4.0,
                            "",
                        );
                    });
                    theme::card_frame().show(ui, |ui| {
                        ui.label(
                            RichText::new("📻 Ekolayzer & Filtreler")
                                .size(13.0)
                                .strong()
                                .color(theme::accent()),
                        );
                        fx_slider(
                            ui,
                            "High-Pass (Hz):",
                            &mut self.afx.params.highpass,
                            0.0..=2000.0,
                            " Hz",
                        );
                        fx_slider(
                            ui,
                            "Low-Pass (Hz):",
                            &mut self.afx.params.lowpass,
                            500.0..=20000.0,
                            " Hz",
                        );
                        fx_slider(
                            ui,
                            "Bas (Bass) dB:",
                            &mut self.afx.params.bass,
                            -15.0..=15.0,
                            " dB",
                        );
                        fx_slider(
                            ui,
                            "Tiz (Treble) dB:",
                            &mut self.afx.params.treble,
                            -15.0..=15.0,
                            " dB",
                        );
                        fx_slider(
                            ui,
                            "Sol/Sağ Denge (Pan):",
                            &mut self.afx.params.panning,
                            -1.0..=1.0,
                            "",
                        );
                    });
                    theme::card_frame().show(ui, |ui| {
                        ui.label(
                            RichText::new("🎤 Vokal & Karaoke")
                                .size(13.0)
                                .strong()
                                .color(theme::accent()),
                        );
                        fx_switch(
                            ui,
                            "Vokal Silici (Karaoke Altyapısı)",
                            &mut self.afx.params.vocal_remove,
                        );
                    });
                    theme::card_frame().show(ui, |ui| {
                        ui.label(
                            RichText::new("🌀 Modülasyon Efektleri")
                                .size(13.0)
                                .strong()
                                .color(theme::accent()),
                        );
                        fx_switch(ui, "Chorus (Çoklu Ses Efekti)", &mut self.afx.params.chorus);
                        fx_switch(
                            ui,
                            "Flanger (Metalik Dalgalanma)",
                            &mut self.afx.params.flanger,
                        );
                        fx_switch(ui, "Vibrato (Ses Titremesi)", &mut self.afx.params.vibrato);
                        fx_switch(
                            ui,
                            "Tremolo (Hacim Titremesi)",
                            &mut self.afx.params.tremolo,
                        );
                    });
                    theme::card_frame().show(ui, |ui| {
                        ui.label(
                            RichText::new("🎭 Yaratıcı & Eğlenceli (FX)")
                                .size(13.0)
                                .strong()
                                .color(theme::accent()),
                        );
                        fx_slider(
                            ui,
                            "Ses Tonu (Pitch):",
                            &mut self.afx.params.pitch,
                            0.5..=2.0,
                            "×",
                        );
                        fx_slider(
                            ui,
                            "Hız (Tempo):",
                            &mut self.afx.params.speed,
                            0.5..=2.0,
                            "×",
                        );
                        fx_switch(ui, "Yankı ve Derinlik (Echo)", &mut self.afx.params.echo);
                        fx_switch(ui, "Tersine Çevir (Reverse)", &mut self.afx.params.reverse);
                    });
                    theme::card_frame().show(ui, |ui| {
                        ui.label(
                            RichText::new("⏱ Geçişler (Fade)")
                                .size(13.0)
                                .strong()
                                .color(theme::accent()),
                        );
                        ui.horizontal(|ui| {
                            ui.label("Fade In (sn):");
                            ui.add(
                                egui::DragValue::new(&mut self.afx.params.fade_in)
                                    .range(0.0..=10.0)
                                    .speed(0.05)
                                    .fixed_decimals(1),
                            );
                            ui.label("Fade Out (sn):");
                            ui.add(
                                egui::DragValue::new(&mut self.afx.params.fade_out)
                                    .range(0.0..=10.0)
                                    .speed(0.05)
                                    .fixed_decimals(1),
                            );
                        });
                    });
                    ui.add_space(4.0);
                    if ui
                        .add(secondary_button("↩ Tüm Efektleri Sıfırla"))
                        .clicked()
                    {
                        self.afx.params = AudioEffectsParams::default();
                    }
                });

            // sağ: dosya, dalga formu, önizleme
            cols[1].vertical(|ui| {
                theme::card_frame().show(ui, |ui| {
                    ui.label(RichText::new("🎙 Dosya & Önizleme").size(14.0).strong());
                    if ui.add(primary_button("📂 Ses Dosyası Seç")).clicked() {
                        if let Some(paths) = rfd::FileDialog::new()
                            .set_title("Ses Seç")
                            .add_filter(
                                "Ses Dosyaları",
                                &["mp3", "wav", "ogg", "aac", "flac", "m4a", "opus", "wma"],
                            )
                            .pick_files()
                        {
                            self.afx.files = paths
                                .iter()
                                .map(|p| p.to_string_lossy().to_string())
                                .collect();
                            self.afx.src = self.afx.files.first().cloned().unwrap_or_default();
                            self.afx.wave_tex = None;
                            self.afx.status.clear();
                            if !self.afx.src.is_empty() {
                                self.afx.wave_busy = true;
                                let src = self.afx.src.clone();
                                let (tx, rx) = mpsc::channel();
                                self.afx.wave_rx = Some(rx);
                                std::thread::spawn(move || {
                                    let _ = tx.send(media::AudioEffectsEngine::waveform(&src));
                                });
                            }
                        }
                    }
                    let info = if self.afx.files.is_empty() {
                        "Dosya seçilmedi".to_string()
                    } else if self.afx.files.len() == 1 {
                        format!("Seçili: {}", util::basename(&self.afx.src))
                    } else {
                        format!("Toplu İşlem: {} dosya seçildi", self.afx.files.len())
                    };
                    ui.label(RichText::new(info).size(11.0).color(theme::txt2()));
                    if let Some(tex) = self.afx.wave_tex.clone() {
                        let s = tex.size_vec2();
                        let sc = (ui.available_width() / s.x).min(1.6);
                        let rect = egui::Rect::from_min_size(ui.cursor().min, s * sc);
                        ui.painter().image(
                            tex.id(),
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        ui.allocate_space(egui::vec2(ui.available_width(), s.y * sc));
                    } else {
                        ui.allocate_space(egui::vec2(ui.available_width(), 80.0));
                    }
                    if !self.afx.status.is_empty() {
                        ui.label(
                            RichText::new(&self.afx.status)
                                .size(11.0)
                                .color(theme::yellow()),
                        );
                    }
                    ui.horizontal(|ui| {
                        if ui.add(danger_button("⏹")).clicked() {
                            if let Some(mut c) = self.afx.preview.take() {
                                let _ = c.kill();
                                let _ = c.wait();
                            }
                            self.afx.status = "⏹ Durduruldu.".into();
                        }
                        if ui
                            .add_enabled(
                                !self.afx.preview_busy && !self.afx.src.is_empty(),
                                primary_button(if self.afx.preview_busy {
                                    "⏳ Hazırlanıyor..."
                                } else {
                                    "▶ Önizle"
                                }),
                            )
                            .clicked()
                        {
                            let src = self.afx.src.clone();
                            let params = self.afx.params.clone();
                            self.afx.preview_busy = true;
                            self.afx.status = "⏳ Önizleme hazırlanıyor (max 45 sn)...".into();
                            let (tx, rx) = mpsc::channel();
                            self.afx.preview_rx = Some(rx);
                            std::thread::spawn(move || {
                                let _ =
                                    tx.send(media::AudioEffectsEngine::preview_file(&params, &src));
                            });
                        }
                    });
                    if ui
                        .add(primary_button("✨ Efektleri Uygula & Kaydet"))
                        .clicked()
                    {
                        if self.afx.files.is_empty() {
                            self.error("Ses dosyası seçin.");
                        } else {
                            let params = self.afx.params.clone();
                            let files = self.afx.files.clone();
                            if files.len() == 1 {
                                let src = files[0].clone();
                                let ext = Path::new(&src)
                                    .extension()
                                    .map(|e| e.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "mp3".into());
                                let dir = self.settings.recorder.output_dir.clone();
                                if let Some(out) = pick_save(
                                    "Dosyayı Kaydet",
                                    &ext,
                                    &format!("efektli.{ext}"),
                                    &dir,
                                ) {
                                    self.run_job(move || {
                                        media::AudioEffectsEngine::apply(&params, &src, &out)?;
                                        Ok(format!("✓ Tamamlandı: {}", util::basename(&out)))
                                    });
                                }
                            } else if let Some(dir) = rfd::FileDialog::new()
                                .set_title("Toplu İşlem: Kaydedilecek Klasörü Seçin")
                                .pick_folder()
                            {
                                self.run_job(move || {
                                    let mut ok = 0usize;
                                    for src in &files {
                                        let ext = Path::new(src)
                                            .extension()
                                            .map(|e| e.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "mp3".into());
                                        let name = Path::new(src)
                                            .file_stem()
                                            .map(|s| s.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "ses".into());
                                        let out = dir
                                            .join(format!("{name}_FX.{ext}"))
                                            .to_string_lossy()
                                            .to_string();
                                        if media::AudioEffectsEngine::apply(&params, src, &out)
                                            .is_ok()
                                        {
                                            ok += 1;
                                        }
                                    }
                                    Ok(format!(
                                        "✓ {ok}/{} dosya işlendi. Klasör: {}",
                                        files.len(),
                                        dir.display()
                                    ))
                                });
                            }
                        }
                    }
                });
            });
        });
    }
}
