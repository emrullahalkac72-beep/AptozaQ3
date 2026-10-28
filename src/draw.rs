//! Ekran üzerine çizim katmanı: veri modeli, ffmpeg karelerine rasterleştirme,
//! ab_glyph ile metin çizimi.
use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Mouse,
    Rectangle,
    Circle,
    Arrow,
    Line,
    Freehand,
    Text,
    Eraser,
}

impl Tool {
    pub const ALL: [Tool; 8] = [
        Tool::Mouse,
        Tool::Rectangle,
        Tool::Circle,
        Tool::Arrow,
        Tool::Line,
        Tool::Freehand,
        Tool::Text,
        Tool::Eraser,
    ];
    pub fn icon(self) -> &'static str {
        match self {
            Tool::Mouse => "🖱",
            Tool::Rectangle => "▭",
            Tool::Circle => "○",
            Tool::Arrow => "➜",
            Tool::Line => "╱",
            Tool::Freehand => "✏",
            Tool::Text => "T",
            Tool::Eraser => "⌫",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Tool::Mouse => "Fare",
            Tool::Rectangle => "Dikdörtgen",
            Tool::Circle => "Daire",
            Tool::Arrow => "Ok",
            Tool::Line => "Çizgi",
            Tool::Freehand => "Kalem",
            Tool::Text => "Metin",
            Tool::Eraser => "Silgi",
        }
    }
}

#[derive(Clone, Debug)]
pub enum DrawOp {
    Rect {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [u8; 3],
        width: f32,
    },
    Ellipse {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [u8; 3],
        width: f32,
    },
    Line {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [u8; 3],
        width: f32,
    },
    Arrow {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        color: [u8; 3],
        width: f32,
    },
    Freehand {
        points: Vec<(f32, f32)>,
        color: [u8; 3],
        width: f32,
    },
    Text {
        x: f32,
        y: f32,
        text: String,
        size: f32,
        color: [u8; 3],
    },
    Erase {
        points: Vec<(f32, f32)>,
        radius: f32,
    },
}

#[derive(Clone)]
pub struct DrawState {
    pub tool: Tool,
    pub color: [u8; 3],
    pub thickness: f32,
    pub ops: Arc<Mutex<Vec<DrawOp>>>,
    pub drag_start: Option<(f32, f32)>,
    pub current: Option<DrawOp>,
    pub text_dialog: bool,
    pub text_value: String,
    pub text_size: f32,
    pub text_pos: Option<(f32, f32)>,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            tool: Tool::Rectangle,
            color: [255, 68, 68],
            thickness: 3.0,
            ops: Arc::new(Mutex::new(Vec::new())),
            drag_start: None,
            current: None,
            text_dialog: false,
            text_value: "Metin".into(),
            text_size: 18.0,
            text_pos: None,
        }
    }
}

impl DrawState {
    pub fn clear(&self) {
        if let Ok(mut o) = self.ops.lock() {
            o.clear();
        }
    }
    pub fn push(&self, op: DrawOp) {
        if let Ok(mut o) = self.ops.lock() {
            o.push(op);
        }
    }
    /// Çizim geçmişinde kullanılacak anlık görüntü.
    pub fn snapshot(&self) -> Vec<DrawOp> {
        self.ops.lock().map(|o| o.clone()).unwrap_or_default()
    }
    /// Kayıt katmanının içeriğini topluca değiştirir (geri al / yinele).
    pub fn restore(&self, ops: Vec<DrawOp>) {
        if let Ok(mut o) = self.ops.lock() {
            *o = ops;
        }
    }
}

/// Kalem noktalarını çok sık kaydetmemek için küçük hareketleri eler.
pub fn push_pen_point(points: &mut Vec<(f32, f32)>, p: (f32, f32), min_dist: f32) {
    let min2 = min_dist.max(0.0).powi(2);
    if let Some(last) = points.last() {
        let dx = p.0 - last.0;
        let dy = p.1 - last.1;
        if dx * dx + dy * dy < min2 {
            return;
        }
    }
    points.push(p);
}

/// Serbest çizimde köşeleri yumuşatır; ilk/son nokta korunur.
pub fn smooth_freehand(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut out = Vec::with_capacity(points.len());
    out.push(points[0]);
    for w in points.windows(3) {
        out.push((
            w[0].0 * 0.2 + w[1].0 * 0.6 + w[2].0 * 0.2,
            w[0].1 * 0.2 + w[1].1 * 0.6 + w[2].1 * 0.2,
        ));
    }
    out.push(*points.last().unwrap_or(&points[0]));
    out
}

static FONT_BYTES: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();

fn font_bytes() -> Option<&'static Vec<u8>> {
    FONT_BYTES
        .get_or_init(|| {
            let candidates = [
                "C:\\Windows\\Fonts\\arialbd.ttf",
                "C:\\Windows\\Fonts\\arial.ttf",
                "C:\\Windows\\Fonts\\segoeui.ttf",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/Library/Fonts/Arial.ttf",
            ];
            for p in candidates {
                if let Ok(b) = std::fs::read(p) {
                    return Some(b);
                }
            }
            None
        })
        .as_ref()
}

