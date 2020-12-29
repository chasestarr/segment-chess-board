use image::{GrayImage, RgbImage};
use std::env;
use std::path::Path;

pub fn write_gray(i: &GrayImage, name: &str) {
  if env::var("DEBUG").is_ok() {
    let path = Path::new("./tmp").join(format!("{}.png", name));
    i.save(path).unwrap();
  }
}

pub fn write_rgb(i: &RgbImage, name: &str) {
  if env::var("DEBUG").is_ok() {
    let path = Path::new("./tmp").join(format!("{}.png", name));
    i.save(path).unwrap();
  }
}
