//! Video / ses düzenleme, ses efektleri ve görselden video üretimi.
//!
//! Bu modül Python sürümündeki `VideoEditor`, `AudioEditor`,
//! `AudioEffectsEngine` ve `ImageToVideoEditor` sınıflarının birebir
//! ffmpeg karşılıklarını içerir.
use crate::timeline::ClipEffects;
use crate::util;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// ------------------------------------------------------------------ video
pub struct VideoEditor;

impl VideoEditor {
    pub fn trim(inp: &str, out: &str, t0: f64, t1: f64) -> Result<()> {
        if !Path::new(inp).exists() {
            return Err(anyhow!("Kaynak video bulunamadı."));
        }
        if !t0.is_finite() || !t1.is_finite() || t0 < 0.0 || t1 <= t0 {
            return Err(anyhow!(
                "Geçerli bir zaman aralığı girin (başlangıç 0 veya daha büyük, bitiş başlangıçtan büyük olmalı)."
            ));
        }
        util::ensure_parent(Path::new(out))?;
        let args: Vec<String> = vec![
            "-y".into(),
            "-ss".into(),
            format!("{t0}"),
            "-i".into(),
            inp.into(),
            "-to".into(),
            format!("{}", t1 - t0),
            "-c:v".into(),
            "libx264".into(),
            "-crf".into(),
            "18".into(),
            "-preset".into(),
            "medium".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "192k".into(),
            out.into(),
        ];
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
    }

    pub fn merge(paths: &[String], out: &str) -> Result<()> {
        if paths.len() < 2 {
            return Err(anyhow!("En az 2 video eklemeniz gerekiyor."));
        }
        util::ensure_parent(Path::new(out))?;
        let lst = format!("{out}_list.txt");
        let mut text = String::new();
        for p in paths {
            text.push_str(&format!(
                "file '{}'\n",
                p.replace('\\', "/").replace('\'', "'\\''")
            ));
        }
        fs::write(&lst, text)?;
        let args: Vec<String> = vec![
            "-y".into(),
            "-f".into(),
            "concat".into(),
            "-safe".into(),
            "0".into(),
            "-i".into(),
            lst.clone(),
            "-c:v".into(),
            "libx264".into(),
            "-crf".into(),
            "18".into(),
            "-preset".into(),
            "medium".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "192k".into(),
            out.into(),
        ];
        let res = util::ffmpeg_ok(&args).map_err(|e| anyhow!(e));
        let _ = fs::remove_file(&lst);
        res
    }

    pub fn convert(inp: &str, out: &str, fmt: &str) -> Result<()> {
        if !Path::new(inp).exists() {
            return Err(anyhow!("Kaynak dosya bulunamadı."));
        }
        util::ensure_parent(Path::new(out))?;
        let mut extra: Vec<String> = match fmt {
            "mp4" | "mkv" | "mov" => vec![
                "-c:v", "libx264", "-crf", "18", "-preset", "medium", "-c:a", "aac", "-b:a", "192k",
            ],
            "avi" => vec![
                "-c:v",
                "libxvid",
                "-q:v",
                "3",
                "-c:a",
                "libmp3lame",
                "-b:a",
                "192k",
            ],
            "gif" => vec!["-vf", "fps=10,scale=640:-1:flags=lanczos", "-loop", "0"],
            "webm" => vec![
                "-c:v",
                "libvpx-vp9",
                "-crf",
                "30",
                "-b:v",
                "0",
                "-c:a",
                "libopus",
                "-b:a",
                "192k",
            ],
            "mp3" => vec!["-vn", "-c:a", "libmp3lame", "-b:a", "192k"],
            "wav" => vec!["-vn", "-c:a", "pcm_s16le"],
            _ => vec!["-c", "copy"],
        }
        .into_iter()
        .map(String::from)
        .collect();
        // GIF için -loop bir çıktı seçeneğidir, girişten sonra kalmalı; zaten öyle.
        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), inp.into()];
        args.append(&mut extra);
        args.push(out.into());
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
    }
}

// ------------------------------------------------------------------ ses
pub struct AudioEditor;