fn blend(buf: &mut [u8], w: u32, h: u32, x: f32, y: f32, color: [u8; 3], alpha: f32) {
    if x < 0.0 || y < 0.0 {
        return;
    }
    let (xi, yi) = (x as u32, y as u32);
    if xi >= w || yi >= h {
        return;
    }
    let a = alpha.clamp(0.0, 1.0);
    let idx = ((yi * w + xi) * 4) as usize;
    if idx + 3 >= buf.len() {
        return;
    }
    let dst_a = buf[idx + 3] as f32 / 255.0;
    let out_a = a + dst_a * (1.0 - a);
    if out_a <= 0.0 {
        return;
    }
    for c in 0..3 {
        let src = color[c] as f32;
        let dst = buf[idx + c] as f32;
        buf[idx + c] = ((src * a + dst * dst_a * (1.0 - a)) / out_a).clamp(0.0, 255.0) as u8;
    }
    buf[idx + 3] = (out_a * 255.0) as u8;
}

fn clear_circle(buf: &mut [u8], w: u32, h: u32, cx: f32, cy: f32, r: f32) {
    let x0 = (cx - r).floor().max(0.0) as u32;
    let y0 = (cy - r).floor().max(0.0) as u32;
    let x1 = ((cx + r).ceil() as i64).min(w as i64 - 1).max(0) as u32;
    let y1 = ((cy + r).ceil() as i64).min(h as i64 - 1).max(0) as u32;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= r * r {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx..idx + 4].fill(0);
            }
        }
    }
}

fn stamp(
    buf: &mut [u8],
    w: u32,
    h: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: [u8; 3],
    alpha: f32,
) {
    let r = radius.max(0.5);
    let x0 = (cx - r).floor().max(0.0) as i64;
    let y0 = (cy - r).floor().max(0.0) as i64;
    let x1 = ((cx + r).ceil() as i64).min(w as i64 - 1);
    let y1 = ((cy + r).ceil() as i64).min(h as i64 - 1);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            if d <= r {
                let edge = (r - d).clamp(0.0, 1.0);
                blend(buf, w, h, x as f32, y as f32, color, alpha * edge);
            }
        }
    }
}

pub fn draw_line_seg(
    buf: &mut [u8],
    w: u32,
    h: u32,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: [u8; 3],
    width: f32,
    alpha: f32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    let steps = (len * 1.5).ceil().max(1.0) as u32;
    let r = (width / 2.0).max(0.5);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        stamp(buf, w, h, x0 + dx * t, y0 + dy * t, r, color, alpha);
    }
}

fn fill_triangle(
    buf: &mut [u8],
    w: u32,
    h: u32,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    color: [u8; 3],
    alpha: f32,
) {
    let minx = a.0.min(b.0).min(c.0).floor().max(0.0) as i64;
    let maxx = a.0.max(b.0).max(c.0).ceil().min(w as f32 - 1.0) as i64;
    let miny = a.1.min(b.1).min(c.1).floor().max(0.0) as i64;
    let maxy = a.1.max(b.1).max(c.1).ceil().min(h as f32 - 1.0) as i64;
    let edge = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| {
        (x - p.0) * (q.1 - p.1) - (y - p.1) * (q.0 - p.0)
    };
    for y in miny..=maxy {
        for x in minx..=maxx {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let e0 = edge(a, b, px, py);
            let e1 = edge(b, c, px, py);
            let e2 = edge(c, a, px, py);
            if (e0 >= 0.0 && e1 >= 0.0 && e2 >= 0.0) || (e0 <= 0.0 && e1 <= 0.0 && e2 <= 0.0) {
                blend(buf, w, h, x as f32, y as f32, color, alpha);
            }
        }
    }
}

pub fn draw_text_rgba(
    buf: &mut [u8],
    w: u32,
    h: u32,
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: [u8; 3],
    opacity: f32,
) {
    let Some(bytes) = font_bytes() else {
        // Font yoksa basit blok göster
        let bh = (size * 0.8).max(2.0);
        for (i, _) in text.chars().enumerate() {
            let bx = x + i as f32 * size * 0.6;
            for yy in 0..bh as u32 {
                draw_line_seg(
                    buf,
                    w,
                    h,
                    bx,
                    y + yy as f32,
                    bx + size * 0.5,
                    y + yy as f32,
                    color,
                    1.0,
                    opacity,
                );
            }
        }
        return;
    };
    let Ok(font) = FontVec::try_from_vec(bytes.clone()) else {
        return;
    };
    let scale = PxScale::from(size.max(1.0));
    let scaled = font.as_scaled(scale);
    let line_h = scaled.ascent() - scaled.descent() + scaled.line_gap();
    let mut cy = y + scaled.ascent();
    for line in text.split('\n') {
        let mut cx = x;
        for ch in line.chars() {
            let gid = font.glyph_id(ch);
            let glyph = gid.with_scale_and_position(scale, ab_glyph::point(cx, cy));
            if let Some(outline) = font.outline_glyph(glyph) {
                let bb = outline.px_bounds();
                outline.draw(|gx, gy, cov| {
                    let px = bb.min.x + gx as f32;
                    let py = bb.min.y + gy as f32;
                    blend(buf, w, h, px, py, color, cov * opacity);
                });
            }
            cx += scaled.h_advance(gid);
        }
        cy += line_h;
    }
}

