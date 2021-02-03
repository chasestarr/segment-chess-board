use image::imageops::{resize, FilterType};
use image::open;
use segment_chess_board::segment_layered;
use std::path::Path;

fn main() {
  // let input_path = Path::new("images/camera_01.png");
  // let input_path = Path::new("images/camera_02.png");
  // let input_path = Path::new("images/camera_03.png");
  // let input_path = Path::new("images/camera_03_warped_02.png");
  // let input_path = Path::new("images/camera_04.png");
  // let input_path = Path::new("images/camera_05.png");
  // let input_path = Path::new("images/camera_05_warped.png");
  // let input_path = Path::new("images/camera_06.png");
  let input_path = Path::new("images/camera_07.png");
  // let input_path = Path::new("images/screenshot_02.png");
  // let input_path = Path::new("images/screenshot_03.png");
  // let input_path = Path::new("images/camera_01_warped.png");
  if !input_path.is_file() {
    panic!("Input file does not exist");
  }

  let input_image = open(input_path).expect(&format!("Could not load image at {:?}", input_path));
  let gray = input_image.into_luma8();
  segment_layered(&gray);
}