impl AudioEditor {
    pub const FORMATS: [&'static str; 8] =
        ["mp3", "wav", "ogg", "aac", "flac", "m4a", "opus", "wma"];

    pub fn codec_args(fmt: &str) -> Vec<String> {
        let v: &[&str] = match fmt.to_lowercase().as_str() {
            "mp3" => &["-c:a", "libmp3lame", "-b:a", "192k"],
            "wav" => &["-c:a", "pcm_s16le"],
            "ogg" => &["-c:a", "libvorbis", "-q:a", "5"],
            "aac" => &["-c:a", "aac", "-b:a", "192k"],
            "flac" => &["-c:a", "flac"],
            "m4a" => &["-c:a", "aac", "-b:a", "192k"],
            "opus" => &["-c:a", "libopus", "-b:a", "128k"],
            "wma" => &["-c:a", "wmav2", "-b:a", "192k"],
            _ => &["-c:a", "copy"],
        };
        v.iter().map(|s| s.to_string()).collect()
    }

    pub fn trim(inp: &str, out: &str, t0: f64, t1: f64) -> Result<()> {
        if !Path::new(inp).exists() {
            return Err(anyhow!("Kaynak ses dosyası bulunamadı."));
        }
        if !t0.is_finite() || !t1.is_finite() || t0 < 0.0 || t1 <= t0 {
            return Err(anyhow!(
                "Geçerli bir zaman aralığı girin (başlangıç 0 veya daha büyük, bitiş başlangıçtan büyük olmalı)."
            ));
        }
        let ext = extension(out);
        let mut args: Vec<String> = vec![
            "-y".into(),
            "-ss".into(),
            format!("{t0}"),
            "-i".into(),
            inp.into(),
            "-t".into(),
            format!("{}", t1 - t0),
            "-vn".into(),
        ];
        args.extend(Self::codec_args(&ext));
        args.push(out.into());
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
    }

    pub fn split(inp: &str, out1: &str, out2: &str, split_time: f64) -> Result<()> {
        if !Path::new(inp).exists() {
            return Err(anyhow!("Kaynak ses dosyası bulunamadı."));
        }
        if !split_time.is_finite() || split_time <= 0.0 {
            return Err(anyhow!("Bölme noktası 0'dan büyük olmalıdır."));
        }
        let dur = util::duration(inp);
        if dur > 0.0 && split_time >= dur - 0.01 {
            return Err(anyhow!("Bölme noktası ses süresinden küçük olmalıdır."));
        }
        let ext = extension(inp);
        let codec = Self::codec_args(&ext);
        let mut a: Vec<String> = vec![
            "-y".into(),
            "-i".into(),
            inp.into(),
            "-t".into(),
            format!("{split_time}"),
            "-vn".into(),
        ];
        a.extend(codec.clone());
        a.push(out1.into());
        let mut b: Vec<String> = vec![
            "-y".into(),
            "-ss".into(),
            format!("{split_time}"),
            "-i".into(),
            inp.into(),
            "-vn".into(),
        ];
        b.extend(codec);
        b.push(out2.into());
        util::ffmpeg_ok(&a).map_err(|e| anyhow!(e))?;
        util::ffmpeg_ok(&b).map_err(|e| anyhow!(e))?;
        Ok(())
    }

    pub fn convert(inp: &str, out: &str, fmt: &str, bitrate: &str) -> Result<()> {
        let mut codec = Self::codec_args(fmt);
        if bitrate != "192k" && codec.len() >= 4 && codec[2] == "-b:a" {
            codec[3] = bitrate.to_string();
        }
        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), inp.into(), "-vn".into()];
        args.extend(codec);
        args.push(out.into());
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
    }

    pub fn merge(paths: &[String], out: &str) -> Result<()> {
        if paths.len() < 2 {
            return Err(anyhow!("En az 2 ses dosyası eklemeniz gerekiyor."));
        }
        let ext = extension(out);
        let codec = Self::codec_args(&ext);
        let mut args: Vec<String> = vec!["-y".into()];
        let mut inputs = Vec::new();
        let mut filters = Vec::new();
        for (i, p) in paths.iter().enumerate() {
            args.extend(["-i".into(), p.clone()]);
            inputs.push(format!("[a{i}]"));
            filters.push(format!(
                "[{i}:a]aresample=44100,aformat=sample_fmts=fltp:channel_layouts=stereo[a{i}]"
            ));
        }
        let concat = format!("{}concat=n={}:v=0:a=1[aout]", inputs.join(""), paths.len());
        filters.push(concat);
        args.extend([
            "-filter_complex".into(),
            filters.join(";"),
            "-map".into(),
            "[aout]".into(),
            "-vn".into(),
        ]);
        args.extend(codec);
        args.push(out.into());
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
    }
}

