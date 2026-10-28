//! Kayıt motoru: ekran (xcap/DXGI), webcam (ffmpeg dshow), mikrofon (ffmpeg dshow),
//! canlı çizim katmanı, imleç vurgusu, filigran, zoom, PiP ve H.264 kodlama.
use crate::draw::{self, DrawOp};
use crate::model::RecorderConfig;
use crate::util;
use anyhow::{anyhow, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use image::{imageops, RgbaImage};
use std::fs::{self, File};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct WebcamFrame {
    pub w: u32,
    pub h: u32,
    pub bgr: Vec<u8>,
    pub seq: u64,
}

#[derive(Debug)]
pub enum EngineEvent {
    Done {
        ok: bool,
        path: Option<String>,
        message: Option<String>,
    },
}

#[derive(Default)]
pub struct TimeState {
    start: Option<Instant>,
    pause_at: Option<Instant>,
    pause_acc: f64,
}

pub struct Shared {
    pub active: AtomicBool,
    pub paused: AtomicBool,
    pub stop: AtomicBool,
    pub audio_muted: AtomicBool,
    pub zoom: AtomicU32,
    pub zoom_center: Mutex<Option<(f32, f32)>>,
    pub ops: Arc<Mutex<Vec<DrawOp>>>,
    pub webcam: Mutex<Option<WebcamFrame>>,
    pub webcam_ok: AtomicBool,
    pub status: Mutex<String>,
    pub error: Mutex<Option<String>>,
    pub time: Mutex<TimeState>,
    seq: AtomicU64,
}

impl Shared {
    pub fn new(ops: Arc<Mutex<Vec<DrawOp>>>) -> Self {
        Self {
            active: AtomicBool::new(false),
            paused: AtomicBool::new(false),
            stop: AtomicBool::new(false),
            audio_muted: AtomicBool::new(false),
            zoom: AtomicU32::new(1.0f32.to_bits()),
            zoom_center: Mutex::new(None),
            ops,
            webcam: Mutex::new(None),
            webcam_ok: AtomicBool::new(false),
            status: Mutex::new("● Hazır".into()),
            error: Mutex::new(None),
            time: Mutex::new(TimeState::default()),
            seq: AtomicU64::new(0),
        }
    }

    pub fn elapsed(&self) -> f64 {
        let t = self.time.lock().unwrap();
        if let Some(start) = t.start {
            let mut e = start.elapsed().as_secs_f64() - t.pause_acc;
            if let Some(p) = t.pause_at {
                e -= p.elapsed().as_secs_f64();
            }
            e.max(0.0)
        } else {
            0.0
        }
    }

    pub fn elapsed_str(&self) -> String {
        util::human_time(self.elapsed())
    }

    pub fn status(&self) -> String {
        self.status.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn set_status(&self, s: impl Into<String>) {
        if let Ok(mut st) = self.status.lock() {
            *st = s.into();
        }
    }

    pub fn zoom(&self) -> f32 {
        f32::from_bits(self.zoom.load(Ordering::Relaxed))
    }

    pub fn set_zoom(&self, z: f32) {
        self.zoom
            .store(z.clamp(1.0, 4.0).to_bits(), Ordering::Relaxed);
    }

    pub fn push_webcam(&self, w: u32, h: u32, bgr: Vec<u8>) {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut wc) = self.webcam.lock() {
            *wc = Some(WebcamFrame { w, h, bgr, seq });
        }
    }

    pub fn webcam_frame(&self) -> Option<WebcamFrame> {
        self.webcam.lock().ok().and_then(|g| g.clone())
    }
}

pub struct Engine {
    pub shared: Arc<Shared>,
    tx: Sender<EngineEvent>,
    rx: Receiver<EngineEvent>,
}

impl Engine {
    pub fn new(ops: Arc<Mutex<Vec<DrawOp>>>) -> Self {
        let (tx, rx) = unbounded();
        Self {
            shared: Arc::new(Shared::new(ops)),
            tx,
            rx,
        }
    }

    pub fn start(&self, cfg: RecorderConfig) -> Result<()> {
        if self.shared.active.load(Ordering::SeqCst) {
            return Err(anyhow!("Kayıt zaten sürüyor."));
        }
        if !cfg.screen_on && !cfg.webcam_on {
            return Err(anyhow!(
                "Lütfen en az bir kayıt kaynağı etkinleştirin (Ekran veya Webcam)."
            ));
        }
        self.shared.active.store(true, Ordering::SeqCst);
        self.shared.stop.store(false, Ordering::SeqCst);
        self.shared.paused.store(false, Ordering::SeqCst);
        self.shared.audio_muted.store(false, Ordering::SeqCst);
        self.shared.set_status("⏳ Başlatılıyor...");
        if let Ok(mut e) = self.shared.error.lock() {
            *e = None;
        }
        {
            let mut t = self.shared.time.lock().unwrap();
            *t = TimeState {
                start: Some(Instant::now()),
                pause_at: None,
                pause_acc: 0.0,
            };
        }
        let shared = self.shared.clone();
        let tx = self.tx.clone();
        std::thread::spawn(move || run_session(shared, cfg, tx));
        Ok(())
    }

    pub fn stop(&self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        self.shared.set_status("⏳ Kaydediliyor...");
    }

    pub fn pause(&self) {
        if self.shared.active.load(Ordering::SeqCst)
            && !self.shared.paused.swap(true, Ordering::SeqCst)
        {
            let mut t = self.shared.time.lock().unwrap();
            t.pause_at = Some(Instant::now());
            drop(t);
            self.shared.set_status("⏸ Duraklatıldı");
        }
    }

    pub fn resume(&self) {
        if self.shared.active.load(Ordering::SeqCst)
            && self.shared.paused.swap(false, Ordering::SeqCst)
        {
            let mut t = self.shared.time.lock().unwrap();
            if let Some(p) = t.pause_at.take() {
                t.pause_acc += p.elapsed().as_secs_f64();
            }
            drop(t);
            self.shared.set_status("● Kayıt Yapılıyor");
        }
    }

    pub fn is_active(&self) -> bool {
        self.shared.active.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.shared.paused.load(Ordering::SeqCst)
    }

    pub fn is_muted(&self) -> bool {
        self.shared.audio_muted.load(Ordering::SeqCst)
    }

    pub fn toggle_mute(&self) -> bool {
        let v = !self.shared.audio_muted.load(Ordering::SeqCst);
        self.shared.audio_muted.store(v, Ordering::SeqCst);
        v
    }

    pub fn poll(&self) -> Option<EngineEvent> {
        self.rx.try_recv().ok()
    }
}

// ---------------------------------------------------------------- WAV yazıcı
struct WavSink {
    file: BufWriter<File>,
    data_len: u64,
}

impl WavSink {
    fn new(path: &str, rate: u32) -> std::io::Result<Self> {
        let mut f = File::create(path)?;
        f.write_all(&[0u8; 44])?;
        let _ = rate;
        Ok(Self {
            file: BufWriter::new(f),
            data_len: 0,
        })
    }
    fn write(&mut self, data: &[u8]) {
        if self.file.write_all(data).is_ok() {
            self.data_len += data.len() as u64;
        }
    }
    fn finalize(mut self, rate: u32) {
        let _ = self.file.flush();
        let mut f = match self.file.into_inner() {
            Ok(f) => f,
            Err(_) => return,
        };
        let _ = f.seek(SeekFrom::Start(0));
        let channels: u16 = 2;
        let bits: u16 = 16;
        let byte_rate = rate * channels as u32 * (bits / 8) as u32;
        let block_align = channels * (bits / 8);
        let mut h = Vec::with_capacity(44);
        h.extend_from_slice(b"RIFF");
        h.extend_from_slice(&((36 + self.data_len) as u32).to_le_bytes());
        h.extend_from_slice(b"WAVEfmt ");
        h.extend_from_slice(&16u32.to_le_bytes());
        h.extend_from_slice(&1u16.to_le_bytes());
        h.extend_from_slice(&channels.to_le_bytes());
        h.extend_from_slice(&rate.to_le_bytes());
        h.extend_from_slice(&byte_rate.to_le_bytes());
        h.extend_from_slice(&block_align.to_le_bytes());
        h.extend_from_slice(&bits.to_le_bytes());
        h.extend_from_slice(b"data");
        h.extend_from_slice(&(self.data_len as u32).to_le_bytes());
        let _ = f.seek(SeekFrom::Start(0));
        let _ = f.write_all(&h);
        let _ = f.flush();
    }
}

// ---------------------------------------------------------------- yardımcılar
fn corner_pos(pos: &str, ow: u32, oh: u32, w: u32, h: u32, margin: i32) -> (i32, i32) {
    let (ow, oh, w, h) = (ow as i32, oh as i32, w as i32, h as i32);
    match pos {
        "top-left" => (margin, margin),
        "top-right" => (w - ow - margin, margin),
        "bottom-left" => (margin, h - oh - margin),
        "center" => ((w - ow) / 2, (h - oh) / 2),
        _ => (w - ow - margin, h - oh - margin),
    }
}

fn bgr_frame_from_rgba(
    raw: &[u8],
    fw: u32,
    fh: u32,
    crop: Option<(u32, u32, u32, u32)>,
    tw: u32,
    th: u32,
) -> Vec<u8> {
    let Some(img) = RgbaImage::from_raw(fw, fh, raw.to_vec()) else {
        return vec![0u8; (tw * th * 3) as usize];
    };
    if fw == 0 || fh == 0 {
        return vec![0u8; (tw * th * 3) as usize];
    }
    let img = if let Some((x, y, w, h)) = crop {
        let x = x.min(fw - 1);
        let y = y.min(fh - 1);
        let w = w.max(1).min(fw - x);
        let h = h.max(1).min(fh - y);
        imageops::crop_imm(&img, x, y, w, h).to_image()
    } else {
        img
    };
    let img = if img.width() != tw || img.height() != th {
        imageops::resize(&img, tw, th, imageops::FilterType::Triangle)
    } else {
        img
    };
    let mut bgr = vec![0u8; (tw * th * 3) as usize];
    for (i, p) in img.pixels().enumerate() {
        bgr[i * 3] = p[2];
        bgr[i * 3 + 1] = p[1];
        bgr[i * 3 + 2] = p[0];
    }
    bgr
}

fn apply_zoom_bgr(frame: &mut Vec<u8>, w: u32, h: u32, zoom: f32, center: Option<(f32, f32)>) {
    if zoom <= 1.001 {
        return;
    }
    let (cx, cy) = center.unwrap_or((w as f32 / 2.0, h as f32 / 2.0));
    let nw = (w as f32 / zoom) as u32;
    let nh = (h as f32 / zoom) as u32;
    let nw = nw.max(2).min(w);
    let nh = nh.max(2).min(h);
    let x0 = (cx - nw as f32 / 2.0).max(0.0).min((w - nw) as f32) as u32;
    let y0 = (cy - nh as f32 / 2.0).max(0.0).min((h - nh) as f32) as u32;
    let mut out = vec![0u8; (w * h * 3) as usize];
    for y in 0..h {
        let sy = y0 + (y * nh / h).min(nh - 1);
        for x in 0..w {
            let sx = x0 + (x * nw / w).min(nw - 1);
            let si = ((sy * w + sx) * 3) as usize;
            let di = ((y * w + x) * 3) as usize;
            out[di..di + 3].copy_from_slice(&frame[si..si + 3]);
        }
    }
    *frame = out;
}

fn paste_bgr(dst: &mut [u8], dw: u32, dh: u32, src: &[u8], sw: u32, sh: u32, ox: i32, oy: i32) {
    for y in 0..sh as i32 {
        let dy = oy + y;
        if dy < 0 || dy >= dh as i32 {
            continue;
        }
        for x in 0..sw as i32 {
            let dx = ox + x;
            if dx < 0 || dx >= dw as i32 {
                continue;
            }
            let si = ((y as u32 * sw + x as u32) * 3) as usize;
            let di = ((dy as u32 * dw + dx as u32) * 3) as usize;
            if si + 3 <= src.len() && di + 3 <= dst.len() {
                dst[di..di + 3].copy_from_slice(&src[si..si + 3]);
            }
        }
    }
}

fn darken_bgr(dst: &mut [u8], w: u32, h: u32, x: i32, y: i32, rw: i32, rh: i32, factor: f32) {
    for yy in y.max(0)..(y + rh).min(h as i32) {
        for xx in x.max(0)..(x + rw).min(w as i32) {
            let i = ((yy as u32 * w + xx as u32) * 3) as usize;
            for c in 0..3 {
                dst[i + c] = (dst[i + c] as f32 * factor) as u8;
            }
        }
    }
}

fn circle_bgr(
    buf: &mut [u8],
    w: u32,
    h: u32,
    cx: f32,
    cy: f32,
    r: f32,
    col: [u8; 3],
    thickness: i32,
    filled: bool,
) {
    let r = r.max(1.0);
    let x0 = (cx - r - 2.0).max(0.0) as i32;
    let y0 = (cy - r - 2.0).max(0.0) as i32;
    let x1 = ((cx + r + 2.0) as i32).min(w as i32 - 1);
    let y1 = ((cy + r + 2.0) as i32).min(h as i32 - 1);
    let inner = (r - thickness.max(1) as f32).max(0.0);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let hit = if filled { d <= r } else { d <= r && d >= inner };
            if hit {
                let i = ((y as u32 * w + x as u32) * 3) as usize;
                buf[i..i + 3].copy_from_slice(&col);
            }
        }
    }
}

