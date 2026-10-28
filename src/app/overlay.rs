//! Saydam çizim katmanı, yüzen araç paneli ve bölge seçici.
use super::*;
use crate::draw::{DrawOp, Tool};

/// Windows: çizim katmanı tamamen GPU'dan bağımsız, kendi Win32 katmanlı penceremizdir
/// (UpdateLayeredWindow, per-pixel alfa). Böylece arka plan asla siyah olamaz.
/// Fare girişi bu pencereye gelir; durum GetCursorPos/GetAsyncKeyState ile okunur.
#[cfg(windows)]
mod native {
    use super::{map_op, DrawOp};
    use std::collections::hash_map::DefaultHasher;
    use std::ffi::c_void;
    use std::hash::{Hash, Hasher};
    use std::ptr::{null, null_mut};
    use std::sync::{Mutex, Once};
    use windows_sys::Win32::Foundation::{POINT, RECT, SIZE};
    use windows_sys::Win32::Graphics::Gdi::{
        CombineRgn, CreateCompatibleDC, CreateDIBSection, CreateRectRgn, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, SelectObject, SetWindowRgn, BITMAPINFO, BLENDFUNCTION, RGN_DIFF,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, FindWindowW, GetCursorPos, GetWindowLongW,
        GetWindowRect, LoadCursorW, RegisterClassW, SetWindowLongW, SetWindowPos, ShowWindow,
        UpdateLayeredWindow, WindowFromPoint, GWL_EXSTYLE, IDC_CROSS, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SW_SHOWNOACTIVATE, ULW_ALPHA, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };

    struct State {
        hwnd: isize,
        dc: isize,
        bmp: isize,
        old: isize,
        bits: usize,
        w: i32,
        h: i32,
        hit: bool,
        prev_down: bool,
        hash: u64,
        panel_key: Option<(i32, i32, i32, i32)>,
    }

    static STATE: Mutex<State> = Mutex::new(State {
        hwnd: 0,
        dc: 0,
        bmp: 0,
        old: 0,
        bits: 0,
        w: 0,
        h: 0,
        hit: true,
        prev_down: false,
        hash: 0,
        panel_key: None,
    });
    static REGISTER: Once = Once::new();

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe fn free(st: &mut State) {
        if st.hwnd != 0 {
            DestroyWindow(st.hwnd as _);
        }
        if st.dc != 0 {
            if st.old != 0 {
                SelectObject(st.dc as _, st.old as _);
            }
            if st.bmp != 0 {
                DeleteObject(st.bmp as _);
            }
            DeleteDC(st.dc as _);
        }
        st.hwnd = 0;
        st.dc = 0;
        st.bmp = 0;
        st.old = 0;
        st.bits = 0;
        st.w = 0;
        st.h = 0;
        st.hash = 0;
        st.prev_down = false;
        st.panel_key = None;
    }

    /// Çizim katmanını kaldırır (çizim modu kapanınca).
    pub fn destroy() {
        let mut st = STATE.lock().unwrap_or_else(|e| e.into_inner());
        if st.hwnd != 0 {
            unsafe { free(&mut st) };
        }
    }

