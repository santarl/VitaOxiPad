use rstar::{primitives::Rectangle, RTree, AABB};
use serde::{Deserialize, Serialize};
use vigem_client::{
    Client, DS4Buttons, DS4ReportExBuilder,
    DS4SpecialButtons, DS4TouchPoint, DS4TouchReport, DpadDirection,
    DS4Status, BatteryStatus,
    DualShock4Wired, TargetId,
};

use std::time::Instant;
use std::{ffi::OsString, time::Duration};

use crate::VitaVirtualDevice;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to connect to the client")]
    ConnectionFailed(#[source] vigem_client::Error),
    #[error("Failed to plugin the target")]
    PluginTargetFailed(#[source] vigem_client::Error),
    #[error("Sending report failed")]
    SendReportFailed(#[source] vigem_client::Error),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub enum TouchAction {
    Dpad(u16),
    Button(u16),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[doc(hidden)]
pub enum TouchConfig {
    Zones(RTree<TouchZone>),
    Touchpad,
}

impl TouchConfig {
    pub fn zones<I: IntoIterator<Item = TouchZone>>(it: I) -> Self {
        TouchConfig::Zones(RTree::bulk_load(it.into_iter().collect()))
    }

    #[inline]
    pub fn touchpad() -> Self {
        TouchConfig::Touchpad
    }
}

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

#[derive(Clone, Debug, Deserialize, Serialize, derive_builder::Builder)]
pub struct Config {
    pub front_touch_config: Option<TouchConfig>,
    pub rear_touch_config: Option<TouchConfig>,
    pub trigger_config: TriggerConfig,
}

// Touch coordinates are in the range [0, 0] x [1920, 887] for the back touchpad
// and [0, 0] x [1920, 1087] for the front touchpad
const FRONT_TOUCHPAD_RECT: (Point, Point) = (Point(0, 0), Point(1920, 1087));
const REAR_TOUCHPAD_RECT: (Point, Point) = (Point(0, 0), Point(1920, 887));

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
                    Some(TouchAction::Button(DS4Buttons::SHOULDER_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point((REAR_TOUCHPAD_RECT.1).0 / 2, (REAR_TOUCHPAD_RECT.0).0),
                        REAR_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(DS4Buttons::SHOULDER_RIGHT)),
                ),
            ])),
            front_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        FRONT_TOUCHPAD_RECT.0,
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.1).1),
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.0).0),
                        FRONT_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_RIGHT)),
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
                    Some(TouchAction::Button(DS4Buttons::TRIGGER_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point((REAR_TOUCHPAD_RECT.1).0 / 2, (REAR_TOUCHPAD_RECT.0).0),
                        REAR_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(DS4Buttons::TRIGGER_RIGHT)),
                ),
            ])),
            front_touch_config: Some(TouchConfig::zones([
                TouchZone::new(
                    (
                        FRONT_TOUCHPAD_RECT.0,
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.1).1),
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point((FRONT_TOUCHPAD_RECT.1).0 / 2, (FRONT_TOUCHPAD_RECT.0).0),
                        FRONT_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_RIGHT)),
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
                    Some(TouchAction::Button(DS4Buttons::SHOULDER_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point(FRONT_TOUCHPAD_RECT.1.x() / 2, FRONT_TOUCHPAD_RECT.0.y()),
                        Point(FRONT_TOUCHPAD_RECT.1.x(), FRONT_TOUCHPAD_RECT.1.y() / 2),
                    ),
                    Some(TouchAction::Button(DS4Buttons::SHOULDER_RIGHT)),
                ),
                TouchZone::new(
                    (
                        Point(FRONT_TOUCHPAD_RECT.0.x(), FRONT_TOUCHPAD_RECT.1.y() / 2),
                        Point(FRONT_TOUCHPAD_RECT.1.x() / 2, FRONT_TOUCHPAD_RECT.1.y()),
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point(FRONT_TOUCHPAD_RECT.1.x() / 2, FRONT_TOUCHPAD_RECT.1.y() / 2),
                        FRONT_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_RIGHT)),
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
                    Some(TouchAction::Button(DS4Buttons::SHOULDER_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point(REAR_TOUCHPAD_RECT.1.x() / 2, REAR_TOUCHPAD_RECT.0.y()),
                        Point(REAR_TOUCHPAD_RECT.1.x(), REAR_TOUCHPAD_RECT.1.y() / 2),
                    ),
                    Some(TouchAction::Button(DS4Buttons::SHOULDER_RIGHT)),
                ),
                TouchZone::new(
                    (
                        Point(REAR_TOUCHPAD_RECT.0.x(), REAR_TOUCHPAD_RECT.1.y() / 2),
                        Point(REAR_TOUCHPAD_RECT.1.x() / 2, REAR_TOUCHPAD_RECT.1.y()),
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_LEFT)),
                ),
                TouchZone::new(
                    (
                        Point(REAR_TOUCHPAD_RECT.1.x() / 2, REAR_TOUCHPAD_RECT.1.y() / 2),
                        REAR_TOUCHPAD_RECT.1,
                    ),
                    Some(TouchAction::Button(DS4Buttons::THUMB_RIGHT)),
                ),
            ])),
            trigger_config: TriggerConfig::Trigger,
        }
    }
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

