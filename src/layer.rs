use crate::bounding_box::{bounding_box, bounding_box_area, bounding_box_offset, dist_squared};
use crate::color::turbo;
use crate::debug::{debug_images, write_rgb};
use crate::lattice::get_points;
use crate::line::get_lines;
use image::GrayImage;
use imageproc::geometric_transformations::Projection;

pub fn layer(image: &GrayImage) -> (Option<Projection>, f32) {
  let (width, height) = image.dimensions();
  let lines = get_lines(&image, 100, 20);
  let mut intersection_points: Vec<(f32, f32)> = Vec::new();
  for a in lines.iter() {
    for b in lines.iter() {
      if let Some(point) = a.intersection(b) {
        intersection_points.push(point);
      }
    }
  }

  let points = get_points(&image, &intersection_points);
  let mbb = bounding_box(&points);
  let mbb_area = bounding_box_area(mbb);
  dbg!(mbb_area);
  let input_area = (width * height) as f32;
  dbg!(input_area);
  let error = 1.0 - mbb_area / input_area;
  dbg!(error);
  let offset = (error * mbb_area).sqrt().max(mbb_area.sqrt() / 6.0);
  dbg!(offset);
  // check error if it should offset
  let offset_mbb = bounding_box_offset(mbb, offset / 4.0);
  println!("{:?}", offset_mbb);

  let closest_offset_mbb: Vec<(f32, f32)> = offset_mbb
    .to_vec()
    .iter()
    .map(|op| {
      let mut closest_index = 0;
      let mut min_d = dist_squared(*op, intersection_points[0]).abs();
      for index in 1..intersection_points.len() {
        let d = dist_squared(*op, intersection_points[index]).abs();
        if d < min_d {
          min_d = d;
          closest_index = index;
        }
      }
      return intersection_points[closest_index];
    })
    .collect();

  let project_from = [
    closest_offset_mbb[0],
    closest_offset_mbb[1],
    closest_offset_mbb[2],
    closest_offset_mbb[3],
  ];

  let project_to = [
    (0.0, 0.0),
    (width as f32, 0.0),
    (width as f32, height as f32),
    (0.0, height as f32),
  ];

  if debug_images() {
    let mut intersection_image = image::DynamicImage::ImageLuma8(image.clone()).to_rgb8();
    let red = image::Rgb::<u8>([255, 0, 0]);
    let green = image::Rgb::<u8>([0, 255, 0]);
    let blue = image::Rgb::<u8>([0, 0, 255]);

    for (x, y) in intersection_points.iter() {
      imageproc::drawing::draw_hollow_circle_mut(
        &mut intersection_image,
        (*x as i32, *y as i32),
        15,
        blue,
      );
      imageproc::drawing::draw_hollow_circle_mut(
        &mut intersection_image,
        (*x as i32, *y as i32),
        3,
        blue,
      );
    }

    for (x, y) in points.iter() {
      imageproc::drawing::draw_hollow_circle_mut(
        &mut intersection_image,
        (*x as i32, *y as i32),
        15,
        red,
      );
      imageproc::drawing::draw_hollow_circle_mut(
        &mut intersection_image,
        (*x as i32, *y as i32),
        3,
        red,
      );
    }

    for line in lines.iter() {
      imageproc::drawing::draw_line_segment_mut(
        &mut intersection_image,
        line.start,
        line.end,
        green,
      );
    }

    write_rgb(&intersection_image, "lattice-intersections");

    let mut mbb_image = image::DynamicImage::ImageLuma8(image.clone()).to_rgb8();
    for i in 0..mbb.len() {
      let p = mbb[i];
      imageproc::drawing::draw_filled_circle_mut(
        &mut mbb_image,
        (p.0 as i32, p.1 as i32),
        3,
        image::Rgb::<u8>([255, 0, 0]),
      );
    }
    for i in 0..offset_mbb.len() {
      let p = offset_mbb[i];
      imageproc::drawing::draw_filled_circle_mut(
        &mut mbb_image,
        (p.0 as i32, p.1 as i32),
        5,
        image::Rgb::<u8>([0, 255, 0]),
      );
    }
    write_rgb(&mbb_image, "mbb-offset");
  }

  return (
    Projection::from_control_points(project_from, project_to),
    error,
  );
}
