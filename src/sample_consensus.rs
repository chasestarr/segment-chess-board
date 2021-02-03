// https://github.com/Elucidation/ChessboardDetect/blob/master/Brutesac.py

use crate::delaunay_triangulation::{triangulate, Triangulation, EMPTY};
use crate::lattice::is_corner;
use crate::line::intersection;
use crate::point::Point;
use image::GrayImage;
use std::collections::HashSet;

fn edges_of_triangle(t: usize) -> [usize; 3] {
  return [3 * t, 3 * t + 1, 3 * t + 2];
}

fn points_of_triangle(tri: &Triangulation, points: &Vec<Point>, t: usize) -> [Point; 3] {
  let edges = edges_of_triangle(t);
  return [
    points[tri.triangles[edges[0]]],
    points[tri.triangles[edges[1]]],
    points[tri.triangles[edges[2]]],
  ];
}

fn triangle_of_edge(e: usize) -> usize {
  return (e as f32 / 3.0).floor() as usize;
}

fn neighbors_of_triangle(tri: &Triangulation, t: usize) -> Vec<usize> {
  let mut neighbors = Vec::new();
  let edges = edges_of_triangle(t);
  for e in 0..edges.len() {
    let opposite = tri.halfedges[edges[e]];
    if opposite != EMPTY {
      let t = triangle_of_edge(opposite);
      neighbors.push(t);
    }
  }
  return neighbors;
}

fn quad_sort(quad: [Point; 4]) -> [Point; 4] {
  let mut sorted_by_y = quad.clone().to_vec();
  sorted_by_y.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

  let mut top = vec![sorted_by_y[0], sorted_by_y[1]];
  if top[1].x < top[0].x {
    top = vec![top[1], top[0]];
  }
  let mut bottom = vec![sorted_by_y[2], sorted_by_y[3]];
  if bottom[1].x < bottom[0].x {
    bottom = vec![bottom[1], bottom[0]];
  }
  return [top[0], top[1], bottom[1], bottom[0]];
}

fn quad_edges_intersect(quad: [Point; 4]) -> bool {
  let mut edges = Vec::new();
  for start in 0..4 {
    let mut end = start + 1;
    if end == 4 {
      end = 0;
    }
    edges.push((quad[start], quad[end]));
  }

  for i in 0..edges.len() {
    for j in 0..edges.len() {
      if i == j {
        continue;
      }

      let (a_start, a_end) = edges[i];
      let (b_start, b_end) = edges[j];
      if a_start == b_start || a_end == b_start || a_start == b_end || a_end == b_end {
        continue;
      }

      if let Some(_int) = intersection(
        (a_start.x as f32, a_start.y as f32),
        (a_end.x as f32, a_end.y as f32),
        (b_start.x as f32, b_start.y as f32),
        (b_end.x as f32, b_end.y as f32),
      ) {
        return true;
      }
    }
  }

  return false;
}

fn quads(points: &Vec<Point>) -> Vec<[Point; 4]> {
  if let Some(tri) = triangulate(points) {
    let mut quads: Vec<[Point; 4]> = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for triangle_index in 0..tri.len() {
      let triangle = points_of_triangle(&tri, points, triangle_index);
      for neighbor_index in neighbors_of_triangle(&tri, triangle_index) {
        if !seen.contains(&(triangle_index, neighbor_index))
          && !seen.contains(&(neighbor_index, triangle_index))
        {
          let neighbor = points_of_triangle(&tri, points, neighbor_index);
          let mut quad = triangle.to_vec();
          quad.extend(neighbor.to_vec());
          quad.sort_by(|a, b| a.partial_cmp(&b).unwrap());
          quad.dedup();
          let sorted_quad = quad_sort([quad[0], quad[1], quad[2], quad[3]]);
          if !quad_edges_intersect(sorted_quad) {
            quads.push(sorted_quad);
          }
          seen.insert((triangle_index, neighbor_index));
        }
      }
    }

    return quads;
  }
  return Vec::new();
}

