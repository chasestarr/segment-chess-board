use crate::line::intersection;

use std::cmp::Ordering;
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

// https://stackoverflow.com/questions/6989100/sort-points-in-clockwise-order
fn sorted_clockwise(p: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let mut points = p.clone();
  let (cx, cy) = points_center(&points);
  points.sort_by(|(ax, ay), (bx, by)| {
    if ax - cx >= 0.0 && bx - cx < 0.0 {
      return Ordering::Greater;
    }
    if ax - cx < 0.0 && bx - cx >= 0.0 {
      return Ordering::Less;
    }
    if ax - cx == 0.0 && bx - cx == 0.0 {
      if ay - cy >= 0.0 || by - cy >= 0.0 {
        if ay > by {
          return Ordering::Greater;
        } else {
          return Ordering::Less;
        }
      }
      if by > ay {
        return Ordering::Greater;
      } else {
        return Ordering::Less;
      }
    }

    let det = (ax - cy) * (by - cy) - (bx - cx) * (ay - cy);
    if det < 0.0 {
      return Ordering::Greater;
    }
    if det > 0.0 {
      return Ordering::Greater;
    }

    return Ordering::Equal;
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

fn dist((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
  let dist_squared = (ax - bx).powf(2.0) + (ay - by).powf(2.0);
  return dist_squared.sqrt();
}

fn transpose(i: (f32, f32), j: (f32, f32), p: (f32, f32)) -> (f32, f32) {
  return (i.0 * p.0 + i.1 * p.1, j.0 * p.0 + j.1 * p.1);
}

// https://www.geometrictools.com/GTE/Mathematics/MinimumAreaBox2.h
fn remove_colinear_points(points: &Vec<(f32, f32)>) -> Vec<(f32, f32)> {
  let mut result = Vec::new();
  let mut edge_prev = sub(points[0], points[points.len() - 1]);
  for i in 0..points.len() {
    let edge_next = sub(points[i], points[(i + 1) % points.len()]);
    let dp = dot_product(edge_prev, perp(edge_next));
    if dp.abs() > 0.01 {
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

fn mbb_smallest_box(i0: usize, i1: usize, points: &Vec<(f32, f32)>) -> [(f32, f32); 4] {
  let ui = sub(points[i1], points[i0]);
  let uj = mult(perp(ui), -1.0);
  let mut box_index = [i1, i1, i1, i1];
  let origin = points[i1];
  let mut support: [(f32, f32); 4] = [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];
  let mut i = 0;
  for point in points.iter() {
    let diff = sub(*point, origin);
    let v = (dot_product(ui, diff), dot_product(uj, diff));
    if v.0 > support[1].0 || (v.0 == support[1].0 && v.1 > support[1].1) {
      box_index[1] = i;
      support[1] = v;
    }
    if v.1 > support[2].1 || (v.1 == support[2].1 && v.0 < support[2].0) {
      box_index[2] = i;
      support[2] = v;
    }
    if v.0 < support[3].0 || (v.1 == support[3].0 && v.1 < support[3].1) {
      box_index[3] = i;
      support[3] = v;
    }
    i += 1;
  }

  return support;
}

fn triangle_area(points: [(f32, f32); 3]) -> f32 {
  let [(x1, y1), (x2, y2), (x3, y3)] = points;
  return ((x1 * y2 + x2 * y3 + x3 * y1 - y1 * x2 - y2 * x3 - y3 * x1) / 2.0).abs();
}

fn bounding_box_area(points: [(f32, f32); 4]) -> f32 {
  let t1 = [points[0], points[1], points[2]];
  let t2 = [points[0], points[2], points[3]];
  return triangle_area(t1) + triangle_area(t2);
}

pub fn bounding_box_offset(points: [(f32, f32); 4], offset: f32) -> [(f32, f32); 4] {
  let mut offsets = Vec::new();
  for i in 0..points.len() {
    let edge = sub(points[(i + 1) % points.len()], points[i]);
    offsets.push(mult(normalized(perp(edge)), offset));
  }

  [
    add(add(points[0], offsets[0]), offsets[3]),
    add(add(points[1], offsets[1]), offsets[0]),
    add(add(points[2], offsets[2]), offsets[1]),
    add(add(points[3], offsets[3]), offsets[2]),
  ]
}

// https://github.com/dbworth/minimum-area-bounding-rectangle/blob/master/python/min_bounding_rect.py
fn oriented_bounding_box(points: &Vec<(f32, f32)>) -> [(f32, f32); 4] {
  let mut edge_angles = Vec::new();
  for i in 0..points.len() {
    let (x, y) = sub(points[(i + 1) % points.len()], points[i]);
    edge_angles.push(y.atan2(x).abs());
  }

  let mut obb = (0.0, std::f32::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
  for i in 0..edge_angles.len() {
    let ri = (
      edge_angles[i].cos(),
      (edge_angles[i] - std::f32::consts::PI / 2.0).cos(),
    );
    let rj = (
      (edge_angles[i] + std::f32::consts::PI / 2.0).cos(),
      edge_angles[i].cos(),
    );
    let rotated_points: Vec<(f32, f32)> = points.iter().map(|p| transpose(ri, rj, *p)).collect();
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
  let pi = (angle.cos(), (angle - std::f32::consts::PI / 2.0).cos());
  let pj = ((angle + std::f32::consts::PI / 2.0).cos(), angle.cos());
  return [
    transpose(pi, pj, (max_x, min_y)),
    transpose(pi, pj, (min_x, min_y)),
    transpose(pi, pj, (min_x, max_y)),
    transpose(pi, pj, (max_x, max_y)),
  ];
}

fn oriented_bounding_box_js(a: usize, b: usize, points: &Vec<(f32, f32)>) -> [(f32, f32); 4] {
  let mut box_area = std::f32::INFINITY;
  let mut box_corners = [points[a], points[b], points[b], points[a]];

  let mut edge_directions = Vec::new();
  for i in 0..points.len() {
    edge_directions.push(normalized(sub(points[(i + 1) % points.len()], points[i])));
  }

  let mut min_pnt = (std::f32::INFINITY, std::f32::INFINITY);
  let mut max_pnt = (std::f32::NEG_INFINITY, std::f32::NEG_INFINITY);
  let mut indices = [0, 0, 0, 0];
  for i in 0..points.len() {
    let p = points[i];
    if p.0 < min_pnt.0 {
      min_pnt.0 = p.0;
      indices[0] = i;
    }
    if p.0 > max_pnt.0 {
      max_pnt.0 = p.0;
      indices[1] = i;
    }
    if p.1 < min_pnt.1 {
      min_pnt.1 = p.1;
      indices[2] = i;
    }
    if p.1 > max_pnt.1 {
      max_pnt.1 = p.1;
      indices[3] = i;
    }

    let mut directions = [(0.0, -1.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)];

    for _ in 0..points.len() {
      let phis = [
        (dot_product(directions[0], edge_directions[indices[0]])).acos(),
        (dot_product(directions[1], edge_directions[indices[1]])).acos(),
        (dot_product(directions[2], edge_directions[indices[2]])).acos(),
        (dot_product(directions[3], edge_directions[indices[3]])).acos(),
      ];
      let mut min_phi_index = 3;
      for j in 0..phis.len() {
        if phis[j] < phis[min_phi_index] {
          min_phi_index = j;
        }
      }

      match min_phi_index {
        0 => {
          directions[0] = edge_directions[0];
          directions[1] = mult(directions[0], -1.0);
          directions[2] = mult(perp(directions[0]), -1.0);
          directions[3] = mult(directions[2], -1.0);
          indices[0] = (indices[0] + 1) % points.len();
        }
        1 => {
          directions[1] = edge_directions[1];
          directions[0] = mult(directions[1], -1.0);
          directions[2] = mult(perp(directions[0]), -1.0);
          directions[3] = mult(directions[2], -1.0);
          indices[1] = (indices[1] + 1) % points.len();
        }
        2 => {
          directions[2] = edge_directions[2];
          directions[3] = mult(directions[2], -1.0);
          directions[0] = mult(perp(directions[3]), -1.0);
          directions[1] = mult(directions[0], -1.0);
          indices[2] = (indices[2] + 1) % points.len();
        }
        3 => {
          directions[3] = edge_directions[3];
          directions[0] = mult(directions[3], -1.0);
          directions[1] = mult(perp(directions[0]), -1.0);
          directions[2] = mult(directions[1], -1.0);
          indices[3] = (indices[3] + 1) % points.len();
        }
        _ => {}
      }

      let top_left = intersection(
        points[indices[0]],
        directions[0],
        points[indices[2]],
        directions[2],
      )
      .unwrap();
      let top_right = intersection(
        points[indices[1]],
        directions[1],
        points[indices[2]],
        directions[2],
      )
      .unwrap();
      let bottom_left = intersection(
        points[indices[3]],
        directions[3],
        points[indices[0]],
        directions[0],
      )
      .unwrap();
      let bottom_right = intersection(
        points[indices[3]],
        directions[3],
        points[indices[1]],
        directions[1],
      )
      .unwrap();
      let area = dist(top_left, top_right) * dist(top_left, bottom_left);
      if area < box_area {
        box_corners = [top_left, top_right, bottom_right, bottom_left];
        box_area = area;
      }
    }
  }

  return box_corners;
}

// https://www.geometrictools.com/Source/ComputationalGeometry.html
// https://www.geometrictools.com/GTE/Mathematics/MinimumAreaBox2.h
fn minimum_bounding_box(hull: &Vec<(f32, f32)>) -> [(f32, f32); 4] {
  // let mut min_b = mbb_smallest_box(0, 1, hull);
  let mut min_b = oriented_bounding_box_js(0, 1, hull);
  let mut min_area = convex_hull_area(&min_b.to_vec());
  for index in 1..hull.len() {
    let mut next_index = index + 1;
    if next_index == hull.len() {
      next_index = 0;
    }
    // let next_b = mbb_smallest_box(index, next_index, hull);
    let next_b = oriented_bounding_box_js(index, next_index, hull);
    let next_area = convex_hull_area(&next_b.to_vec());
    if next_area < min_area {
      min_b = next_b;
      min_area = next_area;
    }
  }

  return min_b;
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
  let mut mbb = oriented_bounding_box(&cluster_hull);
  for i in 0..mbb.len() {
    mbb[i] = (mbb[i].0.abs(), mbb[i].1.abs());
  }

  // let cluster_hull_area = convex_hull_area(&cluster_hull);
  // let image_area = w as f32 * h as f32;
  // let cluster_alpha = (convex_hull_area(&cluster_hull) / 49.0).sqrt();
  // let cluster_beta = cluster_hull.len() as f32 * (5.0 / 100.0);
  // let cluster_centroid = points_center(&cluster_hull);

  if crate::debug::debug_images() {
    let mut hull_image: image::RgbImage = image::ImageBuffer::new(500, 500);
    for i in 0..cluster_hull.len() {
      let p = cluster_hull[i];
      imageproc::drawing::draw_filled_circle_mut(
        &mut hull_image,
        (p.0 as i32, p.1 as i32),
        5,
        image::Rgb::<u8>([255, 255, 255]),
      );
    }

    crate::debug::write_rgb(&hull_image, "convex-hull");

    for i in 0..largest_cluster.len() {
      let p = largest_cluster[i];
      imageproc::drawing::draw_filled_circle_mut(
        &mut hull_image,
        (p.0 as i32, p.1 as i32),
        3,
        image::Rgb::<u8>(crate::color::turbo(i as f32 / largest_cluster.len() as f32)),
      );
    }

    for p in mbb.iter() {
      imageproc::drawing::draw_filled_circle_mut(
        &mut hull_image,
        (p.0 as i32, p.1 as i32),
        10,
        image::Rgb::<u8>([255, 255, 255]),
      );
    }
    crate::debug::write_rgb(&hull_image, "convex-hull-mbb");
  }

  if crate::debug::debug_images() {
    let mut db_clusters_image: image::RgbImage = image::ImageBuffer::new(500, 500);
    let mut db_clusters_max = 0;
    for c in cluster_assignments.iter() {
      if *c > db_clusters_max {
        db_clusters_max = *c;
      }
    }
    for i in 0..points.len() {
      let (x, y) = points[i];
      imageproc::drawing::draw_filled_circle_mut(
        &mut db_clusters_image,
        (x as i32, y as i32),
        3,
        image::Rgb::<u8>(crate::color::turbo(
          cluster_assignments[i] as f32 / db_clusters_max as f32,
        )),
      );
    }
    crate::debug::write_rgb(&db_clusters_image, "heat_map_clusters");
  }

  return mbb;
}
