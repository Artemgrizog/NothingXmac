//! Device catalog: every Nothing / CMF audio product and what it can do.
//!
//! Capabilities follow the Nothing control protocol as verified on a Headphone (1)
//! and as documented by the open-source community for the other models.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Over = 0,
    Stem = 1,
    Bud = 2,
    Neck = 3,
    Open = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spatial {
    None,
    /// on / off
    Switch,
    /// off / fixed / head tracking
    Modes,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ring {
    None,
    /// payload [state]
    Legacy,
    /// payload [0x06, state]
    Single,
    /// payload [0x02 left | 0x03 right, state]
    Buds,
}

/// How the controls are operated (wording of gesture names).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Touch {
    Pinch,
    Tap,
    Press,
}

pub type Presets = &'static [(u8, &'static str)];

pub struct Nc {
    pub off: u8,
    pub transparency: Option<u8>,
    /// Noise-cancellation bytes with their level name ("" = single level).
    pub levels: &'static [(u8, &'static str)],
}

/// A configurable gesture: (device, button, gesture type) and the allowed actions.
pub struct Slot {
    pub device: u8,
    pub button: u8,
    pub kind: u8,
    pub actions: &'static [u8],
}

pub struct Caps {
    #[allow(dead_code)]
    pub single_battery: bool,
    pub presets: Presets,
    /// CMF models select presets through the "listening mode" commands.
    pub listen_mode: bool,
    /// Preset id that selects the custom 3-band EQ (None = no custom EQ).
    pub custom_id: Option<u8>,
    /// Up to 8-band parametric EQ ("Advanced").
    pub advanced_eq: bool,
    pub nc: Option<&'static Nc>,
    /// Number of bass-enhance levels (0 = no bass enhance).
    pub bass: u8,
    pub spatial: Spatial,
    pub in_ear: bool,
    pub latency: bool,
    pub ring: Ring,
    pub fit_test: bool,
    pub dual: bool,
    pub personal_anc: bool,
    pub touch: Touch,
    pub gestures: &'static [Slot],
}

pub struct Model {
    pub code: &'static str,
    /// Other model ids reported by devices of the same product.
    pub alt: &'static [&'static str],
    pub name: &'static str,
    pub pat: &'static str,
    pub exact: bool,
    pub kind: Kind,
    pub dark: bool,
    pub accent: Option<[u8; 3]>,
    pub caps: Caps,
    /// Tested on real hardware (documentation only; the UI treats all models alike).
    #[allow(dead_code)]
    pub verified: bool,
}

/// How a model enters Bluetooth pairing mode (selects the instructions in the guide).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pairing {
    /// headphones: hold the Bluetooth button
    Button = 0,
    /// earbuds: hold the button on the open case
    Case = 1,
    /// earbuds without a case button: touch and hold both earbuds
    Touch = 2,
    /// neckband: hold the power button
    Power = 3,
}

impl Model {
    pub fn is_cmf(&self) -> bool {
        self.name.starts_with("CMF")
    }

    /// Product name with the brand, e.g. "Nothing Ear (3)" or "CMF Buds Pro 2".
    pub fn full_name(&self) -> String {
        if self.is_cmf() || self.name.starts_with("Nothing") {
            self.name.to_string()
        } else {
            format!("Nothing {}", self.name)
        }
    }

    pub fn pairing(&self) -> Pairing {
        match self.kind {
            Kind::Over => Pairing::Button,
            Kind::Neck => Pairing::Power,
            // CMF Buds Neo: wear both earbuds, then touch and hold both for 5 s
            _ if self.code == "B193" => Pairing::Touch,
            _ => Pairing::Case,
        }
    }
}

// ---------------------------------------------------------------- tables

pub const PRESET_CUSTOM: u8 = 5;
pub const PRESET_CUSTOM_CMF: u8 = 6;

const P_NOTHING: Presets = &[(0, "Balanced"), (3, "More bass"), (2, "More treble"), (1, "Voice")];
const P_CMF: Presets = &[(0, "Dirac Opteo"), (1, "Rock"), (2, "Electronic"), (3, "Pop"), (4, "Enhance vocals"), (5, "Classical")];
const P_CMF_2A: Presets = &[(7, "Dirac Opteo"), (1, "Rock"), (2, "Electronic"), (3, "Pop"), (4, "Enhance vocals"), (5, "Classical")];
const P_CMF_2PLUS: Presets = &[(1, "Rock"), (2, "Electronic"), (3, "Pop"), (4, "Enhance vocals"), (5, "Classical")];

