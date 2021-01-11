use std::collections::HashMap;

fn dist((ax, ay): (f32, f32), (bx, by): (f32, f32)) -> f32 {
  let dist_squared = (ax - bx).powf(2.0) + (ay - by).powf(2.0);
  return dist_squared.sqrt();
}

fn gaussian_kernel(distance: f32, kernel_bandwidth: f32) -> f32 {
  return (-1.0 / 2.0 * (distance * distance) / (kernel_bandwidth * kernel_bandwidth)).exp();
}

fn mean_shift_shift_point(
  point: (f32, f32),
  points: &Vec<(f32, f32)>,
  kernel_bandwidth: f32,
) -> (f32, f32) {
  let mut shifted_x = 0.0;
  let mut shifted_y = 0.0;
  let mut total_weight = 0.0;

  for i in 0..points.len() {
    let (other_x, other_y) = points[i];
    let distance = dist(point, (other_x, other_y));
    let weight = gaussian_kernel(distance, kernel_bandwidth);
    shifted_x += other_x * weight;
    shifted_y += other_y * weight;
    total_weight += weight;
  }

  shifted_x /= total_weight;
  shifted_y /= total_weight;
  return (shifted_x, shifted_y);
}

fn mean_shift_shift(points: &Vec<(f32, f32)>, kernel_bandwidth: f32) -> Vec<(f32, f32)> {
  let EPSILON = 50.0;
  let mut shifted_points = points.clone();
  let mut max_shift_distance = 0.0;
  let mut stop_moving: Vec<bool> = Vec::new();
  stop_moving.fill(false);
  while max_shift_distance > EPSILON {
    max_shift_distance = 0.0;
    for i in 0..points.len() {
      if !stop_moving[i] {
        let point = mean_shift_shift_point(points[i], points, kernel_bandwidth);
        let distance = dist(point, points[i]);
        if distance > max_shift_distance {
          max_shift_distance = distance;
        }
        if distance <= EPSILON {
          stop_moving[i] = true;
        }
        shifted_points[i] = point;
      }
    }
  }
  return shifted_points;
}

fn mean_shift_cluster(shifted_points: Vec<(f32, f32)>) -> Vec<usize> {
  let mut clusters: Vec<(f32, f32)> = Vec::new();
  let mut assignments: Vec<usize> = Vec::new();
  for i in 0..shifted_points.len() {
    for j in 0..clusters.len() {
      let distance = dist(clusters[j], shifted_points[i]);
      if distance < 300.0 {
        assignments.push(j);
        break;
      }
    }

    if assignments.len() == 0 || assignments.len() - 1 < i {
      clusters.push(shifted_points[i]);
      assignments.push(clusters.len());
    }
  }
  return assignments;
}

pub fn mean_shift(points: &Vec<(f32, f32)>) -> Vec<usize> {
  let shifted = mean_shift_shift(points, 200.0);
  return mean_shift_cluster(shifted);
}

// https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust
#[derive(Debug)]
struct HashPoint(f32, f32);
impl HashPoint {
  fn key(&self) -> (u32, u32) {
    unsafe { (std::mem::transmute(self.0), std::mem::transmute(self.1)) }
  }
}
impl std::hash::Hash for HashPoint {
  fn hash<H>(&self, state: &mut H)
  where
    H: std::hash::Hasher,
  {
    self.key().hash(state)
  }
}
impl PartialEq for HashPoint {
  fn eq(&self, other: &HashPoint) -> bool {
    self.key() == other.key()
  }
}
impl Eq for HashPoint {}

fn dbscan_range_query(points: &Vec<(f32, f32)>, point: (f32, f32), eps: f32) -> Vec<(f32, f32)> {
  let mut neighbors = Vec::new();
  for other_point in points.iter() {
    if point != *other_point && dist(point, *other_point) <= eps {
      neighbors.push(*other_point);
    }
  }
  return neighbors;
}

// https://allalgorithms.com/docs/dbscan
pub fn dbscan(points: &Vec<(f32, f32)>, eps: f32, min_pts: usize) -> Vec<usize> {
  let mut cluster_index = 0;
  let mut cluster_assignments: HashMap<HashPoint, usize> = HashMap::new();

  for i in 0..points.len() {
    let point = points[i];
    if cluster_assignments.contains_key(&HashPoint(point.0, point.1)) {
      continue;
    }

    let mut neighbors = dbscan_range_query(points, points[i], eps);
    if neighbors.len() < min_pts {
      cluster_assignments.insert(HashPoint(point.0, point.1), 0);
      continue;
    }

    cluster_index += 1;
    cluster_assignments.insert(HashPoint(point.0, point.1), cluster_index);

    while neighbors.len() > 0 {
      if let Some(neighbor) = neighbors.pop() {
        if let Some(index) = cluster_assignments.get_mut(&HashPoint(neighbor.0, neighbor.1)) {
          if *index == 0 {
            *index = cluster_index;
          } else {
            continue;
          }
        }

        cluster_assignments.insert(HashPoint(neighbor.0, neighbor.1), cluster_index);

        let neighbor_neighbors = dbscan_range_query(points, neighbor, eps);
        if neighbor_neighbors.len() >= min_pts {
          for nn in neighbor_neighbors.iter() {
            if !cluster_assignments.contains_key(&HashPoint(nn.0, nn.1)) {
              neighbors.push(*nn);
            }
          }
        }
      }
    }
  }

  let mut result = Vec::new();
  for p in points.iter() {
    if let Some(ci) = cluster_assignments.get(&HashPoint(p.0, p.1)) {
      result.push(*ci)
    } else {
      result.push(0);
    }
  }
  return result;
}