    /// Pencereyi (gerekirse) oluşturur; Fare aracında tıklamayı alttaki uygulamaya geçirir.
    pub fn ensure(x: i32, y: i32, w: i32, h: i32, click_through: bool) {
        if w <= 0 || h <= 0 {
            return;
        }
        let mut st = STATE.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            if st.hwnd == 0 || st.w != w || st.h != h {
                free(&mut st);
                let cname = wide("AptozaOverlayLayer");
                REGISTER.call_once(|| {
                    let mut wc: WNDCLASSW = std::mem::zeroed();
                    wc.lpfnWndProc = Some(DefWindowProcW);
                    wc.hCursor = LoadCursorW(null_mut(), IDC_CROSS);
                    wc.lpszClassName = cname.as_ptr();
                    RegisterClassW(&wc);
                });
                let title = wide("Aptoza Çizim Katmanı");
                let hwnd = CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                    cname.as_ptr(),
                    title.as_ptr(),
                    WS_POPUP,
                    x,
                    y,
                    w,
                    h,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null(),
                );
                if hwnd.is_null() {
                    return;
                }
                let screen = GetDC(null_mut());
                let dc = CreateCompatibleDC(screen);
                let mut bi: BITMAPINFO = std::mem::zeroed();
                bi.bmiHeader.biSize = std::mem::size_of_val(&bi.bmiHeader) as u32;
                bi.bmiHeader.biWidth = w;
                bi.bmiHeader.biHeight = -h; // yukarıdan aşağı
                bi.bmiHeader.biPlanes = 1;
                bi.bmiHeader.biBitCount = 32;
                bi.bmiHeader.biCompression = 0; // BI_RGB
                let mut bits: *mut c_void = null_mut();
                let bmp = CreateDIBSection(screen, &bi, 0, &mut bits, null_mut(), 0);
                ReleaseDC(null_mut(), screen);
                if bmp.is_null() || bits.is_null() {
                    DestroyWindow(hwnd);
                    DeleteDC(dc);
                    return;
                }
                let old = SelectObject(dc, bmp);
                st.hwnd = hwnd as isize;
                st.dc = dc as isize;
                st.bmp = bmp as isize;
                st.old = old as isize;
                st.bits = bits as usize;
                st.w = w;
                st.h = h;
                st.hash = 0;
                // Araç paneli açıksa katmanı panelin hemen altına yerleştir (panel tıklanabilir kalsın).
                let panel_title = wide("Araç ve Zoom Paneli");
                let panel = FindWindowW(null(), panel_title.as_ptr());
                if !panel.is_null() {
                    SetWindowPos(
                        hwnd,
                        panel,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                }
                ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            }
            let hwnd = st.hwnd as _;
            let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            let want = if click_through {
                ex | WS_EX_TRANSPARENT
            } else {
                ex & !WS_EX_TRANSPARENT
            };
            if want != ex {
                SetWindowLongW(hwnd, GWL_EXSTYLE, want as i32);
            }
            // Araç panelinin kapladığı alanı katmandan çıkar: panel tıklamaları panele ulaşsın.
            let ptitle = wide("Araç ve Zoom Paneli");
            let panel = FindWindowW(null(), ptitle.as_ptr());
            let mut key = None;
            if !panel.is_null() {
                let mut r: RECT = std::mem::zeroed();
                if GetWindowRect(panel, &mut r) != 0 {
                    key = Some((r.left - x, r.top - y, r.right - x, r.bottom - y));
                }
            }
            if key != st.panel_key {
                st.panel_key = key;
                match key {
                    Some((l, t, r, b)) => {
                        let full = CreateRectRgn(0, 0, w, h);
                        let hole = CreateRectRgn(l, t, r, b);
                        CombineRgn(full, full, hole, RGN_DIFF);
                        DeleteObject(hole);
                        SetWindowRgn(hwnd, full, 1);
                    }
                    None => {
                        SetWindowRgn(hwnd, null_mut(), 1);
                    }
                }
            }
            if st.hit == click_through {
                st.hit = !click_through;
                st.hash = 0;
            }
        }
    }

    /// (imleç x, imleç y, sol tuş basılı, imleç çizim katmanının üstünde) – ekran pikseli.
    /// Ayrıca önceki karedeki basılı durumu döndürür.
    pub fn mouse() -> Option<(i32, i32, bool, bool, bool)> {
        let mut st = STATE.lock().unwrap_or_else(|e| e.into_inner());
        if st.hwnd == 0 {
            return None;
        }
        unsafe {
            let mut p = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut p) == 0 {
                return None;
            }
            let down = (GetAsyncKeyState(1) as u16) & 0x8000 != 0;
            let over = WindowFromPoint(p) as isize == st.hwnd;
            let was = st.prev_down;
            st.prev_down = down;
            Some((p.x, p.y, down, was, over))
        }
    }

    /// Çizimleri (ekran piksel koordinatlarına çevirerek) katmanlı pencereye basar.
    pub fn update(x: i32, y: i32, w: i32, h: i32, ops: &[DrawOp], ppp: f32) {
        let mut st = STATE.lock().unwrap_or_else(|e| e.into_inner());
        if st.hwnd == 0 || st.w != w || st.h != h {
            return;
        }
        let mapped: Vec<DrawOp> = ops.iter().map(|o| map_op(o, (0.0, 0.0), ppp)).collect();
        let mut hasher = DefaultHasher::new();
        format!("{:?}|{}|{}|{}", mapped, x, y, st.hit).hash(&mut hasher);
        let hash = hasher.finish() | 1;
        if hash == st.hash {
            return;
        }
        st.hash = hash;
        unsafe {
            let rgba = crate::draw::rasterize(&mapped, w as u32, h as u32);
            let dst =
                std::slice::from_raw_parts_mut(st.bits as *mut u8, (w as usize) * (h as usize) * 4);
            for (i, px) in rgba.chunks_exact(4).enumerate() {
                let a = px[3] as u32;
                if a == 0 {
                    // Neredeyse görünmez ama tıklanabilir (alfa=0 pikseller fareyi geçirir).
                    dst[i * 4] = 0;
                    dst[i * 4 + 1] = 0;
                    dst[i * 4 + 2] = 0;
                    dst[i * 4 + 3] = if st.hit { 1 } else { 0 };
                } else {
                    dst[i * 4] = (px[2] as u32 * a / 255) as u8; // B
                    dst[i * 4 + 1] = (px[1] as u32 * a / 255) as u8; // G
                    dst[i * 4 + 2] = (px[0] as u32 * a / 255) as u8; // R
                    dst[i * 4 + 3] = a as u8;
                }
            }
            let pt_dst = POINT { x, y };
            let sz = SIZE { cx: w, cy: h };
            let pt_src = POINT { x: 0, y: 0 };
            let bf = BLENDFUNCTION {
                BlendOp: 0,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: 1,
            };
            UpdateLayeredWindow(
                st.hwnd as _,
                null_mut(),
                &pt_dst,
                &sz,
                st.dc as _,
                &pt_src,
                0,
                &bf,
                ULW_ALPHA,
            );
        }
    }
}

/// Çizim modu kapanınca Windows'taki görünür çizim katmanını kaldırır.
#[cfg(windows)]
pub fn destroy_display() {
    native::destroy();
}

