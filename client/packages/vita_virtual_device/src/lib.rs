use rstar::{primitives::Rectangle, RTree, AABB};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;

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

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Button {
    ThumbRight,
    ThumbLeft,
    Options,
    Share,
    TriggerRight,
    TriggerLeft,
    ShoulderRight,
    ShoulderLeft,
    Triangle,
    Circle,
    Cross,
    Square,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum DpadDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    None,
}

/// Represents an action triggered by a touch input.
#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub enum TouchAction {
    Dpad(DpadDirection),
    Button(Button),
}

/// Point in 2D space (x, y).
#[derive(Clone, Debug, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Point(pub i32, pub i32);

impl Point {
    #[inline]
    pub fn x(&self) -> i32 {
        self.0
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.1
    }
}

/// Implements `rstar::Point` for spatial indexing.
impl rstar::Point for Point {
    type Scalar = i32;

    const DIMENSIONS: usize = 2;

    #[inline]
    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Point(generator(0), generator(1))
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.0,
            1 => self.1,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => unreachable!(),
        }
    }
}

/// Defines a touch zone and the action it triggers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TouchZone {
    rect: Rectangle<Point>,
    /// The emulated action to perform when the touch zone is touched.
    action: Option<TouchAction>,
}

impl TouchZone {
    #[inline]
    pub fn new(rect: (Point, Point), action: Option<TouchAction>) -> Self {
        TouchZone {
            rect: AABB::from_corners(rect.0, rect.1).into(),
            action,
        }
    }
}

/// Enables spatial queries on `TouchZone` objects.
impl rstar::RTreeObject for TouchZone {
    type Envelope = AABB<Point>;

    #[inline]
    fn envelope(&self) -> Self::Envelope {
        self.rect.envelope()
    }
}

impl rstar::PointDistance for TouchZone {
    #[inline]
    fn distance_2(&self, point: &Point) -> i32 {
        self.rect.distance_2(point)
    }

    #[inline]
    fn contains_point(&self, point: &<Self::Envelope as rstar::Envelope>::Point) -> bool {
        self.rect.contains_point(point)
    }

    #[inline]
    fn distance_2_if_less_or_equal(&self, point: &Point, max_distance_2: i32) -> Option<i32> {
        self.rect.distance_2_if_less_or_equal(point, max_distance_2)
    }
}

/// Configuration for touch inputs, can be zones or a touchpad.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[doc(hidden)]
pub enum TouchConfig {
    Zones(RTree<TouchZone>),
    Touchpad,
}

impl TouchConfig {
    /// Creates a `TouchConfig` with specified zones.
    pub fn zones<I: IntoIterator<Item = TouchZone>>(it: I) -> Self {
        TouchConfig::Zones(RTree::bulk_load(it.into_iter().collect()))
    }

    /// Creates a `TouchConfig` representing a touchpad.
    #[inline]
    pub fn touchpad() -> Self {
        TouchConfig::Touchpad
    }
}

/// Configuration for trigger inputs.
#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub enum TriggerConfig {
    Shoulder,
    Trigger,
}

impl Default for TriggerConfig {
    #[inline]
    fn default() -> Self {
        TriggerConfig::Shoulder
    }
}

