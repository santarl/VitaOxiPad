use serde::{Deserialize, Serialize};
use rstar::{primitives::Rectangle, AABB};

use crate::virtual_button::{Button, DpadDirection};

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

/// Represents an action triggered by a touch input.
#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
pub enum TouchAction {
    Dpad(DpadDirection),
    Button(Button),
}

/// Defines a touch zone and the action it triggers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TouchZone {
    pub rect: Rectangle<Point>,
    /// The emulated action to perform when the touch zone is touched.
    pub action: Option<TouchAction>,
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
