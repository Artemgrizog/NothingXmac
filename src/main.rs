#[macro_use]
mod log;
mod art;
mod bt;
mod catalog;
mod dotmatrix;
mod eq;
mod i18n;
mod login;
mod proto;
mod settings;
mod tray;

use catalog::{Model, Ring, Spatial, Touch};
use i18n::tr;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use proto::{cmd, Band, BattId, Event};
use settings::Settings;
use slint::{ComponentHandle, Model as _, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

slint::include_modules!();

/// viewbox heights of the EQ graphs (must match main.slint)
const EQ_H: f32 = 170.0;
const ADV_H: f32 = 230.0;
const ADV_RANGE: f64 = 12.0;
const PANEL_W: f64 = 392.0;

struct Dev {
    info: bt::Paired,
    model: &'static Model,
}

#[derive(PartialEq)]
enum LinkSt {
    None,
    Opening(String, Instant),
    Open(String),
}

/// List models shared by the main window and the menu bar panel; updated in place.
struct Models {
    devices: Rc<VecModel<DeviceItem>>,
    batteries: Rc<VecModel<BattItem>>,
    nc_levels: Rc<VecModel<Preset>>,
    presets_a: Rc<VecModel<Preset>>,
    presets_b: Rc<VecModel<Preset>>,
    gestures: Rc<VecModel<GestureSlot>>,
    actions: Rc<VecModel<ActionItem>>,
    dual: Rc<VecModel<DualItem>>,
    adv: Rc<VecModel<EqBand>>,
    info: Rc<VecModel<InfoRow>>,
}

fn sync<T: Clone + PartialEq + 'static>(m: &VecModel<T>, data: Vec<T>) {
    if m.row_count() != data.len() {
        m.set_vec(data);
        return;
    }
    for (i, d) in data.into_iter().enumerate() {
        if m.row_data(i).as_ref() != Some(&d) {
            m.set_row_data(i, d);
        }
    }
}

struct State {
    ui: slint::Weak<MainWindow>,
    panel: slint::Weak<TrayPanel>,
    m: Models,
    settings: Settings,
    tray: Option<tray::Tray>,
    paired: Vec<Dev>,
    bt_denied: bool,
    selected: Option<String>,
    bt: bt::Worker,
    link: LinkSt,
    connecting: Option<(String, Instant)>,
    scan_inflight: bool,
    next_open: Instant,
    parser: proto::Parser,
    seq: u8,
    error: String,
    last_scan: Instant,
    last_batt: Instant,
    pending: Vec<(Instant, u16, Vec<u8>)>,
    dirty: bool,
    img_cache: HashMap<(&'static str, u32), slint::Image>,
    model_override: HashMap<String, &'static Model>,
    hero_for: String,
    // ---- device state
    batt: Vec<proto::Batt>,
    anc: u8,
    nc_level: u8,
    eq: Option<u8>,
    bass_on: bool,
    bass_level: u8,
    latency: bool,
    spatial: u8,
    firmware: String,
    serial: String,
    dual: Option<bool>,
    dual_devices: Vec<proto::DualDevice>,
    gestures: Vec<(u8, u8, u8, u8)>,
    gesture_open: Option<usize>,
    in_ear: Option<bool>,
    panc: Option<bool>,
    fit: Option<(u8, u8)>,
    fit_until: Option<Instant>,
    ringing: Option<(u8, Instant)>,
    adv_on: Option<bool>,
    /// bands stored in the device (custom / advanced EQ block)
    bands: Vec<Band>,
    /// advanced EQ being edited
    adv: Vec<Band>,
    adv_sel: usize,
    custom: [f32; 3],
    pushed_custom: Option<[f32; 3]>,
    /// logical width of the EQ graphs
    graph_w: f32,
    last_page: i32,
}

fn set_policy(regular: bool) {
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(if regular { NSApplicationActivationPolicy::Regular } else { NSApplicationActivationPolicy::Accessory });
        if regular {
            #[allow(deprecated)]
            app.activateIgnoringOtherApps(true);
        }
    }
}

fn activate() {
    if let Some(mtm) = MainThreadMarker::new() {
        #[allow(deprecated)]
        NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
    }
}

fn show_window(ui: &MainWindow) {
    set_policy(true);
    // show() on an already visible window makes Slint stop repainting it
    if !ui.window().is_visible() {
        let _ = ui.show();
    }
    use slint::winit_030::WinitWindowAccessor;
    ui.window().with_winit_window(|w| w.focus_window());
    ui.window().request_redraw();
}

fn default_advanced() -> Vec<Band> {
    vec![
        Band { kind: 0, gain: 0.0, freq: 60.0, q: 0.7 },
        Band { kind: 1, gain: 0.0, freq: 250.0, q: 1.0 },
        Band { kind: 1, gain: 0.0, freq: 1000.0, q: 1.0 },
        Band { kind: 1, gain: 0.0, freq: 4000.0, q: 1.0 },
        Band { kind: 2, gain: 0.0, freq: 12000.0, q: 0.7 },
    ]
}

fn freq_text(f: f32) -> String {
    if f >= 1000.0 {
        format!("{:.1} kHz", f / 1000.0).replace(".0 ", " ")
    } else {
        format!("{} Hz", f.round() as i32)
    }
}

fn group_name(device: u8, button: u8, kind: catalog::Kind) -> &'static str {
    match device {
        catalog::D_LEFT => tr("Left earbud"),
        catalog::D_RIGHT => tr("Right earbud"),
        catalog::D_CASE => tr("Case knob"),
        _ if kind == catalog::Kind::Over => match button {
            catalog::B_ROLLER => tr("Roller"),
            catalog::B_SLIDER => tr("Slider"),
            _ => tr("Button"),
        },
        _ => tr("Controls"),
    }
}

fn gesture_name(touch: Touch, device: u8, button: u8, gesture: u8) -> &'static str {
    if device == catalog::D_SINGLE && button == catalog::B_SLIDER {
        return tr("Slide");
    }
    if gesture == catalog::G_ROTATE {
        return tr("Rotate");
    }
    let touch = if device == catalog::D_CASE || device == catalog::D_SINGLE { Touch::Press } else { touch };
    tr(match (touch, gesture) {
        (_, catalog::G_SINGLE) => "Press",
        (Touch::Pinch, catalog::G_DOUBLE) => "Double pinch",
        (Touch::Pinch, catalog::G_TRIPLE) => "Triple pinch",
        (Touch::Pinch, catalog::G_HOLD) => "Pinch & hold",
        (Touch::Pinch, catalog::G_DOUBLE_HOLD) => "Double pinch & hold",
        (Touch::Tap, catalog::G_DOUBLE) => "Double tap",
        (Touch::Tap, catalog::G_TRIPLE) => "Triple tap",
        (Touch::Tap, catalog::G_HOLD) => "Tap & hold",
        (Touch::Tap, catalog::G_DOUBLE_HOLD) => "Double tap & hold",
        (_, catalog::G_DOUBLE) => "Double press",
        (_, catalog::G_TRIPLE) => "Triple press",
        (_, catalog::G_HOLD) => "Press & hold",
        (_, catalog::G_DOUBLE_HOLD) => "Double press & hold",
        _ => "Press",
    })
}