pub struct VitaDevice {
    ds4_target: DualShock4Wired<Client>,
    config: Config,
    touch_state: bool,
    touch_start_time: Option<Instant>,
}

impl VitaDevice {
    pub fn create(config_name: &str) -> crate::Result<Self> {
        let client = Client::connect().map_err(Error::ConnectionFailed)?;
        let mut ds4_target = DualShock4Wired::new(client, TargetId::DUALSHOCK4_WIRED);

        ds4_target.plugin().map_err(Error::PluginTargetFailed)?;
        ds4_target.wait_ready().map_err(Error::PluginTargetFailed)?;
        // Wait for the device to be ready, because the ioctl doesn't seem to work
        std::thread::sleep(Duration::from_millis(100));

        // Select the configuration depending on the name
        let config = match config_name {
            "standart" => Config::rear_rl2_front_rl3(),
            "alt_triggers" => Config::rear_rl1_front_rl3_vitatriggers_rl2(),
            "rear_touchpad" => Config::front_top_rl2_bottom_rl3_rear_touchpad(),
            "front_touchpad" => Config::rear_top_rl2_bottom_rl3_front_touchpad(),
            _ => return Err(crate::Error::Windows(Error::InvalidConfig(config_name.to_string()))),
        };

        Ok(VitaDevice {
            ds4_target,
            config: config,
            touch_state: false,
            touch_start_time: None,
        })
    }
}

impl VitaVirtualDevice<&ConfigBuilder> for VitaDevice {
    type Config = Config;

    fn identifiers(&self) -> Option<&[OsString]> {
        None
    }

    #[inline]
    fn get_config(&self) -> &Self::Config {
        &self.config
    }

    #[inline]
    fn set_config(&mut self, config: &ConfigBuilder) -> crate::Result<()> {
        if let Some(front_touch_config) = &config.front_touch_config {
            self.config.front_touch_config = front_touch_config.clone();
        }

        if let Some(rear_touch_config) = &config.rear_touch_config {
            self.config.rear_touch_config = rear_touch_config.clone();
        }

        if let Some(trigger_config) = config.trigger_config {
            self.config.trigger_config = trigger_config;
        }

        Ok(())
    }

    #[inline]
    fn f32_to_i16(value: f32, min_value: f32, max_value: f32) -> i16 {
        let clamped_value = value.clamp(min_value, max_value);
        let normalized_value = (clamped_value - min_value) / (max_value - min_value);
        let scaled_value = normalized_value * 65536.0 - 32768.0;
        scaled_value.round() as i16
    }

