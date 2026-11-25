//! "Open at login" through SMAppService (macOS 13+).

use objc2_service_management::{SMAppService, SMAppServiceStatus};

pub fn enabled() -> bool {
    unsafe { SMAppService::mainAppService().status() == SMAppServiceStatus::Enabled }
}

pub fn set(on: bool) -> bool {
    let svc = unsafe { SMAppService::mainAppService() };
    let r = unsafe { if on { svc.registerAndReturnError() } else { svc.unregisterAndReturnError() } };
    if let Err(e) = &r {
        crate::log!("login item: {e:?}");
    }
    r.is_ok()
}
