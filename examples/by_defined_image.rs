use image::open;
use segment_chess_board::segment;
use std::path::Path;

fn main() {
  // let input_path = Path::new("images/camera_01.png");
  let input_path = Path::new("images/camera_02.png");
  // let input_path = Path::new("images/camera_03.png");
  // let input_path = Path::new("images/camera_03_warped_02.png");
  // let input_path = Path::new("images/screenshot_02.png");
  // let input_path = Path::new("images/screenshot_03.png");
  // let input_path = Path::new("images/camera_01_warped.png");
  if !input_path.is_file() {
    panic!("Input file does not exist");
  }

  let input_image = open(input_path).expect(&format!("Could not load image at {:?}", input_path));
  segment(&input_image);
}