fn extension(path: &str) -> String {
    Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

// ------------------------------------------------------------------ ses efektleri
#[derive(Clone, Debug)]
pub struct AudioEffectsParams {
    pub volume: f32,
    pub noise: bool,
    pub silence: bool,
    pub loudnorm: bool,
    pub compressor: bool,
    pub stereo: bool,
    pub panning: f32,
    pub vocal_remove: bool,
    pub bass: f32,
    pub treble: f32,
    pub highpass: f32,
    pub lowpass: f32,
    pub pitch: f32,
    pub speed: f32,
    pub echo: bool,
    pub reverse: bool,
    pub chorus: bool,
    pub flanger: bool,
    pub tremolo: bool,
    pub vibrato: bool,
    pub fade_in: f32,
    pub fade_out: f32,
}

impl Default for AudioEffectsParams {
    fn default() -> Self {
        Self {
            volume: 1.0,
            noise: false,
            silence: false,
            loudnorm: false,
            compressor: false,
            stereo: false,
            panning: 0.0,
            vocal_remove: false,
            bass: 0.0,
            treble: 0.0,
            highpass: 0.0,
            lowpass: 20000.0,
            pitch: 1.0,
            speed: 1.0,
            echo: false,
            reverse: false,
            chorus: false,
            flanger: false,
            tremolo: false,
            vibrato: false,
            fade_in: 0.0,
            fade_out: 0.0,
        }
    }
}

pub struct AudioEffectsEngine;

impl AudioEffectsEngine {
    /// Python `build_filters` sırası birebir korunmuştur.
    pub fn build_filters(p: &AudioEffectsParams, total_dur: Option<f64>) -> Vec<String> {
        let mut f: Vec<String> = Vec::new();
        if (p.volume - 1.0).abs() > f32::EPSILON {
            f.push(format!("volume={}", p.volume));
        }
        if p.noise {
            f.push("afftdn=nf=-25".into());
        }
        if p.silence {
            f.push("silenceremove=stop_periods=-1:stop_duration=1:stop_threshold=-35dB".into());
        }
        if p.loudnorm {
            f.push("loudnorm=I=-16:TP=-1.5:LRA=11".into());
        }
        if p.compressor {
            f.push("acompressor=threshold=-20dB:ratio=4:attack=5:release=50".into());
        }
        if p.panning != 0.0 {
            let left = (1.0 - p.panning).min(1.0);
            let right = (1.0 + p.panning).min(1.0);
            f.push(format!("pan=stereo|c0={left}*c0|c1={right}*c1"));
        } else if p.stereo {
            f.push("pan=stereo|c0=c0|c1=c0".into());
        }
        if p.vocal_remove {
            f.push("pan=stereo|c0=0.5*c0-0.5*c1|c1=0.5*c1-0.5*c0".into());
        }
        if p.bass != 0.0 {
            f.push(format!("bass=g={}", p.bass));
        }
        if p.treble != 0.0 {
            f.push(format!("treble=g={}", p.treble));
        }
        if p.highpass > 0.0 {
            f.push(format!("highpass=f={}", p.highpass));
        }
        if p.lowpass < 20000.0 {
            f.push(format!("lowpass=f={}", p.lowpass));
        }
        if (p.pitch - 1.0).abs() > f32::EPSILON {
            f.push(format!("asetrate=44100*{},atempo=1/{}", p.pitch, p.pitch));
        }
        if (p.speed - 1.0).abs() > f32::EPSILON {
            f.push(format!("atempo={}", p.speed));
        }
        if p.echo {
            f.push("aecho=0.8:0.9:500:0.4".into());
        }
        if p.reverse {
            f.push("areverse".into());
        }
        if p.chorus {
            f.push("chorus=0.5:0.9:50|60|40:0.4|0.32|0.3:0.25|0.4|0.3:2|2.3|1.3".into());
        }
        if p.flanger {
            f.push("flanger=delay=0".into());
        }
        if p.tremolo {
            f.push("tremolo=f=5.0:d=0.5".into());
        }
        if p.vibrato {
            f.push("vibrato=f=7.0:d=0.5".into());
        }
        if p.fade_in > 0.0 {
            f.push(format!("afade=t=in:ss=0:d={}", p.fade_in));
        }
        if p.fade_out > 0.0 {
            if let Some(d) = total_dur {
                if d > p.fade_out as f64 {
                    f.push(format!(
                        "afade=t=out:st={}:d={}",
                        d - p.fade_out as f64,
                        p.fade_out
                    ));
                }
            }
        }
        f
    }

    pub fn apply(p: &AudioEffectsParams, src: &str, out: &str) -> Result<()> {
        let filters = Self::build_filters(p, Some(util::duration(src)));
        let ext = extension(out);
        let mut args: Vec<String> = vec!["-y".into(), "-i".into(), src.into(), "-vn".into()];
        if !filters.is_empty() {
            args.extend(["-af".into(), filters.join(",")]);
        }
        match ext.as_str() {
            "mp3" => args.extend([
                "-c:a".into(),
                "libmp3lame".into(),
                "-b:a".into(),
                "192k".into(),
            ]),
            "wav" => args.extend(["-c:a".into(), "pcm_s16le".into()]),
            "aac" | "m4a" => {
                args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "192k".into()])
            }
            _ => args.extend([
                "-c:a".into(),
                "libmp3lame".into(),
                "-b:a".into(),
                "192k".into(),
            ]),
        }
        args.push(out.into());
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
    }

    pub fn preview_file(p: &AudioEffectsParams, src: &str) -> Result<PathBuf> {
        let out = util::unique_temp("wav");
        let dur = util::duration(src);
        let filters = Self::build_filters(p, Some(dur.min(45.0)));
        let mut args: Vec<String> = vec![
            "-y".into(),
            "-i".into(),
            src.into(),
            "-t".into(),
            "45".into(),
            "-vn".into(),
        ];
        if !filters.is_empty() {
            args.extend(["-af".into(), filters.join(",")]);
        }
        args.extend([
            "-c:a".into(),
            "pcm_s16le".into(),
            "-ar".into(),
            "44100".into(),
            "-ac".into(),
            "2".into(),
            out.to_string_lossy().to_string(),
        ]);
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))?;
        Ok(out)
    }

    /// Dalga formu PNG üretir (ses efektleri sekmesi).
    pub fn waveform(src: &str) -> Result<PathBuf> {
        let out = util::unique_temp("png");
        let args: Vec<String> = vec![
            "-y".into(),
            "-i".into(),
            src.into(),
            "-filter_complex".into(),
            "showwavespic=s=580x140:colors=0x238636".into(),
            "-frames:v".into(),
            "1".into(),
            out.to_string_lossy().to_string(),
        ];
        util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))?;
        Ok(out)
    }
}

