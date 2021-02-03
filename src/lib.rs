use image::imageops::{blur, contrast, resize, unsharpen, FilterType};
use image::{GrayImage, ImageBuffer};
use imageproc::geometric_transformations::{warp_with, Interpolation, Projection};
use wasm_bindgen::prelude::*;

mod bounding_box;
mod cluster;
mod color;
mod debug;
mod delaunay_triangulation;
mod lattice;
mod layer;
mod line;
mod point;
mod sample_consensus;

use cluster::{dbscan, mean_shift};
use lattice::get_points;
use layer::layer;
use line::get_lines;

// http://wiki.bitplan.com/index.php/PlayChessWithAWebCam/Papers#Stonewall_Chess_Computer_Vision
// https://www.esimov.com/2020/01/pigo-wasm#.X_0caWRKjUL
// https://github.com/esimov/pigo
// https://github.com/ColinEberhardt/wasm-sudoku-solver

pub fn segment_layered(image: &GrayImage) {
    let resized = resize(image, 400, 400, FilterType::Gaussian);
    let mut next_image = resized;
    let mut index = 0;
    loop {
        if index > 4 {
            break;
        }

        let (p, e) = layer(&next_image);
        match p {
            Some(projection) => {
                next_image = warp_with(
                    &next_image,
                    |x, y| projection.invert() * (x, y),
                    Interpolation::Bilinear,
                    image::Luma([0]),
                );
            }
            None => {
                break;
            }
        }

        dbg!(e);
        if e < 0.5 {
            break;
        }
        debug::write_gray(&next_image, format!("layer_{}", index).as_str());
        index += 1;
    }
    debug::write_gray(&next_image, "layer_result");
}

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

fn from_js_image_buffer(width: u32, height: u32, buf: Vec<u8>) -> image::RgbImage {
    return ImageBuffer::from_fn(width, height, |x, y| {
        let channels = 4;
        let pixel_index = (width * channels * y + x * channels) as usize;
        return image::Rgb([buf[pixel_index], buf[pixel_index + 1], buf[pixel_index + 2]]);
    });
}

#[wasm_bindgen]
pub fn wasm_bounding_box(width: u32, height: u32, buf: Vec<u8>) -> Vec<f32> {
    let w = 400;
    let h = 400;
    let img = from_js_image_buffer(width, height, buf);
    let scaled = resize(&img, w, h, FilterType::Triangle);
    let gray = &image::DynamicImage::ImageRgb8(scaled.clone()).into_luma8();
    let lines = get_lines(&gray, 100, 20);
    let mut intersection_points: Vec<(f32, f32)> = Vec::new();
    for a in lines.iter() {
        for b in lines.iter() {
            if let Some(point) = a.intersection(b) {
                intersection_points.push(point);
            }
        }
    }
    let points = get_points(&gray, &intersection_points);
    if points.len() < 4 {
        return vec![];
    }

    let mbb = bounding_box::bounding_box(&points);
    let mbb_area = bounding_box::bounding_box_area(mbb);
    let input_area = (w * h) as f32;
    if mbb_area < input_area / 20.0 {
        return Vec::new();
    }
    let error = 1.0 - mbb_area / input_area;
    let offset = (error * mbb_area).sqrt().max(mbb_area.sqrt() / 6.0);
    // check error if it should offset
    let offset_mbb = bounding_box::bounding_box_offset(mbb, offset / 4.0);

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

    let mut output = Vec::new();
    for (x, y) in closest_offset_mbb.iter() {
        output.push(*x);
        output.push(*y);
    }
    return output;
}

pub fn segment(i: &image::DynamicImage) {
    let input_image_rgb = i.to_rgb8();
    let formatted_rgb = resize(&input_image_rgb, 400, 400, FilterType::Gaussian);
    let formatted_gray = unsharpen(
        &image::DynamicImage::ImageRgb8(formatted_rgb.clone()).into_luma8(),
        2.0,
        50,
    );

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

    let (w, h) = formatted_gray.dimensions();
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
    }

    println!("points {:?}", points);

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
