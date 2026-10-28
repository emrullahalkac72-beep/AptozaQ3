//! Ortak yardımcılar: ffmpeg/ffprobe, dosya yolları, ekran görüntüsü, aygıt listeleri.
use anyhow::{anyhow, Context, Result};
use chrono::Local;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Windows'ta konsol penceresi açmadan komut oluşturur.
pub fn cmd(program: &str) -> Command {
    let mut c = Command::new(program);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

pub fn ffmpeg() -> Command {
    cmd("ffmpeg")
}

pub fn ffprobe_bin() -> Command {
    cmd("ffprobe")
}

pub fn ffplay() -> Command {
    cmd("ffplay")
}

pub fn run(program: &str, args: &[String]) -> Result<Output> {
    let out = cmd(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("{program} çalıştırılamadı"))?;
    Ok(out)
}

pub fn ffmpeg_ok(args: &[String]) -> Result<(), String> {
    let out = run("ffmpeg", args).map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(last_error_lines(&String::from_utf8_lossy(&out.stderr), 8))
    }
}

pub fn last_error_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

pub fn which(name: &str) -> bool {
    run(name, &["-version".to_string()])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn stamp() -> String {
    Local::now().format("%Y%m%d_%H%M%S_%3f").to_string()
}

fn temp_seq() -> u64 {
    TEMP_SEQ.fetch_add(1, Ordering::Relaxed)
}

pub fn human_time(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

pub fn mmss(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn default_output_dir() -> PathBuf {
    home_dir().join("Videos")
}

pub fn settings_path() -> PathBuf {
    home_dir().join(".ekran_kayit_pro.json")
}

/// Timeline otomatik kayıt (kurtarma) dosyası.
pub fn autosave_path() -> PathBuf {
    home_dir().join(".aptoza_timeline_autosave.aptproj")
}

pub fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}

pub fn unique_temp(ext: &str) -> PathBuf {
    let ext = ext.trim_start_matches('.').replace(['\\', '/', ':'], "_");
    let ext = if ext.is_empty() { "tmp" } else { ext.as_str() };
    temp_dir().join(format!(
        "aptoza_{}_{}_{}.{}",
        std::process::id(),
        stamp(),
        temp_seq(),
        ext
    ))
}

pub fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).ok();
    }
    Ok(())
}

/// Aynı klasörde geçici dosya üzerinden yazar; yarım kalmış dosya bırakmaz.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}.{}.{}.tmp",
        path.extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default(),
        std::process::id(),
        temp_seq()
    ));
    fs::write(&tmp, bytes)?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Windows'ta hedef dosya varsa rename başarısız olabilir.
            fs::copy(&tmp, path)?;
            let _ = fs::remove_file(&tmp);
            Ok(())
        }
    }
}

pub fn round_even(v: u32) -> u32 {
    let v = v.max(2);
    v - (v % 2)
}

/// ffprobe süresi (saniye).
pub fn duration(path: &str) -> f64 {
    let out = ffprobe_bin()
        .args([
            "-v",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
            path,
        ])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .trim()
            .parse::<f64>()
            .unwrap_or(0.0),
        Err(_) => 0.0,
    }
}

pub fn has_audio(path: &str) -> bool {
    let out = ffprobe_bin()
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output();
    out.map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false)
}

pub fn dimensions(path: &str) -> (u32, u32) {
    let out = ffprobe_bin()
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0",
            path,
        ])
        .output();
    if let Ok(o) = out {
        let s = String::from_utf8_lossy(&o.stdout);
        let mut it = s.trim().split(',');
        if let (Some(w), Some(h)) = (it.next(), it.next()) {
            if let (Ok(w), Ok(h)) = (w.trim().parse::<u32>(), h.trim().parse::<u32>()) {
                return (w, h);
            }
        }
    }
    (1920, 1080)
}