const NC_FULL: Nc = Nc { off: 5, transparency: Some(7), levels: &[(1, "High"), (2, "Mid"), (3, "Low"), (4, "Adaptive")] };
const NC_3: Nc = Nc { off: 5, transparency: Some(7), levels: &[(1, "High"), (2, "Mid"), (3, "Low")] };
const NC_EAR1: Nc = Nc { off: 5, transparency: Some(7), levels: &[(1, "High"), (3, "Low")] };
const NC_SINGLE: Nc = Nc { off: 5, transparency: Some(7), levels: &[(4, "")] };

// gesture types
pub const G_SINGLE: u8 = 0x01;
pub const G_DOUBLE: u8 = 0x02;
pub const G_TRIPLE: u8 = 0x03;
pub const G_HOLD: u8 = 0x07;
pub const G_DOUBLE_HOLD: u8 = 0x09;
pub const G_ROTATE: u8 = 0x0A;
// devices
pub const D_LEFT: u8 = 0x02;
pub const D_RIGHT: u8 = 0x03;
pub const D_CASE: u8 = 0x04;
pub const D_SINGLE: u8 = 0x06;
// headphone buttons
pub const B_BUTTON: u8 = 0x0A;
pub const B_ROLLER: u8 = 0x01;
pub const B_SLIDER: u8 = 0x05;

/// Action codes understood by the devices.
pub const ACTIONS: &[(u8, &str)] = &[
    (0x01, "No action"),
    (0x02, "Play / pause"),
    (0x08, "Previous track"),
    (0x09, "Next track"),
    (0x16, "Noise cancellation / transparency"),
    (0x14, "Noise cancellation / off"),
    (0x15, "Transparency / off"),
    (0x0A, "Noise control: all modes"),
    (0x0B, "Voice assistant"),
    (0x11, "Game mode"),
    (0x12, "Volume up"),
    (0x13, "Volume down"),
    (0x17, "Volume"),
    (0x18, "Take a photo"),
    (0x1B, "Spatial audio"),
    (0x1D, "Microphone on / off"),
    (0x1F, "News"),
    (0x20, "Channel Hop"),
    (0x21, "Essential Space"),
    (0x22, "Switch EQ preset"),
    (0x23, "Ultra Bass"),
    (0x24, "Treble boost"),
    (0x25, "Recording"),
];

pub fn action_name(code: u8) -> &'static str {
    ACTIONS.iter().find(|(c, _)| *c == code).map(|(_, n)| *n).unwrap_or("Unknown")
}

macro_rules! buds {
    ($( $kind:expr => [$($a:expr),*] ),* $(,)?) => {
        &[
            $( Slot { device: D_LEFT, button: 0x01, kind: $kind, actions: &[$($a),*] }, )*
            $( Slot { device: D_RIGHT, button: 0x01, kind: $kind, actions: &[$($a),*] }, )*
        ]
    };
}

