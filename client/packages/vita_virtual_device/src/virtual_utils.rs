use crate::virtual_button::{Button, DpadDirection};
use crate::virtual_config::{TouchConfig, TriggerConfig};
use crate::virtual_touch::{Point, TouchAction};
use vita_reports::{ButtonsData, TouchReport};

/// Computes the D-Pad direction based on the button states.
pub fn compute_dpad_direction(buttons: &ButtonsData) -> DpadDirection {
    match (buttons.up, buttons.down, buttons.left, buttons.right) {
        (true, false, false, false) => DpadDirection::North,
        (true, false, true, false) => DpadDirection::NorthWest,
        (true, false, false, true) => DpadDirection::NorthEast,
        (false, true, false, false) => DpadDirection::South,
        (false, true, true, false) => DpadDirection::SouthWest,
        (false, true, false, true) => DpadDirection::SouthEast,
        (false, false, true, false) => DpadDirection::West,
        (false, false, false, true) => DpadDirection::East,
        _ => DpadDirection::None,
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

/// Retrieves the list of pressed buttons based on the report and trigger configuration.
pub fn get_pressed_buttons(
    report_buttons: &ButtonsData,
    trigger_config: TriggerConfig,
) -> Vec<Button> {
    let mut buttons = vec![
        (report_buttons.circle, Button::Circle),
        (report_buttons.square, Button::Square),
        (report_buttons.cross, Button::Cross),
        (report_buttons.triangle, Button::Triangle),
        (report_buttons.start, Button::Options),
        (report_buttons.select, Button::Share),
    ];

    // Trigger processing depending on the configuration
    match trigger_config {
        TriggerConfig::Shoulder => {
            buttons.push((report_buttons.lt, Button::ShoulderLeft));
            buttons.push((report_buttons.rt, Button::ShoulderRight));
        }
        TriggerConfig::Trigger => {
            buttons.push((report_buttons.lt, Button::TriggerLeft));
            buttons.push((report_buttons.rt, Button::TriggerRight));
        }
    }

    buttons
        .into_iter()
        .filter_map(|(pressed, button)| if pressed { Some(button) } else { None })
        .collect()
}

/// Processes touch reports and returns a list of touch actions.
pub fn process_touch_reports(
    touch_reports: &[TouchReport],
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
