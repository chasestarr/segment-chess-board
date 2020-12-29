use image::imageops::{contrast, resize, FilterType};
use image::open;
use std::env;
use std::path::Path;

mod debug;
mod lattice;
mod layer;
mod line;

use lattice::get_points;
use line::get_lines;

fn main() {
    if env::args().len() != 2 {
        panic!("Please enter an input file")
    }

    let input_path = env::args().nth(1).unwrap();
    let input_path = Path::new(&input_path);
    if !input_path.is_file() {
        panic!("Input file does not exist");
    }

    let input_image = open(input_path).expect(&format!("Could not load image at {:?}", input_path));
    let input_image_gray = input_image.to_luma8();
    let resized = resize(&input_image_gray, 500, 500, FilterType::Triangle);
    let contrasted = contrast(&resized, 50.0);

    let lines = get_lines(&contrasted);
    let _points = get_points(&contrasted, &lines);
}