    fn send_report(&mut self, report: vita_reports::MainReport) -> crate::Result<()> {
        let dpad = match (
            report.buttons.down,
            report.buttons.left,
            report.buttons.up,
            report.buttons.right,
        ) {
            (true, false, false, false) => DpadDirection::South,
            (true, true, false, false) => DpadDirection::SouthWest,
            (false, true, false, false) => DpadDirection::West,
            (false, true, true, false) => DpadDirection::NorthWest,
            (false, false, true, false) => DpadDirection::North,
            (false, false, true, true) => DpadDirection::NorthEast,
            (false, false, false, true) => DpadDirection::East,
            (true, false, false, true) => DpadDirection::SouthEast,
            _ => DpadDirection::None,
        };

        let mut buttons = DS4Buttons::new()
            .circle(report.buttons.circle)
            .square(report.buttons.square)
            .cross(report.buttons.cross)
            .triangle(report.buttons.triangle)
            .options(report.buttons.start)
            .share(report.buttons.select)
            .dpad(dpad);

        for touch in &report.front_touch.reports {
            if let Some(TouchConfig::Zones(zones)) = &self.config.front_touch_config {
                if let Some(zone) = zones.locate_at_point(&Point(touch.x.into(), touch.y.into())) {
                    if let Some(action) = zone.action {
                        match action {
                            TouchAction::Button(button) => buttons |= button,
                            _ => {}
                        }
                    }
                }
            }
        }

        for touch in &report.back_touch.reports {
            if let Some(TouchConfig::Zones(zones)) = &self.config.rear_touch_config {
                if let Some(zone) = zones.locate_at_point(&Point(touch.x.into(), touch.y.into())) {
                    if let Some(action) = zone.action {
                        match action {
                            TouchAction::Button(button) => buttons |= button,
                            _ => {}
                        }
                    }
                }
            }
        }

        let mut pwr_trigger_l: u8 = 0;
        let mut pwr_trigger_r: u8 = 0;
        match self.config.trigger_config {
            TriggerConfig::Shoulder => {
                if report.buttons.lt {
                    buttons |= DS4Buttons::SHOULDER_LEFT;
                }
                if report.buttons.rt {
                    buttons |= DS4Buttons::SHOULDER_RIGHT;
                }
            }
            TriggerConfig::Trigger => {
                if report.buttons.lt {
                    buttons |= DS4Buttons::TRIGGER_LEFT;
                    pwr_trigger_l = 255;
                }
                if report.buttons.rt {
                    buttons |= DS4Buttons::TRIGGER_RIGHT;
                    pwr_trigger_r = 255;
                }
            }
        }

        let mut is_touching = false;
        let touchpad = if let Some(TouchConfig::Touchpad) = self.config.front_touch_config {
            is_touching = !report.front_touch.reports.is_empty();
            let mut points = report
                .front_touch
                .reports
                .iter()
                // Convert the coordinates to the range for the dualshock 4 touchpad (1920x942) from the vita touchpad
                .map(|report| {
                    DS4TouchPoint::new(
                        report.x as u16,
                        (report.y as f32 * (942.0 / FRONT_TOUCHPAD_RECT.1.y() as f32)) as u16)
                });
            let report = DS4TouchReport::new(0, points.next(), points.next());
            Some(report)
        } else if let Some(TouchConfig::Touchpad) = self.config.rear_touch_config {
            is_touching = !report.back_touch.reports.is_empty();
            let mut points = report
                .back_touch
                .reports
                .iter()
                // Convert the coordinates to the range for the dualshock 4 touchpad (1920x942) from the vita rear touchpad
                .map(|report| {
                    DS4TouchPoint::new(
                        report.x as u16,
                        (report.y as f32 * (942.0 / REAR_TOUCHPAD_RECT.1.y() as f32)) as u16
                    )
                });
            let report = DS4TouchReport::new(0, points.next(), points.next());
            Some(report)
        } else {
            None
        };
        let mut special_buttons = DS4SpecialButtons::new();

        if is_touching && !self.touch_state {
            self.touch_start_time = Some(Instant::now());
        } else if !is_touching && self.touch_state {
            if let Some(start_time) = self.touch_start_time {
                let duration = Instant::now().duration_since(start_time);
                if duration < Duration::from_millis(150) {
                    special_buttons = special_buttons.touchpad(true);
                }
            }
            self.touch_start_time = None;
        }

        self.touch_state = is_touching;

        // Convert the vita accel range [-4.0, 4.0] to the dualshock 4 range [-32768, 32768]
        let accel_x_i16 = Self::f32_to_i16(-report.motion.accelerometer.x, -4.0, 4.0); //inverted
        let accel_y_i16 = Self::f32_to_i16(report.motion.accelerometer.y, -4.0, 4.0);
        let accel_z_i16 = Self::f32_to_i16(-report.motion.accelerometer.z, -4.0, 4.0); //inverted

        // Convert the vita gyro range [-35.0, 35.0] to the dualshock 4 range [-32768, 32768]
        let gyro_x_i16 =  Self::f32_to_i16(report.motion.gyro.x, -35.0, 35.0);
        let gyro_y_i16 =  Self::f32_to_i16(-report.motion.gyro.y, -35.0, 35.0); //inverted
        let gyro_z_i16 =  Self::f32_to_i16(report.motion.gyro.z, -35.0, 35.0);

        let report = DS4ReportExBuilder::new()
            .thumb_lx(report.lx)
            .thumb_ly(report.ly)
            .thumb_rx(report.rx)
            .thumb_ry(report.ry)
            .buttons(buttons)
            .touch_reports(touchpad, None, None)
            .gyro_x(gyro_x_i16)
            .gyro_y(gyro_z_i16)
            .gyro_z(gyro_y_i16)
            .accel_x(accel_x_i16)
            .accel_y(accel_z_i16)
            .accel_z(accel_y_i16)
            .trigger_l(pwr_trigger_l)
            .trigger_r(pwr_trigger_r)
            .status(DS4Status::with_battery_status(
                BatteryStatus::Charging((report.charge_percent / 10).min(10))
            ))
            .special(special_buttons)
            .build();

        self.ds4_target
            .update_ex(&report)
            .map_err(Error::SendReportFailed)?;

        Ok(())
    }
}