/// Araç panelindeki simgeleri yazı tipine bağımlı olmadan vektörel çizer.
fn tool_icon_button(ui: &mut egui::Ui, tool: Tool, selected: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(32.0, 28.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if selected {
            theme::accent()
        } else if resp.hovered() {
            theme::card2().linear_multiply(1.5)
        } else {
            theme::card2()
        };
        painter.rect_filled(rect, 5.0, bg);
        let col = if selected {
            egui::Color32::WHITE
        } else {
            theme::txt()
        };
        let stroke = egui::Stroke::new(1.5_f32, col);
        let c = rect.center();
        let r = egui::Rect::from_center_size(c, egui::vec2(13.0, 11.0));
        match tool {
            Tool::Mouse => {
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        c + egui::vec2(-3.0, -6.5),
                        c + egui::vec2(5.5, 1.5),
                        c + egui::vec2(-1.5, 1.5),
                    ],
                    col,
                    egui::Stroke::NONE,
                ));
                painter.line_segment(
                    [c + egui::vec2(-0.5, 1.0), c + egui::vec2(3.0, 6.0)],
                    egui::Stroke::new(2.6_f32, col),
                );
            }
            Tool::Rectangle => {
                painter.rect_stroke(r, 1.5, stroke, egui::StrokeKind::Inside);
            }
            Tool::Circle => {
                painter.circle_stroke(c, 6.0, stroke);
            }
            Tool::Arrow => {
                painter.line_segment(
                    [c + egui::vec2(-6.0, 5.0), c + egui::vec2(5.0, -5.0)],
                    stroke,
                );
                painter.line_segment(
                    [c + egui::vec2(5.0, -5.0), c + egui::vec2(-0.5, -5.0)],
                    stroke,
                );
                painter.line_segment(
                    [c + egui::vec2(5.0, -5.0), c + egui::vec2(5.0, 0.5)],
                    stroke,
                );
            }
            Tool::Line => {
                painter.line_segment(
                    [c + egui::vec2(-6.0, 5.0), c + egui::vec2(6.0, -5.0)],
                    stroke,
                );
            }
            Tool::Freehand => {
                painter.add(egui::Shape::line(
                    vec![
                        c + egui::vec2(-6.5, 3.5),
                        c + egui::vec2(-3.0, -4.5),
                        c + egui::vec2(0.0, 2.5),
                        c + egui::vec2(3.0, -4.5),
                        c + egui::vec2(6.5, 2.5),
                    ],
                    stroke,
                ));
            }
            Tool::Text => {
                painter.text(
                    c,
                    egui::Align2::CENTER_CENTER,
                    "T",
                    egui::FontId::proportional(15.0),
                    col,
                );
            }
            Tool::Eraser => {
                painter.rect_stroke(r.shrink(0.5), 2.0, stroke, egui::StrokeKind::Inside);
                painter.line_segment(
                    [
                        r.left_top() + egui::vec2(2.5, 2.5),
                        r.right_bottom() - egui::vec2(2.5, 2.5),
                    ],
                    stroke,
                );
            }
        };
    }
    resp
}

fn map_pt(x: f32, y: f32, origin: (f32, f32), ppp: f32) -> (f32, f32) {
    (origin.0 + x * ppp, origin.1 + y * ppp)
}

fn map_op(op: &DrawOp, origin: (f32, f32), ppp: f32) -> DrawOp {
    match op {
        DrawOp::Rect {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            let (a, b) = map_pt(*x0, *y0, origin, ppp);
            let (c, d) = map_pt(*x1, *y1, origin, ppp);
            DrawOp::Rect {
                x0: a,
                y0: b,
                x1: c,
                y1: d,
                color: *color,
                width: width * ppp,
            }
        }
        DrawOp::Ellipse {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            let (a, b) = map_pt(*x0, *y0, origin, ppp);
            let (c, d) = map_pt(*x1, *y1, origin, ppp);
            DrawOp::Ellipse {
                x0: a,
                y0: b,
                x1: c,
                y1: d,
                color: *color,
                width: width * ppp,
            }
        }
        DrawOp::Line {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            let (a, b) = map_pt(*x0, *y0, origin, ppp);
            let (c, d) = map_pt(*x1, *y1, origin, ppp);
            DrawOp::Line {
                x0: a,
                y0: b,
                x1: c,
                y1: d,
                color: *color,
                width: width * ppp,
            }
        }
        DrawOp::Arrow {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            let (a, b) = map_pt(*x0, *y0, origin, ppp);
            let (c, d) = map_pt(*x1, *y1, origin, ppp);
            DrawOp::Arrow {
                x0: a,
                y0: b,
                x1: c,
                y1: d,
                color: *color,
                width: width * ppp,
            }
        }
        DrawOp::Freehand {
            points,
            color,
            width,
        } => DrawOp::Freehand {
            points: points
                .iter()
                .map(|(x, y)| map_pt(*x, *y, origin, ppp))
                .collect(),
            color: *color,
            width: width * ppp,
        },
        DrawOp::Text {
            x,
            y,
            text,
            size,
            color,
        } => {
            let (a, b) = map_pt(*x, *y, origin, ppp);
            DrawOp::Text {
                x: a,
                y: b,
                text: text.clone(),
                size: size * ppp,
                color: *color,
            }
        }
        DrawOp::Erase { points, radius } => DrawOp::Erase {
            points: points
                .iter()
                .map(|(x, y)| map_pt(*x, *y, origin, ppp))
                .collect(),
            radius: radius * ppp,
        },
    }
}