fn sample_points() -> Vec<Point> {
  let mut sample_points = Vec::new();
  for y in 0..14 {
    for x in 0..14 {
      sample_points.push(Point {
        x: x as f64 - 7.0,
        y: y as f64 - 7.0,
      });
    }
  }
  return sample_points;
}

fn transform_sample_points(
  sample_points: &Vec<Point>,
  i: &Point,
  j: &Point,
  t: &Point,
) -> Vec<Point> {
  let mut transformed_sample_points = Vec::new();
  for sample_point in sample_points.iter() {
    transformed_sample_points.push(Point {
      x: sample_point.x * i.x + sample_point.y * j.x + t.x,
      y: sample_point.x * i.y + sample_point.y * j.y + t.y,
    });
  }
  return transformed_sample_points;
}

pub fn grid(image: &GrayImage, points: &Vec<Point>) {
  let quads = quads(points);

  let mut best_points = Vec::new();
  let mut best_transformed = Vec::new();
  let sample_points = sample_points();
  for n in 0..quads.len() {
    let quad = quads[n];
    let i = quad[1] - quad[0];
    let j = quad[3] - quad[0];
    let transformed_sample_points = transform_sample_points(&sample_points, &i, &j, &quad[0]);
    let mut corner_points = Vec::new();
    for i in 0..transformed_sample_points.len() {
      let point = transformed_sample_points[i];
      if is_corner(image, point.x as u32, point.y as u32) {
        corner_points.push(point);
      }
    }
    if corner_points.len() > best_points.len() {
      best_points = corner_points;
      best_transformed = transformed_sample_points;
    }
    // if n == 1 {
    //   best_points = corner_points;
    //   best_transformed = transformed_sample_points;
    // }
  }

  if crate::debug::debug_images() {
    let mut quads_image = image::DynamicImage::ImageLuma8(image.clone()).to_rgb8();
    for i in 0..best_transformed.len() {
      let point = best_transformed[i];
      let c = image::Rgb::<u8>([0, 255, 0]);
      imageproc::drawing::draw_filled_circle_mut(
        &mut quads_image,
        (point.x as i32, point.y as i32),
        3,
        c,
      );
    }
    for i in 0..best_points.len() {
      let point = best_points[i];
      let c = image::Rgb::<u8>([255, 0, 0]);
      imageproc::drawing::draw_filled_circle_mut(
        &mut quads_image,
        (point.x as i32, point.y as i32),
        3,
        c,
      );
    }
    //     for qi in 0..quads.len() {
    //         let quad = quads[qi];
    //         println!("{:?}", quad);
    //         let [r, g, b] = color::turbo(qi as f32 / quads.len() as f32);
    //         let c = image::Rgb::<u8>([r, b, g]);
    //         for n in 0..quad.len() {
    //             let next = (n + 1) % quad.len();
    //             imageproc::drawing::draw_line_segment_mut(
    //                 &mut quads_image,
    //                 (quad[n].x as f32, quad[n].y as f32),
    //                 (quad[next].x as f32, quad[next].y as f32),
    //                 c,
    //             );
    //         }
    //     }
    crate::debug::write_rgb(&quads_image, "quads");
  }
}

#[test]
fn should_find_single_quad() {
  let points = vec![
    Point { x: 0.0, y: 0.0 },
    Point { x: 10.0, y: 0.0 },
    Point { x: 10.0, y: 10.0 },
    Point { x: 0.0, y: 10.0 },
  ];

  let quads = quads(&points);
  assert_eq!(quads.len(), 1);

  for i in 0..points.len() {
    assert_eq!(quads[0][i], points[i]);
  }
}

#[test]
fn should_find_two_quads() {
  let points = vec![
    Point { x: 0.0, y: 0.0 },
    Point { x: 10.0, y: 0.0 },
    Point { x: 20.0, y: 0.0 },
    Point { x: 10.0, y: 10.0 },
    Point { x: 20.0, y: 10.0 },
    Point { x: 0.0, y: 10.0 },
  ];

  let quads = quads(&points);
  println!("{:?}", quads);
  assert_eq!(quads.len(), 2);

  // for i in 0..points.len() {
  //   assert_eq!(quads[0][i], points[i]);
  // }
}
