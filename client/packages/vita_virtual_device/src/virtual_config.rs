use rstar::RTree;
use serde::{Deserialize, Serialize};

use crate::virtual_button::Button;
use crate::virtual_touch::{Point, TouchAction, TouchZone};
use crate::{FRONT_TOUCHPAD_RECT, REAR_TOUCHPAD_RECT};

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
            touchpad_source: None,
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
            touchpad_source: None,
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
            touchpad_source: Some(TouchpadSource::Rear),
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
            touchpad_source: Some(TouchpadSource::Front),
        }
    }
}

/// Configuration for touchpad sourse.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[doc(hidden)]
pub enum TouchpadSource {
    Front,
    Rear,
}

/// Overall configuration for the virtual device.
#[derive(Clone, Debug, Deserialize, Serialize, derive_builder::Builder)]
#[builder(field(public))]
pub struct Config {
    pub front_touch_config: Option<TouchConfig>,
    pub rear_touch_config: Option<TouchConfig>,
    pub trigger_config: TriggerConfig,
    pub touchpad_source: Option<TouchpadSource>,
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Config {
            front_touch_config: Some(TouchConfig::Touchpad),
            rear_touch_config: None,
            trigger_config: TriggerConfig::default(),
            touchpad_source: None,
        }
    }
}