fn paint_op(painter: &egui::Painter, op: &DrawOp, _ppp: f32) {
    match op {
        DrawOp::Rect {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            let rect = egui::Rect::from_two_pos(egui::pos2(*x0, *y0), egui::pos2(*x1, *y1));
            painter.rect_stroke(
                rect,
                egui::CornerRadius::ZERO,
                egui::Stroke::new(*width, Color32::from_rgb(color[0], color[1], color[2])),
                egui::StrokeKind::Inside,
            );
        }
        DrawOp::Ellipse {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            let c = egui::pos2((x0 + x1) * 0.5, (y0 + y1) * 0.5);
            let rx = (x1 - x0).abs() * 0.5;
            let ry = (y1 - y0).abs() * 0.5;
            let steps = ((rx.max(ry) * 1.5).max(24.0)) as usize;
            let pts: Vec<egui::Pos2> = (0..=steps)
                .map(|i| {
                    let a = i as f32 / steps as f32 * std::f32::consts::TAU;
                    egui::pos2(c.x + rx * a.cos(), c.y + ry * a.sin())
                })
                .collect();
            painter.add(egui::Shape::line(
                pts,
                egui::Stroke::new(*width, Color32::from_rgb(color[0], color[1], color[2])),
            ));
        }
        DrawOp::Line {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            painter.line_segment(
                [egui::pos2(*x0, *y0), egui::pos2(*x1, *y1)],
                egui::Stroke::new(*width, Color32::from_rgb(color[0], color[1], color[2])),
            );
        }
        DrawOp::Arrow {
            x0,
            y0,
            x1,
            y1,
            color,
            width,
        } => {
            let col = Color32::from_rgb(color[0], color[1], color[2]);
            painter.line_segment(
                [egui::pos2(*x0, *y0), egui::pos2(*x1, *y1)],
                egui::Stroke::new(*width, col),
            );
            let ang = (y1 - y0).atan2(x1 - x0);
            let len = 15.0 + width;
            for off in [0.5f32, -0.5] {
                let p = egui::pos2(x1 - len * (ang + off).cos(), y1 - len * (ang + off).sin());
                painter.line_segment([egui::pos2(*x1, *y1), p], egui::Stroke::new(*width, col));
            }
        }
        DrawOp::Freehand {
            points,
            color,
            width,
        } => {
            let col = Color32::from_rgb(color[0], color[1], color[2]);
            if points.len() == 1 {
                painter.circle_filled(
                    egui::pos2(points[0].0, points[0].1),
                    (*width * 0.5).max(0.5),
                    col,
                );
            }
            for pair in points.windows(2) {
                painter.line_segment(
                    [
                        egui::pos2(pair[0].0, pair[0].1),
                        egui::pos2(pair[1].0, pair[1].1),
                    ],
                    egui::Stroke::new(*width, col),
                );
            }
        }
        DrawOp::Text {
            x,
            y,
            text,
            size,
            color,
        } => {
            painter.text(
                egui::pos2(*x, *y),
                egui::Align2::LEFT_TOP,
                text,
                egui::FontId::proportional(*size),
                Color32::from_rgb(color[0], color[1], color[2]),
            );
        }
        DrawOp::Erase { .. } => {}
    }
}

fn is_near(op: &DrawOp, p: egui::Pos2, r: f32) -> bool {
    match op {
        DrawOp::Rect { x0, y0, x1, y1, .. } | DrawOp::Ellipse { x0, y0, x1, y1, .. } => {
            egui::Rect::from_two_pos(egui::pos2(*x0, *y0), egui::pos2(*x1, *y1))
                .expand(r)
                .contains(p)
        }
        DrawOp::Line { x0, y0, x1, y1, .. } | DrawOp::Arrow { x0, y0, x1, y1, .. } => {
            let a = egui::pos2(*x0, *y0);
            let b = egui::pos2(*x1, *y1);
            let ab = b - a;
            let t = ((p - a).dot(ab) / ab.length_sq().max(0.001)).clamp(0.0, 1.0);
            (a + ab * t - p).length() <= r
        }
        DrawOp::Freehand { points, .. } | DrawOp::Erase { points, .. } => points
            .iter()
            .any(|(x, y)| (egui::pos2(*x, *y) - p).length() <= r),
        DrawOp::Text { x, y, .. } => (egui::pos2(*x, *y) - p).length() <= r * 3.0,
    }
}

