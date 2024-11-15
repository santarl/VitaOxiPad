use std::time::Duration;
use std::time::Instant;

use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::Write,
    os::fd::AsRawFd,
};

use input_linux::{
    bitmask::BitmaskTrait,
    sys::{input_event, BUS_VIRTUAL},
    AbsoluteAxis, AbsoluteEvent, AbsoluteInfo, AbsoluteInfoSetup, EventKind, EventTime, InputEvent,
    InputId, InputProperty, Key, KeyEvent, KeyState, SynchronizeEvent, UInputHandle,
};

use crate::virtual_button::{Button, DpadDirection};
use crate::virtual_config::{Config, ConfigBuilder, TouchConfig, TriggerConfig};
use crate::virtual_touch::{Point, TouchAction};
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

fn map_button_to_ds4(button: Button) -> Key {
    match button {
        Button::ThumbRight => Key::ButtonThumbr,
        Button::ThumbLeft => Key::ButtonThumbl,
        Button::Options => Key::ButtonSelect,
        Button::Share => Key::ButtonStart,
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

pub struct VitaDevice<F: AsRawFd> {
    config: Config,
    main_handle: UInputHandle<F>,
    touchpad_handle: UInputHandle<F>,
    sensor_handle: UInputHandle<F>,
    previous_front_touches: [Option<TrackingId>; 6],
    previous_back_touches: [Option<TrackingId>; 4],
    ids: Option<Vec<OsString>>,
}

impl<F: AsRawFd> VitaDevice<F> {
    pub fn new(
        uinput_file: F,
        uinput_sensor_file: F,
        uinput_touchpad_file: F,
        config: Config,
    ) -> std::io::Result<Self> {
        let main_handle = UInputHandle::new(uinput_file);
        let id = InputId {
            bustype: BUS_VIRTUAL,
            vendor: 0x054C,
            product: 0x9CC,
            version: 0x8111,
        };

        main_handle.set_evbit(EventKind::Key)?;
        main_handle.set_keybit(map_button_to_ds4(Button::ThumbRight))?;
        main_handle.set_keybit(map_button_to_ds4(Button::ThumbLeft))?;
        main_handle.set_keybit(map_button_to_ds4(Button::Options))?;
        main_handle.set_keybit(map_button_to_ds4(Button::Share))?;
        main_handle.set_keybit(map_button_to_ds4(Button::TriggerRight))?;
        main_handle.set_keybit(map_button_to_ds4(Button::TriggerLeft))?;
        main_handle.set_keybit(map_button_to_ds4(Button::ShoulderRight))?;
        main_handle.set_keybit(map_button_to_ds4(Button::ShoulderLeft))?;
        main_handle.set_keybit(map_button_to_ds4(Button::Triangle))?;
        main_handle.set_keybit(map_button_to_ds4(Button::Circle))?;
        main_handle.set_keybit(map_button_to_ds4(Button::Cross))?;
        main_handle.set_keybit(map_button_to_ds4(Button::Square))?;
        main_handle.set_evbit(EventKind::Absolute)?;

        let joystick_abs_info = AbsoluteInfo {
            flat: 128,
            fuzz: 0, // Already fuzzed
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
        let joystick_x_info = AbsoluteInfoSetup {
            info: joystick_abs_info,
            axis: AbsoluteAxis::X,
        };
        let joystick_y_info = AbsoluteInfoSetup {
            info: joystick_abs_info,
            axis: AbsoluteAxis::Y,
        };
        let joystick_rx_info = AbsoluteInfoSetup {
            info: joystick_abs_info,
            axis: AbsoluteAxis::RX,
        };
        let joystick_ry_info = AbsoluteInfoSetup {
            info: joystick_abs_info,
            axis: AbsoluteAxis::RY,
        };

        // Dpad
        let dpad_up_down = AbsoluteInfoSetup {
            info: dpad_info,
            axis: AbsoluteAxis::Hat0Y,
        };
        let dpad_left_right = AbsoluteInfoSetup {
            info: dpad_info,
            axis: AbsoluteAxis::Hat0X,
        };

        main_handle.create(
            &id,
            b"PS Vita VitaOxiPad",
            0,
            &[
                joystick_x_info,
                joystick_y_info,
                joystick_rx_info,
                joystick_ry_info,
                dpad_up_down,
                dpad_left_right,
            ],
        )?;

        let touchpad_handle = UInputHandle::new(uinput_touchpad_file);

        touchpad_handle.set_evbit(EventKind::Key)?;
        touchpad_handle.set_evbit(EventKind::Absolute)?;
        touchpad_handle.set_evbit(EventKind::Relative)?;
        touchpad_handle.set_propbit(InputProperty::Pointer)?;
        touchpad_handle.set_propbit(InputProperty::ButtonPad)?;
        touchpad_handle.set_keybit(Key::ButtonTouch)?;
        touchpad_handle.set_keybit(Key::ButtonToolFinger)?;
        touchpad_handle.set_keybit(Key::ButtonLeft)?;

        // Touchscreen (front)
        let front_mt_x_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: FRONT_TOUCHPAD_RECT.1 .0 - 1,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchPositionX,
        };
        let front_mt_y_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: FRONT_TOUCHPAD_RECT.1 .1 - 1,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchPositionY,
        };
        let front_abs_x_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: FRONT_TOUCHPAD_RECT.1 .0 - 1,
                ..Default::default()
            },
            axis: AbsoluteAxis::X,
        };
        let front_abs_y_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: FRONT_TOUCHPAD_RECT.1 .1 - 1,
                ..Default::default()
            },
            axis: AbsoluteAxis::Y,
        };
        let front_mt_id_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: 128,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchTrackingId,
        }; //TODO: Query infos
        let front_mt_slot_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: 5,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchSlot,
        }; // According to vitasdk docs
        let front_mt_pressure_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 1,
                maximum: 128,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchPressure,
        }; //TODO: Query infos

        touchpad_handle.create(
            &id,
            b"PS Vita VitaOxiPad (Touchpad)",
            0,
            &[
                front_mt_x_info,
                front_mt_y_info,
                front_abs_x_info,
                front_abs_y_info,
                front_mt_id_info,
                front_mt_slot_info,
                front_mt_pressure_info,
            ],
        )?;

        // Have to create another device because sensors can't be mixed with directional axes
        // and we can't assign the back touch surface along with the touchscreen.
        // So this second device contains info for the motion sensors and the back touch surface.
        let sensor_handle = UInputHandle::new(uinput_sensor_file);

        sensor_handle.set_evbit(EventKind::Absolute)?;
        sensor_handle.set_propbit(InputProperty::Accelerometer)?;

        let accel_abs_info = AbsoluteInfo {
            minimum: -16,
            maximum: 16,
            ..Default::default()
        };
        let accel_x_info = AbsoluteInfoSetup {
            info: accel_abs_info,
            axis: AbsoluteAxis::X,
        };
        let accel_y_info = AbsoluteInfoSetup {
            info: accel_abs_info,
            axis: AbsoluteAxis::Y,
        };
        let accel_z_info = AbsoluteInfoSetup {
            info: accel_abs_info,
            axis: AbsoluteAxis::Z,
        };

        let gyro_abs_info = AbsoluteInfo {
            minimum: -1,
            maximum: 1,
            ..Default::default()
        };
        let gyro_x_info = AbsoluteInfoSetup {
            info: gyro_abs_info,
            axis: AbsoluteAxis::RX,
        };
        let gyro_y_info = AbsoluteInfoSetup {
            info: gyro_abs_info,
            axis: AbsoluteAxis::RY,
        };
        let gyro_z_info = AbsoluteInfoSetup {
            info: gyro_abs_info,
            axis: AbsoluteAxis::RZ,
        };

        let mt_x_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: REAR_TOUCHPAD_RECT.1 .0 - 1,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchPositionX,
        };
        let mt_y_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: REAR_TOUCHPAD_RECT.1 .1 - 1,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchPositionY,
        };
        let mt_id_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: 128,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchTrackingId,
        };
        let mt_slot_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 0,
                maximum: 3,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchSlot,
        };
        let mt_pressure_info = AbsoluteInfoSetup {
            info: AbsoluteInfo {
                minimum: 1,
                maximum: 128,
                ..Default::default()
            },
            axis: AbsoluteAxis::MultitouchPressure,
        };

        sensor_handle.create(
            &id,
            b"PS Vita VitaOxiPad (Motion Sensors)",
            0,
            &[
                accel_x_info,
                accel_y_info,
                accel_z_info,
                gyro_x_info,
                gyro_y_info,
                gyro_z_info,
                mt_x_info,
                mt_y_info,
                mt_id_info,
                mt_slot_info,
                mt_pressure_info,
            ],
        )?;

        let ids = main_handle
            .evdev_name()
            .ok()
            .zip(touchpad_handle.evdev_name().ok())
            .zip(sensor_handle.evdev_name().ok())
            .map(|((main, touchpad), sensor)| [main, touchpad, sensor].to_vec());

        Ok(VitaDevice {
            config: config,
            main_handle,
            touchpad_handle,
            sensor_handle,
            previous_front_touches: [None; 6],
            previous_back_touches: [None; 4],
            ids,
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
        const EVENT_TIME_ZERO: EventTime = EventTime::new(0, 0);
        let syn_event = *SynchronizeEvent::report(EVENT_TIME_ZERO)
            .as_event()
            .as_raw();

        macro_rules! ex_key_event {
            ($report:ident, $report_name:ident, $uinput_name:expr) => {
                KeyEvent::new(
                    EVENT_TIME_ZERO,
                    $uinput_name,
                    KeyState::pressed($report.buttons.$report_name),
                )
            };
        }

        macro_rules! touch_key_event {
            ($report:ident, $uinput_name:ident) => {
                KeyEvent::new(
                    EVENT_TIME_ZERO,
                    Key::$uinput_name,
                    KeyState::pressed($report.force > 0),
                )
            };
        }

        macro_rules! dpad_event {
            ($report:ident, Hat0Y) => {
                AbsoluteEvent::new(
                    EVENT_TIME_ZERO,
                    AbsoluteAxis::Hat0Y,
                    if $report.buttons.up {
                        -1
                    } else if $report.buttons.down {
                        1
                    } else {
                        0
                    },
                )
            };
            ($report:ident, Hat0X) => {
                AbsoluteEvent::new(
                    EVENT_TIME_ZERO,
                    AbsoluteAxis::Hat0X,
                    if $report.buttons.right {
                        1
                    } else if $report.buttons.left {
                        -1
                    } else {
                        0
                    },
                )
            };
        }

        macro_rules! stick_event {
            ($report:ident, $report_name:ident, $uinput_name:ident) => {
                AbsoluteEvent::new(
                    EVENT_TIME_ZERO,
                    AbsoluteAxis::$uinput_name,
                    $report.$report_name.into(),
                )
            };
        }

        macro_rules! mt_event {
            ($report:ident, $report_name:ident, $uinput_name:ident) => {
                AbsoluteEvent::new(
                    EVENT_TIME_ZERO,
                    AbsoluteAxis::$uinput_name,
                    $report.$report_name.into(),
                )
            };
        }

        macro_rules! accel_event {
            ($report:ident, $report_name:ident, $uinput_name:ident) => {
                AbsoluteEvent::new(
                    EVENT_TIME_ZERO,
                    AbsoluteAxis::$uinput_name,
                    $report.motion.accelerometer.$report_name.round() as i32,
                )
            };
        }

        macro_rules! gyro_event {
            ($report:ident, $report_name:ident, $uinput_name:ident) => {
                AbsoluteEvent::new(
                    EVENT_TIME_ZERO,
                    AbsoluteAxis::$uinput_name,
                    $report.motion.gyro.$report_name.round() as i32,
                )
            };
        }

        // Main device events

        let buttons_events: &[InputEvent] = &[
            ex_key_event!(report, triangle, map_button_to_ds4(Button::Triangle)),
            ex_key_event!(report, circle, map_button_to_ds4(Button::Circle)),
            ex_key_event!(report, cross, map_button_to_ds4(Button::Cross)),
            ex_key_event!(report, square, map_button_to_ds4(Button::Square)),
            ex_key_event!(report, lt, map_button_to_ds4(Button::TriggerLeft)),
            ex_key_event!(report, rt, map_button_to_ds4(Button::TriggerRight)),
            ex_key_event!(report, select, map_button_to_ds4(Button::Options)),
            ex_key_event!(report, start, map_button_to_ds4(Button::Share)),
        ]
        .map(|ev| ev.into());

        let dpad_events =
            &[dpad_event!(report, Hat0Y), dpad_event!(report, Hat0X)].map(|ev| ev.into());

        let sticks_events = &[
            stick_event!(report, lx, X),
            stick_event!(report, ly, Y),
            stick_event!(report, rx, RX),
            stick_event!(report, ry, RY),
        ]
        .map(|ev| ev.into());

        let events: Vec<input_event> = [
            buttons_events,
            sticks_events,
            dpad_events,
            // &front_touch_resets_events,
            // &front_touch_events,
        ]
        .concat()
        .into_iter()
        .map(|ev| ev.into())
        .map(|ev: InputEvent| *ev.as_raw())
        .collect();

        self.main_handle
            .write(&events)
            .map_err(Error::WriteEventFailed)?;
        self.main_handle
            .write(&[syn_event])
            .map_err(Error::WriteEventFailed)?;

        let front_touch_resets_events = self
            .previous_front_touches
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| {
                let new_id = report.front_touch.reports.get(slot).map(|r| r.id);

                match (*id, new_id) {
                    (Some(_), None) => Some([
                        AbsoluteEvent::new(
                            EVENT_TIME_ZERO,
                            AbsoluteAxis::MultitouchSlot,
                            slot as i32,
                        ),
                        AbsoluteEvent::new(EVENT_TIME_ZERO, AbsoluteAxis::MultitouchTrackingId, -1),
                    ]),
                    _ => None,
                }
            })
            .flatten()
            .map(|ev| ev.into())
            .collect::<Vec<InputEvent>>();

        self.previous_front_touches = report
            .front_touch
            .reports
            .iter()
            .map(|report| Some(report.id))
            .chain(
                std::iter::repeat(None)
                    .take(self.previous_front_touches.len() - report.front_touch.reports.len()),
            )
            .collect::<Vec<Option<u8>>>()
            .try_into()
            .unwrap();

        let front_touch_events: Vec<_> = report
            .front_touch
            .reports
            .into_iter()
            .enumerate()
            .map(|(slot, report)| {
                let mut events = vec![
                    AbsoluteEvent::new(EVENT_TIME_ZERO, AbsoluteAxis::MultitouchSlot, slot as i32)
                        .into(),
                    mt_event!(report, x, MultitouchPositionX).into(),
                    mt_event!(report, y, MultitouchPositionY).into(),
                    mt_event!(report, x, X).into(),
                    mt_event!(report, y, Y).into(),
                    mt_event!(report, id, MultitouchTrackingId).into(),
                    mt_event!(report, force, MultitouchPressure).into(),
                ];

                if report.force > 0 {
                    events.push(
                        KeyEvent::new(EVENT_TIME_ZERO, Key::ButtonTouch, KeyState::PRESSED).into(),
                    );
                    events.push(
                        KeyEvent::new(EVENT_TIME_ZERO, Key::ButtonToolFinger, KeyState::PRESSED)
                            .into(),
                    );
                }

                events
            })
            .flatten()
            .collect::<Vec<InputEvent>>();

        let events: Vec<input_event> = front_touch_resets_events
            .iter()
            .chain(front_touch_events.iter())
            .map(|ev| (*ev).into())
            .map(|ev: InputEvent| *ev.as_raw())
            .collect();

        self.touchpad_handle
            .write(&events)
            .map_err(Error::WriteEventFailed)?;
        self.touchpad_handle
            .write(&[syn_event])
            .map_err(Error::WriteEventFailed)?;

        // Sensors device events

        let motion_events: &[InputEvent] = &[
            accel_event!(report, x, X),
            accel_event!(report, z, Y),
            accel_event!(report, y, Z),
            gyro_event!(report, x, RX),
            gyro_event!(report, z, RY),
            gyro_event!(report, y, RZ),
        ]
        .map(|ev| ev.into());

        let back_touch_resets_events = self
            .previous_back_touches
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| {
                let new_id = report.back_touch.reports.get(slot).map(|r| r.id);

                match (*id, new_id) {
                    (Some(_), None) => Some([
                        AbsoluteEvent::new(
                            EVENT_TIME_ZERO,
                            AbsoluteAxis::MultitouchSlot,
                            slot as i32,
                        ),
                        AbsoluteEvent::new(EVENT_TIME_ZERO, AbsoluteAxis::MultitouchTrackingId, -1),
                    ]),
                    _ => None,
                }
            })
            .flatten()
            .map(|ev| ev.into())
            .collect::<Vec<InputEvent>>();

        self.previous_back_touches = report
            .back_touch
            .reports
            .iter()
            .map(|report| Some(report.id))
            .chain(
                std::iter::repeat(None)
                    .take(self.previous_back_touches.len() - report.back_touch.reports.len()),
            )
            .collect::<Vec<Option<u8>>>()
            .try_into()
            .unwrap();

        let back_touch_events: Vec<_> = report
            .back_touch
            .reports
            .into_iter()
            .enumerate()
            .map(|(slot, report)| {
                [
                    AbsoluteEvent::new(EVENT_TIME_ZERO, AbsoluteAxis::MultitouchSlot, slot as i32),
                    mt_event!(report, x, MultitouchPositionX),
                    mt_event!(report, y, MultitouchPositionY),
                    mt_event!(report, id, MultitouchTrackingId),
                    mt_event!(report, force, MultitouchPressure),
                ]
                .map(|event| event.into())
            })
            .flatten()
            .collect::<Vec<InputEvent>>();

        let events: Vec<input_event> =
            [motion_events, &back_touch_resets_events, &back_touch_events]
                .concat()
                .into_iter()
                .map(|ev| ev.into())
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
