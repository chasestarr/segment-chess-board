use image::{GrayImage, RgbImage};
use std::fs;
use std::path::Path;

pub fn debug_images() -> bool {
  return std::env::var("DEBUG").is_ok();
}

pub fn write_gray(i: &GrayImage, name: &str) {
  if debug_images() {
    let output_dir = Path::new("./tmp");
    if !output_dir.is_dir() {
      fs::create_dir(output_dir).expect("Failed to create output directory")
    }

    let path = output_dir.join(format!("{}.png", name));
    i.save(path).unwrap();
  }
}

pub fn write_rgb(i: &RgbImage, name: &str) {
  if debug_images() {
    let output_dir = Path::new("./tmp");
    if !output_dir.is_dir() {
      fs::create_dir(output_dir).expect("Failed to create output directory")
    }

    let path = output_dir.join(format!("{}.png", name));
    i.save(path).unwrap();
  }
}
