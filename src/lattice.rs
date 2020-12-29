use chfft::RFft1D;
use image::{DynamicImage, GrayImage, Rgb};
use imageproc::drawing::{draw_hollow_circle_mut, draw_line_segment_mut};
use std::collections::HashSet;

use crate::debug;
use crate::line::Line;

#[inline]
fn get_circle(i: &GrayImage, x: u32, y: u32, r: u32) -> Option<Vec<u8>> {
  let (width, height) = i.dimensions();
  if x < r || x > width - (r + 1) || y < r || y > height - (r + 1) {
    return None;
  }

  let mut circle: Vec<u8> = Vec::new();
  circle.push(i.get_pixel(x, y - r)[0]);
  for index in 0..r {
    circle.push(i.get_pixel(x + index + 1, y - (r - index))[0]);
  }
  circle.push(i.get_pixel(x, y + r)[0]);
  for index in 0..r {
    circle.push(i.get_pixel(x + (r - index), y + index + 1)[0]);
  }
  circle.push(i.get_pixel(x + r, y)[0]);
  for index in 0..r {
    circle.push(i.get_pixel(x - (index + 1), y + (r - index))[0]);
  }
  circle.push(i.get_pixel(x - r, y)[0]);
  for index in 0..r {
    circle.push(i.get_pixel(x - (r - index), y - (index + 1))[0]);
  }

  return Some(circle);
}

fn is_corner(i: &GrayImage, x: u32, y: u32) -> bool {
  let r = 5;
  match get_circle(i, x, y, r) {
    Some(values) => {
      if std::env::var("DEBUG").is_ok() {
        let cropped =
          image::imageops::crop_imm(i, x - r + 1, y - r + 1, r * 2 + 1, r * 2 + 1).to_image();
        debug::write_gray(&cropped, format!("lattice-corner-{}-{}", y, x).as_str());
      }

      let input: Vec<f64> = values.iter().map(|v| *v as f64).collect();
      let mut fft = RFft1D::<f64>::new(values.len());
      let output = fft.forward(&input);

      let mag_1 = (output[1].re.powf(2.0) + output[1].im.powf(2.0)).sqrt();
      let mag_2 = (output[2].re.powf(2.0) + output[2].im.powf(2.0)).sqrt();

      return mag_2 > mag_1;
    }
    None => {
      return false;
    }
  }
}

fn distance_squared((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
  let a = bx - ax;
  let b = by - ay;
  return a.powf(2.0) + b.powf(2.0);
}

fn unique_within_dist(points: &Vec<(f32, f32)>, r: f32) -> Vec<(f32, f32)> {
  let r_squared = r.powf(2.0);
  let mut seen_indices = HashSet::new();
  let mut output = Vec::new();

  for current_index in 0..points.len() {
    if seen_indices.contains(&current_index) {
      continue;
    }

    let mut neighbors: Vec<(f32, f32)> = Vec::new();
    for neighbor_index in 0..points.len() {
      if current_index == neighbor_index {
        continue;
      }

      let ds = distance_squared(points[current_index], points[neighbor_index]);
      if ds <= r_squared {
        neighbors.push(points[neighbor_index]);
        seen_indices.insert(neighbor_index);
      }
    }

    if neighbors.len() == 0 {
      output.push(points[current_index]);
      seen_indices.insert(current_index);
    } else {
      let mut x_sum = 0.0;
      let mut y_sum = 0.0;
      for (nx, ny) in neighbors.iter() {
        x_sum += nx;
        y_sum += ny;
      }
      let len = neighbors.len() as f32;
      output.push((x_sum / len, y_sum / len));
    }
  }

  return output;
}

pub fn get_points(i: &GrayImage, lines: &Vec<Line>) -> Vec<(f32, f32)> {
  let mut intersection_points: Vec<(f32, f32)> = Vec::new();
  for a in lines.iter() {
    for b in lines.iter() {
      match a.intersection(b) {
        Some(point) => intersection_points.push(point),
        None => {}
      }
    }
  }

  let mut all_corner_points: Vec<(f32, f32)> = Vec::new();
  for point in intersection_points.iter() {
    if is_corner(&i, point.0 as u32, point.1 as u32) {
      all_corner_points.push(*point);
    }
  }

  println!("{}", all_corner_points.len());

  let corner_points = unique_within_dist(&all_corner_points, 5.0);

  println!("{}", corner_points.len());

  if std::env::var("DEBUG").is_ok() {
    let mut intersection_image = DynamicImage::ImageLuma8(i.clone()).to_rgb8();
    let red = Rgb::<u8>([255, 0, 0]);
    let green = Rgb::<u8>([0, 255, 0]);
    let blue = Rgb::<u8>([0, 0, 255]);
    for (x, y) in intersection_points.iter() {
      draw_hollow_circle_mut(&mut intersection_image, (*x as i32, *y as i32), 3, red);
    }
    for (x, y) in corner_points.iter() {
      draw_hollow_circle_mut(&mut intersection_image, (*x as i32, *y as i32), 15, blue);
      draw_hollow_circle_mut(&mut intersection_image, (*x as i32, *y as i32), 3, blue);
    }

    for line in lines.iter() {
      draw_line_segment_mut(&mut intersection_image, line.start, line.end, green);
    }

    debug::write_rgb(&intersection_image, "lattice-intersections");
  }

  return corner_points;
}
