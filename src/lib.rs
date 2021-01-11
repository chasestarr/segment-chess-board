use image::imageops::{contrast, resize, FilterType};
use imageproc::geometric_transformations::{warp_with, Interpolation, Projection};

mod bounding_box;
mod cluster;
mod color;
mod debug;
mod lattice;
mod layer;
mod line;

use cluster::{dbscan, mean_shift};
use lattice::get_points;
use line::get_lines;

pub fn segment(i: &image::DynamicImage) {
    let input_image_rgb = i.to_rgb8();
    let formatted_rgb = resize(&input_image_rgb, 500, 500, FilterType::Triangle);
    let formatted_gray = &image::DynamicImage::ImageRgb8(formatted_rgb.clone()).into_luma8();

    let lines = get_lines(&formatted_gray, 100, 30);
    let points = get_points(&formatted_gray, &lines);

    let (w, h) = formatted_gray.dimensions();

    let clustered = mean_shift(&points);
    let mut clusters_image: image::RgbImage = image::ImageBuffer::new(w, h);
    let mut clusters_max = 0;
    for c in clustered.iter() {
        if *c > clusters_max {
            clusters_max = *c;
        }
    }
    for i in 0..points.len() {
        let (x, y) = points[i];
        imageproc::drawing::draw_filled_circle_mut(
            &mut clusters_image,
            (x as i32, y as i32),
            3,
            image::Rgb::<u8>(color::turbo(clustered[i] as f32 / clusters_max as f32)),
        );
    }
    debug::write_rgb(&clusters_image, "clusters");

    let db_clustered = dbscan(&points, 75.0, 5);
    let mut db_clusters_image: image::RgbImage = image::ImageBuffer::new(w, h);
    let mut db_clusters_max = 0;
    for c in db_clustered.iter() {
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
            image::Rgb::<u8>(color::turbo(
                db_clustered[i] as f32 / db_clusters_max as f32,
            )),
        );
    }
    debug::write_rgb(&db_clusters_image, "db_clusters");

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for (x, y) in points.iter() {
        sum_x += x;
        sum_y += y;
    }
    let avg_x = sum_x / points.len() as f32;
    let avg_y = sum_y / points.len() as f32;
    println!("{}, {}", avg_x, avg_y);

    let mut center_image: image::RgbImage = image::ImageBuffer::new(w, h);
    imageproc::drawing::draw_filled_circle_mut(
        &mut center_image,
        (
            (sum_x / points.len() as f32) as i32,
            (sum_y / points.len() as f32) as i32,
        ),
        2,
        image::Rgb::<u8>([59, 189, 255]),
    );
    let mut max_dist: i32 = 0;
    let mut min_dist: i32 = 100000;
    let dists: Vec<i32> = points
        .iter()
        .map(|(x, y)| ((x - avg_x).abs() + (y - avg_y).abs()) as i32)
        .collect();
    for dist in dists.iter() {
        if *dist > max_dist {
            max_dist = *dist;
        }
        if *dist < min_dist {
            min_dist = *dist;
        }
    }
    let range = max_dist - min_dist;
    for index in 0..dists.len() {
        let d = dists[index];
        let (x, y) = points[index];
        let v = 1.0 - (d - min_dist) as f32 / range as f32;
        imageproc::drawing::draw_filled_circle_mut(
            &mut center_image,
            (x as i32, y as i32),
            3,
            image::Rgb::<u8>(color::turbo(v)),
        );
    }
    debug::write_rgb(&center_image, "center-of-points");

    let mbb = bounding_box::bounding_box(&points);
    println!("{:?}", mbb);
    let mbb_area = bounding_box::convex_hull_area(&mbb.to_vec());
    let input_area = (w * h) as f32;
    let error = 1.0 - mbb_area / input_area;
    let offset = (error * mbb_area).sqrt().max(mbb_area.sqrt() / 6.0);
    // check error if it should offset
    let offset_mbb = bounding_box::bounding_box_offset(mbb, offset / 4.0);
    println!("offset mbb {:?}", offset_mbb);
    // println!("error {}, offset {}", error, offset);

    if crate::debug::debug_images() {
        let mut mbb_image = image::DynamicImage::ImageLuma8(formatted_gray.clone()).to_rgb8();
        // let mut mbb_image: image::RgbImage = image::ImageBuffer::new(500, 500);
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
                3,
                image::Rgb::<u8>([0, 255, 0]),
            );
        }
        crate::debug::write_rgb(&mbb_image, "mbb-offset");
    }

    let projection = Projection::from_control_points(
        offset_mbb,
        [
            (0.0, h as f32),
            (w as f32, h as f32),
            (w as f32, 0.0),
            (0.0, 0.0),
        ],
    )
    .expect("Could not compute projection matrix")
    .invert();

    let warped_rgb = warp_with(
        &formatted_rgb,
        |x, y| projection * (x, y),
        Interpolation::Bilinear,
        image::Rgb([0, 0, 0]),
    );
    crate::debug::write_rgb(&warped_rgb, "warped");
}