const G_BUDS_PRO: &[Slot] = buds![
    G_DOUBLE => [0x02, 0x08, 0x09, 0x0B],
    G_TRIPLE => [0x08, 0x09, 0x0B],
    G_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x0B],
    G_DOUBLE_HOLD => [0x12, 0x13, 0x0B, 0x01],
];
const G_CMF: &[Slot] = buds![
    G_DOUBLE => [0x02, 0x08, 0x09, 0x0B, 0x01],
    G_TRIPLE => [0x08, 0x09, 0x0B, 0x01],
    G_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x0B, 0x01],
    G_DOUBLE_HOLD => [0x12, 0x13, 0x0B, 0x01],
];
const G_CMF_NEWS: &[Slot] = buds![
    G_DOUBLE => [0x02, 0x08, 0x09, 0x0B, 0x1F, 0x01],
    G_TRIPLE => [0x08, 0x09, 0x0B, 0x1F, 0x01],
    G_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x0B, 0x1F, 0x01],
    G_DOUBLE_HOLD => [0x12, 0x13, 0x0B, 0x1F, 0x01],
];
const G_CMF_PRO2: &[Slot] = &[
    Slot { device: D_LEFT, button: 0x01, kind: G_DOUBLE, actions: &[0x02, 0x08, 0x09, 0x0B, 0x01] },
    Slot { device: D_LEFT, button: 0x01, kind: G_TRIPLE, actions: &[0x08, 0x09, 0x0B, 0x01] },
    Slot { device: D_LEFT, button: 0x01, kind: G_HOLD, actions: &[0x16, 0x14, 0x15, 0x0A, 0x0B, 0x01] },
    Slot { device: D_LEFT, button: 0x01, kind: G_DOUBLE_HOLD, actions: &[0x12, 0x13, 0x0B, 0x01] },
    Slot { device: D_RIGHT, button: 0x01, kind: G_DOUBLE, actions: &[0x02, 0x08, 0x09, 0x0B, 0x01] },
    Slot { device: D_RIGHT, button: 0x01, kind: G_TRIPLE, actions: &[0x08, 0x09, 0x0B, 0x01] },
    Slot { device: D_RIGHT, button: 0x01, kind: G_HOLD, actions: &[0x16, 0x14, 0x15, 0x0A, 0x0B, 0x01] },
    Slot { device: D_RIGHT, button: 0x01, kind: G_DOUBLE_HOLD, actions: &[0x12, 0x13, 0x0B, 0x01] },
    Slot { device: D_CASE, button: 0x01, kind: G_SINGLE, actions: &[0x02, 0x08, 0x09, 0x0B, 0x11, 0x01] },
    Slot { device: D_CASE, button: 0x01, kind: G_DOUBLE, actions: &[0x02, 0x08, 0x09, 0x0B, 0x11, 0x01] },
    Slot { device: D_CASE, button: 0x01, kind: G_TRIPLE, actions: &[0x02, 0x08, 0x09, 0x0B, 0x11, 0x01] },
    Slot { device: D_CASE, button: 0x01, kind: G_HOLD, actions: &[0x16, 0x14, 0x15, 0x0A, 0x0B, 0x11, 0x01] },
    Slot { device: D_CASE, button: 0x01, kind: G_DOUBLE_HOLD, actions: &[0x12, 0x13, 0x0B, 0x11, 0x01] },
    Slot { device: D_CASE, button: 0x01, kind: G_ROTATE, actions: &[0x17, 0x01] },
];
const G_EAR_PINCH: &[Slot] = buds![
    G_DOUBLE => [0x08, 0x09, 0x0B],
    G_TRIPLE => [0x08, 0x09, 0x0B],
    G_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x12, 0x13, 0x0B],
    G_DOUBLE_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x12, 0x13, 0x0B, 0x01],
];
const G_EAR1: &[Slot] = buds![
    G_TRIPLE => [0x08, 0x09, 0x0B, 0x01],
    G_HOLD => [0x0A, 0x01],
];
const G_EAR3: &[Slot] = buds![
    G_DOUBLE => [0x09, 0x08, 0x0B],
    G_TRIPLE => [0x09, 0x08, 0x0B, 0x01],
    G_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x0B, 0x12, 0x13, 0x1D, 0x01],
    G_DOUBLE_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x0B, 0x12, 0x13, 0x01],
];
const G_EAR3A: &[Slot] = buds![
    G_DOUBLE => [0x09, 0x08, 0x0B],
    G_TRIPLE => [0x09, 0x08, 0x0B, 0x01],
    G_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x12, 0x13, 0x25, 0x0B, 0x1D, 0x01],
    G_DOUBLE_HOLD => [0x16, 0x14, 0x15, 0x0A, 0x12, 0x13, 0x25, 0x0B, 0x01],
];
const G_EAR_OPEN: &[Slot] = buds![
    G_DOUBLE => [0x08, 0x09, 0x0B],
    G_TRIPLE => [0x08, 0x09, 0x0B],
    G_HOLD => [0x12, 0x13, 0x0B],
    G_DOUBLE_HOLD => [0x12, 0x13, 0x0B, 0x01],
];
const G_NECKBAND: &[Slot] = &[
    Slot { device: D_SINGLE, button: 0x01, kind: G_DOUBLE, actions: &[0x08, 0x09, 0x0B] },
    Slot { device: D_SINGLE, button: 0x01, kind: G_TRIPLE, actions: &[0x08, 0x09, 0x0B] },
    Slot { device: D_SINGLE, button: 0x01, kind: G_HOLD, actions: &[0x16, 0x14, 0x15, 0x0A] },
];
const G_HEADPHONE_1: &[Slot] = &[
    Slot { device: D_SINGLE, button: B_BUTTON, kind: G_SINGLE, actions: &[0x20, 0x0B, 0x1F, 0x16, 0x14, 0x15, 0x0A, 0x1B, 0x1D, 0x22, 0x01] },
    Slot { device: D_SINGLE, button: B_BUTTON, kind: G_HOLD, actions: &[0x20, 0x0B, 0x1F, 0x16, 0x14, 0x15, 0x0A, 0x1B, 0x1D, 0x22, 0x01] },
    Slot { device: D_SINGLE, button: B_ROLLER, kind: G_HOLD, actions: &[0x16, 0x14, 0x15, 0x0A, 0x01] },
];
const G_HEADPHONE_A: &[Slot] = &[
    Slot { device: D_SINGLE, button: B_BUTTON, kind: G_SINGLE, actions: &[0x20, 0x0B, 0x1F, 0x16, 0x14, 0x15, 0x0A, 0x18, 0x1D, 0x22, 0x01] },
    Slot { device: D_SINGLE, button: B_BUTTON, kind: G_HOLD, actions: &[0x20, 0x0B, 0x1F, 0x16, 0x14, 0x15, 0x0A, 0x1D, 0x22, 0x01] },
    Slot { device: D_SINGLE, button: B_ROLLER, kind: G_HOLD, actions: &[0x16, 0x14, 0x15, 0x0A, 0x01] },
];
const G_CMF_HEADPHONE: &[Slot] = &[
    Slot { device: D_SINGLE, button: B_BUTTON, kind: G_SINGLE, actions: &[0x20, 0x0B, 0x1F, 0x1B, 0x1D, 0x22, 0x16, 0x14, 0x15, 0x0A, 0x01] },
    Slot { device: D_SINGLE, button: B_BUTTON, kind: G_HOLD, actions: &[0x20, 0x0B, 0x1F, 0x1B, 0x1D, 0x22, 0x16, 0x14, 0x15, 0x0A, 0x01] },
    Slot { device: D_SINGLE, button: B_ROLLER, kind: G_HOLD, actions: &[0x16, 0x14, 0x15, 0x0A, 0x01] },
    Slot { device: D_SINGLE, button: B_SLIDER, kind: G_SINGLE, actions: &[0x23, 0x24] },
];