impl App {
    /// Windows: GPU yüzeyine bağımlı olmayan, katmanlı pencereyle çizim katmanı.
    #[cfg(windows)]
    pub fn draw_overlay_viewport(&mut self, ctx: &egui::Context) {
        let (mx, my, mw, mh) = match self.settings.recorder.region.filter(|r| r.valid()) {
            Some(r) => (r.x as f32, r.y as f32, r.w as f32, r.h as f32),
            None => {
                let m = super::record::primary_monitor_rect();
                (m.0 as f32, m.1 as f32, m.2 as f32, m.3 as f32)
            }
        };
        let ppp = ctx.pixels_per_point();
        let origin = (mx, my);
        let tool = self.draw.tool;
        let color = self.draw.color;
        let thickness = self.draw.thickness;
        let mut ops = self.overlay_ops.clone();
        let ops_before = ops.clone();
        let rec_before = self.draw.snapshot();
        let mut freehand = self.overlay_freehand.clone();
        let mut drag_start = self.draw.drag_start;
        let mut text_state = self.overlay_text.clone();
        let mut new_rec: Vec<DrawOp> = Vec::new();
        let mut text_commit: Option<(f32, f32, String)> = None;
        let mut preview: Option<DrawOp> = None;

        native::ensure(
            mx as i32,
            my as i32,
            mw as i32,
            mh as i32,
            tool == Tool::Mouse,
        );

        let shape = |s: (f32, f32), p: egui::Pos2| -> DrawOp {
            match tool {
                Tool::Rectangle => DrawOp::Rect {
                    x0: s.0,
                    y0: s.1,
                    x1: p.x,
                    y1: p.y,
                    color,
                    width: thickness,
                },
                Tool::Circle => DrawOp::Ellipse {
                    x0: s.0,
                    y0: s.1,
                    x1: p.x,
                    y1: p.y,
                    color,
                    width: thickness,
                },
                Tool::Line => DrawOp::Line {
                    x0: s.0,
                    y0: s.1,
                    x1: p.x,
                    y1: p.y,
                    color,
                    width: thickness,
                },
                _ => DrawOp::Arrow {
                    x0: s.0,
                    y0: s.1,
                    x1: p.x,
                    y1: p.y,
                    color,
                    width: thickness,
                },
            }
        };

        if let Some((cx, cy, down, was_down, over)) = native::mouse() {
            let p = egui::pos2((cx as f32 - mx) / ppp, (cy as f32 - my) / ppp);
            let pressed = down && !was_down && over;
            let released = !down && was_down;
            let is_shape = matches!(
                tool,
                Tool::Rectangle | Tool::Circle | Tool::Line | Tool::Arrow
            );
            if tool == Tool::Mouse {
                drag_start = None;
                freehand.clear();
            } else if tool == Tool::Text {
                if pressed && text_state.is_none() {
                    text_state = Some((p, String::new()));
                }
            } else {
                if pressed {
                    drag_start = Some((p.x, p.y));
                    freehand = vec![(p.x, p.y)];
                }
                if down {
                    if let Some(s) = drag_start {
                        if is_shape {
                            preview = Some(shape(s, p));
                        } else {
                            crate::draw::push_pen_point(&mut freehand, (p.x, p.y), 1.0);
                        }
                    }
                }
                if released {
                    if let Some(s) = drag_start.take() {
                        if is_shape {
                            let dist = ((p.x - s.0).powi(2) + (p.y - s.1).powi(2)).sqrt();
                            if dist > 2.0 {
                                let op = shape(s, p);
                                new_rec.push(map_op(&op, origin, ppp));
                                ops.push(op);
                            }
                        } else if tool == Tool::Freehand && !freehand.is_empty() {
                            let op = DrawOp::Freehand {
                                points: crate::draw::smooth_freehand(&freehand),
                                color,
                                width: thickness,
                            };
                            new_rec.push(map_op(&op, origin, ppp));
                            ops.push(op);
                        } else if tool == Tool::Eraser && !freehand.is_empty() {
                            let op = DrawOp::Erase {
                                points: freehand.clone(),
                                radius: thickness * 2.0,
                            };
                            ops.retain(|old| {
                                !freehand
                                    .iter()
                                    .any(|q| is_near(old, egui::pos2(q.0, q.1), 14.0))
                            });
                            new_rec.push(map_op(&op, origin, ppp));
                        }
                    }
                    freehand.clear();
                }
            }
        }

        // Metin girişi: ayrı küçük, opak pencere.
        if let Some((p, text)) = text_state.clone() {
            let mut txt = text;
            let mut done = false;
            let mut cancel = false;
            let sx = mx / ppp + p.x;
            let sy = my / ppp + p.y;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("overlay_text_entry"),
                egui::ViewportBuilder::default()
                    .with_title("Metin Ekle")
                    .with_decorations(false)
                    .with_always_on_top()
                    .with_resizable(false)
                    .with_position([sx, sy])
                    .with_inner_size([250.0, 66.0]),
                |ctx, _class| {
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::new()
                                .fill(Color32::from_rgb(24, 28, 34))
                                .inner_margin(egui::Margin::same(6)),
                        )
                        .show(ctx, |ui| {
                            let r = ui.add(
                                egui::TextEdit::singleline(&mut txt)
                                    .hint_text("Metin yazın...")
                                    .desired_width(230.0),
                            );
                            r.request_focus();
                            ui.horizontal(|ui| {
                                if ui.small_button("✔ Ekle").clicked() {
                                    done = true;
                                }
                                if ui.small_button("✖").clicked() {
                                    cancel = true;
                                }
                            });
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                done = true;
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                cancel = true;
                            }
                        });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        cancel = true;
                    }
                },
            );
            if done && !txt.trim().is_empty() {
                text_commit = Some((p.x, p.y, txt));
                text_state = None;
            } else if cancel {
                text_state = None;
            } else {
                text_state = Some((p, txt));
            }
        }

        if let Some((x, y, text)) = text_commit {
            let op = DrawOp::Text {
                x,
                y,
                text,
                size: self.draw.text_size,
                color,
            };
            ops.push(op.clone());
            new_rec.push(map_op(&op, origin, ppp));
        }

        // Görünür katmanı güncelle (kayıt sırasında tamamlanan + süren çizimler).
        let mut disp = ops.clone();
        if let Some(op) = preview.take() {
            disp.push(op);
        }
        if tool == Tool::Freehand && freehand.len() > 1 {
            disp.push(DrawOp::Freehand {
                points: freehand.clone(),
                color,
                width: thickness,
            });
        }
        native::update(mx as i32, my as i32, mw as i32, mh as i32, &disp, ppp);

        self.overlay_ops = ops;
        self.overlay_freehand = freehand;
        self.overlay_text = text_state;
        self.draw.drag_start = drag_start;
        if !new_rec.is_empty() {
            self.draw_history_push(rec_before, ops_before);
        }
        for op in new_rec {
            self.draw.push(op);
        }
        // Fare durumu yoklandığı için çizim modunda sürekli yenile.
        ctx.request_repaint_after(Duration::from_millis(16));
    }

    #[cfg(not(windows))]
    pub fn draw_overlay_viewport(&mut self, ctx: &egui::Context) {
        let (mx, my, mw, mh) = match self.settings.recorder.region.filter(|r| r.valid()) {
            Some(r) => (r.x as f32, r.y as f32, r.w as f32, r.h as f32),
            None => {
                let m = super::record::primary_monitor_rect();
                (m.0 as f32, m.1 as f32, m.2 as f32, m.3 as f32)
            }
        };
        let ppp = ctx.pixels_per_point();
        let origin = (mx, my);
        let mut ops = self.overlay_ops.clone();
        let ops_before = ops.clone();
        let rec_before = self.draw.snapshot();
        let tool = self.draw.tool;
        let color = self.draw.color;
        let thickness = self.draw.thickness;
        let mut drag_start = self.draw.drag_start;
        let mut freehand = self.overlay_freehand.clone();
        let mut text_state = self.overlay_text.clone();
        let mut new_rec: Vec<DrawOp> = Vec::new();
        let mut text_commit: Option<(f32, f32, String)> = None;
        let mut preview: Option<DrawOp> = None;
        // Fare-geçirgenlik modu çalışma anında değiştirilince Windows'ta pencere
        // saydamlığı bozulup siyaha dönebiliyor. Bu yüzden "Fare" aracı ile çizim
        // araçları için ayrı pencereler (ayrı ViewportId) kullanılır; her pencere
        // doğru bayraklarla baştan oluşturulur ve sonradan değiştirilmez.
        let passthrough = tool == Tool::Mouse;
        let viewport = egui::ViewportBuilder::default()
            .with_title(if passthrough {
                "Aptoza Çizim Fare"
            } else {
                "Aptoza Çizim"
            })
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(passthrough)
            .with_position([mx / ppp, my / ppp])
            .with_inner_size([mw / ppp, mh / ppp])
            .with_transparent(true);
        // Windows'ta Fare aracında giriş penceresine gerek yok (çizimler katmanlı pencerede).
        let show_input = !(cfg!(windows) && passthrough);
        if show_input {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(if passthrough {
                    "draw_overlay_pass"
                } else {
                    "draw_overlay_draw"
                }),
                viewport,
                |ctx, _class| {
                    ctx.set_visuals({
                        let mut visuals = ctx.style().visuals.clone();
                        visuals.window_fill = Color32::TRANSPARENT;
                        visuals.panel_fill = Color32::TRANSPARENT;
                        visuals.extreme_bg_color = Color32::TRANSPARENT;
                        visuals
                    });
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE.fill(Color32::TRANSPARENT))
                        .show(ctx, |ui| {
                            let size = ui.available_size();
                            let (resp, painter) =
                                ui.allocate_painter(size, egui::Sense::click_and_drag());
                            let local = |p: egui::Pos2| {
                                egui::pos2(p.x - resp.rect.min.x, p.y - resp.rect.min.y)
                            };
                            for op in &ops {
                                paint_op(&painter, op, ppp);
                            }

                            let pos = resp.interact_pointer_pos().map(local);
                            if tool == Tool::Text {
                                if resp.clicked() {
                                    if let Some(p) = pos {
                                        text_state = Some((p, String::new()));
                                    }
                                }
                            } else if resp.drag_started() {
                                if let Some(p) = pos {
                                    drag_start = Some((p.x, p.y));
                                    if tool == Tool::Freehand || tool == Tool::Eraser {
                                        freehand = vec![(p.x, p.y)];
                                    }
                                }
                            } else if resp.dragged() {
                                if let Some(p) = pos {
                                    match tool {
                                        Tool::Freehand => crate::draw::push_pen_point(
                                            &mut freehand,
                                            (p.x, p.y),
                                            1.0,
                                        ),
                                        Tool::Eraser => crate::draw::push_pen_point(
                                            &mut freehand,
                                            (p.x, p.y),
                                            1.0,
                                        ),
                                        Tool::Rectangle
                                        | Tool::Circle
                                        | Tool::Line
                                        | Tool::Arrow => {
                                            if let Some((sx, sy)) = drag_start {
                                                let op = match tool {
                                                    Tool::Rectangle => DrawOp::Rect {
                                                        x0: sx,
                                                        y0: sy,
                                                        x1: p.x,
                                                        y1: p.y,
                                                        color,
                                                        width: thickness,
                                                    },
                                                    Tool::Circle => DrawOp::Ellipse {
                                                        x0: sx,
                                                        y0: sy,
                                                        x1: p.x,
                                                        y1: p.y,
                                                        color,
                                                        width: thickness,
                                                    },
                                                    Tool::Line => DrawOp::Line {
                                                        x0: sx,
                                                        y0: sy,
                                                        x1: p.x,
                                                        y1: p.y,
                                                        color,
                                                        width: thickness,
                                                    },
                                                    _ => DrawOp::Arrow {
                                                        x0: sx,
                                                        y0: sy,
                                                        x1: p.x,
                                                        y1: p.y,
                                                        color,
                                                        width: thickness,
                                                    },
                                                };
                                                paint_op(&painter, &op, 1.0);
                                                preview = Some(op);
                                            }
                                        }
                                        _ => {}
                                    }
                                    if tool == Tool::Freehand && freehand.len() > 1 {
                                        paint_op(
                                            &painter,
                                            &DrawOp::Freehand {
                                                points: freehand.clone(),
                                                color,
                                                width: thickness,
                                            },
                                            1.0,
                                        );
                                    }
                                }
                            } else if resp.drag_stopped() {
                                if let Some((sx, sy)) = drag_start.take() {
                                    if tool == Tool::Freehand {
                                        if !freehand.is_empty() {
                                            let op = DrawOp::Freehand {
                                                points: crate::draw::smooth_freehand(&freehand),
                                                color,
                                                width: thickness,
                                            };
                                            ops.push(op.clone());
                                            new_rec.push(map_op(&op, origin, ppp));
                                        }
                                        freehand.clear();
                                    } else if tool == Tool::Eraser {
                                        if !freehand.is_empty() {
                                            let op = DrawOp::Erase {
                                                points: freehand.clone(),
                                                radius: thickness * 2.0,
                                            };
                                            let hit: Vec<egui::Pos2> = freehand
                                                .iter()
                                                .map(|q| egui::pos2(q.0, q.1))
                                                .collect();
                                            ops.retain(|old| {
                                                !hit.iter().any(|q| is_near(old, *q, 14.0))
                                            });
                                            new_rec.push(map_op(&op, origin, ppp));
                                        }
                                        freehand.clear();
                                    } else if let Some(p) = pos {
                                        let dist = ((p.x - sx).powi(2) + (p.y - sy).powi(2)).sqrt();
                                        if dist > 2.0 {
                                            let op = match tool {
                                                Tool::Rectangle => DrawOp::Rect {
                                                    x0: sx,
                                                    y0: sy,
                                                    x1: p.x,
                                                    y1: p.y,
                                                    color,
                                                    width: thickness,
                                                },
                                                Tool::Circle => DrawOp::Ellipse {
                                                    x0: sx,
                                                    y0: sy,
                                                    x1: p.x,
                                                    y1: p.y,
                                                    color,
                                                    width: thickness,
                                                },
                                                Tool::Line => DrawOp::Line {
                                                    x0: sx,
                                                    y0: sy,
                                                    x1: p.x,
                                                    y1: p.y,
                                                    color,
                                                    width: thickness,
                                                },
                                                Tool::Arrow => DrawOp::Arrow {
                                                    x0: sx,
                                                    y0: sy,
                                                    x1: p.x,
                                                    y1: p.y,
                                                    color,
                                                    width: thickness,
                                                },
                                                _ => DrawOp::Line {
                                                    x0: sx,
                                                    y0: sy,
                                                    x1: p.x,
                                                    y1: p.y,
                                                    color,
                                                    width: thickness,
                                                },
                                            };
                                            new_rec.push(map_op(&op, origin, ppp));
                                            ops.push(op);
                                        }
                                    }
                                }
                            }

                            if resp.hovered() {
                                ctx.set_cursor_icon(match tool {
                                    Tool::Mouse => egui::CursorIcon::Default,
                                    Tool::Text => egui::CursorIcon::Text,
                                    _ => egui::CursorIcon::Crosshair,
                                });
                            }
                        });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        text_state = None;
                    }
                },
            );
        }

        // Metin girişi: görünmez giriş penceresinde okunamayacağından ayrı küçük pencere.
        if let Some((p, text)) = text_state.clone() {
            let mut txt = text;
            let mut done = false;
            let mut cancel = false;
            let sx = mx / ppp + p.x;
            let sy = my / ppp + p.y;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("overlay_text_entry"),
                egui::ViewportBuilder::default()
                    .with_title("Metin Ekle")
                    .with_decorations(false)
                    .with_always_on_top()
                    .with_resizable(false)
                    .with_position([sx, sy])
                    .with_inner_size([250.0, 66.0]),
                |ctx, _class| {
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::new()
                                .fill(Color32::from_rgb(24, 28, 34))
                                .inner_margin(egui::Margin::same(6)),
                        )
                        .show(ctx, |ui| {
                            let r = ui.add(
                                egui::TextEdit::singleline(&mut txt)
                                    .hint_text("Metin yazın...")
                                    .desired_width(230.0),
                            );
                            r.request_focus();
                            ui.horizontal(|ui| {
                                if ui.small_button("✔ Ekle").clicked() {
                                    done = true;
                                }
                                if ui.small_button("✖").clicked() {
                                    cancel = true;
                                }
                            });
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                done = true;
                            }
                            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                cancel = true;
                            }
                        });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        cancel = true;
                    }
                },
            );
            if done && !txt.trim().is_empty() {
                text_commit = Some((p.x, p.y, txt));
                text_state = None;
            } else if cancel {
                text_state = None;
            } else {
                text_state = Some((p, txt));
            }
        }

        if let Some((x, y, text)) = text_commit {
            let op = DrawOp::Text {
                x,
                y,
                text,
                size: self.draw.text_size,
                color,
            };
            ops.push(op.clone());
            new_rec.push(map_op(&op, origin, ppp));
        }
        #[cfg(windows)]
        {
            let mut disp = ops.clone();
            if let Some(op) = preview.take() {
                disp.push(op);
            }
            if tool == Tool::Freehand && freehand.len() > 1 {
                disp.push(DrawOp::Freehand {
                    points: freehand.clone(),
                    color,
                    width: thickness,
                });
            }
            native::update(mx as i32, my as i32, mw as i32, mh as i32, &disp, ppp);
        }
        #[cfg(not(windows))]
        let _ = preview;
        self.overlay_ops = ops;
        self.overlay_freehand = freehand;
        self.overlay_text = text_state;
        self.draw.drag_start = drag_start;
        if !new_rec.is_empty() {
            self.draw_history_push(rec_before, ops_before);
        }
        for op in new_rec {
            self.draw.push(op);
        }
        if self.engine.is_active() {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
    }

    pub fn tools_viewport(&mut self, ctx: &egui::Context) {
        let mon = super::record::primary_monitor_rect();
        let width = 566.0;
        let ppp = ctx.pixels_per_point();
        let x = mon.0 as f32 / ppp + (mon.2 as f32 / ppp - width) * 0.5;
        let y = mon.1 as f32 / ppp + 10.0;

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("tool_panel"),
            egui::ViewportBuilder::default()
                .with_title("Araç ve Zoom Paneli")
                .with_decorations(true)
                .with_resizable(true)
                .with_always_on_top()
                .with_position([x.max(0.0), y.max(0.0)])
                .with_inner_size([width, 96.0]),
            |ctx, _class| {
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(theme::card())
                            .inner_margin(egui::Margin::symmetric(8, 6)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            for t in Tool::ALL {
                                if tool_icon_button(ui, t, self.draw.tool == t)
                                    .on_hover_text(t.label())
                                    .clicked()
                                {
                                    self.draw.tool = t;
                                }
                            }
                            ui.separator();
                            let mut rgb = self.draw.color;
                            if ui.color_edit_button_srgb(&mut rgb).changed() {
                                self.draw.color = rgb;
                            }
                            ui.add_sized(
                                [86.0, 20.0],
                                egui::Slider::new(&mut self.draw.thickness, 1.0..=16.0)
                                    .show_value(false),
                            )
                            .on_hover_text(format!(
                                "Çizgi kalınlığı: {:.0} px",
                                self.draw.thickness
                            ));
                            if ui
                                .add_enabled(!self.draw_undo.is_empty(), egui::Button::new("↩"))
                                .on_hover_text("Son çizimi geri al (Ctrl+Z)")
                                .clicked()
                            {
                                self.drawing_undo();
                            }
                            if ui
                                .add_enabled(!self.draw_redo.is_empty(), egui::Button::new("↪"))
                                .on_hover_text("Çizimi yinele (Ctrl+Y)")
                                .clicked()
                            {
                                self.drawing_redo();
                            }
                            if ui
                                .small_button("Temizle")
                                .on_hover_text("Tüm çizimleri temizle")
                                .clicked()
                            {
                                self.overlay_ops.clear();
                                self.draw.clear();
                                self.clear_draw_history();
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("×").on_hover_text("Paneli kapat").clicked()
                                    {
                                        self.tool_open = false;
                                        self.draw_mode = false;
                                    }
                                },
                            );
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Zoom").size(11.0).color(theme::txt2()));
                            let mut z = self.engine.shared.zoom();
                            if ui
                                .add_sized(
                                    [150.0, 18.0],
                                    egui::Slider::new(&mut z, 1.0..=4.0)
                                        .fixed_decimals(2)
                                        .suffix("×")
                                        .show_value(true),
                                )
                                .changed()
                            {
                                self.settings.recorder.zoom = z;
                                self.engine.shared.set_zoom(z);
                            }
                            if ui
                                .small_button("1×")
                                .on_hover_text("Zoom'u sıfırla")
                                .clicked()
                            {
                                self.settings.recorder.zoom = 1.0;
                                self.engine.shared.set_zoom(1.0);
                            }
                            ui.label(
                                RichText::new(format!(
                                    "{} | {}",
                                    self.draw.tool.label(),
                                    if self.engine.is_active() {
                                        "Kayıt"
                                    } else {
                                        "Hazır"
                                    }
                                ))
                                .size(11.0)
                                .color(theme::accent2()),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.checkbox(
                                        &mut self.draw_mode,
                                        RichText::new("Çizim katmanı").size(11.0),
                                    );
                                },
                            );
                        });
                    });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.tool_open = false;
                }
            },
        );
    }

    pub fn region_viewport(&mut self, ctx: &egui::Context) {
        let Some(shot) = self.region_select.clone() else {
            return;
        };
        let Some(tex) = self.region_tex.clone() else {
            return;
        };
        let ppp = ctx.pixels_per_point();
        let (mx, my) = (self.region_mon.0 as f32, self.region_mon.1 as f32);
        let sw = shot.width() as f32;
        let sh = shot.height() as f32;
        let mut result: Option<Option<Region>> = None;
        let mut drag = self.region_drag;

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("region_select"),
            egui::ViewportBuilder::default()
                .with_title("Bölge Seç")
                .with_decorations(false)
                .with_always_on_top()
                .with_position([mx / ppp, my / ppp])
                .with_inner_size([sw / ppp, sh / ppp]),
            |ctx, _class| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(Color32::BLACK).inner_margin(egui::Margin::ZERO))
                    .show(ctx, |ui| {
                        let rect = ui.max_rect();
                        ui.painter().image(
                            tex.id(),
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        ui.painter().rect_filled(
                            rect,
                            egui::CornerRadius::ZERO,
                            Color32::from_black_alpha(100),
                        );
                        let resp = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                        let pos = resp.interact_pointer_pos();
                        if resp.drag_started() {
                            if let Some(p) = pos {
                                drag = Some((p, p));
                            }
                        } else if resp.dragged() {
                            if let (Some((s, _)), Some(p)) = (drag, pos) {
                                drag = Some((s, p));
                            }
                        } else if resp.drag_stopped() {
                            if let (Some((s, _)), Some(p)) = (drag, pos) {
                                let e = p;
                                let r = egui::Rect::from_two_pos(s, e);
                                if r.width() > 10.0 && r.height() > 10.0 {
                                    let scale_x = sw / rect.width();
                                    let scale_y = sh / rect.height();
                                    let x = mx + (r.min.x - rect.min.x) * scale_x;
                                    let y = my + (r.min.y - rect.min.y) * scale_y;
                                    let w = r.width() * scale_x;
                                    let h = r.height() * scale_y;
                                    result = Some(Some(Region {
                                        x: x.round() as i32,
                                        y: y.round() as i32,
                                        w: w.round() as u32,
                                        h: h.round() as u32,
                                    }));
                                } else {
                                    result = Some(None);
                                }
                            }
                        }
                        if let Some((s, e)) = drag {
                            let r = egui::Rect::from_two_pos(s, e);
                            let col = Color32::from_rgb(35, 134, 54);
                            ui.painter().rect_stroke(
                                r,
                                egui::CornerRadius::ZERO,
                                egui::Stroke::new(2.0_f32, col),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(r.min, egui::vec2(r.width(), 22.0)),
                                egui::CornerRadius::ZERO,
                                Color32::from_black_alpha(180),
                            );
                            let info = format!("{}×{}", r.width(), r.height());
                            ui.painter().text(
                                r.min + egui::vec2(6.0, 3.0),
                                egui::Align2::LEFT_TOP,
                                info,
                                egui::FontId::monospace(13.0),
                                Color32::WHITE,
                            );
                        }
                        ui.painter().text(
                            rect.center_top() + egui::vec2(0.0, 24.0),
                            egui::Align2::CENTER_TOP,
                            "Bölge seçmek için fareyle sürükleyin  •  ESC: iptal  •  Enter: tam ekran",
                            egui::FontId::proportional(14.0),
                            Color32::WHITE,
                        );
                    });
                if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.viewport().close_requested()) {
                    result = Some(None);
                } else if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    result = Some(Some(Region { x: mx as i32, y: my as i32, w: sw as u32, h: sh as u32 }));
                }
            },
        );

        self.region_drag = drag;
        if let Some(r) = result {
            self.region_select = None;
            self.region_tex = None;
            self.region_drag = None;
            self.set_region(r.filter(|x| x.valid()));
        }
    }
}
