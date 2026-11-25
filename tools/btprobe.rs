// Debug probe: opens the Nothing control channel and sends raw commands.
// Build as an example inside an app bundle (Bluetooth needs NSBluetoothAlwaysUsageDescription), then run:
//   NX_ADDR=aa-bb-cc-dd-ee-ff probe c007 "f00f:01 05 00"
// Arguments are hex commands with optional ":payload". Output goes to /tmp/nothingx-probe2.log.
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{define_class, msg_send, AllocAnyThread, ClassType};
use objc2_foundation::{NSDate, NSObjectProtocol, NSRunLoop, NSString};
use objc2_io_bluetooth::{IOBluetoothDevice, IOBluetoothRFCOMMChannel};
use std::io::Write;
use std::sync::Mutex;

static RX: Mutex<Vec<u8>> = Mutex::new(Vec::new());

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "NXDelegate"]
    struct Delegate;
    unsafe impl NSObjectProtocol for Delegate {}
    impl Delegate {
        #[unsafe(method(rfcommChannelData:data:length:))]
        fn data(&self, _ch: *mut AnyObject, data: *mut u8, len: usize) {
            let s = unsafe { std::slice::from_raw_parts(data, len) };
            RX.lock().unwrap().extend_from_slice(s);
        }
        #[unsafe(method(rfcommChannelClosed:))]
        fn closed(&self, _ch: *mut AnyObject) { RX.lock().unwrap().extend_from_slice(b"<CLOSED>"); }
    }
);

fn crc(b: &[u8]) -> u16 {
    let mut c: u16 = 0xFFFF;
    for &x in b { c ^= x as u16; for _ in 0..8 { c = if c & 1 != 0 { (c >> 1) ^ 0xA001 } else { c >> 1 }; } }
    c
}
fn frame(cmd: u16, payload: &[u8], seq: u8) -> Vec<u8> {
    let mut v = vec![0x55, 0x60, 0x01, cmd as u8, (cmd >> 8) as u8, payload.len() as u8, 0, seq];
    v.extend_from_slice(payload);
    let c = crc(&v); v.push(c as u8); v.push((c >> 8) as u8); v
}
fn pump(sec: f64) { unsafe { NSRunLoop::currentRunLoop().runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(sec)); } }
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(" ") }

fn main() {
    let mut log = std::fs::File::create("/tmp/nothingx-probe2.log").unwrap();
    macro_rules! p { ($($a:tt)*) => {{ let s = format!($($a)*); println!("{s}"); writeln!(log, "{s}").ok(); log.flush().ok(); }} }
    unsafe {
        let dev = IOBluetoothDevice::deviceWithAddressString(Some(&NSString::from_str(&std::env::var("NX_ADDR").expect("set NX_ADDR=aa-bb-cc-dd-ee-ff")))).unwrap();
        p!("connected={}", dev.isConnected());
        if std::env::args().any(|a| a == "reconnect") {
            p!("closeConnection ret={:#x}", dev.closeConnection());
            pump(4.0);
            p!("after close connected={}", dev.isConnected());
            p!("openConnection ret={:#x}", dev.openConnection());
            pump(4.0);
            p!("after open connected={}", dev.isConnected());
        }
        let del: Retained<Delegate> = msg_send![Delegate::alloc(), init];
        let mut chan: Option<Retained<IOBluetoothRFCOMMChannel>> = None;
        let r = dev.openRFCOMMChannelSync_withChannelID_delegate(Some(&mut chan), 15, Some(&del));
        p!("open ret={r:#x} chan={}", chan.is_some());
        let Some(chan) = chan else { return };
        p!("isOpen={} mtu={}", chan.isOpen(), chan.getMTU());
        let reqs: &[(&str, u16, &[u8])] = &[
            ("battery", 0xC007, &[]), ("anc", 0xC01E, &[]), ("firmware", 0xC042, &[]),
            ("eq", 0xC01F, &[]), ("listening", 0xC050, &[]), ("bass", 0xC04E, &[]),
            ("latency", 0xC041, &[]), ("spatial", 0xC04F, &[]), ("inear", 0xC00E, &[]), ("serial", 0xC006, &[]), ("model", 0xC01C, &[]),
        ];
        let extra: Vec<(String,u16,Vec<u8>)> = std::env::args().skip(1).filter_map(|a| { let mut it = a.splitn(2, ':'); let c = u16::from_str_radix(it.next()?,16).ok()?; let pl: Vec<u8> = it.next().unwrap_or("").split_whitespace().filter_map(|h| u8::from_str_radix(h,16).ok()).collect(); Some((a.clone(),c,pl)) }).collect();
        let reqs: Vec<(&str,u16,&[u8])> = if extra.is_empty() { reqs.to_vec() } else { extra.iter().map(|(n,c,pl)|(n.as_str(),*c,&pl[..])).collect() };
        for (i, (name, cmd, pl)) in reqs.iter().enumerate() {
            let mut f = frame(*cmd, pl, i as u8 + 1);
            let r = chan.writeSync_length(f.as_mut_ptr() as *mut _, f.len() as u16);
            p!("TX {name} {} ret={r:#x}", hex(&f));
            pump(0.6);
            let rx = std::mem::take(&mut *RX.lock().unwrap());
            p!("RX {}", hex(&rx));
        }
        chan.closeChannel();
    }
}