pub fn hex_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim().trim_start_matches('#').as_bytes();
    if h.len() >= 6 && h[..6].iter().all(u8::is_ascii_hexdigit) {
        let parse = |a: &[u8]| {
            std::str::from_utf8(a)
                .ok()
                .and_then(|s| u8::from_str_radix(s, 16).ok())
                .unwrap_or(255)
        };
        (parse(&h[0..2]), parse(&h[2..4]), parse(&h[4..6]))
    } else {
        (255, 255, 255)
    }
}

pub fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub fn ff_hex(hex: &str) -> String {
    let (r, g, b) = hex_rgb(hex);
    format!("{r:02X}{g:02X}{b:02X}")
}

/// ffmpeg drawtext için metin kaçışı.
pub fn escape_drawtext(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            ':' => out.push_str("\\:"),
            ',' => out.push_str("\\,"),
            '%' => out.push_str("\\%"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            ';' => out.push_str("\\;"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(ch),
        }
    }
    out
}

/// ffmpeg filtre grafiğinde dosya yolu kaçışı (movie=...).
pub fn escape_filter_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            ':' => out.push_str("\\:"),
            ',' => out.push_str("\\,"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            ';' => out.push_str("\\;"),
            '\n' | '\r' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

/// Platforma uygun, ffmpeg drawtext'in okuyabileceği font yolu.
pub fn windows_font() -> String {
    let mut candidates: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        let win = std::env::var("windir").unwrap_or_else(|_| "C:\\Windows".into());
        for name in ["arialbd.ttf", "arial.ttf", "segoeui.ttf", "tahoma.ttf"] {
            candidates.push(Path::new(&win).join("Fonts").join(name));
        }
    }
    #[cfg(target_os = "macos")]
    {
        for p in [
            "/Library/Fonts/Arial.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ] {
            candidates.push(PathBuf::from(p));
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        for p in [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ] {
            candidates.push(PathBuf::from(p));
        }
    }
    let path = candidates
        .into_iter()
        .find(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "sans".to_string());
    path.replace('\\', "/")
        .replace(':', "\\:")
        .replace('\'', "\\'")
}

/// dshow aygıtlarını listeler (video/audio adları).
pub fn dshow_devices(kind: &str) -> Vec<String> {
    let out = ffmpeg()
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    let mut names = Vec::new();
    if let Ok(o) = out {
        let text = String::from_utf8_lossy(&o.stderr).to_string();
        for line in text.lines() {
            let low = line.to_lowercase();
            if low.contains(&format!("({kind})")) {
                if let (Some(a), Some(b)) = (line.find('"'), line.rfind('"')) {
                    if b > a + 1 {
                        names.push(line[a + 1..b].to_string());
                    }
                }
            }
        }
    }
    names
}

pub fn audio_devices() -> Vec<String> {
    dshow_devices("audio")
}

pub fn video_devices() -> Vec<String> {
    dshow_devices("video")
}

/// Ana monitör / bölge yakalama (xcap) - RGBA döner.
pub fn capture_rgba(region: Option<(i32, i32, u32, u32)>) -> Result<image::RgbaImage> {
    let monitors = xcap::Monitor::all().context("Monitörler listelenemedi")?;
    let mon = monitors
        .iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .or_else(|| monitors.first())
        .ok_or_else(|| anyhow!("Monitör bulunamadı"))?;
    let img = mon.capture_image().context("Ekran görüntüsü alınamadı")?;
    if let Some((x, y, w, h)) = region {
        let mx = mon.x().unwrap_or(0) as i64;
        let my = mon.y().unwrap_or(0) as i64;
        let rx = x as i64 - mx;
        let ry = y as i64 - my;
        let rw = w as i64;
        let rh = h as i64;
        if rw <= 0
            || rh <= 0
            || rx < 0
            || ry < 0
            || rx + rw > img.width() as i64
            || ry + rh > img.height() as i64
        {
            return Err(anyhow!("Seçili bölge monitör sınırlarının dışında."));
        }
        return Ok(
            image::imageops::crop_imm(&img, rx as u32, ry as u32, rw as u32, rh as u32).to_image(),
        );
    }
    Ok(img)
}

pub fn save_rgba(path: &Path, img: &image::RgbaImage, fmt: &str) -> Result<()> {
    ensure_parent(path)?;
    let format = match fmt.to_lowercase().as_str() {
        "jpg" | "jpeg" => image::ImageFormat::Jpeg,
        "bmp" => image::ImageFormat::Bmp,
        "webp" => image::ImageFormat::WebP,
        "tiff" | "tif" => image::ImageFormat::Tiff,
        _ => image::ImageFormat::Png,
    };
    image::save_buffer_with_format(
        path,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ColorType::Rgba8,
        format,
    )
    .with_context(|| format!("Görsel kaydedilemedi: {}", path.display()))
}

pub fn load_rgba(path: &str) -> Result<image::RgbaImage> {
    Ok(image::open(path).context("Görsel açılamadı")?.to_rgba8())
}

pub fn icon_data() -> Option<egui::IconData> {
    let data = include_bytes!("../AQ2.ico");
    let dir = ico::IconDir::read(std::io::Cursor::new(data.as_slice())).ok()?;
    let mut best: Option<ico::IconImage> = None;
    for e in dir.entries() {
        if let Ok(img) = e.decode() {
            if best
                .as_ref()
                .map(|b| b.width() * b.height() < img.width() * img.height())
                .unwrap_or(true)
            {
                best = Some(img);
            }
        }
    }
    let img = best?;
    Some(egui::IconData {
        rgba: img.rgba_data().to_vec(),
        width: img.width(),
        height: img.height(),
    })
}

#[cfg(windows)]
pub fn cursor_pos() -> Option<(i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut p = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut p) };
    if ok != 0 {
        Some((p.x, p.y))
    } else {
        None
    }
}

