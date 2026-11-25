use objc2::rc::Retained;
use objc2_foundation::{NSArray, NSDate, NSRunLoop, NSString};
use objc2_io_bluetooth::{IOBluetoothDevice, IOBluetoothSDPServiceRecord};
use std::io::Write;

fn pump(sec: f64) {
    unsafe {
        let rl = NSRunLoop::currentRunLoop();
        rl.runUntilDate(&NSDate::dateWithTimeIntervalSinceNow(sec));
    }
}

fn main() {
    let mut log = std::fs::File::create("/tmp/nothingx-probe.log").unwrap();
    macro_rules! p { ($($a:tt)*) => {{ let s = format!($($a)*); println!("{s}"); writeln!(log, "{s}").ok(); log.flush().ok(); }} }
    unsafe {
        let paired: Option<Retained<NSArray>> = IOBluetoothDevice::pairedDevices();
        let Some(paired) = paired else { p!("no paired devices (permission?)"); pump(1.0); return };
        let mut target: Option<Retained<IOBluetoothDevice>> = None;
        for o in paired.iter() {
            let d: Retained<IOBluetoothDevice> = Retained::cast_unchecked(o);
            let name = d.name().to_string();
            let addr = d.addressString().map(|n| n.to_string()).unwrap_or_default();
            p!("paired: {name} [{addr}] connected={}", d.isConnected());
            if name.contains("Nothing") || name.contains("CMF") { target = Some(d); }
        }
        let Some(d) = target else { p!("no nothing device"); return };
        p!("SDP query...");
        let r = d.performSDPQuery(None);
        p!("sdp ret {r}");
        pump(4.0);
        if let Some(svcs) = d.services() {
            for o in svcs.iter() {
                let rec: Retained<IOBluetoothSDPServiceRecord> = Retained::cast_unchecked(o);
                let name = rec.getServiceName().map(|n| n.to_string()).unwrap_or_default();
                let mut ch: u8 = 0;
                let rr = rec.getRFCOMMChannelID(&mut ch);
                p!("service '{name}' rfcomm_ret={rr} ch={ch}\n{:?}\n", rec);
            }
        } else { p!("no services"); }
        let _ = NSString::from_str("x");
    }
}