fn make_bordered_pip(frame: &[u8], w: u32, h: u32) -> (Vec<u8>, u32, u32) {
    let b = 3u32;
    let (nw, nh) = (w + b * 2, h + b * 2);
    let mut out = vec![220u8; (nw * nh * 3) as usize];
    paste_bgr(&mut out, nw, nh, frame, w, h, b as i32, b as i32);
    (out, nw, nh)
}

// ---------------------------------------------------------------- oturum
fn choose_monitor(cfg: &RecorderConfig) -> Result<(xcap::Monitor, i32, i32, u32, u32)> {
    let monitors = xcap::Monitor::all().map_err(|e| anyhow!("Monitör listesi: {e}"))?;
    let mon = if let Some(r) = cfg.region {
        monitors
            .iter()
            .find(|m| {
                let (mx, my) = (m.x().unwrap_or(0), m.y().unwrap_or(0));
                r.x >= mx
                    && r.y >= my
                    && r.x < mx + m.width().unwrap_or(1) as i32
                    && r.y < my + m.height().unwrap_or(1) as i32
            })
            .cloned()
            .or_else(|| {
                monitors
                    .iter()
                    .find(|m| m.is_primary().unwrap_or(false))
                    .cloned()
            })
            .or_else(|| monitors.first().cloned())
    } else {
        monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .cloned()
            .or_else(|| monitors.first().cloned())
    };
    let mon = mon.ok_or_else(|| anyhow!("Monitör bulunamadı"))?;
    let mx = mon.x().unwrap_or(0);
    let my = mon.y().unwrap_or(0);
    let mw = mon.width().unwrap_or(1920);
    let mh = mon.height().unwrap_or(1080);
    Ok((mon, mx, my, mw, mh))
}

