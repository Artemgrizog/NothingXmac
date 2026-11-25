//! Bluetooth Classic transport on macOS (IOBluetooth RFCOMM).
//!
//! IOBluetooth only works on the main thread, and its *synchronous* calls spin the
//! main run loop (re-entering the UI event loop => abort). So everything here uses
//! the asynchronous IOBluetooth API on the main thread; results are collected by
//! the delegate and handed to the controller as events via `events()`.

use crate::proto::UUID_NT_LINK;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{define_class, msg_send, AllocAnyThread, DefinedClass};
use objc2_foundation::{NSObjectProtocol, NSString};
use objc2_io_bluetooth::{IOBluetoothDevice, IOBluetoothRFCOMMChannel, IOBluetoothSDPServiceRecord, IOBluetoothSDPUUID};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct Paired {
    pub name: String,
    pub address: String,
    pub connected: bool,
    /// Device advertises the Nothing control service ("NT LINK") in its cached SDP records.
    pub has_nt: bool,
}

pub enum Cmd {
    Scan,
    Open(String),
    Send(Vec<u8>),
    Close,
    Connect(String),
}

#[derive(Debug)]
pub enum Ev {
    /// `None`: Bluetooth access denied / unavailable.
    Paired(Option<Vec<Paired>>),
    Opened(String),
    /// address, reason, retry soon (SDP still loading)
    OpenFailed(String, String, bool),
    /// The control channel is held by another host (e.g. Nothing X on a phone).
    Busy(String),
    NotNothing(String),
    Closed(String),
    Data(Vec<u8>),
    ConnectDone(bool),
}

#[derive(Default)]
struct Shared {
    rx: Vec<u8>,
    open_status: Option<i32>,
    closed: bool,
    write_bufs: HashMap<usize, Vec<u8>>,
    next_ref: usize,
    connect_status: Option<i32>,
}

struct Ivars {
    shared: Arc<Mutex<Shared>>,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "NXBluetoothDelegate"]
    #[ivars = Ivars]
    struct Delegate;

    unsafe impl NSObjectProtocol for Delegate {}

    impl Delegate {
        #[unsafe(method(rfcommChannelData:data:length:))]
        fn data(&self, _ch: *mut AnyObject, data: *mut c_void, len: usize) {
            if data.is_null() || len == 0 {
                return;
            }
            let s = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
            self.ivars().shared.lock().unwrap().rx.extend_from_slice(s);
        }

        #[unsafe(method(rfcommChannelOpenComplete:status:))]
        fn open_complete(&self, _ch: *mut AnyObject, status: i32) {
            crate::log!("bt: open complete status={status:#x}");
            self.ivars().shared.lock().unwrap().open_status = Some(status);
        }

        #[unsafe(method(rfcommChannelClosed:))]
        fn closed(&self, _ch: *mut AnyObject) {
            crate::log!("bt: channel closed");
            self.ivars().shared.lock().unwrap().closed = true;
        }

        #[unsafe(method(rfcommChannelWriteComplete:refcon:status:))]
        fn write_complete(&self, _ch: *mut AnyObject, refcon: *mut c_void, _status: i32) {
            self.ivars().shared.lock().unwrap().write_bufs.remove(&(refcon as usize));
        }

        #[unsafe(method(connectionComplete:status:))]
        fn connection_complete(&self, _dev: *mut AnyObject, status: i32) {
            crate::log!("bt: connection complete status={status:#x}");
            self.ivars().shared.lock().unwrap().connect_status = Some(status);
        }
    }
);

impl Delegate {
    fn new(shared: Arc<Mutex<Shared>>) -> Retained<Self> {
        let this = Self::alloc().set_ivars(Ivars { shared });
        unsafe { msg_send![super(this), init] }
    }
}

fn device(addr: &str) -> Option<Retained<IOBluetoothDevice>> {
    unsafe { IOBluetoothDevice::deviceWithAddressString(Some(&NSString::from_str(addr))) }
}

/// address -> exposes the NT LINK service (checked once per device; the lookup is slow)
static NT_CACHE: Mutex<Option<HashMap<String, bool>>> = Mutex::new(None);

/// kBluetoothDeviceClassMajorAudio
const CLASS_MAJOR_AUDIO: u32 = 0x04;

