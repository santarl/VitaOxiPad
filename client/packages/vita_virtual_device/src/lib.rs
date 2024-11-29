use std::ffi::OsString;

mod virtual_button;
mod virtual_config;
mod virtual_touch;
mod virtual_utils;

pub use virtual_touch::Point;

// Error handling that includes platform-specific errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Linux(#[from] linux::Error),
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    Windows(#[from] windows::Error),
}

const FRONT_TOUCHPAD_RECT: (Point, Point) = (Point(0, 0), Point(1920, 1087));
const REAR_TOUCHPAD_RECT: (Point, Point) = (Point(0, 0), Point(1920, 887));

type Result<T> = std::result::Result<T, Error>;

/// A trait for creating and using a virtual device.
pub trait VitaVirtualDevice<ConfigSetter: ?Sized>: Sized {
    type Config;

    fn get_config(&self) -> &Self::Config;
    fn identifiers(&self) -> Option<&[OsString]>;
    fn set_config(&mut self, config: ConfigSetter) -> Result<()>;
    fn send_report(&mut self, report: vita_reports::MainReport) -> Result<()>;
}

/// Helper function to convert a `f32` value to `i16` within specified bounds.
#[inline]
pub fn f32_to_i16(value: f32, min_value: f32, max_value: f32) -> i16 {
    let clamped_value = value.clamp(min_value, max_value);
    let normalized_value = (clamped_value - min_value) / (max_value - min_value);
    let scaled_value = normalized_value * 65536.0 - 32768.0;
    scaled_value.round() as i16
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        pub use linux::VitaDevice;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        pub use windows::VitaDevice;
    }
}