/// Çizim işlemlerini RGBA tampona aktarır.
pub fn rasterize(ops: &[DrawOp], w: u32, h: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    // Çizimler arka planla karışmamalı: tam opak çiz, kenarlarda yalnızca
    // anti-aliasing için kısmi kapsama kullan.
    let alpha = 1.0;
    for op in ops {
        match op {
            DrawOp::Rect {
                x0,
                y0,
                x1,
                y1,
                color,
                width,
            } => {
                let xa = (*x0).min(*x1);
                let ya = (*y0).min(*y1);
                let xb = (*x0).max(*x1);
                let yb = (*y0).max(*y1);
                draw_line_seg(&mut buf, w, h, xa, ya, xb, ya, *color, *width, alpha);
                draw_line_seg(&mut buf, w, h, xb, ya, xb, yb, *color, *width, alpha);
                draw_line_seg(&mut buf, w, h, xb, yb, xa, yb, *color, *width, alpha);
                draw_line_seg(&mut buf, w, h, xa, yb, xa, ya, *color, *width, alpha);
            }
            DrawOp::Ellipse {
                x0,
                y0,
                x1,
                y1,
                color,
                width,
            } => {
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let rx = (x1 - x0).abs() / 2.0;
                let ry = (y1 - y0).abs() / 2.0;
                let steps = ((rx.max(ry) * 2.0).max(24.0)) as u32;
                for i in 0..steps {
                    let a = i as f32 / steps as f32 * std::f32::consts::TAU;
                    let a2 = (i + 1) as f32 / steps as f32 * std::f32::consts::TAU;
                    draw_line_seg(
                        &mut buf,
                        w,
                        h,
                        cx + rx * a.cos(),
                        cy + ry * a.sin(),
                        cx + rx * a2.cos(),
                        cy + ry * a2.sin(),
                        *color,
                        *width,
                        alpha,
                    );
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
                draw_line_seg(&mut buf, w, h, *x0, *y0, *x1, *y1, *color, *width, alpha);
            }
            DrawOp::Arrow {
                x0,
                y0,
                x1,
                y1,
                color,
                width,
            } => {
                draw_line_seg(&mut buf, w, h, *x0, *y0, *x1, *y1, *color, *width, alpha);
                let ang = (y1 - y0).atan2(x1 - x0);
                let len = 15.0 + *width;
                let a1 = (x1 - len * (ang + 0.5).cos(), y1 - len * (ang + 0.5).sin());
                let a2 = (x1 - len * (ang - 0.5).cos(), y1 - len * (ang - 0.5).sin());
                fill_triangle(&mut buf, w, h, (*x1, *y1), a1, a2, *color, alpha);
            }
            DrawOp::Freehand {
                points,
                color,
                width,
            } => {
                if points.len() == 1 {
                    stamp(
                        &mut buf,
                        w,
                        h,
                        points[0].0,
                        points[0].1,
                        (*width / 2.0).max(0.5),
                        *color,
                        alpha,
                    );
                }
                for pair in points.windows(2) {
                    draw_line_seg(
                        &mut buf, w, h, pair[0].0, pair[0].1, pair[1].0, pair[1].1, *color, *width,
                        alpha,
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
                draw_text_rgba(&mut buf, w, h, text, *x, *y, *size, *color, alpha);
            }
            DrawOp::Erase { points, radius } => {
                for p in points {
                    clear_circle(&mut buf, w, h, p.0, p.1, *radius);
                }
            }
        }
    }
    buf
}

/// RGBA çizim katmanını BGR kareye alpha-blend eder.
pub fn composite_over_bgr(frame: &mut [u8], w: u32, h: u32, layer: &[u8]) {
    let n = (w * h) as usize;
    for i in 0..n {
        let a = layer[i * 4 + 3] as f32 / 255.0;
        if a <= 0.0 {
            continue;
        }
        let inv = 1.0 - a;
        for c in 0..3 {
            let src = layer[i * 4 + c] as f32;
            let dst = frame[i * 3 + c] as f32;
            frame[i * 3 + c] = (dst * inv + src * a).clamp(0.0, 255.0) as u8;
        }
    }
}
