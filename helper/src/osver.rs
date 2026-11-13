//! Windows build detection.
//!
//! The Quick Settings hook targets `ShellHost.exe`, which only hosts Quick Settings since
//! Windows 11 24H2 (build 26100). On older builds the panel lives in `ShellExperienceHost.exe`
//! with a different layout, so the hook must not be attempted there.

use std::sync::OnceLock;

/// First build whose Quick Settings panel is hosted by `ShellHost.exe` (Windows 11 24H2).
pub const MIN_QUICK_SETTINGS_BUILD: u32 = 26100;

#[repr(C)]
struct OsVersionInfoW {
    size: u32,
    major: u32,
    minor: u32,
    build: u32,
    platform_id: u32,
    csd_version: [u16; 128],
}

#[link(name = "ntdll")]
extern "system" {
    /// Unlike `GetVersionEx`, this is not subject to manifest-based version lies.
    fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
}

fn detect() -> u32 {
    // Lets the unsupported-system path be exercised on a supported machine.
    if let Some(fake) = std::env::var("BRIGHTTRAY_FAKE_BUILD")
        .ok()
        .and_then(|v| v.trim().parse().ok())
    {
        return fake;
    }
    let mut info = OsVersionInfoW {
        size: std::mem::size_of::<OsVersionInfoW>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform_id: 0,
        csd_version: [0; 128],
    };
    if unsafe { RtlGetVersion(&mut info) } == 0 {
        info.build
    } else {
        0
    }
}

/// The Windows build number (e.g. 19045, 22631, 26100), or 0 if it could not be read.
pub fn build() -> u32 {
    static BUILD: OnceLock<u32> = OnceLock::new();
    *BUILD.get_or_init(detect)
}

/// Whether the Quick Settings hook can work on this system. An unreadable build number counts
/// as supported: the hook validates everything it patches and gives up safely on its own.
pub fn quick_settings_supported() -> bool {
    let build = build();
    build == 0 || build >= MIN_QUICK_SETTINGS_BUILD
}