/// Capabilities shared by most Nothing earbuds; individual models override fields.
const EAR_BASE: Caps = Caps {
    single_battery: false,
    presets: P_NOTHING,
    listen_mode: false,
    custom_id: Some(PRESET_CUSTOM),
    advanced_eq: false,
    nc: Some(&NC_FULL),
    bass: 0,
    spatial: Spatial::None,
    in_ear: true,
    latency: true,
    ring: Ring::Buds,
    fit_test: false,
    dual: true,
    personal_anc: false,
    touch: Touch::Pinch,
    gestures: G_EAR_PINCH,
};

const CMF_BASE: Caps = Caps {
    single_battery: false,
    presets: P_CMF,
    listen_mode: true,
    custom_id: Some(PRESET_CUSTOM_CMF),
    advanced_eq: false,
    nc: Some(&NC_FULL),
    bass: 5,
    spatial: Spatial::None,
    in_ear: true,
    latency: true,
    ring: Ring::Buds,
    fit_test: false,
    dual: true,
    personal_anc: false,
    touch: Touch::Tap,
    gestures: G_CMF,
};

const HEADPHONE_BASE: Caps = Caps {
    single_battery: true,
    presets: P_NOTHING,
    listen_mode: false,
    custom_id: Some(PRESET_CUSTOM),
    advanced_eq: true,
    nc: Some(&NC_FULL),
    bass: 5,
    spatial: Spatial::Modes,
    in_ear: true,
    latency: true,
    ring: Ring::Single,
    fit_test: false,
    dual: true,
    personal_anc: false,
    touch: Touch::Press,
    gestures: G_HEADPHONE_1,
};

const YELLOW: Option<[u8; 3]> = Some([250, 214, 40]);
const ORANGE: Option<[u8; 3]> = Some([255, 98, 40]);

macro_rules! m {
    ($code:expr, [$($alt:expr),*], $name:expr, $pat:expr, $exact:expr, $kind:expr, $dark:expr, $acc:expr, $ver:expr, $caps:expr) => {
        Model { code: $code, alt: &[$($alt),*], name: $name, pat: $pat, exact: $exact, kind: $kind, dark: $dark, accent: $acc, verified: $ver, caps: $caps }
    };
}

