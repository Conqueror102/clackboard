//! A small low-latency mixer. Key presses push voices; the audio callback mixes them with
//! per-voice pitch (by resampling) and stereo pan. The output stream lives on its own thread
//! and is rebuilt when the default output device changes or fails.

use super::pack::Segment;
use rodio::cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::cpal::{self, SampleFormat, Stream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MAX_VOICES: usize = 48;

pub struct Voice {
    seg: Segment,
    /// Position in source frames.
    pos: f64,
    /// Source frames per output frame (rate ratio × pitch).
    step: f64,
    gl: f32,
    gr: f32,
    /// Short fade-out over the last frames so clips never end in a click.
    fade: usize,
}

struct Shared {
    pending: Mutex<Vec<Voice>>,
    /// Output sample rate (set when the stream starts).
    rate: AtomicU32,
    /// Master volume as f32 bits.
    volume: AtomicU32,
    failed: AtomicBool,
}

#[derive(Clone)]
pub struct Engine {
    shared: Arc<Shared>,
}

impl Engine {
    /// Start the audio thread. Never fails: with no output device it stays silent and keeps
    /// trying, so plugging headphones in later just works.
    pub fn start() -> Self {
        let shared = Arc::new(Shared {
            pending: Mutex::new(Vec::new()),
            rate: AtomicU32::new(48_000),
            volume: AtomicU32::new(0.7f32.to_bits()),
            failed: AtomicBool::new(false),
        });
        let s = shared.clone();
        let _ = std::thread::Builder::new().name("clackboard-audio".into()).spawn(move || run(s));
        Self { shared }
    }

    pub fn set_volume(&self, v: f32) {
        self.shared.volume.store(v.clamp(0.0, 2.0).to_bits(), Ordering::Relaxed);
    }

    /// Queue a sound. `pan` is -1 (left) … 1 (right); `pitch` 1.0 is the recorded pitch.
    pub fn play(&self, seg: &Segment, gain: f32, pan: f32, pitch: f32) {
        let out_rate = self.shared.rate.load(Ordering::Relaxed).max(8_000) as f64;
        let step = seg.buf.rate as f64 / out_rate * pitch.clamp(0.25, 4.0) as f64;
        // Equal-power pan.
        let angle = (pan.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
        let voice = Voice {
            seg: seg.clone(),
            pos: seg.start as f64,
            step,
            gl: gain * angle.cos() * std::f32::consts::SQRT_2,
            gr: gain * angle.sin() * std::f32::consts::SQRT_2,
            fade: ((seg.buf.rate as usize) / 400).max(8), // ~2.5 ms
        };
        if let Ok(mut p) = self.shared.pending.lock() {
            if p.len() >= MAX_VOICES {
                p.remove(0);
            }
            p.push(voice);
        }
    }
}

fn run(shared: Arc<Shared>) {
    loop {
        let host = cpal::default_host();
        let device = host.default_output_device();
        let name = device.as_ref().and_then(|d| d.name().ok());
        let stream = device.and_then(|d| match open(&d, shared.clone()) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("sound: could not open output: {e}");
                None
            }
        });
        shared.failed.store(false, Ordering::Relaxed);
        // Watch for a failed stream or a new default device, then rebuild.
        loop {
            std::thread::sleep(Duration::from_millis(1500));
            if shared.failed.load(Ordering::Relaxed) || stream.is_none() {
                break;
            }
            let now = cpal::default_host().default_output_device().and_then(|d| d.name().ok());
            if now != name {
                break;
            }
        }
        drop(stream);
    }
}

fn open(device: &cpal::Device, shared: Arc<Shared>) -> Result<Stream, String> {
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    shared.rate.store(config.sample_rate.0, Ordering::Relaxed);
    let channels = config.channels as usize;
    let fail = shared.clone();
    let on_error = move |e: cpal::StreamError| {
        eprintln!("sound: output error: {e}");
        fail.failed.store(true, Ordering::Relaxed);
    };
    let stream = match format {
        SampleFormat::F32 => build::<f32>(device, &config, channels, shared, on_error),
        SampleFormat::I16 => build::<i16>(device, &config, channels, shared, on_error),
        SampleFormat::U16 => build::<u16>(device, &config, channels, shared, on_error),
        SampleFormat::I32 => build::<i32>(device, &config, channels, shared, on_error),
        other => return Err(format!("unsupported sample format {other:?}")),
    }?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    shared: Arc<Shared>,
    on_error: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let mut voices: Vec<Voice> = Vec::with_capacity(MAX_VOICES);
    let mut mix: Vec<f32> = Vec::new();
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                if let Ok(mut p) = shared.pending.try_lock() {
                    voices.extend(p.drain(..));
                }
                while voices.len() > MAX_VOICES {
                    voices.remove(0);
                }
                let frames = data.len() / channels.max(1);
                mix.clear();
                mix.resize(frames * 2, 0.0);
                for v in voices.iter_mut() {
                    render(v, &mut mix);
                }
                voices.retain(|v| (v.pos as usize) + 1 < v.seg.end);
                let vol = f32::from_bits(shared.volume.load(Ordering::Relaxed));
                for (i, frame) in data.chunks_mut(channels.max(1)).enumerate() {
                    let l = soft_clip(mix[i * 2] * vol);
                    let r = soft_clip(mix[i * 2 + 1] * vol);
                    for (c, s) in frame.iter_mut().enumerate() {
                        let x = match (channels, c) {
                            (1, _) => (l + r) * 0.5,
                            (_, 0) => l,
                            (_, 1) => r,
                            _ => 0.0,
                        };
                        *s = T::from_sample(x);
                    }
                }
            },
            on_error,
            None,
        )
        .map_err(|e| e.to_string())
}

fn render(v: &mut Voice, out: &mut [f32]) {
    let s = &v.seg.buf.samples;
    let end = v.seg.end;
    for frame in out.chunks_mut(2) {
        let i = v.pos as usize;
        if i + 1 >= end {
            break;
        }
        let t = (v.pos - i as f64) as f32;
        let (l0, r0, l1, r1) = (s[i * 2], s[i * 2 + 1], s[i * 2 + 2], s[i * 2 + 3]);
        let left = end - i;
        let env = if left < v.fade { left as f32 / v.fade as f32 } else { 1.0 };
        frame[0] += (l0 + (l1 - l0) * t) * v.gl * env;
        frame[1] += (r0 + (r1 - r0) * t) * v.gr * env;
        v.pos += v.step;
    }
}

/// Keep many overlapping presses from hard-clipping.
fn soft_clip(x: f32) -> f32 {
    if x.abs() <= 0.8 {
        x
    } else {
        x.signum() * (0.8 + 0.2 * ((x.abs() - 0.8) / 0.2).tanh())
    }
}
