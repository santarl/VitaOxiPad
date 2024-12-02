use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{File, OpenOptions},
    io::Write,
    os::fd::AsRawFd,
};

use std::time::{SystemTime, UNIX_EPOCH};

use input_linux::{
    sys::{input_event, BUS_VIRTUAL},
    AbsoluteAxis, AbsoluteEvent, AbsoluteInfo, AbsoluteInfoSetup, EventKind, EventTime, InputEvent,
    InputId, InputProperty, Key, KeyEvent, KeyState, SynchronizeEvent, UInputHandle,
};

use crate::virtual_button::{Button, DpadDirection};
use crate::virtual_config::{Config, ConfigBuilder, TouchConfig, TouchpadSource};
use crate::virtual_touch::{Point, TouchAction};
use crate::virtual_utils::{compute_dpad_direction, get_pressed_buttons};
use crate::{f32_to_i16, VitaVirtualDevice, FRONT_TOUCHPAD_RECT, REAR_TOUCHPAD_RECT};

type TrackingId = u8;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to create uinput device")]
    DeviceCreationFailed(#[source] std::io::Error),
    #[error("Failed to write uinput device event")]
    WriteEventFailed(#[source] std::io::Error),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

// Constants for front touchpad
const FRONT_TOUCHPAD_MAX_X: i32 = FRONT_TOUCHPAD_RECT.1 .0 - 1;
const FRONT_TOUCHPAD_MAX_Y: i32 = FRONT_TOUCHPAD_RECT.1 .1 - 1;
const FRONT_TOUCHPAD_MAX_SLOTS: usize = 6;

// Constants for rear touchpad
const REAR_TOUCHPAD_MAX_X: i32 = REAR_TOUCHPAD_RECT.1 .0 - 1;
const REAR_TOUCHPAD_MAX_Y: i32 = REAR_TOUCHPAD_RECT.1 .1 - 1;
const REAR_TOUCHPAD_MAX_SLOTS: usize = 4;

fn get_current_event_time() -> EventTime {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    EventTime::new(now.as_secs() as i64, now.subsec_nanos() as i64)
}

fn map_button_to_ds4(button: Button) -> Key {
    match button {
        Button::ThumbRight => Key::ButtonThumbr,
        Button::ThumbLeft => Key::ButtonThumbl,
        Button::Options => Key::ButtonStart,
        Button::Share => Key::ButtonSelect,
        Button::TriggerRight => Key::ButtonTR,
        Button::TriggerLeft => Key::ButtonTL,
        Button::ShoulderRight => Key::ButtonTR2,
        Button::ShoulderLeft => Key::ButtonTL2,
        Button::Triangle => Key::ButtonNorth,
        Button::Circle => Key::ButtonEast,
        Button::Cross => Key::ButtonSouth,
        Button::Square => Key::ButtonWest,
    }
}

/// Converts DpadDirection to axis values suitable for uinput.
pub fn dpad_direction_to_axis_values(direction: DpadDirection) -> (i32, i32) {
    match direction {
        DpadDirection::North => (0, -1),
        DpadDirection::NorthEast => (1, -1),
        DpadDirection::East => (1, 0),
        DpadDirection::SouthEast => (1, 1),
        DpadDirection::South => (0, 1),
        DpadDirection::SouthWest => (-1, 1),
        DpadDirection::West => (-1, 0),
        DpadDirection::NorthWest => (-1, -1),
        DpadDirection::None => (0, 0),
    }
}

/// Processes touch reports and returns a list of touch actions.
pub fn process_touch_reports(
    touch_reports: &[vita_reports::TouchReport],
    touch_config: &Option<TouchConfig>,
) -> Vec<TouchAction> {
    let mut actions = Vec::new();
    if let Some(TouchConfig::Zones(zones)) = touch_config {
        for touch in touch_reports {
            if let Some(zone) = zones.locate_at_point(&Point(touch.x.into(), touch.y.into())) {
                if let Some(action) = zone.action {
                    actions.push(action);
                }
            }
        }
    }
    actions
}

pub struct VitaDevice<F: AsRawFd> {
    config: Config,
    main_handle: UInputHandle<F>,
    touchpad_handle: UInputHandle<F>,
    sensor_handle: UInputHandle<F>,
    keyboard_handle: UInputHandle<F>,
    previous_front_touches: Vec<Option<TrackingId>>,
    previous_rear_touches: Vec<Option<TrackingId>>,
    touch_state: bool,
    ids: Option<Vec<OsString>>,
    previous_buttons: HashSet<Button>,
    previous_hat_x: i32,
    previous_hat_y: i32,
}

impl<F: AsRawFd> VitaDevice<F> {
    pub fn new(
        uinput_file: F,
        uinput_sensor_file: F,
        uinput_touchpad_file: F,
        uinput_keyboard_file: F,
        config: Config,
    ) -> std::io::Result<Self> {
        let main_handle = UInputHandle::new(uinput_file);
        let id = InputId {
            bustype: BUS_VIRTUAL,
            vendor: 0x054C,
            product: 0x9CC,
            version: 0x8111,
        };

        // Configure main device
        main_handle.set_evbit(EventKind::Key)?;
        for button in [
            Button::ThumbRight,
            Button::ThumbLeft,
            Button::Options,
            Button::Share,
            Button::TriggerRight,
            Button::TriggerLeft,
            Button::ShoulderRight,
            Button::ShoulderLeft,
            Button::Triangle,
            Button::Circle,
            Button::Cross,
            Button::Square,
        ] {
            main_handle.set_keybit(map_button_to_ds4(button))?;
        }
        main_handle.set_evbit(EventKind::Absolute)?;

        let joystick_abs_info = AbsoluteInfo {
            flat: 128,
            fuzz: 0,
            maximum: 255,
            minimum: 0,
            resolution: 255,
            ..Default::default()
        };

        let dpad_info = AbsoluteInfo {
            fuzz: 0,
            maximum: 1,
            minimum: -1,
            resolution: 3,
            ..Default::default()
        };

        // Sticks
        let joystick_axes = [
            AbsoluteInfoSetup {
                info: joystick_abs_info,
                axis: AbsoluteAxis::X,
            },
            AbsoluteInfoSetup {
                info: joystick_abs_info,
                axis: AbsoluteAxis::Y,
            },
            AbsoluteInfoSetup {
                info: joystick_abs_info,
                axis: AbsoluteAxis::RX,
            },
            AbsoluteInfoSetup {
                info: joystick_abs_info,
                axis: AbsoluteAxis::RY,
            },
        ];

        // Dpad
        let dpad_axes = [
            AbsoluteInfoSetup {
                info: dpad_info,
                axis: AbsoluteAxis::Hat0Y,
            },
            AbsoluteInfoSetup {
                info: dpad_info,
                axis: AbsoluteAxis::Hat0X,
            },
        ];

        // Combine axes into a single vector
        let axes: Vec<AbsoluteInfoSetup> = joystick_axes
            .iter()
            .cloned()
            .chain(dpad_axes.iter().cloned())
            .collect();

        main_handle.create(&id, b"PS Vita VitaOxiPad", 0, &axes)?;

        // Configure touchpad device
        let touchpad_handle = UInputHandle::new(uinput_touchpad_file);

        touchpad_handle.set_evbit(EventKind::Key)?;
        touchpad_handle.set_evbit(EventKind::Absolute)?;
        touchpad_handle.set_evbit(EventKind::Relative)?;
        touchpad_handle.set_propbit(InputProperty::Pointer)?;
        touchpad_handle.set_propbit(InputProperty::ButtonPad)?;
        touchpad_handle.set_keybit(Key::ButtonTouch)?;
        touchpad_handle.set_keybit(Key::ButtonToolFinger)?;
        touchpad_handle.set_keybit(Key::ButtonLeft)?;

        let (max_x, max_y, max_slots) = match config.touchpad_source {
            Some(TouchpadSource::Front) | None => (
                FRONT_TOUCHPAD_MAX_X,
                FRONT_TOUCHPAD_MAX_Y,
                FRONT_TOUCHPAD_MAX_SLOTS,
            ),
            Some(TouchpadSource::Rear) => (
                REAR_TOUCHPAD_MAX_X,
                REAR_TOUCHPAD_MAX_Y,
                REAR_TOUCHPAD_MAX_SLOTS,
            ),
        };

        // Set up touchpad axes based on the determined parameters
        let touchpad_axes = [
            AbsoluteInfoSetup {
                info: AbsoluteInfo {
                    minimum: 0,
                    maximum: max_x,
                    ..Default::default()
                },
                axis: AbsoluteAxis::MultitouchPositionX,
            },
            AbsoluteInfoSetup {
                info: AbsoluteInfo {
                    minimum: 0,
                    maximum: max_y,
                    ..Default::default()
                },
                axis: AbsoluteAxis::MultitouchPositionY,
            },
            AbsoluteInfoSetup {
                info: AbsoluteInfo {
                    minimum: 0,
                    maximum: max_x,
                    ..Default::default()
                },
                axis: AbsoluteAxis::X,
            },
            AbsoluteInfoSetup {
                info: AbsoluteInfo {
                    minimum: 0,
                    maximum: max_y,
                    ..Default::default()
                },
                axis: AbsoluteAxis::Y,
            },
            AbsoluteInfoSetup {
                info: AbsoluteInfo {
                    minimum: -1,
                    maximum: 128,
                    ..Default::default()
                },
                axis: AbsoluteAxis::MultitouchTrackingId,
            },
            AbsoluteInfoSetup {
                info: AbsoluteInfo {
                    minimum: 0,
                    maximum: (max_slots - 1) as i32,
                    ..Default::default()
                },
                axis: AbsoluteAxis::MultitouchSlot,
            },
            AbsoluteInfoSetup {
                info: AbsoluteInfo {
                    minimum: 0,
                    maximum: 128,
                    ..Default::default()
                },
                axis: AbsoluteAxis::MultitouchPressure,
            },
        ];

        touchpad_handle.create(&id, b"PS Vita VitaOxiPad (Touchpad)", 0, &touchpad_axes)?;

        // Configure sensor device
        let sensor_handle = UInputHandle::new(uinput_sensor_file);

        sensor_handle.set_evbit(EventKind::Absolute)?;
        sensor_handle.set_propbit(InputProperty::Accelerometer)?;

        let accel_abs_info = AbsoluteInfo {
            minimum: -32768,
            maximum: 32768,
            ..Default::default()
        };

        let gyro_abs_info = accel_abs_info;

        let sensor_axes = [
            AbsoluteInfoSetup {
                info: accel_abs_info,
                axis: AbsoluteAxis::X,
            },
            AbsoluteInfoSetup {
                info: accel_abs_info,
                axis: AbsoluteAxis::Y,
            },
            AbsoluteInfoSetup {
                info: accel_abs_info,
                axis: AbsoluteAxis::Z,
            },
            AbsoluteInfoSetup {
                info: gyro_abs_info,
                axis: AbsoluteAxis::RX,
            },
            AbsoluteInfoSetup {
                info: gyro_abs_info,
                axis: AbsoluteAxis::RY,
            },
            AbsoluteInfoSetup {
                info: gyro_abs_info,
                axis: AbsoluteAxis::RZ,
            },
        ];

        sensor_handle.create(&id, b"PS Vita VitaOxiPad (Motion Sensors)", 0, &sensor_axes)?;

        let ids = main_handle
            .evdev_name()
            .ok()
            .zip(touchpad_handle.evdev_name().ok())
            .zip(sensor_handle.evdev_name().ok())
            .map(|((main, touchpad), sensor)| [main, touchpad, sensor].to_vec());

        Ok(VitaDevice {
            config,
            main_handle,
            touchpad_handle,
            sensor_handle,
            previous_front_touches: vec![None; FRONT_TOUCHPAD_MAX_SLOTS],
            previous_rear_touches: vec![None; REAR_TOUCHPAD_MAX_SLOTS],
            touch_state: false,
            ids,
            previous_buttons: HashSet::new(),
            previous_hat_x: 0,
            previous_hat_y: 0,
        })
    }
}

impl VitaDevice<File> {
    pub fn create(config_name: &str) -> crate::Result<Self> {
        // Select the configuration depending on the name
        let config = match config_name {
            "standart" => Config::rear_rl2_front_rl3(),
            "alt_triggers" => Config::rear_rl1_front_rl3_vitatriggers_rl2(),
            "rear_touchpad" => Config::front_top_rl2_bottom_rl3_rear_touchpad(),
            "front_touchpad" => Config::rear_top_rl2_bottom_rl3_front_touchpad(),
            _ => {
                return Err(crate::Error::Linux(Error::InvalidConfig(
                    config_name.to_string(),
                )))
            }
        };

        let uinput_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")
            .map_err(Error::DeviceCreationFailed)?;

        let uinput_sensor_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")
            .map_err(Error::DeviceCreationFailed)?;

        let uinput_touchpad_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")
            .map_err(Error::DeviceCreationFailed)?;

        let device = Self::new(
            uinput_file,
            uinput_sensor_file,
            uinput_touchpad_file,
            config,
        )
        .map_err(Error::DeviceCreationFailed)?;

        Ok(device)
    }
}

impl<F: AsRawFd + Write> VitaVirtualDevice<&ConfigBuilder> for VitaDevice<F> {
    type Config = Config;

    fn identifiers(&self) -> Option<&[OsString]> {
        self.ids.as_ref().map(|ids| ids.as_slice())
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

    fn send_report(&mut self, report: vita_reports::MainReport) -> crate::Result<()> {
        let syn_event = *SynchronizeEvent::report(get_current_event_time())
            .as_event()
            .as_raw();

        // Calculate D-Pad direction
        let dpad_direction = compute_dpad_direction(&report.buttons);
        let (mut hat_x_value, mut hat_y_value) = dpad_direction_to_axis_values(dpad_direction);

        // Get pressed buttons
        let pressed_buttons = get_pressed_buttons(&report.buttons, self.config.trigger_config);
        let mut pressed_buttons_set: HashSet<Button> = pressed_buttons.iter().cloned().collect();

        // Process touch actions
        let front_touch_actions =
            process_touch_reports(&report.front_touch.reports, &self.config.front_touch_config);
        let rear_touch_actions =
            process_touch_reports(&report.back_touch.reports, &self.config.rear_touch_config);

        for action in front_touch_actions.into_iter().chain(rear_touch_actions) {
            match action {
                TouchAction::Button(button) => {
                    pressed_buttons_set.insert(button);
                }
                TouchAction::Dpad(direction) => {
                    let (x, y) = dpad_direction_to_axis_values(direction);
                    hat_x_value = x;
                    hat_y_value = y;
                }
            }
        }

        // Compute buttons to press and release
        let buttons_to_press: Vec<Button> = pressed_buttons_set
            .difference(&self.previous_buttons)
            .cloned()
            .collect();
        let buttons_to_release: Vec<Button> = self
            .previous_buttons
            .difference(&pressed_buttons_set)
            .cloned()
            .collect();

        // Create button press events
        let button_press_events: Vec<InputEvent> = buttons_to_press
            .iter()
            .map(|&button| {
                KeyEvent::new(
                    get_current_event_time(),
                    map_button_to_ds4(button),
                    KeyState::PRESSED,
                )
                .into()
            })
            .collect();

        // Create button release events
        let button_release_events: Vec<InputEvent> = buttons_to_release
            .iter()
            .map(|&button| {
                KeyEvent::new(
                    get_current_event_time(),
                    map_button_to_ds4(button),
                    KeyState::RELEASED,
                )
                .into()
            })
            .collect();

        // Update previous_buttons
        self.previous_buttons = pressed_buttons_set;

        // Create D-Pad events if values have changed
        let mut dpad_events = Vec::new();
        if hat_x_value != self.previous_hat_x {
            dpad_events
                .push(AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::Hat0X, hat_x_value).into());
            self.previous_hat_x = hat_x_value;
        }
        if hat_y_value != self.previous_hat_y {
            dpad_events
                .push(AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::Hat0Y, hat_y_value).into());
            self.previous_hat_y = hat_y_value;
        }

        // Create stick events (always send)
        let stick_events = create_stick_events(&report);

        // Write main device events
        let events: Vec<input_event> = button_press_events
            .iter()
            .chain(button_release_events.iter())
            .chain(dpad_events.iter())
            .chain(stick_events.iter())
            .map(|ev| (*ev).into())
            .map(|ev: InputEvent| *ev.as_raw())
            .collect();

        self.main_handle
            .write(&events)
            .map_err(Error::WriteEventFailed)?;
        self.main_handle
            .write(&[syn_event])
            .map_err(Error::WriteEventFailed)?;

        match self.config.touchpad_source {
            Some(TouchpadSource::Front) => {
                // Handle front touch events
                let touch_events = create_touch_events(
                    &report.front_touch.reports,
                    &mut self.previous_front_touches,
                    6,
                    &mut self.touch_state,
                );

                let events: Vec<input_event> = touch_events
                    .iter()
                    .map(|ev| (*ev).into())
                    .map(|ev: InputEvent| *ev.as_raw())
                    .collect();

                self.touchpad_handle
                    .write(&events)
                    .map_err(Error::WriteEventFailed)?;
                self.touchpad_handle
                    .write(&[syn_event])
                    .map_err(Error::WriteEventFailed)?;
            }
            Some(TouchpadSource::Rear) => {
                // Handle rear touch events
                let touch_events = create_touch_events(
                    &report.back_touch.reports,
                    &mut self.previous_rear_touches,
                    4,
                    &mut self.touch_state,
                );

                let events: Vec<input_event> = touch_events
                    .iter()
                    .map(|ev| (*ev).into())
                    .map(|ev: InputEvent| *ev.as_raw())
                    .collect();

                self.touchpad_handle
                    .write(&events)
                    .map_err(Error::WriteEventFailed)?;
                self.touchpad_handle
                    .write(&[syn_event])
                    .map_err(Error::WriteEventFailed)?;
            }
            None => {
                // Do nothing
            }
        }
        // Handle motion sensor events
        let motion_events = create_motion_events(&report);

        let events: Vec<input_event> = motion_events
            .iter()
            .map(|ev| (*ev).into())
            .map(|ev: InputEvent| *ev.as_raw())
            .collect();

        self.sensor_handle
            .write(&events)
            .map_err(Error::WriteEventFailed)?;
        self.sensor_handle
            .write(&[syn_event])
            .map_err(Error::WriteEventFailed)?;

        Ok(())
    }
}

fn create_motion_events(report: &vita_reports::MainReport) -> Vec<InputEvent> {
    // Convert the vita accel range [-4.0, 4.0] to the range [-32768, 32768]
    let accel_x_i16 = f32_to_i16(-report.motion.accelerometer.x, -4.0, 4.0); // inverted
    let accel_y_i16 = f32_to_i16(report.motion.accelerometer.y, -4.0, 4.0);
    let accel_z_i16 = f32_to_i16(-report.motion.accelerometer.z, -4.0, 4.0); // inverted

    // Convert the vita gyro range [-35.0, 35.0] to the range [-32768, 32768]
    let gyro_x_i16 = f32_to_i16(report.motion.gyro.x, -35.0, 35.0);
    let gyro_y_i16 = f32_to_i16(-report.motion.gyro.y, -35.0, 35.0); // inverted
    let gyro_z_i16 = f32_to_i16(report.motion.gyro.z, -35.0, 35.0);

    vec![
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::X, accel_x_i16 as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::Y, accel_z_i16 as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::Z, accel_y_i16 as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::RX, gyro_x_i16 as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::RY, gyro_z_i16 as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::RZ, gyro_y_i16 as i32).into(),
    ]
}

