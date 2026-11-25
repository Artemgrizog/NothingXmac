//! Nothing / CMF serial protocol (RFCOMM "NT LINK" service).
//!
//! Frame: 55 | 60 | 01 | cmd_lo cmd_hi | len_lo len_hi | seq | payload | crc16_le
//! CRC-16/MODBUS (poly 0xA001 reflected, init 0xFFFF) over everything before the CRC.
//! Unsolicited notifications may use control 0x00 (no CRC).

pub const UUID_NT_LINK: [u8; 16] = [
    0xae, 0xac, 0x4a, 0x03, 0xdf, 0xf5, 0x49, 0x8f, 0x84, 0x3a, 0x34, 0x48, 0x7c, 0xf1, 0x33, 0xeb,
];

pub mod cmd {
    pub const BATTERY_GET: u16 = 0xC007;
    pub const SERIAL_GET: u16 = 0xC006;
    pub const FIRMWARE_GET: u16 = 0xC042;
    pub const MODEL_GET: u16 = 0xC01C;
    pub const ANC_GET: u16 = 0xC01E;
    pub const ANC_SET: u16 = 0xF00F;
    pub const EQ_GET: u16 = 0xC01F;
    pub const EQ_SET: u16 = 0xF010;
    pub const CUSTOM_EQ_GET: u16 = 0xC044;
    pub const CUSTOM_EQ_SET: u16 = 0xF041;
    pub const BASS_GET: u16 = 0xC04E;
    pub const BASS_SET: u16 = 0xF051;
    pub const LATENCY_GET: u16 = 0xC041;
    pub const LATENCY_SET: u16 = 0xF040;
    pub const SPATIAL_GET: u16 = 0xC04F;
    pub const SPATIAL_SET: u16 = 0xF052;
    pub const DUAL_GET: u16 = 0xC027;
    pub const DUAL_SET: u16 = 0xF01A;
    pub const DUAL_DEVICES_GET: u16 = 0xC028;
    pub const GESTURES_GET: u16 = 0xC018;
    pub const GESTURE_SET: u16 = 0xF003;
    pub const IN_EAR_GET: u16 = 0xC00E;
    pub const IN_EAR_SET: u16 = 0xF004;
    pub const RING_SET: u16 = 0xF002;
    pub const FIT_TEST_SET: u16 = 0xF014;
    pub const PERSONAL_ANC_GET: u16 = 0xC020;
    pub const PERSONAL_ANC_SET: u16 = 0xF011;
    pub const ADVANCED_EQ_GET: u16 = 0xC04C;
    pub const ADVANCED_EQ_SET: u16 = 0xF04F;
    pub const LISTEN_GET: u16 = 0xC050;
    pub const LISTEN_SET: u16 = 0xF01D;
}

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= b as u16;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
        }
    }
    crc
}

pub fn encode(cmd: u16, payload: &[u8], seq: u8) -> Vec<u8> {
    let mut v = vec![0x55, 0x60, 0x01, cmd as u8, (cmd >> 8) as u8, payload.len() as u8, 0x00, seq];
    v.extend_from_slice(payload);
    let c = crc16(&v);
    v.push(c as u8);
    v.push((c >> 8) as u8);
    v
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub cmd: u16,
    #[allow(dead_code)]
    pub seq: u8,
    pub payload: Vec<u8>,
}

#[derive(Default)]
pub struct Parser {
    buf: Vec<u8>,
}