// ------------------------------------------------------------------ görselden video
#[derive(Clone, Debug)]
pub struct ImageItem {
    pub path: String,
    pub duration: f32,
    pub effects: ClipEffects,
}

impl ImageItem {
    pub fn new(path: String, duration: f32) -> Self {
        Self {
            path,
            duration,
            effects: ClipEffects::default(),
        }
    }
    pub fn name(&self) -> String {
        util::basename(&self.path)
    }
}

/// `ImageToVideoModule._build_vf_list` birebir karşılığı.
pub fn build_image_vf(
    eff: &ClipEffects,
    w: u32,
    h: u32,
    dur: Option<f64>,
    is_preview: bool,
) -> Vec<String> {
    let mut vf: Vec<String> = Vec::new();
    if eff.rotate {
        vf.push("transpose=1".into());
    }
    if eff.mirror {
        vf.push("hflip".into());
    }
    if eff.vflip {
        vf.push("vflip".into());
    }
    if eff.invert {
        vf.push("negate".into());
    }
    if is_preview {
        vf.push("scale=400:400:force_original_aspect_ratio=decrease".into());
        vf.push("pad=400:400:(ow-iw)/2:(oh-ih)/2:color=black".into());
    } else {
        vf.push(format!(
            "scale={w}:{h}:force_original_aspect_ratio=decrease"
        ));
        vf.push(format!("pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:color=black"));
    }
    if eff.brightness != 0.0 || eff.contrast != 1.0 || eff.saturation != 1.0 || eff.gamma != 1.0 {
        vf.push(format!(
            "eq=brightness={}:contrast={}:saturation={}:gamma={}",
            eff.brightness, eff.contrast, eff.saturation, eff.gamma
        ));
    }
    if eff.hue != 0.0 {
        vf.push(format!("hue=h={}", eff.hue));
    }
    if eff.blur > 0.0 {
        vf.push(format!("boxblur={}:1", eff.blur));
    }
    if eff.sepia {
        vf.push(
            "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131:0:0:0:0:1".into(),
        );
    } else if eff.bw {
        vf.push("hue=s=0".into());
    } else if eff.red_filter {
        vf.push("colorchannelmixer=1:0:0:0:0:0:0:0:0:0:0:0:0:0:0:1".into());
    } else if eff.blue_filter {
        vf.push("colorchannelmixer=0:0:0:0:0:0:0:0:0:0:1:0:0:0:0:1".into());
    }
    if eff.vignette {
        vf.push("vignette".into());
    }
    if eff.sharpen {
        vf.push("unsharp=5:5:1.0:5:5:0.0".into());
    }
    if !is_preview {
        if let Some(d) = dur {
            if eff.fade_in > 0.0 {
                vf.push(format!("fade=t=in:st=0:d={}", eff.fade_in));
            }
            if eff.fade_out > 0.0 {
                let st = (d - eff.fade_out as f64).max(0.0);
                vf.push(format!("fade=t=out:st={st}:d={}", eff.fade_out));
            }
        }
    }
    vf
}