#[cfg(not(windows))]
pub fn cursor_pos() -> Option<(i32, i32)> {
    None
}

/// Arka planda ffmpeg çalıştırıp bittiğinde dosya üretir; durumu string olarak paylaşır.
pub fn spawn_job<F>(status: std::sync::Arc<std::sync::Mutex<String>>, job: F)
where
    F: FnOnce() -> Result<String> + Send + 'static,
{
    std::thread::spawn(move || {
        if let Ok(mut s) = status.lock() {
            *s = "⏳ İşleniyor... Lütfen bekleyin.".into();
        }
        match job() {
            Ok(msg) => {
                if let Ok(mut s) = status.lock() {
                    *s = msg;
                }
            }
            Err(e) => {
                if let Ok(mut s) = status.lock() {
                    *s = format!("✗ Hata: {}", shorten_tail(&e.to_string(), 400));
                }
            }
        }
    });
}

/// Kullanıcı metinlerini sınırlar; komut satırını bozabilecek kontrol
/// karakterlerini (sekme ve satır sonu hariç) temizler.
pub fn clean_text(s: &str, max_chars: usize) -> String {
    s.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(max_chars)
        .collect()
}

pub fn shorten(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n).collect();
        out.push('…');
        out
    }
}

pub fn shorten_tail(s: &str, n: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        return s.to_string();
    }
    let start = chars.len().saturating_sub(n.saturating_sub(1));
    format!("…{}", chars[start..].iter().collect::<String>())
}

pub fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

pub fn list_dir_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok().map(|x| x.path())).collect())
        .unwrap_or_default();
    v.sort();
    v
}

pub fn read_all_stdout(mut child: std::process::Child) -> Vec<u8> {
    let mut buf = Vec::new();
    if let Some(mut s) = child.stdout.take() {
        let _ = s.read_to_end(&mut buf);
    }
    buf
}

pub fn now_instant() -> Instant {
    Instant::now()
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