pub static MODELS: &[Model] = &[
    // ---- Nothing headphones
    m!("B170", [], "Headphone (1)", "headphone (1)", false, Kind::Over, false, None, true, HEADPHONE_BASE),
    m!("B17P", [], "Headphone (1) Pro", "headphone (1) pro", false, Kind::Over, true, None, false, HEADPHONE_BASE),
    m!("B186", ["B198"], "Headphone (a)", "headphone (a)", false, Kind::Over, false, YELLOW, false,
        Caps { bass: 3, in_ear: false, gestures: G_HEADPHONE_A, ..HEADPHONE_BASE }),
    // ---- Nothing earbuds
    m!("B173", [], "Ear (3)", "ear (3)", false, Kind::Stem, false, None, false,
        Caps { advanced_eq: true, spatial: Spatial::Switch, fit_test: true, personal_anc: true, touch: Touch::Press, gestures: G_EAR3, ..EAR_BASE }),
    m!("B190", [], "Ear (3a)", "ear (3a)", false, Kind::Stem, false, YELLOW, false,
        Caps { advanced_eq: true, spatial: Spatial::Switch, fit_test: true, gestures: G_EAR3A, ..EAR_BASE }),
    m!("B171", [], "Ear", "ear", true, Kind::Stem, false, None, false,
        Caps { advanced_eq: true, bass: 5, fit_test: true, personal_anc: true, ..EAR_BASE }),
    m!("B155", [], "Ear (2)", "ear (2)", false, Kind::Stem, false, None, false,
        Caps { advanced_eq: true, fit_test: true, personal_anc: true, ..EAR_BASE }),
    m!("B183", ["B162"], "Ear (a)", "ear (a)", false, Kind::Stem, false, YELLOW, false,
        Caps { bass: 5, fit_test: true, ..EAR_BASE }),
    m!("B181", [], "Ear (1)", "ear (1)", false, Kind::Stem, false, None, false,
        Caps { custom_id: None, nc: Some(&NC_EAR1), in_ear: false, ring: Ring::Legacy, dual: false, touch: Touch::Tap, gestures: G_EAR1, ..EAR_BASE }),
    m!("B157", [], "Ear (stick)", "ear (stick)", false, Kind::Stem, false, None, false,
        Caps { advanced_eq: true, nc: None, dual: false, ..EAR_BASE }),
    m!("B174", [], "Ear (open)", "ear (open)", false, Kind::Open, false, None, false,
        Caps { nc: None, in_ear: false, ring: Ring::None, dual: false, gestures: G_EAR_OPEN, ..EAR_BASE }),
    // ---- CMF
    m!("B175", [], "CMF Headphone Pro", "headphone pro", false, Kind::Over, true, ORANGE, false,
        Caps { advanced_eq: false, spatial: Spatial::None, gestures: G_CMF_HEADPHONE, ..HEADPHONE_BASE }),
    m!("B172", ["B187"], "CMF Buds Pro 2", "buds pro 2", false, Kind::Bud, true, ORANGE, false,
        Caps { spatial: Spatial::Switch, fit_test: true, gestures: G_CMF_PRO2, ..CMF_BASE }),
    m!("B163", [], "CMF Buds Pro", "buds pro", false, Kind::Bud, true, ORANGE, false,
        Caps { presets: P_NOTHING, listen_mode: false, custom_id: Some(PRESET_CUSTOM), nc: Some(&NC_3), bass: 0, dual: false, gestures: G_BUDS_PRO, ..CMF_BASE }),
    m!("B184", [], "CMF Buds 2 Plus", "buds 2 plus", false, Kind::Bud, false, ORANGE, false,
        Caps { presets: P_CMF_2PLUS, spatial: Spatial::Switch, fit_test: true, ..CMF_BASE }),
    m!("B185", [], "CMF Buds 2a", "buds 2a", false, Kind::Bud, false, ORANGE, false,
        Caps { presets: P_CMF_2A, nc: Some(&NC_SINGLE), in_ear: false, gestures: G_CMF_NEWS, ..CMF_BASE }),
    m!("B179", [], "CMF Buds 2", "buds 2", false, Kind::Bud, false, ORANGE, false,
        Caps { spatial: Spatial::Switch, in_ear: false, fit_test: true, gestures: G_CMF_NEWS, ..CMF_BASE }),
    m!("B193", [], "CMF Buds Neo", "buds neo", false, Kind::Bud, false, ORANGE, false,
        Caps { spatial: Spatial::Switch, in_ear: false, ..CMF_BASE }),
    m!("B168", [], "CMF Buds", "cmf buds", true, Kind::Bud, false, ORANGE, false,
        Caps { nc: Some(&NC_SINGLE), ..CMF_BASE }),
    m!("B164", [], "CMF Neckband Pro", "neckband pro", false, Kind::Neck, true, ORANGE, false,
        Caps {
            single_battery: true,
            presets: P_NOTHING,
            listen_mode: false,
            custom_id: Some(PRESET_CUSTOM),
            spatial: Spatial::Switch,
            in_ear: false,
            ring: Ring::Single,
            gestures: G_NECKBAND,
            ..CMF_BASE
        }),
];

