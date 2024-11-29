use serde::{Deserialize, Serialize};

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