fn paired_devices() -> Option<Vec<Paired>> {
    unsafe {
        let t0 = Instant::now();
        let arr = IOBluetoothDevice::pairedDevices()?;
        let t_list = t0.elapsed().as_millis();
        let mut v = Vec::new();
        for o in arr.iter() {
            let t1 = Instant::now();
            let d: Retained<IOBluetoothDevice> = Retained::cast_unchecked(o);
            let name = d.name().to_string();
            let address = d.addressString().map(|n| n.to_string()).unwrap_or_default();
            // Only unrecognised audio devices need the (slow) service lookup, e.g. a
            // pair of Nothing headphones the user renamed in Bluetooth settings.
            let has_nt = if crate::catalog::identify_or_generic(&name).is_some() || d.deviceClassMajor() != CLASS_MAJOR_AUDIO {
                false
            } else {
                let mut cache = NT_CACHE.lock().unwrap();
                *cache.get_or_insert_with(HashMap::new).entry(address.clone()).or_insert_with(|| {
                    IOBluetoothSDPUUID::uuidWithBytes_length(UUID_NT_LINK.as_ptr() as *const _, 16)
                        .map_or(false, |u| d.getServiceRecordForUUID(Some(&u)).is_some())
                })
            };
            let connected = d.isConnected();
            let ms = t1.elapsed().as_millis();
            if ms > 100 {
                crate::log!("perf: device {name} took {ms} ms (has_nt={has_nt})");
            }
            v.push(Paired { name, address, connected, has_nt });
        }
        if t_list > 100 {
            crate::log!("perf: pairedDevices {t_list} ms");
        }
        Some(v)
    }
}

fn find_channel(dev: &IOBluetoothDevice) -> Option<u8> {
    unsafe {
        let uuid = IOBluetoothSDPUUID::uuidWithBytes_length(UUID_NT_LINK.as_ptr() as *const _, 16)?;
        let mut ch: u8 = 0;
        if let Some(rec) = dev.getServiceRecordForUUID(Some(&uuid)) {
            if rec.getRFCOMMChannelID(&mut ch) == 0 {
                return Some(ch);
            }
        }
        let svcs = dev.services()?;
        for o in svcs.iter() {
            let rec: Retained<IOBluetoothSDPServiceRecord> = Retained::cast_unchecked(o);
            let name = rec.getServiceName().map(|n| n.to_string()).unwrap_or_default();
            if name == "NT LINK" && rec.getRFCOMMChannelID(&mut ch) == 0 {
                return Some(ch);
            }
        }
        None
    }
}

/// kIOReturnError: what IOBluetooth reports when the device refuses the channel.
const IO_RETURN_ERROR: i32 = 0xe00002bc_u32 as i32;

struct Chan {
    addr: String,
    chan: Retained<IOBluetoothRFCOMMChannel>,
    _dev: Retained<IOBluetoothDevice>,
    _del: Retained<Delegate>,
    shared: Arc<Mutex<Shared>>,
    started: Instant,
    open: bool,
}

struct Conn {
    _dev: Retained<IOBluetoothDevice>,
    _del: Retained<Delegate>,
    shared: Arc<Mutex<Shared>>,
    started: Instant,
}

#[derive(Default)]
struct Inner {
    chan: Option<Chan>,
    conn: Option<Conn>,
    queue: Vec<Ev>,
}

/// Bluetooth driver with a message-style interface.
///
/// Listing paired devices runs on a background thread: the first IOBluetooth call
/// blocks until the user answers the Bluetooth permission prompt, and must not
/// freeze the window. RFCOMM work stays on the main thread (asynchronous API).
pub struct Worker {
    inner: RefCell<Inner>,
    scan_tx: std::sync::mpsc::Sender<()>,
    scan_rx: std::sync::mpsc::Receiver<Option<Vec<Paired>>>,
}

