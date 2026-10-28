//! AptozaQ3 — modern ekran kaydedici, video/ses editörü ve timeline.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod app;
mod draw;
mod media;
mod model;
mod recorder;
mod theme;
mod timeline;
mod util;

fn main() -> eframe::Result<()> {
    let size = default_size();
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("AptozaQ3 — Ekran Kayıt Pro")
        .with_inner_size(size)
        .with_min_inner_size([950.0, 560.0])
        .with_position([14.0, 14.0])
        .with_transparent(true)
        .with_resizable(true);
    if let Some(icon) = util::icon_data() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        // Windows'ta wgpu swapchain'i opak olduğundan şeffaf çizim katmanı siyah
        // görünüyordu; OpenGL (glow) ile per-pixel alfa düzgün çalışır.
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "AptozaQ3",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::new()))),
    )
}

/// Ekranın fiziksel boyutu ve DPI ölçeğine göre sığan mantıksal pencere boyutu.
#[cfg(windows)]
fn default_size() -> [f32; 2] {
    use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SetProcessDPIAware, SM_CXSCREEN, SM_CYSCREEN,
    };
    unsafe {
        // DPI ölçeğini doğru okumak için süreci DPI farkındalığına geçir.
        SetProcessDPIAware();
        let sw = GetSystemMetrics(SM_CXSCREEN) as f32;
        let sh = GetSystemMetrics(SM_CYSCREEN) as f32;
        let dpi = GetDpiForSystem();
        let scale = if dpi == 0 {
            1.0
        } else {
            (dpi as f32 / 96.0).max(1.0)
        };
        // Python sürümündeki 1280x740 fiziksel boyutunu mantıksal puana çevir,
        // ekran çalışma alanına sığdır.
        let w = (sw / scale - 30.0).min(1280.0).max(950.0);
        let h = (sh / scale - 95.0).min(740.0).max(560.0);
        [w, h]
    }
}

#[cfg(not(windows))]
fn default_size() -> [f32; 2] {
    [1280.0, 740.0]
}
