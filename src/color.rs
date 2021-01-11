fn dot(left: &Vec<f32>, right: &Vec<f32>) -> f32 {
  let mut result = 0.0;
  for i in 0..left.len() {
    result += left[i] * right[i];
  }
  return result;
}

pub fn turbo(unconstrained_x: f32) -> [u8; 3] {
  let red4 = vec![0.13572138, 4.61539260, -42.66032258, 132.13108234];
  let green4 = vec![0.09140261, 2.19418839, 4.84296658, -14.18503333];
  let blue4 = vec![0.10667330, 12.64194608, -60.58204836, 110.36276771];
  let red2 = vec![-152.94239396, 59.28637943];
  let green2 = vec![4.27729857, 2.82956604];
  let blue2 = vec![-89.90310912, 27.34824973];

  let x = unconstrained_x.min(1.0).max(0.0);
  let v4 = vec![1.0, x, x * x, x * x * x];
  let v2 = vec![v4[2] * v4[2], v4[3] * v4[2]];
  return [
    ((dot(&v4, &red4) + dot(&v2, &red2)) * 255.0) as u8,
    ((dot(&v4, &green4) + dot(&v2, &green2)) * 255.0) as u8,
    ((dot(&v4, &blue4) + dot(&v2, &blue2)) * 255.0) as u8,
  ];
}
