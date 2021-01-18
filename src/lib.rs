use image::imageops::{blur, contrast, resize, FilterType};
use image::ImageBuffer;
use imageproc::geometric_transformations::{warp_with, Interpolation, Projection};
use wasm_bindgen::prelude::*;

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

// http://wiki.bitplan.com/index.php/PlayChessWithAWebCam/Papers#Stonewall_Chess_Computer_Vision
// https://www.esimov.com/2020/01/pigo-wasm#.X_0caWRKjUL
// https://github.com/esimov/pigo
// https://github.com/ColinEberhardt/wasm-sudoku-solver

#[wasm_bindgen]
pub fn list(width: u32, height: u32, buf: Vec<u8>) -> Vec<u8> {
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        let channels = 4;
        let pixel_index = (width * channels * y + x * channels) as usize;
        return image::Rgba([
            buf[pixel_index],
            buf[pixel_index + 1],
            buf[pixel_index + 2],
            buf[pixel_index + 3],
        ]);
    });
    return blur(&img, 5.0).into_vec();
}

fn test_mbb() {
    let hull = vec![
        (90.0, 120.0),
        (130.0, 100.0),
        (190.0, 120.0),
        (180.0, 220.0),
        (70.0, 190.0),
    ];
    let mbb = crate::bounding_box::oriented_bounding_box(&hull);
    let mut img: image::RgbImage = image::ImageBuffer::new(300, 300);
    for index in 0..hull.len() {
        let p = hull[index];
        imageproc::drawing::draw_filled_circle_mut(
            &mut img,
            (p.0 as i32, p.1 as i32),
            3,
            image::Rgb::<u8>([0, 128, 255]),
        );
        imageproc::drawing::draw_line_segment_mut(
            &mut img,
            hull[index],
            hull[(index + 1) % hull.len()],
            image::Rgb::<u8>([0, 255, 100]),
        );
    }
    for index in 0..mbb.len() {
        let p = mbb[index];
        imageproc::drawing::draw_filled_circle_mut(
            &mut img,
            (p.0 as i32, p.1 as i32),
            3,
            image::Rgb::<u8>([255, 128, 0]),
        );
        imageproc::drawing::draw_line_segment_mut(
            &mut img,
            mbb[index],
            mbb[(index + 1) % mbb.len()],
            image::Rgb::<u8>([255, 0, 128]),
        );
    }
    debug::write_rgb(&img, "test-mbb");
}

pub fn segment(i: &image::DynamicImage) {
    let input_image_rgb = contrast(&i.to_rgb8(), 3.0);
    let formatted_rgb = resize(&input_image_rgb, 400, 400, FilterType::Triangle);
    let formatted_gray = &image::DynamicImage::ImageRgb8(formatted_rgb.clone()).into_luma8();

    let lines = get_lines(&formatted_gray, 100, 20);

    let mut intersection_points: Vec<(f32, f32)> = Vec::new();
    for a in lines.iter() {
        for b in lines.iter() {
            if let Some(point) = a.intersection(b) {
                intersection_points.push(point);
            }
        }
    }

    let points = get_points(&formatted_gray, &intersection_points);

    if crate::debug::debug_images() {
        let mut intersection_image =
            image::DynamicImage::ImageLuma8(formatted_gray.clone()).to_rgb8();
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

        debug::write_rgb(&intersection_image, "lattice-intersections");
    }

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
    let mbb_area = bounding_box::bounding_box_area(mbb);
    let input_area = (w * h) as f32;
    println!("input_area {}, mbb_area {:?}", input_area, mbb_area);
    let error = 1.0 - mbb_area / input_area;
    let offset = (error * mbb_area).sqrt().max(mbb_area.sqrt() / 6.0);
    println!("error {}, offset {:?}", error, offset);
    // check error if it should offset
    let offset_mbb = bounding_box::bounding_box_offset(mbb, offset / 4.0);
    println!("offset mbb {:?}", offset_mbb);
    // println!("error {}, offset {}", error, offset);

    let closest_offset_mbb: Vec<(f32, f32)> = offset_mbb
        .to_vec()
        .iter()
        .map(|op| {
            let mut closest_index = 0;
            let mut min_d = bounding_box::dist_squared(*op, intersection_points[0]).abs();
            for index in 1..intersection_points.len() {
                let d = bounding_box::dist_squared(*op, intersection_points[index]).abs();
                if d < min_d {
                    min_d = d;
                    closest_index = index;
                }
            }
            return intersection_points[closest_index];
        })
        .collect();

    if crate::debug::debug_images() {
        let mut mbb_image = image::DynamicImage::ImageLuma8(formatted_gray.clone()).to_rgb8();
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
        for i in 0..closest_offset_mbb.len() {
            let p = closest_offset_mbb[i];
            imageproc::drawing::draw_filled_circle_mut(
                &mut mbb_image,
                (p.0 as i32, p.1 as i32),
                3,
                image::Rgb::<u8>([0, 0, 255]),
            );
        }
        crate::debug::write_rgb(&mbb_image, "mbb-offset");
    }

    let projection = Projection::from_control_points(
        [
            closest_offset_mbb[0],
            closest_offset_mbb[1],
            closest_offset_mbb[2],
            closest_offset_mbb[3],
        ],
        [
            (0.0, 0.0),
            (w as f32, 0.0),
            (w as f32, h as f32),
            (0.0, h as f32),
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
