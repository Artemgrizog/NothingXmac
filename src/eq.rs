//! Frequency response of the device EQ (low shelf / peak / high shelf biquads).

use crate::proto::Band;
use std::f64::consts::PI;

pub const F_MIN: f64 = 20.0;
pub const F_MAX: f64 = 20000.0;

fn mag_db(kind: u8, f0: f64, q: f64, gain_db: f64, f: f64) -> f64 {
    let fs = 48000.0;
    let a = 10f64.powf(gain_db / 40.0);
    let w0 = 2.0 * PI * f0.min(fs * 0.45) / fs;
    let (sn, cs) = (w0.sin(), w0.cos());
    let alpha = sn / (2.0 * q.max(0.05));
    let sa = 2.0 * a.sqrt() * alpha;
    let (b0, b1, b2, a0, a1, a2) = match kind {
        0 => (
            a * ((a + 1.0) - (a - 1.0) * cs + sa),
            2.0 * a * ((a - 1.0) - (a + 1.0) * cs),
            a * ((a + 1.0) - (a - 1.0) * cs - sa),
            (a + 1.0) + (a - 1.0) * cs + sa,
            -2.0 * ((a - 1.0) + (a + 1.0) * cs),
            (a + 1.0) + (a - 1.0) * cs - sa,
        ),
        2 => (
            a * ((a + 1.0) + (a - 1.0) * cs + sa),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * cs),
            a * ((a + 1.0) + (a - 1.0) * cs - sa),
            (a + 1.0) - (a - 1.0) * cs + sa,
            2.0 * ((a - 1.0) - (a + 1.0) * cs),
            (a + 1.0) - (a - 1.0) * cs - sa,
        ),
        _ => (1.0 + alpha * a, -2.0 * cs, 1.0 - alpha * a, 1.0 + alpha / a, -2.0 * cs, 1.0 - alpha / a),
    };
    let w = 2.0 * PI * f / fs;
    let (c1, s1, c2, s2) = (w.cos(), w.sin(), (2.0 * w).cos(), (2.0 * w).sin());
    let nr = b0 + b1 * c1 + b2 * c2;
    let ni = -(b1 * s1 + b2 * s2);
    let dr = a0 + a1 * c1 + a2 * c2;
    let di = -(a1 * s1 + a2 * s2);
    10.0 * ((nr * nr + ni * ni) / (dr * dr + di * di)).log10()
}

/// Position of a frequency on the log axis, 0..1.
pub fn freq_to_x(f: f64) -> f64 {
    ((f / F_MIN).log10() / (F_MAX / F_MIN).log10()).clamp(0.0, 1.0)
}

pub fn x_to_freq(x: f64) -> f64 {
    F_MIN * (F_MAX / F_MIN).powf(x.clamp(0.0, 1.0))
}

/// SVG path of the summed response in a `w` x `h` box, `range` dB above and below 0.
pub fn curve(bands: &[Band], w: f32, h: f32, range: f64) -> String {
    let mut s = String::new();
    let n = 160;
    for i in 0..=n {
        let x = i as f64 / n as f64;
        let f = x_to_freq(x);
        let db: f64 = bands.iter().map(|b| mag_db(b.kind, b.freq as f64, b.q as f64, b.gain as f64, f)).sum();
        let y = (0.5 - db.clamp(-range, range) / (2.0 * range)) * h as f64;
        s.push_str(&format!("{} {:.1} {:.1} ", if i == 0 { "M" } else { "L" }, x * w as f64, y));
    }
    s
}

/// The fixed 3-band "custom" EQ (bass low shelf, mid peak, treble high shelf).
pub fn simple_bands(bass: f32, mid: f32, treble: f32) -> [Band; 3] {
    [
        Band { kind: 1, gain: mid, freq: 980.0, q: 0.66 },
        Band { kind: 2, gain: treble, freq: 3500.0, q: 1.0 },
        Band { kind: 0, gain: bass, freq: 140.0, q: 0.8 },
    ]
}