fn run_session(shared: Arc<Shared>, cfg: RecorderConfig, tx: Sender<EngineEvent>) {
    let result = (|| -> Result<Option<String>> {
        fs::create_dir_all(&cfg.output_dir).ok();
        let base = format!(
            "{}{}kayit_{}",
            cfg.output_dir.trim_end_matches(['\\', '/']),
            std::path::MAIN_SEPARATOR,
            util::stamp()
        );
        let vid_tmp = format!("{base}_v.mp4");
        let aud_raw_path = format!("{base}_a.raw");
        let aud_tmp = format!("{base}_a.wav");
        let out_path = format!("{base}.mp4");
        let err_log = format!("{base}_ffmpeg_err.log");

        let mon_info = choose_monitor(&cfg).ok();
        let (mut tw, mut th, region_abs, mon_origin, mon_size) = if cfg.screen_on {
            if let Some((_, mx, my, mw, mh)) = mon_info.as_ref() {
                if let Some(r) = cfg.region.filter(|r| r.valid()) {
                    (r.w, r.h, Some((r.x, r.y, r.w, r.h)), (*mx, *my), (*mw, *mh))
                } else {
                    (*mw, *mh, None, (*mx, *my), (*mw, *mh))
                }
            } else {
                (1920, 1080, None, (0, 0), (1920, 1080))
            }
        } else {
            (
                cfg.webcam_res.0,
                cfg.webcam_res.1,
                None,
                (0, 0),
                (cfg.webcam_res.0, cfg.webcam_res.1),
            )
        };
        tw = util::round_even(tw);
        th = util::round_even(th);

        // Encoder
        let mut enc_args: Vec<String> = vec![
            "-y".into(),
            "-f".into(),
            "rawvideo".into(),
            "-vcodec".into(),
            "rawvideo".into(),
            "-s".into(),
            format!("{tw}x{th}"),
            "-pix_fmt".into(),
            "bgr24".into(),
            "-r".into(),
            cfg.fps.to_string(),
            "-i".into(),
            "pipe:".into(),
            "-vcodec".into(),
            "libx264".into(),
            "-preset".into(),
            "ultrafast".into(),
            "-tune".into(),
            "zerolatency".into(),
            "-crf".into(),
            cfg.quality.to_string(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-colorspace".into(),
            "bt709".into(),
            "-color_trc".into(),
            "bt709".into(),
            "-color_primaries".into(),
            "bt709".into(),
            "-color_range".into(),
            "tv".into(),
            "-fps_mode".into(),
            "cfr".into(),
            "-threads".into(),
            "0".into(),
            vid_tmp.clone(),
        ];
        let stderr = File::create(&err_log).ok();
        let mut enc = util::ffmpeg()
            .args(&enc_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(stderr.map(Stdio::from).unwrap_or_else(Stdio::null))
            .spawn();
        if let Ok(proc) = enc.as_mut() {
            std::thread::sleep(Duration::from_millis(150));
            if let Ok(Some(st)) = proc.try_wait() {
                // -fps_mode desteklenmiyorsa onsuz tekrar dene
                if !st.success() {
                    enc_args.retain(|a| a.as_str() != "-fps_mode" && a.as_str() != "cfr");
                    let stderr = File::create(&err_log).ok();
                    enc = util::ffmpeg()
                        .args(&enc_args)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::null())
                        .stderr(stderr.map(Stdio::from).unwrap_or_else(Stdio::null))
                        .spawn();
                }
            }
        }
        let mut enc = match enc {
            Ok(e) => e,
            Err(_) => {
                return Err(anyhow!(
                    "FFmpeg başlatılamadı. FFmpeg'in PATH'te olduğundan emin olun."
                ))
            }
        };
        let stdin = enc.stdin.take();

        // Webcam
        let webcam_thread = if cfg.webcam_on {
            let sh = shared.clone();
            let cfg2 = cfg.clone();
            Some(std::thread::spawn(move || webcam_loop(sh, cfg2)))
        } else {
            None
        };
        if cfg.webcam_on {
            // ilk kareyi bekle (en fazla ~3 sn)
            let t0 = Instant::now();
            while shared.webcam_frame().is_none() && t0.elapsed() < Duration::from_secs(3) {
                if shared.stop.load(Ordering::SeqCst) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            if shared.webcam_frame().is_none() {
                shared.webcam_ok.store(false, Ordering::SeqCst);
            }
        }

        // Ses
        let audio_thread = if cfg.audio_on {
            let sh = shared.clone();
            let cfg2 = cfg.clone();
            let raw = aud_raw_path.clone();
            Some(std::thread::spawn(move || audio_loop(sh, cfg2, raw)))
        } else {
            None
        };

        // Ekran / video
        let sh = shared.clone();
        let cfg2 = cfg.clone();
        let (stdin, region, origin, msize) = (stdin, region_abs, mon_origin, mon_size);
        let has_mon = mon_info.is_some();
        let video_thread = std::thread::spawn(move || {
            screen_loop(sh, cfg2, stdin, tw, th, region, origin, msize, has_mon)
        });
        shared.set_status("● Kayıt Yapılıyor");

        while !shared.stop.load(Ordering::SeqCst) {
            let _ = shared.elapsed();
            if shared.paused.load(Ordering::SeqCst) {
                shared.set_status("⏸ Duraklatıldı");
            } else {
                shared.set_status("● Kayıt Yapılıyor");
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        let enc_status = video_thread.join().unwrap_or(None);
        let _ = enc.wait();
        if let Some(t) = webcam_thread {
            let _ = t.join();
        }
        if let Some(t) = audio_thread {
            let _ = t.join();
        }

        let mut has_audio = false;
        if cfg.audio_on {
            if let Ok(meta) = fs::metadata(&aud_raw_path) {
                if meta.len() > 1024 {
                    // raw -> wav
                    if finalize_audio(&aud_raw_path, &aud_tmp, cfg.audio_rate).is_ok() {
                        has_audio = true;
                    }
                }
            }
        }

        if !std::path::Path::new(&vid_tmp).exists() {
            let msg =
                read_log_tail(&err_log).unwrap_or_else(|| "Video dosyası oluşturulamadı.".into());
            return Err(anyhow!(msg));
        }
        let _ = enc_status;

        // Birleştirme
        let crf = cfg.quality.to_string();
        let mut merged = false;
        if has_audio {
            let args: Vec<String> = vec![
                "-y".into(),
                "-i".into(),
                vid_tmp.clone(),
                "-i".into(),
                aud_tmp.clone(),
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "medium".into(),
                "-crf".into(),
                crf.clone(),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                "192k".into(),
                "-af".into(),
                "aresample=async=1".into(),
                "-movflags".into(),
                "+faststart".into(),
                "-shortest".into(),
                out_path.clone(),
            ];
            if util::ffmpeg_ok(&args).is_ok() {
                merged = true;
                let _ = fs::remove_file(&vid_tmp);
                let _ = fs::remove_file(&aud_tmp);
            }
        }
        if !merged {
            let args: Vec<String> = vec![
                "-y".into(),
                "-i".into(),
                vid_tmp.clone(),
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "medium".into(),
                "-crf".into(),
                crf,
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-movflags".into(),
                "+faststart".into(),
                out_path.clone(),
            ];
            if util::ffmpeg_ok(&args).is_ok() {
                let _ = fs::remove_file(&vid_tmp);
                merged = true;
            }
        }
        if !merged {
            let _ = fs::rename(&vid_tmp, &out_path);
        }
        let _ = fs::remove_file(&aud_raw_path);
        let _ = fs::remove_file(&aud_tmp);
        let _ = fs::remove_file(&err_log);

        let ok = fs::metadata(&out_path)
            .map(|m| m.len() > 2048)
            .unwrap_or(false);
        if ok {
            Ok(Some(out_path))
        } else {
            Err(anyhow!(
                "Video dosyası oluşturulamadı. FFmpeg kurulumunu, çıktı klasörü izinlerini ve disk alanını kontrol edin."
            ))
        }
    })();

    shared.set_status(match &result {
        Ok(_) => "✓ Tamamlandı",
        Err(_) => "✗ Hata",
    });
    shared.active.store(false, Ordering::SeqCst);
    let event = match result {
        Ok(Some(path)) => EngineEvent::Done {
            ok: true,
            path: Some(path),
            message: None,
        },
        Ok(None) => EngineEvent::Done {
            ok: false,
            path: None,
            message: Some("Video oluşturulamadı.".into()),
        },
        Err(e) => {
            let msg = e.to_string();
            if let Ok(mut err) = shared.error.lock() {
                *err = Some(msg.clone());
            }
            EngineEvent::Done {
                ok: false,
                path: None,
                message: Some(msg),
            }
        }
    };
    let _ = tx.send(event);
}

fn read_log_tail(path: &str) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    if text.trim().is_empty() {
        None
    } else {
        Some(format!(
            "FFmpeg hatası:\n{}",
            util::last_error_lines(&text, 8)
        ))
    }
}

fn finalize_audio(raw: &str, wav: &str, rate: u32) -> Result<()> {
    let mut src = File::open(raw)?;
    let mut dst = WavSink::new(wav, rate)?;
    let mut buf = vec![0u8; 65536];
    loop {
        let n = src.read(&mut buf)?;
        if n == 0 {
            break;
        }
        dst.write(&buf[..n]);
    }
    dst.finalize(rate);
    Ok(())
}

// ---------------------------------------------------------------- webcam
fn webcam_loop(shared: Arc<Shared>, cfg: RecorderConfig) {
    let mut name = cfg.cam_name.clone();
    if name.is_empty() {
        let vids = util::video_devices();
        name = vids
            .get(cfg.cam_idx)
            .cloned()
            .or_else(|| vids.first().cloned())
            .unwrap_or_default();
    }
    if name.is_empty() {
        if let Ok(mut e) = shared.error.lock() {
            *e = Some("Webcam bulunamadı.".into());
        }
        return;
    }
    let (w, h) = (cfg.webcam_res.0.max(2), cfg.webcam_res.1.max(2));
    let mut args: Vec<String> = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-f".into(),
        "dshow".into(),
        "-rtbufsize".into(),
        "200M".into(),
        "-framerate".into(),
        cfg.fps.to_string(),
        "-video_size".into(),
        format!("{w}x{h}"),
        "-i".into(),
        format!("video={name}"),
        "-vf".into(),
        format!("scale={w}:{h}"),
        "-pix_fmt".into(),
        "bgr24".into(),
        "-f".into(),
        "rawvideo".into(),
        "pipe:1".into(),
    ];
    let mut child = util::ffmpeg()
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    if let Ok(c) = child.as_mut() {
        std::thread::sleep(Duration::from_millis(400));
        if let Ok(Some(_)) = c.try_wait() {
            args.retain(|a| a != "-video_size" && !a.contains('x'));
            // yeniden dene (çözünürlük zorlaması olmadan)
            child = util::ffmpeg()
                .args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn();
        }
    }
    let Ok(mut child) = child else { return };
    let Some(mut stdout) = child.stdout.take() else {
        return;
    };
    let frame_size = (w * h * 3) as usize;
    let mut buf = vec![0u8; frame_size];
    shared.webcam_ok.store(true, Ordering::SeqCst);
    while !shared.stop.load(Ordering::SeqCst) {
        if stdout.read_exact(&mut buf).is_err() {
            break;
        }
        shared.push_webcam(w, h, buf.clone());
    }
    let _ = child.kill();
    let _ = child.wait();
    shared.webcam_ok.store(false, Ordering::SeqCst);
}

// ---------------------------------------------------------------- ses
fn audio_loop(shared: Arc<Shared>, cfg: RecorderConfig, raw_path: String) {
    let mut name = cfg.audio_device.clone();
    if name.is_empty() {
        let devs = util::audio_devices();
        name = devs.first().cloned().unwrap_or_default();
    }
    if name.is_empty() {
        return;
    }
    let raw_file = match File::create(&raw_path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut sink = BufWriter::new(raw_file);
    let args: Vec<String> = vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-f".into(),
        "dshow".into(),
        "-rtbufsize".into(),
        "100M".into(),
        "-i".into(),
        format!("audio={name}"),
        "-ac".into(),
        "2".into(),
        "-ar".into(),
        cfg.audio_rate.to_string(),
        "-f".into(),
        "s16le".into(),
        "pipe:1".into(),
    ];
    let mut child = match util::ffmpeg()
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    let Some(mut stdout) = child.stdout.take() else {
        return;
    };
    let mut buf = vec![0u8; 4096];
    while !shared.stop.load(Ordering::SeqCst) {
        let n = match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        if shared.paused.load(Ordering::SeqCst) {
            continue;
        }
        let data: &[u8] = if shared.audio_muted.load(Ordering::SeqCst) {
            &[0u8; 4096][..n]
        } else {
            &buf[..n]
        };
        if sink.write_all(data).is_err() {
            break;
        }
    }
    let _ = sink.flush();
    let _ = child.kill();
    let _ = child.wait();
}

// ---------------------------------------------------------------- ekran
fn screen_loop(
    shared: Arc<Shared>,
    cfg: RecorderConfig,
    mut stdin: Option<ChildStdin>,
    tw: u32,
    th: u32,
    region: Option<(i32, i32, u32, u32)>,
    mon_origin: (i32, i32),
    mon_size: (u32, u32),
    has_mon: bool,
) -> Option<std::process::ExitStatus> {
    let mut recorder = None;
    let mut last_rgba: Option<(Vec<u8>, u32, u32)> = None;
    let mut still_mon = None;
    if has_mon && cfg.screen_on {
        if let Ok((mon, _, _, _, _)) = choose_monitor(&cfg) {
            if let Ok((rec, rx)) = mon.video_recorder() {
                let _ = rec.start();
                recorder = Some((rec, rx));
            } else {
                still_mon = Some(mon);
            }
        }
    }
    let mut last_still = Instant::now() - Duration::from_secs(5);
    // filigran görseli
    let wm_img: Option<RgbaImage> = cfg
        .wm_image
        .as_ref()
        .and_then(|p| image::open(p).ok())
        .map(|i| imageops::resize(&i.to_rgba8(), 150, 60, imageops::FilterType::Lanczos3));

    let mut active = 0.0f64;
    let mut written = 0u64;
    let mut last_time = Instant::now();
    let mut pipe_dead = false;

    while !shared.stop.load(Ordering::SeqCst) {
        if shared.paused.load(Ordering::SeqCst) {
            last_time = Instant::now();
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }
        // kare topla (kanal tıkanmasın diye sürekli boşalt)
        if let Some((rec, rx)) = recorder.as_ref() {
            match rx.recv_timeout(Duration::from_millis(3)) {
                Ok(f) => last_rgba = Some((f.raw, f.width, f.height)),
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    let _ = rec.stop();
                    recorder = None;
                }
                Err(_) => {}
            }
        } else if cfg.screen_on {
            if last_still.elapsed() > Duration::from_millis(33) {
                if let Some(mon) = still_mon.as_ref() {
                    if let Ok(img) = mon.capture_image() {
                        last_rgba = Some((
                            img.into_raw(),
                            mon.width().unwrap_or(tw),
                            mon.height().unwrap_or(th),
                        ));
                    }
                }
                last_still = Instant::now();
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        if !cfg.screen_on || last_rgba.is_none() {
            if let Some(wf) = shared.webcam_frame() {
                last_rgba = Some((bgr_to_rgba(&wf.bgr), wf.w, wf.h));
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last_time).as_secs_f64();
        last_time = now;
        active += dt;
        let expected = (active * cfg.fps as f64) as u64;
        let to_write = expected.saturating_sub(written);
        if to_write == 0 {
            std::thread::sleep(Duration::from_millis(3));
            continue;
        }
        let blank = vec![0u8; (tw * th * 3) as usize];
        let mut bgr = if let Some((raw, fw, fh)) = last_rgba.as_ref() {
            let crop = if let Some((rx, ry, rw, rh)) = region {
                let sx = if mon_size.0 > 0 {
                    *fw as f32 / mon_size.0 as f32
                } else {
                    1.0
                };
                let sy = if mon_size.1 > 0 {
                    *fh as f32 / mon_size.1 as f32
                } else {
                    1.0
                };
                Some((
                    (((rx - mon_origin.0) as f32) * sx).max(0.0) as u32,
                    (((ry - mon_origin.1) as f32) * sy).max(0.0) as u32,
                    (rw as f32 * sx) as u32,
                    (rh as f32 * sy) as u32,
                ))
            } else {
                None
            };
            bgr_frame_from_rgba(raw, *fw, *fh, crop, tw, th)
        } else {
            blank
        };

        apply_zoom_bgr(
            &mut bgr,
            tw,
            th,
            shared.zoom(),
            *shared.zoom_center.lock().unwrap(),
        );

        // PiP
        if cfg.pip_enabled && cfg.webcam_on {
            if let Some(wf) = shared.webcam_frame() {
                let (pw, ph) = (
                    util::round_even(cfg.pip_size.0),
                    util::round_even(cfg.pip_size.1),
                );
                if let Some(src) = RgbaImage::from_raw(wf.w, wf.h, bgr_to_rgba(&wf.bgr)) {
                    let small = imageops::resize(&src, pw, ph, imageops::FilterType::Triangle);
                    let mut sb = vec![0u8; (pw * ph * 3) as usize];
                    for (i, p) in small.pixels().enumerate() {
                        sb[i * 3] = p[2];
                        sb[i * 3 + 1] = p[1];
                        sb[i * 3 + 2] = p[0];
                    }
                    let (pb, bw, bh) = make_bordered_pip(&sb, pw, ph);
                    let (ox, oy) = corner_pos(&cfg.pip_pos, bw, bh, tw, th, 20);
                    darken_bgr(&mut bgr, tw, th, ox + 6, oy + 6, bw as i32, bh as i32, 0.45);
                    paste_bgr(&mut bgr, tw, th, &pb, bw, bh, ox, oy);
                }
            }
        }

        // çizim
        let ops = shared.ops.lock().map(|o| o.clone()).unwrap_or_default();
        if !ops.is_empty() {
            let layer = draw::rasterize(&ops, tw, th);
            draw::composite_over_bgr(&mut bgr, tw, th, &layer);
        }

        // imleç
        if cfg.cursor_hl {
            if let Some((cx, cy)) = util::cursor_pos() {
                let mut x = cx as f32 - mon_origin.0 as f32;
                let mut y = cy as f32 - mon_origin.1 as f32;
                if let Some((rx, ry, rw, rh)) = region {
                    x -= rx as f32;
                    y -= ry as f32;
                    if rw > 0 && rh > 0 {
                        x *= tw as f32 / rw as f32;
                        y *= th as f32 / rh as f32;
                    }
                }
                let (r, g, b) = util::hex_rgb(&cfg.cursor_color);
                let col = [b, g, r];
                let sz = cfg.cursor_sz.max(4) as f32;
                circle_bgr(&mut bgr, tw, th, x, y, sz, col, 2, false);
                circle_bgr(&mut bgr, tw, th, x, y, 4.0, col, 1, true);
                circle_bgr(&mut bgr, tw, th, x, y, sz + 4.0, col, 1, false);
            }
        }

        // filigran
        if cfg.wm_enabled {
            let mut wm_layer = vec![0u8; (tw * th * 4) as usize];
            if !cfg.wm_text.trim().is_empty() {
                let text = cfg.wm_text.as_str();
                let size = 22.0f32;
                let twidth = (text.chars().count() as f32 * size * 0.62).max(20.0) as i32;
                let theight = (size * 1.4) as i32;
                let (px, py) = corner_pos(
                    &cfg.wm_pos,
                    twidth as u32 + 8,
                    theight as u32 + 4,
                    tw,
                    th,
                    16,
                );
                // arka plan
                for yy in py.max(0)..(py + theight + 4).min(th as i32) {
                    for xx in px.max(0)..(px + twidth + 8).min(tw as i32) {
                        let i = ((yy as u32 * tw + xx as u32) * 4) as usize;
                        wm_layer[i + 3] = (255.0 * cfg.wm_opacity * 0.5) as u8;
                    }
                }
                draw::draw_text_rgba(
                    &mut wm_layer,
                    tw,
                    th,
                    text,
                    (px + 4) as f32,
                    (py + 2) as f32,
                    size,
                    [255, 255, 255],
                    cfg.wm_opacity,
                );
            }
            if let Some(img) = wm_img.as_ref() {
                let (px, py) = corner_pos(&cfg.wm_pos, img.width(), img.height(), tw, th, 16);
                for y in 0..img.height() {
                    for x in 0..img.width() {
                        let (dx, dy) = (px + x as i32, py + y as i32);
                        if dx < 0 || dy < 0 || dx >= tw as i32 || dy >= th as i32 {
                            continue;
                        }
                        let p = img.get_pixel(x, y);
                        let a = p[3] as f32 / 255.0 * cfg.wm_opacity;
                        let i = ((dy as u32 * tw + dx as u32) * 4) as usize;
                        wm_layer[i] = p[0];
                        wm_layer[i + 1] = p[1];
                        wm_layer[i + 2] = p[2];
                        wm_layer[i + 3] = (a * 255.0) as u8;
                    }
                }
            }
            draw::composite_over_bgr(&mut bgr, tw, th, &wm_layer);
        }

        if let Some(stdin) = stdin.as_mut() {
            for _ in 0..to_write {
                if stdin.write_all(&bgr).is_err() {
                    pipe_dead = true;
                    break;
                }
            }
        } else {
            pipe_dead = true;
        }
        written += to_write;
        if pipe_dead {
            break;
        }
    }

    if let Some((rec, _rx)) = recorder {
        let _ = rec.stop();
    }
    drop(stdin);
    None
}

fn bgr_to_rgba(bgr: &[u8]) -> Vec<u8> {
    let mut out = vec![255u8; bgr.len() / 3 * 4];
    for i in 0..bgr.len() / 3 {
        out[i * 4] = bgr[i * 3 + 2];
        out[i * 4 + 1] = bgr[i * 3 + 1];
        out[i * 4 + 2] = bgr[i * 3];
        out[i * 4 + 3] = 255;
    }
    out
}

/// Webcam çözünürlük sorgusu (dshow destekli çözünürlükler denenir).
pub fn query_webcam_resolutions(cam_idx: usize) -> Vec<(u32, u32)> {
    let vids = util::video_devices();
    let name = vids
        .get(cam_idx)
        .cloned()
        .or_else(|| vids.first().cloned())
        .unwrap_or_default();
    if name.is_empty() {
        return vec![(1280, 720)];
    }
    let mut supported = Vec::new();
    for (w, h) in [
        (3840, 2160),
        (1920, 1080),
        (1280, 720),
        (854, 480),
        (640, 360),
    ] {
        let child = util::ffmpeg()
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "dshow",
                "-video_size",
                &format!("{w}x{h}"),
                "-i",
                &format!("video={name}"),
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(mut c) = child {
            std::thread::sleep(Duration::from_millis(700));
            match c.try_wait() {
                Ok(Some(st)) if st.success() => supported.push((w, h)),
                Ok(Some(_)) => {}
                _ => {
                    let _ = c.kill();
                }
            }
        }
    }
    if supported.is_empty() {
        vec![(1280, 720)]
    } else {
        supported
    }
}

/// Kamera cihaz listesinin boş olması durumunda kullanılacak komut denemesi.
pub fn camera_command_debug(name: &str) -> Command {
    let mut c = util::ffmpeg();
    c.args([
        "-f",
        "dshow",
        "-i",
        &format!("video={name}"),
        "-frames:v",
        "1",
        "-f",
        "null",
        "-",
    ]);
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_screen_short_clip() {
        let dir = std::env::temp_dir().join("aptoza_record_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut cfg = crate::model::RecorderConfig::default();
        cfg.screen_on = true;
        cfg.webcam_on = false;
        cfg.audio_on = false;
        cfg.fps = 30;
        cfg.quality = 26;
        cfg.output_dir = dir.to_string_lossy().to_string();

        let eng = Engine::new(Arc::new(Mutex::new(Vec::new())));
        eng.start(cfg).expect("kayıt başlatılamadı");
        std::thread::sleep(Duration::from_secs(3));
        eng.stop();

        let mut done = None;
        for _ in 0..300 {
            if let Some(EngineEvent::Done { ok, path, message }) = eng.poll() {
                done = Some((ok, path, message));
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let (ok, path, message) = done.expect("kayıt tamamlanmadı");
        assert!(ok, "kayıt hatası: {message:?}");
        let path = path.expect("çıktı yolu yok");
        let meta = std::fs::metadata(&path).expect("kayıt dosyası yok");
        assert!(meta.len() > 10_000, "kayıt çok küçük: {}", meta.len());
        let dur = crate::util::duration(&path);
        assert!(dur > 1.5, "kayıt süresi beklenenden kısa: {dur}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
