use crate::virtual_button::{Button, DpadDirection};
use crate::virtual_config::TriggerConfig;
use vita_reports::ButtonsData;

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
        (report_buttons.ps, Button::PSButton),
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
