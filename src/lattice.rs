use chfft::RFft1D;
use image::GrayImage;
use std::collections::HashSet;

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

pub fn get_points(i: &GrayImage, intersection_points: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let mut all_corner_points: Vec<(f32, f32)> = Vec::new();
  for point in intersection_points.iter() {
    if is_corner(&i, point.0 as u32, point.1 as u32) {
      all_corner_points.push(*point);
    }
  }

  return unique_within_dist(&all_corner_points, 5.0);
}
