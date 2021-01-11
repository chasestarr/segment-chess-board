use image::open;
use segment_chess_board::segment;
use std::env;
use std::path::Path;

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
  segment(&input_image);
}