impl Worker {
    pub fn spawn() -> Worker {
        let (scan_tx, req_rx) = std::sync::mpsc::channel::<()>();
        let (res_tx, scan_rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("bt-scan".into())
            .spawn(move || {
                while req_rx.recv().is_ok() {
                    while req_rx.try_recv().is_ok() {}
                    let list = objc2::rc::autoreleasepool(|_| paired_devices());
                    if res_tx.send(list).is_err() {
                        break;
                    }
                }
            })
            .expect("spawn bt-scan thread");
        Worker { inner: RefCell::new(Inner::default()), scan_tx, scan_rx }
    }

    fn close_inner(inner: &mut Inner, notify: bool) {
        if let Some(c) = inner.chan.take() {
            unsafe {
                c.chan.setDelegate(None);
                c.chan.closeChannel();
            }
            if notify {
                inner.queue.push(Ev::Closed(c.addr));
            }
        }
    }

    pub fn cmd(&self, c: Cmd) {
        let mut inner = self.inner.borrow_mut();
        match c {
            Cmd::Scan => {
                let _ = self.scan_tx.send(());
            }
            Cmd::Close => Self::close_inner(&mut inner, false),
            Cmd::Open(addr) => {
                Self::close_inner(&mut inner, false);
                let Some(dev) = device(&addr) else {
                    inner.queue.push(Ev::OpenFailed(addr, "unknown device".into(), false));
                    return;
                };
                let ch = match find_channel(&dev) {
                    Some(c) => c,
                    None => {
                        let have = unsafe { dev.services() }.map_or(false, |s| s.count() > 0);
                        if have {
                            inner.queue.push(Ev::NotNothing(addr));
                        } else {
                            unsafe { dev.performSDPQuery(None) };
                            inner.queue.push(Ev::OpenFailed(addr, "reading services".into(), true));
                        }
                        return;
                    }
                };
                let shared = Arc::new(Mutex::new(Shared::default()));
                let del = Delegate::new(shared.clone());
                let mut chan: Option<Retained<IOBluetoothRFCOMMChannel>> = None;
                let r = unsafe { dev.openRFCOMMChannelAsync_withChannelID_delegate(Some(&mut chan), ch, Some(&del)) };
                crate::log!("bt: openRFCOMMChannelAsync ch={ch} ret={r:#x}");
                match (r, chan) {
                    (0, Some(c)) => {
                        inner.chan = Some(Chan { addr, chan: c, _dev: dev, _del: del, shared, started: Instant::now(), open: false });
                    }
                    (IO_RETURN_ERROR, _) => inner.queue.push(Ev::Busy(addr)),
                    (r, _) => inner.queue.push(Ev::OpenFailed(addr, format!("RFCOMM {r:#x}"), false)),
                }
            }
            Cmd::Send(bytes) => {
                let Some(c) = inner.chan.as_ref().filter(|c| c.open) else { return };
                let mut s = c.shared.lock().unwrap();
                s.next_ref = s.next_ref.wrapping_add(1).max(1);
                let id = s.next_ref;
                let mut buf = bytes;
                let ptr = buf.as_mut_ptr() as *mut c_void;
                let len = buf.len() as u16;
                s.write_bufs.insert(id, buf);
                drop(s);
                let r = unsafe { c.chan.writeAsync_length_refcon(ptr, len, id as *mut c_void) };
                if r != 0 {
                    crate::log!("bt: write failed {r:#x}");
                    c.shared.lock().unwrap().write_bufs.remove(&id);
                }
            }
            Cmd::Connect(addr) => {
                let Some(dev) = device(&addr) else {
                    inner.queue.push(Ev::ConnectDone(false));
                    return;
                };
                let shared = Arc::new(Mutex::new(Shared::default()));
                let del = Delegate::new(shared.clone());
                let r = unsafe { dev.openConnection_(Some(&del)) };
                if r == 0 {
                    inner.conn = Some(Conn { _dev: dev, _del: del, shared, started: Instant::now() });
                } else {
                    inner.queue.push(Ev::ConnectDone(false));
                }
            }
        }
    }

    pub fn events(&self) -> Vec<Ev> {
        let mut inner = self.inner.borrow_mut();
        for list in self.scan_rx.try_iter() {
            inner.queue.push(Ev::Paired(list));
        }
        // control channel
        let mut close = None;
        if let Some(c) = inner.chan.as_mut() {
            let (status, closed, data) = {
                let mut s = c.shared.lock().unwrap();
                (s.open_status, s.closed, std::mem::take(&mut s.rx))
            };
            let mut evs = vec![];
            if !c.open {
                let is_open = unsafe { c.chan.isOpen() };
                match status {
                    Some(0) => c.open = true,
                    None if is_open => c.open = true,
                    Some(IO_RETURN_ERROR) => close = Some(Ev::Busy(c.addr.clone())),
                    Some(e) => close = Some(Ev::OpenFailed(c.addr.clone(), format!("RFCOMM {e:#x}"), false)),
                    None if closed => close = Some(Ev::Busy(c.addr.clone())),
                    None if c.started.elapsed() > Duration::from_secs(12) => close = Some(Ev::Busy(c.addr.clone())),
                    None => {}
                }
                if c.open {
                    evs.push(Ev::Opened(c.addr.clone()));
                }
            } else if closed {
                close = Some(Ev::Closed(c.addr.clone()));
            }
            if c.open && !data.is_empty() {
                evs.push(Ev::Data(data));
            }
            inner.queue.extend(evs);
        }
        if let Some(ev) = close {
            Self::close_inner(&mut inner, false);
            inner.queue.push(ev);
        }
        // connect request
        let done = inner.conn.as_ref().and_then(|c| match c.shared.lock().unwrap().connect_status {
            Some(s) => Some(s == 0),
            None if c.started.elapsed() > Duration::from_secs(20) => Some(false),
            None => None,
        });
        if let Some(ok) = done {
            inner.conn = None;
            inner.queue.push(Ev::ConnectDone(ok));
        }
        std::mem::take(&mut inner.queue)
    }
}