fn action_label(m: &Model, code: u8) -> &'static str {
    // Ear (1) only knows "switch noise control" on 0x0A
    if code == 0x0A && m.code == "B181" {
        return tr("Noise cancellation / transparency");
    }
    tr(catalog::action_name(code))
}

impl State {
    fn new(ui: slint::Weak<MainWindow>, panel: slint::Weak<TrayPanel>, m: Models, settings: Settings) -> Self {
        let now = Instant::now();
        State {
            ui,
            panel,
            m,
            tray: None,
            paired: vec![],
            bt_denied: false,
            selected: if settings.last_device.is_empty() { None } else { Some(settings.last_device.clone()) },
            bt: bt::Worker::spawn(),
            link: LinkSt::None,
            connecting: None,
            scan_inflight: false,
            next_open: now,
            parser: proto::Parser::default(),
            seq: 0,
            error: String::new(),
            last_scan: now - Duration::from_secs(10),
            last_batt: now,
            pending: vec![],
            dirty: true,
            img_cache: HashMap::new(),
            model_override: HashMap::new(),
            hero_for: String::new(),
            batt: vec![],
            anc: 0,
            nc_level: 1,
            eq: None,
            bass_on: false,
            bass_level: 1,
            latency: false,
            spatial: 0,
            firmware: String::new(),
            serial: String::new(),
            dual: None,
            dual_devices: vec![],
            gestures: vec![],
            gesture_open: None,
            in_ear: None,
            panc: None,
            fit: None,
            fit_until: None,
            ringing: None,
            adv_on: None,
            bands: vec![],
            adv: vec![],
            adv_sel: 0,
            custom: [settings.eq_bass, settings.eq_mid, settings.eq_treble],
            pushed_custom: None,
            graph_w: 600.0,
            last_page: 0,
            settings,
        }
    }

    fn img(&mut self, m: &'static Model, size: u32) -> slint::Image {
        self.img_cache.entry((m.code, size)).or_insert_with(|| art::model_image(m, size)).clone()
    }

    fn sel_dev(&self) -> Option<&Dev> {
        let a = self.selected.as_ref()?;
        self.paired.iter().find(|d| &d.info.address == a)
    }

