use std::fmt;
use std::ops::{Add, Sub};

/// Near-duplicate points (where both `x` and `y` only differ within this value)
/// will not be included in the triangulation for robustness.
pub const EPSILON: f64 = f64::EPSILON * 2.0;

/// Represents a 2D point in the input vector.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Point {
  pub x: f64,
  pub y: f64,
}

impl fmt::Debug for Point {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[{}, {}]", self.x, self.y)
  }
}

impl Point {
  pub fn dist2(&self, p: &Self) -> f64 {
    let dx = self.x - p.x;
    let dy = self.y - p.y;
    dx * dx + dy * dy
  }

  pub fn nearly_equals(&self, p: &Self) -> bool {
    (self.x - p.x).abs() <= EPSILON && (self.y - p.y).abs() <= EPSILON
  }
}

impl From<(f32, f32)> for Point {
  fn from((x, y): (f32, f32)) -> Self {
    return Point {
      x: x as f64,
      y: y as f64,
    };
  }
}

impl Add for Point {
  type Output = Point;
  fn add(self, right: Point) -> Self::Output {
    Point {
      x: self.x + right.x,
      y: self.y + right.y,
    }
  }
}

impl Sub for Point {
  type Output = Point;
  fn sub(self, right: Point) -> Self::Output {
    Point {
      x: self.x - right.x,
      y: self.y - right.y,
    }
  }
}
