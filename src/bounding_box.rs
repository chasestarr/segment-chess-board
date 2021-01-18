use std::collections::HashMap;

fn points_in_bounds(points: &Vec<(f32, f32)>, (w, h): (u32, u32)) -> Vec<(f32, f32)> {
  let mut in_bounds = Vec::new();
  for (x, y) in points.iter() {
    if *x >= 0.0 && *y >= 0.0 && *x < w as f32 && *y < h as f32 {
      in_bounds.push((*x, *y));
    }
  }
  return in_bounds;
}

fn points_center(points: &Vec<(f32, f32)>) -> (f32, f32) {
  let mut x_sum = 0.0;
  let mut y_sum = 0.0;
  for (x, y) in points.iter() {
    x_sum += *x;
    y_sum += *y;
  }
  let len = points.len() as f32;
  return (x_sum / len, y_sum / len);
}

fn angle_between_vectors(start: (f32, f32), end: (f32, f32)) -> f32 {
  let opposite = start.1 - end.1;
  let adjacent = end.0 - start.0;
  let mut angle = opposite.atan2(adjacent);
  if angle < 0.0 {
    angle = angle + std::f32::consts::PI * 2.0;
  }
  let deg = angle * (180.0 / std::f32::consts::PI);
  return deg;
}

fn sorted_clockwise(p: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let mut points = p.clone();
  let (cx, cy) = points_center(&points);
  points.sort_by(|a, b| {
    let a_deg = angle_between_vectors((1.0, 0.0), sub(*a, (cx, cy)));
    let b_deg = angle_between_vectors((1.0, 0.0), sub(*b, (cx, cy)));
    return a_deg.partial_cmp(&b_deg).unwrap();
  });
  return points;
}

fn sorted_leftmost(p: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let mut points = p.clone();
  points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
  return points;
}

