use image::{DynamicImage, GrayImage, Rgb};
use imageproc::drawing::draw_line_segment_mut;
use imageproc::edges::canny;
use imageproc::hough::{detect_lines, LineDetectionOptions, PolarLine};

use crate::debug;

#[derive(Debug)]
pub struct Line {
  pub start: (f32, f32),
  pub end: (f32, f32),
}

impl Line {
  pub fn new(start: (f32, f32), end: (f32, f32)) -> Self {
    Line { start, end }
  }

  pub fn intersection(&self, other: &Line) -> Option<(f32, f32)> {
    let (self_start_x, self_start_y) = self.start;
    let (self_end_x, self_end_y) = self.end;
    let (other_start_x, other_start_y) = other.start;
    let (other_end_x, other_end_y) = other.end;

    let a_self = self_end_y - self_start_y;
    let b_self = self_start_x - self_end_x;
    let c_self = a_self * self_start_x + b_self * self_start_y;

    let a_other = other_end_y - other_start_y;
    let b_other = other_start_x - other_end_x;
    let c_other = a_other * other_start_x + b_other * other_start_y;

    let determinant = a_self * b_other - a_other * b_self;
    if determinant == 0.0 {
      return None;
    }

    let x = (b_other * c_self - b_self * c_other) / determinant;
    let y = (a_self * c_other - a_other * c_self) / determinant;
    return Some((x, y));
  }
}

fn draw_polar_line<P>(i: &mut imageproc::definitions::Image<P>, line: PolarLine, color: P)
where
  P: image::Pixel + 'static,
{
  if let Some(line) = polar_line_points(line, i.width(), i.height()) {
    draw_line_segment_mut(i, line.start, line.end, color);
  }
}

// copied from imageproc internals https://github.com/image-rs/imageproc/blob/master/src/hough.rs#L133
fn polar_line_points(line: PolarLine, image_width: u32, image_height: u32) -> Option<Line> {
  let r = line.r;
  let m = line.angle_in_degrees;
  let w = image_width as f32;
  let h = image_height as f32;

  // Vertical line
  if m == 0 {
    return if r >= 0.0 && r <= w {
      Some(Line::new((r, 0.0), (r, h)))
    } else {
      None
    };
  }

  // Horizontal line
  if m == 90 {
    return if r >= 0.0 && r <= h {
      Some(Line::new((0.0, r), (w, r)))
    } else {
      None
    };
  }

  let theta = (m as f32).to_radians();
  let (sin, cos) = theta.sin_cos();

  let right_y = cos.mul_add(-w, r) / sin;
  let left_y = r / sin;
  let bottom_x = sin.mul_add(-h, r) / cos;
  let top_x = r / cos;

  let mut start = None;

  if right_y >= 0.0 && right_y <= h {
    let right_intersect = (w, right_y);
    if let Some(s) = start {
      return Some(Line::new(s, right_intersect));
    }
    start = Some(right_intersect);
  }

  if left_y >= 0.0 && left_y <= h {
    let left_intersect = (0.0, left_y);
    if let Some(s) = start {
      return Some(Line::new(s, left_intersect));
    }
    start = Some(left_intersect);
  }

  if bottom_x >= 0.0 && bottom_x <= w {
    let bottom_intersect = (bottom_x, h);
    if let Some(s) = start {
      return Some(Line::new(s, bottom_intersect));
    }
    start = Some(bottom_intersect);
  }

  if top_x >= 0.0 && top_x <= w {
    let top_intersect = (top_x, 0.0);
    if let Some(s) = start {
      return Some(Line::new(s, top_intersect));
    }
  }

  None
}

pub fn get_lines(i: &GrayImage) -> Vec<Line> {
  let edges = canny(&i, 50.0, 80.0);
  debug::write_gray(&edges, "line-canny");
  let (image_width, image_height) = i.dimensions();

  let lines = detect_lines(
    &edges,
    LineDetectionOptions {
      vote_threshold: 100,
      suppression_radius: 30,
    },
  );

  if std::env::var("DEBUG").is_ok() {
    let mut lines_image = DynamicImage::ImageLuma8(i.clone()).to_rgb8();
    let len = lines.len();
    for index in 0..lines.len() {
      let v = (index as f32 / len as f32 * 255.0) as u8;
      let mut color = Rgb::<u8>([100, v, 200]);
      if index == 0 {
        color = Rgb::<u8>([255, 0, 0]);
      } else if index == len - 1 {
        color = Rgb::<u8>([0, 255, 0]);
      }
      draw_polar_line(&mut lines_image, lines[index], color);
    }
    debug::write_rgb(&lines_image, "line-polar-lines");
  }

  let mut lines_points: Vec<Line> = Vec::new();
  for line in lines {
    match polar_line_points(line, image_width, image_height) {
      Some(l) => lines_points.push(l),
      None => {}
    }
  }
  return lines_points;
}