/// Timeline klip efektleri için tek karelik önizleme üretir.
pub fn preview_video_frame(path: &str, t: f64, vf: &[String], out: &Path) -> Result<()> {
    let mut args: Vec<String> = vec![
        "-y".into(),
        "-ss".into(),
        format!("{t:.2}"),
        "-i".into(),
        path.into(),
        "-vframes".into(),
        "1".into(),
    ];
    if !vf.is_empty() {
        args.extend(["-vf".into(), vf.join(",")]);
    }
    args.push(out.to_string_lossy().to_string());
    util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
}

pub fn preview_image(path: &str, eff: &ClipEffects, out: &Path) -> Result<()> {
    let vf = build_image_vf(eff, 400, 400, None, true);
    let args: Vec<String> = vec![
        "-y".into(),
        "-i".into(),
        path.into(),
        "-vframes".into(),
        "1".into(),
        "-vf".into(),
        vf.join(","),
        out.to_string_lossy().to_string(),
    ];
    util::ffmpeg_ok(&args).map_err(|e| anyhow!(e))
}

pub fn generate_images(
    items: &[ImageItem],
    out: &str,
    w: u32,
    h: u32,
    status: Arc<Mutex<String>>,
) -> Result<()> {
    if items.is_empty() {
        return Err(anyhow!("Lütfen en az bir görsel ekleyin."));
    }
    if Path::new(out).is_dir() || out.ends_with(['\\', '/']) {
        return Err(anyhow!(
            "Çıktı yolu bir klasör. Lütfen video için bir dosya adı seçin (ör. gorsel_video.mp4)."
        ));
    }
    let w = util::round_even(w.clamp(16, 16_384));
    let h = util::round_even(h.clamp(16, 16_384));
    let tmp = util::temp_dir().join(format!("aptoza_img2vid_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;
    util::ensure_parent(Path::new(out))?;
    let res = (|| -> Result<()> {
        let mut clips: Vec<String> = Vec::new();
        for (i, item) in items.iter().enumerate() {
            if !Path::new(&item.path).exists() {
                return Err(anyhow!("Görsel bulunamadı: {}", item.path));
            }
            if let Ok(mut s) = status.lock() {
                *s = format!("⏳ Görsel {}/{} işleniyor...", i + 1, items.len());
            }
            let clip = tmp.join(format!("clip_{i:03}.mp4"));
            let dur = item.duration.max(0.1);
            let vf = build_image_vf(&item.effects, w, h, Some(dur as f64), false);
            let mut args: Vec<String> = vec![
                "-y".into(),
                "-loop".into(),
                "1".into(),
                "-t".into(),
                format!("{dur}"),
                "-i".into(),
                item.path.clone(),
                "-c:v".into(),
                "libx264".into(),
                "-pix_fmt".into(),
                "yuv420p".into(),
                "-r".into(),
                "30".into(),
                "-fps_mode".into(),
                "cfr".into(),
            ];
            if !vf.is_empty() {
                args.extend(["-vf".into(), vf.join(",")]);
            }
            args.push(clip.to_string_lossy().to_string());
            util::ffmpeg_ok(&args).map_err(|e| anyhow!("Görsel işleme hatası: {e}"))?;
            clips.push(clip.to_string_lossy().to_string());
        }
        if let Ok(mut s) = status.lock() {
            *s = "⚙ Klipler birleştiriliyor...".into();
        }
        let mut filter = String::new();
        for i in 0..clips.len() {
            filter.push_str(&format!(
                "[{i}:v]scale={w}:{h}:flags=lanczos,setsar=1,fps=30,format=yuv420p,setpts=PTS-STARTPTS[v{i}];"
            ));
        }
        for i in 0..clips.len() {
            filter.push_str(&format!("[v{i}]"));
        }
        filter.push_str(&format!("concat=n={}:v=1:a=0[outv]", clips.len()));

        let mut args: Vec<String> = vec!["-y".into()];
        for clip in &clips {
            args.extend(["-i".into(), clip.clone()]);
        }
        args.extend([
            "-filter_complex".into(),
            filter,
            "-map".into(),
            "[outv]".into(),
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            "medium".into(),
            "-crf".into(),
            "18".into(),
            "-pix_fmt".into(),
            "yuv420p".into(),
            "-r".into(),
            "30".into(),
            "-fps_mode".into(),
            "cfr".into(),
            "-an".into(),
            "-movflags".into(),
            "+faststart".into(),
            "-f".into(),
            "mp4".into(),
            out.into(),
        ]);
        util::ffmpeg_ok(&args).map_err(|e| anyhow!("Efektli klipleri birleştirme hatası: {e}"))?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&tmp);
    res?;
    if let Ok(mut s) = status.lock() {
        *s = "✅ Video başarıyla oluşturuldu!".into();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_effects_apply_and_waveform() {
        let dir = std::env::temp_dir().join("aptoza_audio_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("tone.wav");
        let out = dir.join("fx.mp3");
        util::ffmpeg_ok(&[
            "-y".to_string(),
            "-f".to_string(),
            "lavfi".to_string(),
            "-i".to_string(),
            "sine=frequency=440:duration=2".to_string(),
            "-ac".to_string(),
            "2".to_string(),
            src.to_string_lossy().to_string(),
        ])
        .unwrap();

        let mut p = AudioEffectsParams::default();
        p.loudnorm = true;
        p.compressor = true;
        p.bass = 4.0;
        p.treble = 2.0;
        p.echo = true;
        p.chorus = true;
        p.fade_in = 0.2;
        p.fade_out = 0.2;
        AudioEffectsEngine::apply(&p, &src.to_string_lossy(), &out.to_string_lossy())
            .expect("efekt uygulanamadı");
        let meta = std::fs::metadata(&out).expect("çıktı yok");
        assert!(meta.len() > 10_000, "çıktı çok küçük: {}", meta.len());

        let wave = AudioEffectsEngine::waveform(&src.to_string_lossy()).expect("dalga formu");
        let wmeta = std::fs::metadata(&wave).unwrap();
        assert!(wmeta.len() > 200, "dalga formu çok küçük: {}", wmeta.len());
        let _ = std::fs::remove_file(wave);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn images_to_video_pipeline() {
        let dir = std::env::temp_dir().join("aptoza_img2vid_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut images = Vec::new();
        for (name, color, size) in [("a.png", "red", "320x240"), ("b.png", "blue", "640x360")] {
            let p = dir.join(name);
            util::ffmpeg_ok(&[
                "-y".to_string(),
                "-f".to_string(),
                "lavfi".to_string(),
                "-i".to_string(),
                format!("color=c={color}:s={size}"),
                "-frames:v".to_string(),
                "1".to_string(),
                p.to_string_lossy().to_string(),
            ])
            .unwrap();
            let mut item = ImageItem::new(p.to_string_lossy().to_string(), 0.7);
            if name == "b.png" {
                item.effects.mirror = true;
            }
            images.push(item);
        }

        let out = dir.join("img2vid.mp4");
        let status = Arc::new(Mutex::new(String::new()));
        generate_images(&images, &out.to_string_lossy(), 641, 361, status.clone())
            .expect("görselden video üretilemedi");
        let meta = std::fs::metadata(&out).expect("çıktı yok");
        assert!(meta.len() > 2000, "çıktı çok küçük: {}", meta.len());
        let dimensions = util::ffprobe_bin()
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=s=x:p=0",
                out.to_str().unwrap(),
            ])
            .output()
            .expect("ffprobe çalıştırılamadı");
        assert!(dimensions.status.success(), "ffprobe başarısız oldu");
        assert_eq!(
            String::from_utf8_lossy(&dimensions.stdout).trim(),
            "640x360",
            "çıktı çözünürlüğü çift sayılara yuvarlanmalı"
        );
        let dur = util::duration(&out.to_string_lossy());
        assert!(dur > 1.0 && dur < 2.2, "beklenmeyen süre: {dur}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
