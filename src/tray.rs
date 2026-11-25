//! Menu bar item: an NSStatusItem whose click toggles the NothingX panel.

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{define_class, msg_send, sel, AllocAnyThread, MainThreadMarker};
use objc2_app_kit::*;
use objc2_foundation::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

static CLICKED: AtomicBool = AtomicBool::new(false);

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "NXStatusTarget"]
    struct StatusTarget;
    unsafe impl NSObjectProtocol for StatusTarget {}
    impl StatusTarget {
        #[unsafe(method(statusClicked:))]
        fn status_clicked(&self, _sender: &AnyObject) {
            CLICKED.store(true, Ordering::Relaxed);
        }
    }
);

/// True once per click on the menu bar icon.
pub fn take_click() -> bool {
    CLICKED.swap(false, Ordering::Relaxed)
}

pub struct Tray {
    item: Retained<NSStatusItem>,
    _target: Retained<StatusTarget>,
    mtm: MainThreadMarker,
    title: RefCell<String>,
}

/// 18pt template icon: dot-matrix headphones.
fn icon_png() -> Vec<u8> {
    use tiny_skia::*;
    let mut pm = Pixmap::new(36, 36).unwrap();
    let mut pb = PathBuilder::new();
    for i in 0..9 {
        let a = std::f32::consts::PI * (1.0 + i as f32 / 8.0);
        pb.push_circle(18.0 + 13.0 * a.cos(), 20.0 + 13.0 * a.sin(), 1.9);
    }
    for side in [-1.0f32, 1.0] {
        for r in 0..3 {
            for c in 0..2 {
                pb.push_circle(18.0 + side * (13.0 - c as f32 * 4.6), 21.5 + r as f32 * 4.6, 2.0);
            }
        }
    }
    let mut p = Paint::default();
    p.set_color_rgba8(0, 0, 0, 255);
    p.anti_alias = true;
    pm.fill_path(&pb.finish().unwrap(), &p, FillRule::Winding, Transform::identity(), None);
    pm.encode_png().unwrap()
}

/// Where to put the panel: centre x and bottom y of the icon, plus the horizontal
/// bounds of its screen. Top-left based points, as winit expects.
pub struct Anchor {
    pub center_x: f64,
    pub bottom: f64,
    pub screen_left: f64,
    pub screen_right: f64,
}

impl Tray {
    pub fn new(mtm: MainThreadMarker) -> Tray {
        unsafe {
            let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
            let target: Retained<StatusTarget> = msg_send![StatusTarget::alloc(), init];
            if let Some(button) = item.button(mtm) {
                if let Some(img) = NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(&icon_png())) {
                    img.setSize(NSSize::new(18.0, 18.0));
                    img.setTemplate(true);
                    button.setImage(Some(&img));
                }
                button.setImagePosition(NSCellImagePosition::ImageLeft);
                button.setTarget(Some(&target));
                button.setAction(Some(sel!(statusClicked:)));
                button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
            }
            Tray { item, _target: target, mtm, title: RefCell::new(String::from("-")) }
        }
    }

    pub fn set_title(&self, t: &str) {
        if *self.title.borrow() == t {
            return;
        }
        *self.title.borrow_mut() = t.to_string();
        if let Some(button) = self.item.button(self.mtm) {
            let s = if t.is_empty() { String::new() } else { format!(" {t}") };
            button.setTitle(&NSString::from_str(&s));
        }
    }

    pub fn anchor(&self) -> Option<Anchor> {
        let button = self.item.button(self.mtm)?;
        let win = button.window()?;
        let f = win.frame();
        let primary_h = NSScreen::screens(self.mtm).firstObject()?.frame().size.height;
        let sf = win.screen().map(|s| s.frame()).unwrap_or(f);
        Some(Anchor {
            center_x: f.origin.x + f.size.width / 2.0,
            bottom: primary_h - f.origin.y,
            screen_left: sf.origin.x,
            screen_right: sf.origin.x + sf.size.width,
        })
    }
}