pub static GENERIC: Model = Model {
    code: "NTHG",
    alt: &[],
    name: "Nothing device",
    pat: "",
    exact: false,
    kind: Kind::Stem,
    dark: false,
    accent: None,
    verified: false,
    caps: Caps { gestures: &[], advanced_eq: false, ..EAR_BASE },
};

/// Model from the id the device reports (e.g. 0xB170 -> "B170").
pub fn by_model_id(id: u16) -> Option<&'static Model> {
    let code = format!("{id:X}");
    MODELS.iter().find(|m| m.code == code || m.alt.contains(&code.as_str()))
}

/// Identify a model from its Bluetooth name.
pub fn identify(bt_name: &str) -> Option<&'static Model> {
    let lower = bt_name.trim().to_lowercase();
    let stripped = lower.strip_prefix("nothing ").unwrap_or(&lower).trim().to_string();
    if !(lower.starts_with("nothing") || lower.starts_with("cmf") || lower.starts_with("ear") || lower.starts_with("headphone") || lower.starts_with("buds")) {
        return None;
    }
    let mut best: Option<&Model> = None;
    for m in MODELS {
        let hit = if m.exact { stripped == m.pat } else { stripped.contains(m.pat) };
        if hit && best.map_or(true, |b| m.pat.len() > b.pat.len()) {
            best = Some(m);
        }
    }
    best
}

/// Like `identify`, but falls back to a generic profile for unknown Nothing / CMF audio products.
pub fn identify_or_generic(bt_name: &str) -> Option<&'static Model> {
    if let Some(m) = identify(bt_name) {
        return Some(m);
    }
    let l = bt_name.to_lowercase();
    if (l.starts_with("nothing") || l.starts_with("cmf")) && (l.contains("ear") || l.contains("buds") || l.contains("headphone") || l.contains("neckband")) {
        return Some(&GENERIC);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_resolve_to_the_right_model() {
        let cases = [
            ("Nothing Headphone (1)", "B170"),
            ("Nothing Headphone (1) Pro", "B17P"),
            ("Nothing ear (1)", "B181"),
            ("Nothing Ear (2)", "B155"),
            ("Nothing Ear (3a)", "B190"),
            ("Nothing Ear", "B171"),
            ("CMF Buds", "B168"),
            ("CMF Buds 2", "B179"),
            ("CMF Buds 2a", "B185"),
            ("CMF Buds 2 Plus", "B184"),
            ("CMF Buds Pro", "B163"),
            ("CMF Buds Pro 2", "B172"),
            ("CMF Headphone Pro", "B175"),
            ("CMF Neckband Pro", "B164"),
        ];
        for (name, code) in cases {
            assert_eq!(identify(name).map(|m| m.code), Some(code), "{name}");
        }
        assert!(identify("AirPods Pro").is_none());
        assert_eq!(by_model_id(0xB170).map(|m| m.code), Some("B170"));
        assert_eq!(by_model_id(0xB187).map(|m| m.code), Some("B172"));
        assert_eq!(identify("CMF Buds Neo").map(|m| m.code), Some("B193"));
    }

    #[test]
    fn product_names_and_pairing() {
        assert_eq!(by_model_id(0xB170).unwrap().full_name(), "Nothing Headphone (1)");
        assert_eq!(by_model_id(0xB172).unwrap().full_name(), "CMF Buds Pro 2");
        assert_eq!(GENERIC.full_name(), "Nothing device");
        assert_eq!(by_model_id(0xB170).unwrap().pairing(), Pairing::Button);
        assert_eq!(by_model_id(0xB193).unwrap().pairing(), Pairing::Touch);
        assert_eq!(by_model_id(0xB164).unwrap().pairing(), Pairing::Power);
        assert_eq!(by_model_id(0xB155).unwrap().pairing(), Pairing::Case);
    }
}