impl Parser {
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn push(&mut self, data: &[u8]) -> Vec<Frame> {
        self.buf.extend_from_slice(data);
        let mut out = Vec::new();
        loop {
            match self.buf.iter().position(|&b| b == 0x55) {
                Some(0) => {}
                Some(i) => {
                    self.buf.drain(..i);
                }
                None => {
                    self.buf.clear();
                    break;
                }
            }
            if self.buf.len() < 8 {
                break;
            }
            let control = self.buf[1] as u16 | (self.buf[2] as u16) << 8;
            let has_crc = control & 0x20 != 0;
            let plen = self.buf[5] as usize | (self.buf[6] as usize) << 8;
            if plen > 2048 {
                self.buf.remove(0);
                continue;
            }
            let total = 8 + plen + if has_crc { 2 } else { 0 };
            if self.buf.len() < total {
                break;
            }
            out.push(Frame {
                cmd: self.buf[3] as u16 | (self.buf[4] as u16) << 8,
                seq: self.buf[7],
                payload: self.buf[8..8 + plen].to_vec(),
            });
            self.buf.drain(..total);
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BattId {
    Left,
    Right,
    Case,
    Single,
    Other(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct Batt {
    pub id: BattId,
    pub level: u8,
    pub charging: bool,
}

/// One band of the device's parametric EQ block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    /// 0 = low shelf, 1 = peak, 2 = high shelf
    pub kind: u8,
    pub gain: f32,
    pub freq: f32,
    pub q: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DualDevice {
    pub name: String,
    pub mac: [u8; 6],
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
    Battery(Vec<Batt>),
    /// current mode byte, remembered noise-cancellation level (1..=4) if reported
    Anc(u8, Option<u8>),
    EqPreset(u8),
    Bass { on: bool, level: u8 },
    Latency(bool),
    Spatial(u8),
    Firmware(String),
    Serial(String),
    /// Model id, e.g. 0xB170.
    Model(u16),
    Dual(bool),
    /// Bands stored in the device's EQ block (custom / advanced EQ).
    EqBands(Vec<Band>),
    /// (device, button, gesture type, action)
    Gestures(Vec<(u8, u8, u8, u8)>),
    InEar(bool),
    DualDevices(Vec<DualDevice>),
    /// ear tip fit test result: left, right (0 good, 1 poor, >=2 not measured)
    FitTest(u8, u8),
    PersonalAnc(bool),
    AdvancedEq(bool),
    /// CMF "listening mode" EQ preset
    ListenMode(u8),
}

fn f32_at(p: &[u8], o: usize) -> Option<f32> {
    let b = p.get(o..o + 4)?;
    Some(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

pub fn decode(f: &Frame) -> Option<Event> {
    let p = &f.payload;
    match f.cmd {
        0xE001 | 0x4007 => {
            let count = *p.first()? as usize;
            let mut v = Vec::new();
            for i in 0..count {
                let id = *p.get(1 + i * 2)?;
                let lv = *p.get(2 + i * 2)?;
                v.push(Batt {
                    id: match id {
                        2 => BattId::Left,
                        3 => BattId::Right,
                        4 => BattId::Case,
                        6 => BattId::Single,
                        o => BattId::Other(o),
                    },
                    level: (lv & 0x7F).min(100),
                    charging: lv & 0x80 != 0,
                });
            }
            Some(Event::Battery(v))
        }
        0xE003 | 0x401E => Some(Event::Anc(*p.get(1)?, p.get(4).copied().filter(|l| (1..=4).contains(l)))),
        0x401F => Some(Event::EqPreset(*p.first()?)),
        // 0x7051 / 0x7040 are write acknowledgements with a status byte, not state
        0x404E => Some(Event::Bass { on: *p.first()? != 0, level: p.get(1).copied().unwrap_or(2) / 2 }),
        0x4041 => Some(Event::Latency(*p.first()? == 1)),
        0x404F => Some(Event::Spatial(*p.first()?)),
        0x4042 => Some(Event::Firmware(ascii(p))),
        0x4006 => {
            // "6,2,1.0.1.81\n6,4,M346...\n" records; kind 4 = serial
            let text = ascii(p);
            for line in text.lines() {
                let mut it = line.splitn(3, ',');
                let (_, k, v) = (it.next(), it.next(), it.next());
                if k == Some("4") {
                    if let Some(v) = v {
                        return Some(Event::Serial(v.trim().to_string()));
                    }
                }
            }
            None
        }
        0x401C => Some(Event::Model(*p.first()? as u16 | (*p.get(1)? as u16) << 8)),
        0x4027 | 0xE006 => Some(Event::Dual(*p.first()? == 1)),
        0x4044 => Some(Event::EqBands(parse_bands(p))),
        0x4018 => {
            let n = *p.first()? as usize;
            let mut v = Vec::new();
            for i in 0..n {
                let r = p.get(1 + i * 4..5 + i * 4)?;
                v.push((r[0], r[1], r[2], r[3]));
            }
            Some(Event::Gestures(v))
        }
        0x400E => Some(Event::InEar(*p.get(2)? == 1)),
        0x4028 => Some(Event::DualDevices(parse_dual_devices(p))),
        0xE00D => Some(Event::FitTest(*p.first()?, *p.get(1)?)),
        0x4020 => Some(Event::PersonalAnc(*p.first()? == 1)),
        0x404C => Some(Event::AdvancedEq(*p.first()? == 1)),
        0x4050 => Some(Event::ListenMode(*p.first()?)),
        _ => None,
    }
}

/// EQ block: [count][total gain f32][count x (kind u8, gain f32, freq f32, q f32)]...
pub fn parse_bands(p: &[u8]) -> Vec<Band> {
    let n = (p.first().copied().unwrap_or(0) as usize).min(8);
    let mut v = Vec::new();
    for i in 0..n {
        let o = 5 + i * 13;
        let (Some(kind), Some(gain), Some(freq), Some(q)) = (p.get(o).copied(), f32_at(p, o + 1), f32_at(p, o + 5), f32_at(p, o + 9)) else { break };
        if !(gain.is_finite() && freq.is_finite() && q.is_finite()) || freq <= 0.0 {
            continue;
        }
        v.push(Band { kind: kind.min(2), gain: gain.clamp(-12.0, 12.0), freq: freq.clamp(20.0, 20000.0), q: q.clamp(0.1, 10.0) });
    }
    v
}

/// Advanced EQ block (up to 8 bands, always 109 bytes like the device returns it).
pub fn advanced_eq_payload(bands: &[Band]) -> Vec<u8> {
    let bands = &bands[..bands.len().min(8)];
    let highest = bands.iter().map(|b| b.gain.abs()).fold(0.0f32, f32::max);
    let mut v = vec![bands.len() as u8];
    v.extend_from_slice(&(-highest).to_le_bytes());
    for b in bands {
        v.push(b.kind);
        v.extend_from_slice(&b.gain.to_le_bytes());
        v.extend_from_slice(&b.freq.to_le_bytes());
        v.extend_from_slice(&b.q.to_le_bytes());
    }
    v.resize(109, 0);
    v
}

/// Reply to DUAL_DEVICES_GET: [?, ?, count, records...]; record = flags, mac[6], len, name.
fn parse_dual_devices(p: &[u8]) -> Vec<DualDevice> {
    let mut v = Vec::new();
    let count = p.get(2).copied().unwrap_or(0) as usize;
    let mut i = 3;
    for _ in 0..count {
        let Some(&flags) = p.get(i) else { break };
        let Some(mac) = p.get(i + 1..i + 7) else { break };
        let Some(&len_b) = p.get(i + 7) else { break };
        let len = if len_b & 0x80 != 0 { 0x1F } else { (len_b & 0x7F) as usize };
        let name_bytes = p.get(i + 8..(i + 8 + len).min(p.len())).unwrap_or(&[]);
        let mut name = String::from_utf8_lossy(name_bytes).trim_end_matches('\0').to_string();
        if len_b & 0x80 != 0 {
            name.push('…');
        }
        v.push(DualDevice { name, mac: mac.try_into().unwrap_or([0; 6]), connected: flags & 0x0F == 0x01 });
        i += 8 + len;
    }
    v
}

fn ascii(p: &[u8]) -> String {
    p.iter().filter(|b| **b == b'\n' || (0x20..0x7f).contains(*b)).map(|b| *b as char).collect()
}

/// Payload for the 3-band custom EQ (bass / mid / treble in dB, -6..=6).
pub fn custom_eq_payload(bass: f32, mid: f32, treble: f32) -> Vec<u8> {
    let mut t: [u8; 53] = [
        0x03, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0x75, 0x44, 0xc3, 0xf5, 0x28, 0x3f, 0x02, 0, 0, 0, 0, 0, 0xc0,
        0x5a, 0x45, 0, 0, 0x80, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0x0c, 0x43, 0xcd, 0xcc, 0x4c, 0x3f, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let highest = bass.abs().max(mid.abs()).max(treble.abs());
    t[1..5].copy_from_slice(&(-highest).to_le_bytes());
    t[6..10].copy_from_slice(&mid.to_le_bytes());
    t[19..23].copy_from_slice(&treble.to_le_bytes());
    t[32..36].copy_from_slice(&bass.to_le_bytes());
    t.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_vectors() {
        let f = [0x55, 0x60, 0x01, 0x0f, 0xf0, 0x03, 0x00, 0xcb, 0x01, 0x07, 0x00];
        assert_eq!(crc16(&f), 0xafc5);
    }

    #[test]
    fn parse_notification_and_ack_in_one_read() {
        // ANC change notification (control 0x0320, with CRC) followed by the write ack
        let raw = [
            0x55, 0x20, 0x03, 0x03, 0xe0, 0x03, 0x00, 0x00, 0x01, 0x01, 0x00, 0xf2, 0x0b, //
            0x55, 0x60, 0x01, 0x0f, 0x70, 0x01, 0x00, 0x0e, 0x00, 0x9b, 0xe3,
        ];
        let fr = Parser::default().push(&raw);
        assert_eq!(fr.len(), 2);
        assert!(matches!(decode(&fr[0]), Some(Event::Anc(1, None))));
        assert_eq!(fr[1].cmd, 0x700F);
    }

    #[test]
    fn anc_read_reports_remembered_level() {
        let f = Frame { cmd: 0x401E, seq: 0, payload: vec![0x01, 0x07, 0x00, 0x02, 0x04, 0x00] };
        assert!(matches!(decode(&f), Some(Event::Anc(7, Some(4)))));
    }

    #[test]
    fn split_frames_are_reassembled() {
        let raw = [0x55, 0x60, 0x01, 0x07, 0x40, 0x03, 0x00, 0x01, 0x01, 0x06, 0x4b, 0x0b, 0xbd];
        let mut p = Parser::default();
        assert!(p.push(&raw[..5]).is_empty());
        assert_eq!(p.push(&raw[5..]).len(), 1);
    }

    #[test]
    fn custom_eq_roundtrip() {
        let mut payload = custom_eq_payload(3.0, -1.5, 2.0);
        payload.resize(109, 0); // the device answers with a 109-byte block
        let f = Frame { cmd: 0x4044, seq: 0, payload };
        match decode(&f) {
            Some(Event::EqBands(b)) => {
                // template order: mid (peak), treble (high shelf), bass (low shelf)
                assert_eq!(b.len(), 3);
                assert_eq!((b[0].kind, b[0].gain), (1, -1.5));
                assert_eq!((b[1].kind, b[1].gain), (2, 2.0));
                assert_eq!((b[2].kind, b[2].gain), (0, 3.0));
            }
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn advanced_eq_roundtrip() {
        let bands = vec![
            Band { kind: 0, gain: 3.0, freq: 100.0, q: 0.7 },
            Band { kind: 1, gain: -2.0, freq: 400.0, q: 1.0 },
            Band { kind: 2, gain: 2.0, freq: 8000.0, q: 0.7 },
        ];
        let p = advanced_eq_payload(&bands);
        assert_eq!(p.len(), 109);
        assert_eq!(f32::from_le_bytes([p[1], p[2], p[3], p[4]]), -3.0);
        assert_eq!(parse_bands(&p), bands);
    }

    #[test]
    fn dual_device_list() {
        // captured from Headphone (1): 2 devices, first connected
        let mut p = vec![0x01, 0x00, 0x02, 0x11, 1, 2, 3, 4, 5, 6, 3];
        p.extend_from_slice(b"Mac");
        p.extend_from_slice(&[0x00, 6, 5, 4, 3, 2, 1, 5]);
        p.extend_from_slice(b"Phone");
        let d = parse_dual_devices(&p);
        assert_eq!(d.len(), 2);
        assert_eq!((d[0].name.as_str(), d[0].connected), ("Mac", true));
        assert_eq!((d[1].name.as_str(), d[1].connected), ("Phone", false));
    }

    #[test]
    fn parse_battery() {
        let raw = [0x55, 0x60, 0x01, 0x07, 0x40, 0x03, 0x00, 0x01, 0x01, 0x06, 0x4b, 0x0b, 0xbd];
        let mut p = Parser::default();
        let fr = p.push(&raw);
        assert_eq!(fr.len(), 1);
        match decode(&fr[0]) {
            Some(Event::Battery(b)) => assert_eq!(b[0].level, 75),
            e => panic!("{e:?}"),
        }
    }
}