    fn model(&self) -> Option<&'static Model> {
        self.sel_dev().map(|d| d.model)
    }

    fn caps(&self) -> &'static catalog::Caps {
        &self.model().unwrap_or(&catalog::GENERIC).caps
    }

    fn ready(&self) -> bool {
        matches!(self.link, LinkSt::Open(_))
    }

    fn busy(&self) -> bool {
        self.connecting.is_some() || matches!(self.link, LinkSt::Opening(..))
    }

    fn reset_device_state(&mut self) {
        self.batt.clear();
        self.anc = 0;
        self.eq = None;
        self.bass_on = false;
        self.latency = false;
        self.spatial = 0;
        self.firmware.clear();
        self.serial.clear();
        self.dual = None;
        self.dual_devices.clear();
        self.gestures.clear();
        self.gesture_open = None;
        self.in_ear = None;
        self.panc = None;
        self.fit = None;
        self.fit_until = None;
        self.ringing = None;
        self.adv_on = None;
        self.bands.clear();
        self.adv.clear();
        self.pending.clear();
    }

    fn drop_link(&mut self) {
        if self.link != LinkSt::None {
            self.bt.cmd(bt::Cmd::Close);
        }
        self.link = LinkSt::None;
        self.parser.clear();
        self.reset_device_state();
        self.dirty = true;
    }

    fn send(&mut self, c: u16, payload: &[u8]) {
        if !self.ready() {
            return;
        }
        self.seq = if self.seq >= 250 { 1 } else { self.seq + 1 };
        let frame = proto::encode(c, payload, self.seq);
        log!("tx {}", log::hex(&frame));
        self.bt.cmd(bt::Cmd::Send(frame));
    }

    fn queue(&mut self, delay_ms: u64, c: u16, payload: &[u8]) {
        self.pending.retain(|(_, q, _)| *q != c);
        self.pending.push((Instant::now() + Duration::from_millis(delay_ms), c, payload.to_vec()));
    }

    fn eq_get_cmd(&self) -> u16 {
        if self.caps().listen_mode { cmd::LISTEN_GET } else { cmd::EQ_GET }
    }

    fn on_link_ready(&mut self) {
        let caps = self.caps();
        let mut qs: Vec<(u16, Vec<u8>)> = vec![
            (cmd::MODEL_GET, vec![]),
            (cmd::BATTERY_GET, vec![]),
            (cmd::ANC_GET, vec![]),
            (self.eq_get_cmd(), vec![]),
            (cmd::FIRMWARE_GET, vec![]),
            (cmd::SERIAL_GET, vec![]),
        ];
        if caps.latency {
            qs.push((cmd::LATENCY_GET, vec![]));
        }
        if caps.custom_id.is_some() || caps.advanced_eq {
            qs.push((cmd::CUSTOM_EQ_GET, vec![]));
        }
        if caps.advanced_eq {
            qs.push((cmd::ADVANCED_EQ_GET, vec![]));
        }
        if caps.bass > 0 {
            qs.push((cmd::BASS_GET, vec![]));
        }
        if caps.spatial != Spatial::None {
            qs.push((cmd::SPATIAL_GET, vec![]));
        }
        if !caps.gestures.is_empty() {
            qs.push((cmd::GESTURES_GET, vec![]));
        }
        if caps.in_ear {
            qs.push((cmd::IN_EAR_GET, vec![]));
        }
        if caps.personal_anc {
            qs.push((cmd::PERSONAL_ANC_GET, vec![]));
        }
        if caps.dual {
            qs.push((cmd::DUAL_GET, vec![]));
            qs.push((cmd::DUAL_DEVICES_GET, vec![0x00]));
        }
        let now = Instant::now();
        for (i, (c, p)) in qs.into_iter().enumerate() {
            // the first write right after the channel opens is sometimes dropped
            self.pending.push((now + Duration::from_millis(200 + i as u64 * 70), c, p));
        }
        self.last_batt = Instant::now();
        self.error.clear();
        self.dirty = true;
    }

    // ------------------------------------------------------------ scanning / link

    fn request_scan(&mut self) {
        self.last_scan = Instant::now();
        if !self.scan_inflight {
            self.scan_inflight = true;
            self.bt.cmd(bt::Cmd::Scan);
        }
    }

    fn apply_scan(&mut self, list: Option<Vec<bt::Paired>>) {
        self.scan_inflight = false;
        let before: Vec<(String, bool)> = self.paired.iter().map(|d| (d.info.address.clone(), d.info.connected)).collect();
        match list {
            None => {
                self.bt_denied = true;
                self.paired.clear();
            }
            Some(list) => {
                self.bt_denied = false;
                let ov = &self.model_override;
                let mut v: Vec<Dev> = list
                    .into_iter()
                    .filter_map(|p| {
                        let m = ov
                            .get(&p.address)
                            .copied()
                            .or_else(|| catalog::identify_or_generic(&p.name))
                            .or_else(|| p.has_nt.then_some(&catalog::GENERIC))?;
                        Some(Dev { info: p, model: m })
                    })
                    .collect();
                v.sort_by_key(|d| !d.info.connected);
                self.paired = v;
            }
        }
        let after: Vec<(String, bool)> = self.paired.iter().map(|d| (d.info.address.clone(), d.info.connected)).collect();
        if before != after {
            self.dirty = true;
        }
        let valid = self.selected.as_ref().map_or(false, |a| self.paired.iter().any(|d| &d.info.address == a));
        if !valid {
            let pick = self.paired.iter().find(|d| d.info.connected).or(self.paired.first()).map(|d| d.info.address.clone());
            if pick != self.selected {
                self.drop_link();
                self.selected = pick;
                self.dirty = true;
            }
        }
        self.manage_link();
    }

    fn manage_link(&mut self) {
        let (connected, addr) = match self.sel_dev() {
            Some(d) => (d.info.connected, d.info.address.clone()),
            None => (false, String::new()),
        };
        if self.ready() && !connected {
            log!("link: device disconnected");
            self.drop_link();
        }
        if connected && self.link == LinkSt::None && self.connecting.is_none() && Instant::now() >= self.next_open {
            log!("link: opening control channel to {addr}");
            self.bt.cmd(bt::Cmd::Open(addr.clone()));
            self.link = LinkSt::Opening(addr, Instant::now());
            self.parser.clear();
            self.reset_device_state();
            self.dirty = true;
        }
    }

    fn handle_bt(&mut self, ev: bt::Ev) {
        let now = Instant::now();
        let is_cur = |s: &LinkSt, a: &str| match s {
            LinkSt::Opening(x, _) | LinkSt::Open(x) => x == a,
            LinkSt::None => false,
        };
        match ev {
            bt::Ev::Paired(list) => self.apply_scan(list),
            bt::Ev::Opened(a) => {
                if matches!(&self.link, LinkSt::Opening(x, _) if *x == a) {
                    log!("link: open");
                    self.link = LinkSt::Open(a);
                    self.on_link_ready();
                }
            }
            bt::Ev::OpenFailed(a, why, retry) => {
                if is_cur(&self.link, &a) {
                    log!("link: open failed: {why}");
                    self.link = LinkSt::None;
                    if !retry {
                        self.error = tr("Could not connect").into();
                    }
                    self.next_open = now + Duration::from_secs(if retry { 2 } else { 6 });
                    self.dirty = true;
                }
            }
            bt::Ev::Busy(a) => {
                if is_cur(&self.link, &a) {
                    log!("link: control channel busy (another app/host holds it)");
                    self.link = LinkSt::None;
                    self.error = tr("Busy: close Nothing X on your phone").into();
                    self.next_open = now + Duration::from_secs(8);
                    self.dirty = true;
                }
            }
            bt::Ev::NotNothing(a) => {
                if is_cur(&self.link, &a) {
                    self.link = LinkSt::None;
                    self.error = tr("Control service not found").into();
                    self.next_open = now + Duration::from_secs(30);
                    self.dirty = true;
                }
            }
            bt::Ev::Closed(a) => {
                if is_cur(&self.link, &a) {
                    log!("link: closed by device");
                    self.link = LinkSt::None;
                    self.parser.clear();
                    self.reset_device_state();
                    self.next_open = now + Duration::from_secs(2);
                    self.dirty = true;
                }
            }
            bt::Ev::Data(rx) => {
                if self.ready() {
                    log!("rx {}", log::hex(&rx));
                    for f in self.parser.push(&rx) {
                        if let Some(ev) = proto::decode(&f) {
                            self.handle_event(ev);
                        }
                    }
                }
            }
            bt::Ev::ConnectDone(ok) => {
                self.connecting = None;
                if !ok {
                    self.error = tr("Could not connect").into();
                }
                self.next_open = now;
                self.request_scan();
                self.dirty = true;
            }
        }
    }

    fn handle_event(&mut self, e: Event) {
        match e {
            Event::Battery(b) => {
                if !b.is_empty() {
                    self.batt = b;
                }
            }
            Event::Anc(m, remembered) => {
                self.anc = m;
                let levels = self.caps().nc.map(|n| n.levels).unwrap_or(&[]);
                if levels.iter().any(|(b, _)| *b == m) {
                    self.nc_level = m;
                } else if let Some(l) = remembered.filter(|l| levels.iter().any(|(b, _)| b == l)) {
                    self.nc_level = l;
                }
            }
            Event::EqPreset(p) | Event::ListenMode(p) => self.eq = Some(p),
            Event::Bass { on, level } => {
                self.bass_on = on;
                self.bass_level = level.clamp(1, 5);
            }
            Event::Latency(l) => self.latency = l,
            Event::Spatial(s) => self.spatial = s.min(2),
            Event::Firmware(f) => self.firmware = f,
            Event::Serial(s) => self.serial = s,
            Event::Model(id) => {
                if let (Some(m), Some(addr)) = (catalog::by_model_id(id), self.selected.clone()) {
                    if self.model_override.get(&addr).map(|x| x.code) != Some(m.code) {
                        log!("model: device reports {:X} -> {}", id, m.name);
                        self.model_override.insert(addr.clone(), m);
                        if let Some(d) = self.paired.iter_mut().find(|d| d.info.address == addr) {
                            d.model = m;
                        }
                        self.hero_for.clear();
                    }
                }
            }
            Event::Dual(d) => self.dual = Some(d),
            Event::EqBands(b) => {
                // the simple custom EQ is the 3-band low shelf / peak / high shelf block
                if b.len() == 3 {
                    let g = |k: u8| b.iter().find(|x| x.kind == k).map(|x| (x.gain.clamp(-6.0, 6.0) * 10.0).round() / 10.0).unwrap_or(0.0);
                    self.custom = [g(0), g(1), g(2)];
                }
                if self.adv.is_empty() || !b.is_empty() {
                    self.adv = if b.is_empty() { default_advanced() } else { b.clone() };
                    self.adv_sel = self.adv_sel.min(self.adv.len().saturating_sub(1));
                }
                self.bands = b;
            }
            Event::Gestures(g) => self.gestures = g,
            Event::InEar(v) => self.in_ear = Some(v),
            Event::DualDevices(d) => self.dual_devices = d,
            Event::FitTest(l, r) => {
                self.fit = Some((l, r));
                self.fit_until = None;
            }
            Event::PersonalAnc(v) => self.panc = Some(v),
            Event::AdvancedEq(v) => self.adv_on = Some(v),
        }
        self.dirty = true;
    }

    fn tick(&mut self) {
        let now = Instant::now();
        for ev in self.bt.events() {
            self.handle_bt(ev);
        }
        if self.ready() {
            let mut due = vec![];
            self.pending.retain(|(t, c, p)| {
                if *t <= now {
                    due.push((*c, p.clone()));
                    false
                } else {
                    true
                }
            });
            for (c, p) in due {
                self.send(c, &p);
            }
            if now.duration_since(self.last_batt) > Duration::from_secs(30) {
                self.last_batt = now;
                self.send(cmd::BATTERY_GET, &[]);
            }
            if self.fit_until.map_or(false, |t| now > t) {
                self.fit_until = None;
                self.dirty = true;
            }
            if let Some((side, t)) = self.ringing {
                if now.duration_since(t) > Duration::from_secs(25) {
                    self.ring(side as i32, false);
                }
            }
        }
        if now.duration_since(self.last_scan) > Duration::from_millis(2000) {
            self.request_scan();
        }
        if let Some(ui) = self.ui.upgrade() {
            let win = ui.window();
            let w = (win.size().width as f32 / win.scale_factor() - 212.0).max(200.0);
            if (w - self.graph_w).abs() > 0.5 {
                self.graph_w = w;
                ui.set_graph_w(w);
                self.pushed_custom = None;
                self.dirty = true;
            }
        }
        if tray::take_click() {
            log!("ui: menu bar icon clicked");
            self.toggle_panel();
        }
        if let Some(ui) = self.ui.upgrade() {
            let page = ui.get_page();
            if page != self.last_page {
                log!("ui: page {} -> {}", self.last_page, page);
                self.last_page = page;
            }
        }
        if self.dirty {
            self.dirty = false;
            self.refresh_ui();
        }
    }

    // ------------------------------------------------------------ actions

    fn set_anc(&mut self, byte: u8) {
        if !self.ready() {
            return;
        }
        self.send(cmd::ANC_SET, &[0x01, byte, 0x00]);
        self.anc = byte;
        if self.caps().nc.map_or(false, |n| n.levels.iter().any(|(b, _)| *b == byte)) {
            self.nc_level = byte;
        }
        self.queue(500, cmd::ANC_GET, &[]);
        self.dirty = true;
    }

    fn set_spatial(&mut self, v: u8) {
        if !self.ready() {
            return;
        }
        self.send(cmd::SPATIAL_SET, &[v]);
        self.spatial = v;
        self.queue(500, cmd::SPATIAL_GET, &[]);
        self.dirty = true;
    }

    fn set_bass(&mut self, on: bool, level: u8) {
        if !self.ready() {
            return;
        }
        let level = level.clamp(1, self.caps().bass.max(1));
        self.send(cmd::BASS_SET, &[on as u8, level * 2]);
        self.bass_on = on;
        self.bass_level = level;
        self.queue(500, cmd::BASS_GET, &[]);
        self.dirty = true;
    }

    fn set_latency(&mut self, on: bool) {
        if !self.ready() {
            return;
        }
        self.send(cmd::LATENCY_SET, &[if on { 1 } else { 2 }, 0]);
        self.latency = on;
        self.queue(500, cmd::LATENCY_GET, &[]);
        self.dirty = true;
    }

    fn set_in_ear(&mut self, on: bool) {
        if !self.ready() {
            return;
        }
        self.send(cmd::IN_EAR_SET, &[0x01, 0x01, on as u8]);
        self.in_ear = Some(on);
        self.queue(500, cmd::IN_EAR_GET, &[]);
        self.dirty = true;
    }

    fn set_dual(&mut self, on: bool) {
        if !self.ready() {
            return;
        }
        self.send(cmd::DUAL_SET, &[on as u8]);
        self.dual = Some(on);
        self.queue(500, cmd::DUAL_GET, &[]);
        self.queue(900, cmd::DUAL_DEVICES_GET, &[0x00]);
        self.dirty = true;
    }

    fn set_panc(&mut self, on: bool) {
        if !self.ready() {
            return;
        }
        self.send(cmd::PERSONAL_ANC_SET, &[on as u8]);
        self.panc = Some(on);
        self.queue(500, cmd::PERSONAL_ANC_GET, &[]);
        self.dirty = true;
    }

    /// side: 6 = whole device, 2 = left, 3 = right; on=false stops.
    fn ring(&mut self, side: i32, on: bool) {
        if !self.ready() {
            return;
        }
        let ring = self.caps().ring;
        let side_b = side.clamp(0, 6) as u8;
        let st = on as u8;
        match ring {
            Ring::None => return,
            Ring::Legacy => self.send(cmd::RING_SET, &[st]),
            Ring::Single => self.send(cmd::RING_SET, &[0x06, st]),
            Ring::Buds => {
                if on {
                    self.send(cmd::RING_SET, &[side_b, 0x01]);
                } else {
                    let s = self.ringing.map(|(s, _)| s).unwrap_or(side_b);
                    self.send(cmd::RING_SET, &[s, 0x00]);
                }
            }
        }
        self.ringing = if on { Some((side_b, Instant::now())) } else { None };
        self.dirty = true;
    }

    fn fit_test(&mut self) {
        if !self.ready() {
            return;
        }
        self.send(cmd::FIT_TEST_SET, &[0x01]);
        self.fit = None;
        self.fit_until = Some(Instant::now() + Duration::from_secs(20));
        self.dirty = true;
    }

    fn set_gesture(&mut self, idx: usize, action: u8) {
        let Some(slot) = self.caps().gestures.get(idx) else { return };
        let (d, b, k) = (slot.device, slot.button, slot.kind);
        self.gesture_open = None;
        if self.ready() {
            self.send(cmd::GESTURE_SET, &[0x01, d, b, k, action]);
            match self.gestures.iter_mut().find(|g| (g.0, g.1, g.2) == (d, b, k)) {
                Some(g) => g.3 = action,
                None => self.gestures.push((d, b, k, action)),
            }
            self.queue(500, cmd::GESTURES_GET, &[]);
        }
        self.dirty = true;
    }

    fn set_eq_preset(&mut self, id: u8) {
        if !self.ready() {
            return;
        }
        let caps = self.caps();
        let set = if caps.listen_mode { cmd::LISTEN_SET } else { cmd::EQ_SET };
        self.send(set, &[id, 0]);
        if Some(id) == caps.custom_id {
            let c = self.custom;
            self.send(cmd::CUSTOM_EQ_SET, &proto::custom_eq_payload(c[0], c[1], c[2]));
        }
        self.eq = Some(id);
        if caps.advanced_eq {
            self.adv_on = Some(false);
            self.queue(700, cmd::ADVANCED_EQ_GET, &[]);
        }
        let get = self.eq_get_cmd();
        self.queue(500, get, &[]);
        self.dirty = true;
    }

    fn set_eq_custom(&mut self, b: f32, m: f32, t: f32) {
        let r = |v: f32| (v.clamp(-6.0, 6.0) * 10.0).round() / 10.0;
        self.custom = [r(b), r(m), r(t)];
        self.settings.eq_bass = self.custom[0];
        self.settings.eq_mid = self.custom[1];
        self.settings.eq_treble = self.custom[2];
        self.settings.save();
        let caps = self.caps();
        if let (true, Some(cid)) = (self.ready(), caps.custom_id) {
            if self.eq != Some(cid) || self.adv_on == Some(true) {
                let set = if caps.listen_mode { cmd::LISTEN_SET } else { cmd::EQ_SET };
                self.send(set, &[cid, 0]);
            }
            let c = self.custom;
            self.send(cmd::CUSTOM_EQ_SET, &proto::custom_eq_payload(c[0], c[1], c[2]));
            self.eq = Some(cid);
            if caps.advanced_eq {
                self.adv_on = Some(false);
            }
            self.bands = eq::simple_bands(c[0], c[1], c[2]).to_vec();
        }
        self.dirty = true;
    }

    fn set_eq_mode(&mut self, advanced: bool) {
        if !self.ready() {
            return;
        }
        if advanced {
            if self.adv.is_empty() {
                self.adv = if self.bands.is_empty() { default_advanced() } else { self.bands.clone() };
            }
            self.adv_commit();
        } else {
            // selecting a simple preset turns the advanced EQ off on the device
            let id = self.eq.unwrap_or(self.caps().presets.first().map(|p| p.0).unwrap_or(0));
            self.set_eq_preset(id);
        }
        self.dirty = true;
    }

    fn adv_commit(&mut self) {
        if !self.ready() || self.adv.is_empty() {
            return;
        }
        let payload = proto::advanced_eq_payload(&self.adv);
        self.send(cmd::CUSTOM_EQ_SET, &payload);
        if self.adv_on != Some(true) {
            self.send(cmd::ADVANCED_EQ_SET, &[0x01]);
            self.adv_on = Some(true);
            self.queue(600, cmd::ADVANCED_EQ_GET, &[]);
        }
        self.bands = self.adv.clone();
        self.dirty = true;
    }

    fn adv_drag(&mut self, i: usize, dx: f32, dy: f32) {
        let Some(b) = self.adv.get_mut(i) else { return };
        let x = eq::freq_to_x(b.freq as f64) + dx as f64;
        let y = (0.5 - b.gain as f64 / (2.0 * ADV_RANGE)) + dy as f64;
        b.freq = eq::x_to_freq(x).round() as f32;
        b.gain = (((0.5 - y) * 2.0 * ADV_RANGE).clamp(-ADV_RANGE, ADV_RANGE) * 10.0).round() as f32 / 10.0;
        self.adv_sel = i;
        self.push_adv();
    }

    fn adv_edit(&mut self, i: usize, fx: f32, gain: f32, q: f32) {
        let Some(b) = self.adv.get_mut(i) else { return };
        b.freq = eq::x_to_freq(fx as f64).round() as f32;
        b.gain = (gain.clamp(-12.0, 12.0) * 10.0).round() / 10.0;
        b.q = (q.clamp(0.3, 6.0) * 100.0).round() / 100.0;
        self.push_adv();
    }

    fn adv_add(&mut self) {
        if self.adv.len() >= 8 {
            return;
        }
        // put the new band in the widest gap of the log axis
        let mut xs: Vec<f64> = self.adv.iter().map(|b| eq::freq_to_x(b.freq as f64)).collect();
        xs.push(0.0);
        xs.push(1.0);
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (mut best, mut gap) = (0.5, 0.0);
        for w in xs.windows(2) {
            if w[1] - w[0] > gap {
                gap = w[1] - w[0];
                best = (w[0] + w[1]) / 2.0;
            }
        }
        self.adv.push(Band { kind: 1, gain: 0.0, freq: eq::x_to_freq(best).round() as f32, q: 1.0 });
        self.adv.sort_by(|a, b| a.freq.partial_cmp(&b.freq).unwrap());
        self.adv_sel = self.adv.iter().position(|b| b.gain == 0.0 && (eq::freq_to_x(b.freq as f64) - best).abs() < 0.01).unwrap_or(0);
        self.adv_commit();
    }

    /// Recompute graph points + curve for the advanced EQ without touching the device.
    fn push_adv(&mut self) {
        let Some(ui) = self.ui.upgrade() else { return };
        let rows: Vec<EqBand> = self
            .adv
            .iter()
            .map(|b| EqBand {
                kind: b.kind as i32,
                gain: b.gain,
                freq: b.freq,
                q: b.q,
                x: eq::freq_to_x(b.freq as f64) as f32,
                y: (0.5 - b.gain as f64 / (2.0 * ADV_RANGE)) as f32,
            })
            .collect();
        sync(&self.m.adv, rows);
        ui.set_adv_curve(eq::curve(&self.adv, self.graph_w, ADV_H, ADV_RANGE).into());
        let sel = self.adv_sel.min(self.adv.len().saturating_sub(1));
        ui.set_adv_selected(if self.adv.is_empty() { -1 } else { sel as i32 });
        if let Some(b) = self.adv.get(sel) {
            ui.set_adv_fx(eq::freq_to_x(b.freq as f64) as f32);
            ui.set_adv_gain(b.gain);
            ui.set_adv_q(b.q);
            ui.set_adv_kind(b.kind as i32);
            ui.set_adv_freq_text(freq_text(b.freq).into());
        }
    }

    fn select(&mut self, idx: usize) {
        if let Some(d) = self.paired.get(idx) {
            let a = d.info.address.clone();
            if self.selected.as_deref() != Some(&a) {
                self.drop_link();
                self.connecting = None;
                self.error.clear();
                self.selected = Some(a.clone());
                self.next_open = Instant::now();
                self.settings.last_device = a;
                self.settings.save();
                self.manage_link();
                self.dirty = true;
            }
        }
    }

    fn reconnect(&mut self) {
        if self.busy() {
            return;
        }
        let Some(d) = self.sel_dev() else { return };
        let (addr, connected) = (d.info.address.clone(), d.info.connected);
        self.error.clear();
        self.next_open = Instant::now();
        if connected {
            self.manage_link();
        } else {
            log!("connect: asking macOS to connect {addr}");
            self.bt.cmd(bt::Cmd::Connect(addr.clone()));
            self.connecting = Some((addr, Instant::now()));
        }
        self.dirty = true;
    }

    fn open_page(&mut self, page: i32) {
        log!("ui: open page {page} from the menu bar panel");
        if let Some(p) = self.panel.upgrade() {
            let _ = p.hide();
        }
        if let Some(ui) = self.ui.upgrade() {
            ui.set_page(page);
            show_window(&ui);
        }
    }

    fn toggle_panel(&mut self) {
        let Some(p) = self.panel.upgrade() else { return };
        if p.window().is_visible() {
            let _ = p.hide();
            return;
        }
        // the click that took focus away from the panel already closed it
        if PANEL_HIDDEN_AT.with(|t| t.get().map_or(false, |t| t.elapsed() < Duration::from_millis(300))) {
            return;
        }
        if let Some(a) = self.tray.as_ref().and_then(|t| t.anchor()) {
            let x = (a.center_x - PANEL_W / 2.0).clamp(a.screen_left + 8.0, a.screen_right - PANEL_W - 8.0);
            p.window().set_position(slint::LogicalPosition::new(x as f32, (a.bottom + 6.0) as f32));
        }
        self.refresh_ui();
        let _ = p.show();
        activate();
        use slint::winit_030::WinitWindowAccessor;
        p.window().with_winit_window(|w| w.focus_window());
    }

    fn set_language(&mut self, lang: &str) {
        self.settings.language = lang.to_string();
        self.settings.save();
        i18n::apply(lang);
        self.dirty = true;
    }

    // ------------------------------------------------------------ UI

    fn batteries(&self) -> (Vec<BattItem>, String) {
        let find = |id: BattId| self.batt.iter().find(|b| b.id == id).copied();
        let item = |b: proto::Batt, label: &str| BattItem { label: label.into(), pct: b.level as i32, charging: b.charging };
        if let Some(s) = find(BattId::Single) {
            return (vec![item(s, "")], format!("{}%", s.level));
        }
        let (l, c, r) = (find(BattId::Left), find(BattId::Case), find(BattId::Right));
        let mut v = vec![];
        if let Some(l) = l {
            v.push(item(l, tr("L")));
        }
        if let Some(c) = c {
            v.push(item(c, tr("C")));
        }
        if let Some(r) = r {
            v.push(item(r, tr("R")));
        }
        if v.is_empty() {
            if let Some(b) = self.batt.first() {
                v.push(item(*b, ""));
            }
        }
        let title = match (l, r) {
            (Some(l), Some(r)) => format!("{}%", l.level.min(r.level)),
            (Some(x), None) | (None, Some(x)) => format!("{}%", x.level),
            _ => v.first().map(|b| format!("{}%", b.pct)).unwrap_or_default(),
        };
        (v, title)
    }

    fn refresh_ui(&mut self) {
        let Some(ui) = self.ui.upgrade() else { return };
        let panel = self.panel.upgrade();
        let model = self.model();
        let caps = self.caps();
        let ready = self.ready();

        // device chips
        let entries: Vec<(String, String, bool, &'static Model)> =
            self.paired.iter().map(|d| (d.info.name.clone(), d.info.address.clone(), d.info.connected, d.model)).collect();
        let items: Vec<DeviceItem> = entries
            .into_iter()
            .map(|(name, address, connected, m)| DeviceItem { name: name.into(), address: address.into(), connected, img: self.img(m, 128) })
            .collect();
        sync(&self.m.devices, items);
        let sel_idx = self.paired.iter().position(|d| Some(&d.info.address) == self.selected.as_ref()).map(|i| i as i32).unwrap_or(-1);

        let sel_info = self.sel_dev().map(|d| (d.info.address.clone(), d.info.name.clone()));
        let shown_name: String = match (&sel_info, model) {
            (Some((_, n)), Some(m)) => {
                let n = n.trim_start_matches("Nothing ").trim_start_matches("nothing ");
                if n.is_empty() { m.name.to_string() } else { n.to_string() }
            }
            _ => String::new(),
        };
        if let (Some((addr, _)), Some(m)) = (&sel_info, model) {
            if self.hero_for != *addr || self.hero_for.is_empty() {
                self.hero_for = addr.clone();
                let hero = self.img(m, 600);
                ui.set_hero(hero);
            }
        }
        let status: String = if self.busy() {
            tr("Connecting…").into()
        } else if !self.error.is_empty() {
            self.error.clone()
        } else {
            tr("Reconnect").into()
        };
        let (batts, batt_title) = self.batteries();
        sync(&self.m.batteries, if ready { batts } else { vec![] });

        // noise control
        let nc = caps.nc;
        let levels: Vec<Preset> = nc
            .map(|n| n.levels.iter().map(|(b, name)| Preset { name: tr(name).into(), id: *b as i32 }).collect())
            .unwrap_or_default();
        sync(&self.m.nc_levels, levels);
        let anc_mode = match nc {
            Some(n) if self.anc == n.off => 0,
            Some(n) if Some(self.anc) == n.transparency => 2,
            Some(n) if n.levels.iter().any(|(b, _)| *b == self.anc) => 1,
            _ => -1,
        };

        // EQ presets (two rows of at most 4)
        let mut presets: Vec<Preset> = caps.presets.iter().map(|(id, n)| Preset { name: tr(n).into(), id: *id as i32 }).collect();
        if let Some(cid) = caps.custom_id {
            presets.push(Preset { name: tr("Custom").into(), id: cid as i32 });
        }
        let split = presets.len().min(4);
        let rest = presets.split_off(split);
        sync(&self.m.presets_a, presets);
        sync(&self.m.presets_b, rest);
        let eq_name: String = if self.adv_on == Some(true) {
            tr("Advanced").into()
        } else {
            match self.eq {
                None => String::new(),
                Some(id) if Some(id) == caps.custom_id => tr("Custom").into(),
                Some(id) => caps.presets.iter().find(|(i, _)| *i == id).map(|(_, n)| tr(n).to_string()).unwrap_or_else(|| tr("Custom").into()),
            }
        };
        if self.pushed_custom != Some(self.custom) {
            self.pushed_custom = Some(self.custom);
            ui.set_eq_bass(self.custom[0]);
            ui.set_eq_mid(self.custom[1]);
            ui.set_eq_treble(self.custom[2]);
            let c = self.custom;
            ui.set_eq_curve(eq::curve(&eq::simple_bands(c[0], c[1], c[2]), self.graph_w, EQ_H, 8.0).into());
        }
        if self.adv.is_empty() && caps.advanced_eq {
            self.adv = default_advanced();
        }
        self.push_adv();

        // gestures
        let mut slots = vec![];
        let mut last_group = "";
        if let Some(m) = model {
            for s in caps.gestures {
                let group = group_name(s.device, s.button, m.kind);
                let current = self.gestures.iter().find(|g| (g.0, g.1, g.2) == (s.device, s.button, s.kind)).map(|g| g.3);
                slots.push(GestureSlot {
                    group: group.into(),
                    header: group != last_group,
                    label: gesture_name(caps.touch, s.device, s.button, s.kind).into(),
                    action: current.map(|a| action_label(m, a)).unwrap_or("—").into(),
                });
                last_group = group;
            }
        }
        sync(&self.m.gestures, slots);
        match (self.gesture_open, model) {
            (Some(i), Some(m)) if i < caps.gestures.len() => {
                let s = &caps.gestures[i];
                let current = self.gestures.iter().find(|g| (g.0, g.1, g.2) == (s.device, s.button, s.kind)).map(|g| g.3);
                let acts: Vec<ActionItem> = s.actions.iter().map(|a| ActionItem { name: action_label(m, *a).into(), code: *a as i32, selected: Some(*a) == current }).collect();
                sync(&self.m.actions, acts);
                ui.set_gesture_title(format!("{} · {}", group_name(s.device, s.button, m.kind), gesture_name(caps.touch, s.device, s.button, s.kind)).to_uppercase().into());
                ui.set_gesture_open(i as i32);
            }
            _ => ui.set_gesture_open(-1),
        }

        // dual connection devices
        let dual: Vec<DualItem> = self.dual_devices.iter().map(|d| DualItem { name: d.name.clone().into(), connected: d.connected }).collect();
        sync(&self.m.dual, dual);

        // about
        let address = sel_info.as_ref().map(|s| s.0.replace('-', ":").to_uppercase()).unwrap_or_default();
        let info = vec![
            InfoRow { k: tr("Name").into(), v: shown_name.clone().into() },
            InfoRow { k: tr("Model").into(), v: model.map(|m| format!("{}{}", m.code, if m.verified { tr(" · tested") } else { tr(" · experimental") })).unwrap_or_default().into() },
            InfoRow { k: tr("Firmware").into(), v: self.firmware.clone().into() },
            InfoRow { k: tr("Serial number").into(), v: self.serial.clone().into() },
            InfoRow { k: tr("Bluetooth address").into(), v: address.into() },
        ];
        sync(&self.m.info, info);

        let fit_text = |v: u8| -> SharedString {
            match v {
                0 => tr("Good fit").into(),
                1 => tr("Try another ear tip").into(),
                _ => tr("Not measured").into(),
            }
        };

        // ---- main window scalars
        ui.set_selected(sel_idx);
        ui.set_connected(ready);
        ui.set_busy(self.busy());
        ui.set_bt_denied(self.bt_denied);
        ui.set_status(status.clone().into());
        ui.set_device_name(shown_name.clone().into());
        ui.set_firmware(self.firmware.clone().into());
        ui.set_has_anc(nc.is_some());
        ui.set_has_transparency(nc.map_or(false, |n| n.transparency.is_some()));
        ui.set_spatial_kind(match caps.spatial {
            Spatial::None => 0,
            Spatial::Switch => 1,
            Spatial::Modes => 2,
        });
        ui.set_has_bass(caps.bass > 0);
        ui.set_bass_max(caps.bass.max(1) as i32);
        ui.set_has_latency(caps.latency);
        ui.set_has_custom_eq(caps.custom_id.is_some());
        ui.set_has_advanced_eq(caps.advanced_eq);
        ui.set_has_in_ear(caps.in_ear);
        ui.set_has_ring(caps.ring != Ring::None);
        ui.set_ring_buds(caps.ring == Ring::Buds);
        ui.set_has_fit(caps.fit_test);
        ui.set_has_dual(caps.dual);
        ui.set_has_panc(caps.personal_anc);
        ui.set_has_gestures(!caps.gestures.is_empty());
        ui.set_anc_mode(anc_mode);
        ui.set_nc_level(self.nc_level as i32);
        ui.set_spatial(self.spatial as i32);
        ui.set_bass_on(self.bass_on);
        ui.set_bass_level(self.bass_level as i32);
        ui.set_latency_on(self.latency);
        ui.set_in_ear_on(self.in_ear.unwrap_or(false));
        ui.set_dual_on(self.dual.unwrap_or(false));
        ui.set_panc_on(self.panc.unwrap_or(false));
        ui.set_fit_running(self.fit_until.is_some());
        ui.set_fit_left(self.fit.map(|f| fit_text(f.0)).unwrap_or_default());
        ui.set_fit_right(self.fit.map(|f| fit_text(f.1)).unwrap_or_default());
        ui.set_ringing(self.ringing.map(|r| r.0 as i32).unwrap_or(-1));
        ui.set_eq_preset(if self.adv_on == Some(true) { -1 } else { self.eq.map(|e| e as i32).unwrap_or(-1) });
        ui.set_eq_name(eq_name.into());
        ui.set_eq_mode(if self.adv_on == Some(true) { 1 } else { 0 });
        ui.set_menu_battery(self.settings.menu_battery);
        ui.set_language(self.settings.language.clone().into());

        // ---- menu bar panel
        if let Some(p) = &panel {
            p.set_device_name(if shown_name.is_empty() { "NothingX".into() } else { shown_name.to_lowercase().into() });
            p.set_connected(ready);
            p.set_busy(self.busy());
            p.set_has_device(sel_idx >= 0);
            p.set_status(status.into());
            p.set_has_anc(nc.is_some());
            p.set_has_transparency(nc.map_or(false, |n| n.transparency.is_some()));
            p.set_anc_mode(anc_mode);
            p.set_nc_level(self.nc_level as i32);
            p.set_selected(sel_idx);
        }

        // ---- menu bar title
        if let Some(t) = &self.tray {
            t.set_title(if self.settings.menu_battery && ready { &batt_title } else { "" });
        }
    }
}