fn cross_product((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
  return (ax * by) - (ay * bx);
}

fn dot_product((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
  return ax * bx + ay * by;
}

fn add((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> (f32, f32) {
  return (ax + bx, ay + by);
}

fn sub((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> (f32, f32) {
  return (ax - bx, ay - by);
}

fn mult((x, y): (f32, f32), s: f32) -> (f32, f32) {
  return (x * s, y * s);
}

fn perp((x, y): (f32, f32)) -> (f32, f32) {
  return (-y, x);
}

fn normalized((x, y): (f32, f32)) -> (f32, f32) {
  let len = (x * x + y * y).sqrt();
  return (x / len, y / len);
}

pub fn dist_squared((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
  return (ax - bx).powf(2.0) + (ay - by).powf(2.0);
}

fn change_basis(i: (f32, f32), j: (f32, f32), p: (f32, f32)) -> (f32, f32) {
  return (i.0 * p.0 + i.1 * p.1, j.0 * p.0 + j.1 * p.1);
}

fn transpose(points: &Vec<(f32, f32)>) -> [Vec<f32>; 2] {
  let mut x = Vec::new();
  let mut y = Vec::new();
  for p in points.iter() {
    x.push(p.0);
    y.push(p.1);
  }
  return [x, y];
}

fn rotate_points(basis: [[f32; 2]; 2], points: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let transposed = transpose(points);
  let mut x = Vec::new();
  let mut y = Vec::new();
  for i in 0..points.len() {
    x.push(basis[0][0] * transposed[0][i] + basis[0][1] * transposed[1][i]);
    y.push(basis[1][0] * transposed[0][i] + basis[1][1] * transposed[1][i]);
  }
  let mut result = Vec::new();
  for i in 0..points.len() {
    result.push((x[i], y[i]));
  }
  return result;
}

// https://www.geometrictools.com/GTE/Mathematics/MinimumAreaBox2.h
fn remove_colinear_points(points: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let mut result = Vec::new();
  let mut edge_prev = sub(points[0], points[points.len() - 1]);
  for i in 0..points.len() {
    let edge_next = sub(points[i], points[(i + 1) % points.len()]);
    let dp = dot_product(edge_prev, perp(edge_next));
    if dp.abs() > 0.1 {
      result.push(points[i]);
    }
    edge_prev = edge_next;
  }
  return result;
}

// https://en.wikipedia.org/wiki/Gift_wrapping_algorithm
fn convex_hull_giftwrap(points: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let mut hull = Vec::new();
  let sorted = sorted_leftmost(points);
  let left_most = sorted[0];
  let mut current_vertex = left_most;
  hull.push(current_vertex);
  let mut next_vertex = sorted[1];
  let mut index = 0;

  loop {
    let checking = sorted[index];
    let a = (
      next_vertex.0 - current_vertex.0,
      next_vertex.1 - current_vertex.1,
    );
    let b = (checking.0 - current_vertex.0, checking.1 - current_vertex.1);
    let cross = cross_product(a, b);
    if cross < 0.0 {
      next_vertex = checking;
    }

    index += 1;
    if index == sorted.len() {
      if next_vertex == left_most {
        break;
      } else {
        hull.push(next_vertex);
        current_vertex = next_vertex;
        index = 0;
        next_vertex = left_most;
      }
    }
  }

  return remove_colinear_points(&hull);
}

pub fn convex_hull_area(hull: &Vec<(f32, f32)>) -> f32 {
  let mut area = 0.0;
  for i in 0..hull.len() - 1 {
    let ax = hull[i].0 - hull[0].0;
    let ay = hull[i].1 - hull[0].1;
    let bx = hull[i + 1].0 - hull[0].0;
    let by = hull[i + 1].1 - hull[0].1;
    area += cross_product((ax, ay), (bx, by));
  }
  return area.abs() / 2.0;
}

fn triangle_area(points: [(f32, f32); 3]) -> f32 {
  let [(x1, y1), (x2, y2), (x3, y3)] = points;
  return ((x1 * y2 + x2 * y3 + x3 * y1 - y1 * x2 - y2 * x3 - y3 * x1) / 2.0).abs();
}

pub fn bounding_box_area(points: [(f32, f32); 4]) -> f32 {
  let t1 = [points[0], points[1], points[2]];
  let t2 = [points[0], points[2], points[3]];
  return triangle_area(t1) + triangle_area(t2);
}

pub fn bounding_box_sort(points: [(f32, f32); 4]) -> [(f32, f32); 4] {
  let mut sorted_by_y = points.clone().to_vec();
  sorted_by_y.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

  let mut top = vec![sorted_by_y[0], sorted_by_y[1]];
  if top[1].0 < top[0].0 {
    top = vec![top[1], top[0]];
  }
  let mut bottom = vec![sorted_by_y[2], sorted_by_y[3]];
  if bottom[1].0 < bottom[0].0 {
    bottom = vec![bottom[1], bottom[0]];
  }
  return [top[0], top[1], bottom[1], bottom[0]];
}

pub fn bounding_box_offset(points: [(f32, f32); 4], offset: f32) -> [(f32, f32); 4] {
  let mut offsets = Vec::new();
  for i in 0..points.len() {
    let edge = sub(points[(i + 1) % points.len()], points[i]);
    offsets.push(mult(normalized(perp(edge)), -offset));
  }

  [
    add(add(points[0], offsets[0]), offsets[3]),
    add(add(points[1], offsets[1]), offsets[0]),
    add(add(points[2], offsets[2]), offsets[1]),
    add(add(points[3], offsets[3]), offsets[2]),
  ]
}

// https://github.com/dbworth/minimum-area-bounding-rectangle/blob/master/python/min_bounding_rect.py
pub fn oriented_bounding_box(ps: &Vec<(f32, f32)>) -> [(f32, f32); 4] {
  let mut points = sorted_clockwise(ps);
  points.reverse();

  let mut edge_angles = Vec::new();
  for i in 0..points.len() {
    let current = points[i];
    let next = points[(i + 1) % points.len()];
    let (x, y) = sub(next, current);
    edge_angles.push(y.atan2(x));
  }

  let mut obb = (0.0, std::f32::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
  for i in 0..edge_angles.len() {
    let ri = (edge_angles[i].cos(), -edge_angles[i].sin());
    let rj = (edge_angles[i].sin(), edge_angles[i].cos());

    let rotated_points: Vec<(f32, f32)> = points.iter().map(|p| change_basis(ri, rj, *p)).collect();

    let mut min_x = std::f32::INFINITY;
    let mut max_x = std::f32::NEG_INFINITY;
    let mut min_y = std::f32::INFINITY;
    let mut max_y = std::f32::NEG_INFINITY;
    for (x, y) in rotated_points.iter() {
      if *x < min_x {
        min_x = *x;
      }
      if *x > max_x {
        max_x = *x;
      }
      if *y < min_y {
        min_y = *y;
      }
      if *y > max_y {
        max_y = *y;
      }
    }
    let width = max_x - min_x;
    let height = max_y - min_y;
    let area = width * height;
    if area < obb.1 {
      obb = (
        edge_angles[i],
        area,
        width,
        height,
        min_x,
        max_x,
        min_y,
        max_y,
      );
    }
  }

  let angle = obb.0;
  let min_x = obb.4;
  let max_x = obb.5;
  let min_y = obb.6;
  let max_y = obb.7;

  let iri = (angle.cos(), angle.sin());
  let irj = (-angle.sin(), angle.cos());

  return bounding_box_sort([
    change_basis(iri, irj, (min_x, min_y)),
    change_basis(iri, irj, (max_x, min_y)),
    change_basis(iri, irj, (max_x, max_y)),
    change_basis(iri, irj, (min_x, max_y)),
  ]);
}

pub fn bounding_box(points: &Vec<(f32, f32)>) -> [(f32, f32); 4] {
  let hull = convex_hull_giftwrap(points);

  let alpha = (convex_hull_area(&hull) / 15.0).sqrt();
  let cluster_assignments = crate::cluster::dbscan(&points, alpha, 5);
  let mut point_clusters: HashMap<usize, Vec<(f32, f32)>> = HashMap::new();
  for i in 0..points.len() {
    if cluster_assignments[i] != 0 {
      if let Some(cluster) = point_clusters.get_mut(&cluster_assignments[i]) {
        cluster.push(points[i]);
      } else {
        point_clusters.insert(cluster_assignments[i], vec![points[i]]);
      }
    }
  }

  let mut largest_cluster = Vec::new();
  for (_, cluster) in point_clusters.iter() {
    if cluster.len() > largest_cluster.len() {
      largest_cluster = cluster.to_vec();
    }
  }

  let cluster_hull = convex_hull_giftwrap(&largest_cluster);
  let mbb = oriented_bounding_box(&cluster_hull);

  if crate::debug::debug_images() {
    let mut hull_image: image::RgbImage = image::ImageBuffer::new(400, 400);
    for i in 0..largest_cluster.len() {
      let p = largest_cluster[i];
      imageproc::drawing::draw_filled_circle_mut(
        &mut hull_image,
        (p.0 as i32, p.1 as i32),
        3,
        image::Rgb::<u8>(crate::color::turbo(i as f32 / largest_cluster.len() as f32)),
      );
    }

    for i in 0..cluster_hull.len() {
      let p = cluster_hull[i];
      imageproc::drawing::draw_filled_circle_mut(
        &mut hull_image,
        (p.0 as i32, p.1 as i32),
        5,
        image::Rgb::<u8>([255, 255, 255]),
      );
      imageproc::drawing::draw_line_segment_mut(
        &mut hull_image,
        cluster_hull[i],
        cluster_hull[(i + 1) % cluster_hull.len()],
        image::Rgb::<u8>([255, 0, 100]),
      );
    }

    for index in 0..mbb.len() {
      let p = mbb[index];
      imageproc::drawing::draw_filled_circle_mut(
        &mut hull_image,
        (p.0 as i32, p.1 as i32),
        10,
        image::Rgb::<u8>(crate::color::turbo(index as f32 / mbb.len() as f32)),
      );
      imageproc::drawing::draw_filled_circle_mut(
        &mut hull_image,
        (p.0 as i32, p.1 as i32),
        3,
        image::Rgb::<u8>([255, 0, 0]),
      );
      imageproc::drawing::draw_line_segment_mut(
        &mut hull_image,
        mbb[index],
        mbb[(index + 1) % mbb.len()],
        image::Rgb::<u8>([0, 255, 100]),
      );
    }
    crate::debug::write_rgb(&hull_image, "convex-hull-mbb");
  }

  if cluster_hull.len() == 4 {
    return [
      cluster_hull[0],
      cluster_hull[1],
      cluster_hull[2],
      cluster_hull[3],
    ];
  }
  return mbb;
}