fn create_stick_events(report: &vita_reports::MainReport) -> Vec<InputEvent> {
    vec![
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::X, report.lx as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::Y, report.ly as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::RX, report.rx as i32).into(),
        AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::RY, report.ry as i32).into(),
    ]
}

fn create_touch_events(
    touch_reports: &[vita_reports::TouchReport],
    previous_touches: &mut [Option<TrackingId>],
    max_slots: usize,
    touch_state: &mut bool,
) -> Vec<InputEvent> {
    let mut events = Vec::new();

    // Reset slots where touches have ended
    for slot in 0..max_slots {
        let old_id = previous_touches[slot];
        let new_id = touch_reports.get(slot).map(|r| r.id);

        if old_id.is_some() && new_id.is_none() {
            events.push(
                AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::MultitouchSlot, slot as i32)
                    .into(),
            );
            events.push(
                AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::MultitouchTrackingId, -1).into(),
            );
            previous_touches[slot] = None;
        }
    }

    let mut any_touch_active = false;

    // Update slots with current touches
    for (slot, touch) in touch_reports.iter().enumerate() {
        if slot >= max_slots {
            break;
        }
        events.push(
            AbsoluteEvent::new(get_current_event_time(), AbsoluteAxis::MultitouchSlot, slot as i32).into(),
        );
        events.push(
            AbsoluteEvent::new(
                get_current_event_time(),
                AbsoluteAxis::MultitouchTrackingId,
                touch.id as i32,
            )
            .into(),
        );
        events.push(
            AbsoluteEvent::new(
                get_current_event_time(),
                AbsoluteAxis::MultitouchPositionX,
                touch.x as i32,
            )
            .into(),
        );
        events.push(
            AbsoluteEvent::new(
                get_current_event_time(),
                AbsoluteAxis::MultitouchPositionY,
                touch.y as i32,
            )
            .into(),
        );
        events.push(
            AbsoluteEvent::new(
                get_current_event_time(),
                AbsoluteAxis::MultitouchPressure,
                touch.force as i32,
            )
            .into(),
        );

        if touch.force > 0 {
            any_touch_active = true;
        }

        previous_touches[slot] = Some(touch.id);
    }

    if any_touch_active && !*touch_state {
        // Touch started
        events.push(KeyEvent::new(get_current_event_time(), Key::ButtonTouch, KeyState::PRESSED).into());
        events
            .push(KeyEvent::new(get_current_event_time(), Key::ButtonToolFinger, KeyState::PRESSED).into());
        *touch_state = true;
    } else if !any_touch_active && *touch_state {
        // Touch ended
        events.push(KeyEvent::new(get_current_event_time(), Key::ButtonTouch, KeyState::RELEASED).into());
        events
            .push(KeyEvent::new(get_current_event_time(), Key::ButtonToolFinger, KeyState::RELEASED).into());
        *touch_state = false;
    }

    events
}