thread_local! {
    static PANEL_HIDDEN_AT: Cell<Option<Instant>> = const { Cell::new(None) };
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--render-icon" {
        art::app_icon().save_png(&args[2]).expect("write icon");
        return;
    }
    if args.len() == 4 && args[1] == "--render-dmg-background" {
        art::dmg_background(1.0).save_png(&args[2]).expect("write background");
        art::dmg_background(2.0).save_png(&args[3]).expect("write background @2x");
        return;
    }

    // Dark, edge-to-edge windows like a native Mac app.
    {
        use slint::winit_030::winit;
        use winit::platform::macos::WindowAttributesExtMacOS;
        let renderer = std::env::var("NOTHINGX_RENDERER").unwrap_or_else(|_| "femtovg".into());
        let _ = slint::BackendSelector::new()
            .backend_name("winit".into())
            .renderer_name(renderer)
            .with_winit_window_attributes_hook(|a: winit::window::WindowAttributes| {
                a.with_titlebar_transparent(true)
                    .with_title_hidden(true)
                    .with_fullsize_content_view(true)
                    .with_theme(Some(winit::window::Theme::Dark))
            })
            .select();
    }

    log::init();
    log!("NothingX {} starting", env!("CARGO_PKG_VERSION"));
    let settings = Settings::load();
    let ui = MainWindow::new().unwrap();
    let panel = TrayPanel::new().unwrap();
    i18n::apply(&settings.language);
    ui.set_version(env!("CARGO_PKG_VERSION").into());
    ui.set_launch_at_login(login::enabled());

    let models = Models {
        devices: Rc::new(VecModel::default()),
        batteries: Rc::new(VecModel::default()),
        nc_levels: Rc::new(VecModel::default()),
        presets_a: Rc::new(VecModel::default()),
        presets_b: Rc::new(VecModel::default()),
        gestures: Rc::new(VecModel::default()),
        actions: Rc::new(VecModel::default()),
        dual: Rc::new(VecModel::default()),
        adv: Rc::new(VecModel::default()),
        info: Rc::new(VecModel::default()),
    };
    ui.set_devices(ModelRc::from(models.devices.clone()));
    ui.set_batteries(ModelRc::from(models.batteries.clone()));
    ui.set_nc_levels(ModelRc::from(models.nc_levels.clone()));
    ui.set_presets_a(ModelRc::from(models.presets_a.clone()));
    ui.set_presets_b(ModelRc::from(models.presets_b.clone()));
    ui.set_gesture_slots(ModelRc::from(models.gestures.clone()));
    ui.set_gesture_actions(ModelRc::from(models.actions.clone()));
    ui.set_dual_devices(ModelRc::from(models.dual.clone()));
    ui.set_adv_bands(ModelRc::from(models.adv.clone()));
    ui.set_info(ModelRc::from(models.info.clone()));
    panel.set_devices(ModelRc::from(models.devices.clone()));
    panel.set_batteries(ModelRc::from(models.batteries.clone()));
    panel.set_nc_levels(ModelRc::from(models.nc_levels.clone()));

    let st = Rc::new(RefCell::new(State::new(ui.as_weak(), panel.as_weak(), models, settings)));
    {
        let mut s = st.borrow_mut();
        let mut nothing = vec![];
        let mut cmf = vec![];
        for m in catalog::MODELS {
            let item = ModelItem { name: m.name.into(), code: m.code.into(), img: s.img(m, 300), verified: m.verified, cmf: m.is_cmf(), kind: m.kind as i32 };
            if item.cmf { cmf.push(item) } else { nothing.push(item) }
        }
        ui.set_models_nothing(ModelRc::new(VecModel::from(nothing)));
        ui.set_models_cmf(ModelRc::new(VecModel::from(cmf)));
    }

    macro_rules! hook {
        ($w:ident . $on:ident, |$s:ident $(, $a:ident)*| $body:block) => {{
            let st = st.clone();
            $w.$on(move |$($a),*| {
                #[allow(unused_mut)]
                let Ok(mut $s) = st.try_borrow_mut() else { return };
                $body
            });
        }};
    }
    hook!(ui.on_select_device, |s, i| { s.select(i as usize); });
    hook!(ui.on_reconnect, |s| { s.reconnect(); });
    hook!(ui.on_set_anc, |s, b| { s.set_anc(b as u8); });
    hook!(ui.on_set_spatial, |s, v| { s.set_spatial(v as u8); });
    hook!(ui.on_set_eq_preset, |s, id| { s.set_eq_preset(id as u8); });
    hook!(ui.on_set_eq_mode, |s, m| { s.set_eq_mode(m == 1); });
    hook!(ui.on_set_eq_custom, |s, b, m, t| { s.set_eq_custom(b, m, t); });
    hook!(ui.on_adv_drag, |s, i, dx, dy| { s.adv_drag(i as usize, dx, dy); });
    hook!(ui.on_adv_edit, |s, i, fx, g, q| { s.adv_edit(i as usize, fx, g, q); });
    hook!(ui.on_adv_set_kind, |s, i, k| {
        if let Some(b) = s.adv.get_mut(i as usize) {
            b.kind = k.clamp(0, 2) as u8;
        }
        s.push_adv();
        s.adv_commit();
    });
    hook!(ui.on_adv_select, |s, i| {
        s.adv_sel = i.max(0) as usize;
        s.push_adv();
    });
    hook!(ui.on_adv_commit, |s| { s.adv_commit(); });
    hook!(ui.on_adv_add, |s| { s.adv_add(); });
    hook!(ui.on_adv_remove, |s, i| {
        if s.adv.len() > 1 && (i as usize) < s.adv.len() {
            s.adv.remove(i as usize);
            s.adv_sel = s.adv_sel.min(s.adv.len() - 1);
            s.adv_commit();
        }
    });
    hook!(ui.on_adv_reset, |s| {
        s.adv = default_advanced();
        s.adv_sel = 0;
        s.adv_commit();
    });
    hook!(ui.on_set_bass, |s, on, l| { s.set_bass(on, l as u8); });
    hook!(ui.on_set_latency, |s, on| { s.set_latency(on); });
    hook!(ui.on_set_in_ear, |s, on| { s.set_in_ear(on); });
    hook!(ui.on_set_dual, |s, on| { s.set_dual(on); });
    hook!(ui.on_set_panc, |s, on| { s.set_panc(on); });
    hook!(ui.on_ring, |s, side| { if side < 0 { s.ring(0, false) } else { s.ring(side, true) } });
    hook!(ui.on_fit_test, |s| { s.fit_test(); });
    hook!(ui.on_open_gesture, |s, i| {
        s.gesture_open = Some(i as usize);
        s.dirty = true;
    });
    hook!(ui.on_set_gesture, |s, i, code| { s.set_gesture(i as usize, code as u8); });
    hook!(ui.on_set_language, |s, l| { s.set_language(&l); });
    hook!(ui.on_set_menu_battery, |s, v| {
        s.settings.menu_battery = v;
        s.settings.save();
        s.dirty = true;
    });
    {
        let weak = ui.as_weak();
        ui.on_set_launch_at_login(move |on| {
            let ok = login::set(on);
            if let Some(ui) = weak.upgrade() {
                ui.set_launch_at_login(if ok { on } else { login::enabled() });
            }
        });
    }
    {
        let weak = ui.as_weak();
        ui.on_preview_eq(move |b, m, t| {
            if let Some(ui) = weak.upgrade() {
                let w = ui.get_graph_w();
                ui.set_eq_curve(eq::curve(&eq::simple_bands(b, m, t), w, EQ_H, 8.0).into());
            }
        });
    }
    ui.on_open_log(|| {
        if let Ok(home) = std::env::var("HOME") {
            let _ = std::process::Command::new("open").arg(format!("{home}/Library/Logs/NothingX.log")).spawn();
        }
    });
    ui.on_open_bluetooth_settings(|| {
        let _ = std::process::Command::new("open").arg("x-apple.systempreferences:com.apple.BluetoothSettings").spawn();
    });

    // ---- panel
    hook!(panel.on_set_anc, |s, b| { s.set_anc(b as u8); });
    hook!(panel.on_open_page, |s, p| { s.open_page(p); });
    hook!(panel.on_reconnect, |s| { s.reconnect(); });
    hook!(panel.on_select_device, |s, i| { s.select(i as usize); });
    hook!(panel.on_quit, |s| {
        s.drop_link();
        let _ = slint::quit_event_loop();
    });
    {
        let weak = panel.as_weak();
        panel.on_dismiss(move || {
            if let Some(p) = weak.upgrade() {
                let _ = p.hide();
            }
        });
    }
    {
        use slint::winit_030::{winit::event::WindowEvent, EventResult, WinitWindowAccessor};
        panel.window().on_winit_window_event(|w, ev| {
            if let WindowEvent::Focused(false) = ev {
                if w.is_visible() {
                    let _ = w.hide();
                    PANEL_HIDDEN_AT.with(|t| t.set(Some(Instant::now())));
                }
            }
            EventResult::Propagate
        });
    }

    // closing the window keeps the app alive in the menu bar
    ui.window().on_close_requested(|| {
        set_policy(false);
        slint::CloseRequestResponse::HideWindow
    });

    let _ = ui.show();

    // the menu bar item is created once the event loop (NSApplication) is running
    let tray_timer = Timer::default();
    {
        let st = st.clone();
        tray_timer.start(TimerMode::SingleShot, Duration::from_millis(50), move || {
            if let Some(mtm) = MainThreadMarker::new() {
                if let Ok(mut s) = st.try_borrow_mut() {
                    s.tray = Some(tray::Tray::new(mtm));
                    s.dirty = true;
                }
            }
        });
    }
    let tick = Timer::default();
    {
        let st = st.clone();
        tick.start(TimerMode::Repeated, Duration::from_millis(100), move || {
            if let Ok(mut s) = st.try_borrow_mut() {
                s.tick();
            }
        });
    }

    slint::run_event_loop_until_quit().unwrap();
}