impl Config {
    #[inline]
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    #[inline]
    pub fn rear_rl2_front_rl3() -> Self {
        Config {
            rear_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        REAR_TOUCHPAD_RECT.0,
                        Point((REAR_TOUCHPAD_RECT.1).0 / 2, (REAR_TOUCHPAD_RECT.1).1),
                    ),
                    Some(TouchAction::Button(Button::ShoulderLeft)),
                ),
                TouchZone::new(
                    (
                        Point((REAR_TOUCHPAD_RECT.1).0 / 2, (REAR_TOUCHPAD_RECT.0).0),
                        REAR_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(Button::ShoulderRight)),
                ),
            ])),
            front_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        FRONT_TOUCHPAD_RECT.0,
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.1).1),
                    ),
                    Some(TouchAction::Button(Button::ThumbLeft)),
                ),
                TouchZone::new(
                    (
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.0).0),
                        FRONT_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(Button::ThumbRight)),
                ),
            ])),
            trigger_config: TriggerConfig::Trigger,
        }
    }

    #[inline]
    pub fn rear_rl1_front_rl3_vitatriggers_rl2() -> Self {
        Config {
            rear_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        REAR_TOUCHPAD_RECT.0,
                        Point((REAR_TOUCHPAD_RECT.1).0 / 2, (REAR_TOUCHPAD_RECT.1).1),
                    ),
                    Some(TouchAction::Button(Button::TriggerLeft)),
                ),
                TouchZone::new(
                    (
                        Point((REAR_TOUCHPAD_RECT.1).0 / 2, (REAR_TOUCHPAD_RECT.0).0),
                        REAR_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(Button::TriggerRight)),
                ),
            ])),
            front_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        FRONT_TOUCHPAD_RECT.0,
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.1).1),
                    ),
                    Some(TouchAction::Button(Button::ThumbLeft)),
                ),
                TouchZone::new(
                    (
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.0).0),
                        FRONT_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(Button::ThumbRight)),
                ),
            ])),
            trigger_config: TriggerConfig::Shoulder,
        }
    }

    #[inline]
    pub fn front_top_rl2_bottom_rl3_rear_touchpad() -> Self {
        Config {
            front_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        FRONT_TOUCHPAD_RECT.0,
                        Point(FRONT_TOUCHPAD_RECT.1.x() / 2, FRONT_TOUCHPAD_RECT.1.y() / 2),
                    ),
                    Some(TouchAction::Button(Button::ShoulderLeft)),
                ),
                TouchZone::new(
                    (
                        Point(FRONT_TOUCHPAD_RECT.1.x() / 2, FRONT_TOUCHPAD_RECT.0.y()),
                        Point(FRONT_TOUCHPAD_RECT.1.x(), FRONT_TOUCHPAD_RECT.1.y() / 2),
                    ),
                    Some(TouchAction::Button(Button::ShoulderRight)),
                ),
                TouchZone::new(
                    (
                        Point(FRONT_TOUCHPAD_RECT.0.x(), FRONT_TOUCHPAD_RECT.1.y() / 2),
                        Point(FRONT_TOUCHPAD_RECT.1.x() / 2, FRONT_TOUCHPAD_RECT.1.y()),
                    ),
                    Some(TouchAction::Button(Button::ThumbLeft)),
                ),
                TouchZone::new(
                    (
                        Point(FRONT_TOUCHPAD_RECT.1.x() / 2, FRONT_TOUCHPAD_RECT.1.y() / 2),
                        FRONT_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(Button::ThumbRight)),
                ),
            ])),
            rear_touch_config: Some(TouchConfig::Touchpad),
            trigger_config: TriggerConfig::Trigger,
        }
    }

    #[inline]
    pub fn rear_top_rl2_bottom_rl3_front_touchpad() -> Self {
        Config {
            front_touch_config: Some(TouchConfig::Touchpad),
            rear_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        REAR_TOUCHPAD_RECT.0,
                        Point(REAR_TOUCHPAD_RECT.1.x() / 2, REAR_TOUCHPAD_RECT.1.y() / 2),
                    ),
                    Some(TouchAction::Button(Button::ShoulderLeft)),
                ),
                TouchZone::new(
                    (
                        Point(REAR_TOUCHPAD_RECT.1.x() / 2, REAR_TOUCHPAD_RECT.0.y()),
                        Point(REAR_TOUCHPAD_RECT.1.x(), REAR_TOUCHPAD_RECT.1.y() / 2),
                    ),
                    Some(TouchAction::Button(Button::ShoulderRight)),
                ),
                TouchZone::new(
                    (
                        Point(REAR_TOUCHPAD_RECT.0.x(), REAR_TOUCHPAD_RECT.1.y() / 2),
                        Point(REAR_TOUCHPAD_RECT.1.x() / 2, REAR_TOUCHPAD_RECT.1.y()),
                    ),
                    Some(TouchAction::Button(Button::ThumbLeft)),
                ),
                TouchZone::new(
                    (
                        Point(REAR_TOUCHPAD_RECT.1.x() / 2, REAR_TOUCHPAD_RECT.1.y() / 2),
                        REAR_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(Button::ThumbRight)),
                ),
            ])),
            trigger_config: TriggerConfig::Trigger,
        }
    }
}

/// Overall configuration for the virtual device.
#[derive(Clone, Debug, Deserialize, Serialize, derive_builder::Builder)]
pub struct Config {
    pub front_touch_config: Option<TouchConfig>,
    pub rear_touch_config: Option<TouchConfig>,
    pub trigger_config: TriggerConfig,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Config {
            front_touch_config: Some(TouchConfig::Touchpad),
            rear_touch_config: None,
            trigger_config: TriggerConfig::default(),
        }
    }
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
