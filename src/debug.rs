use image::{GrayImage, RgbImage};
use std::env;
use std::fs;
use std::path::Path;

pub fn write_gray(i: &GrayImage, name: &str) {
  if env::var("DEBUG").is_ok() {
    let output_dir = Path::new("./tmp");
    if !output_dir.is_dir() {
      fs::create_dir(output_dir).expect("Failed to create output directory")
    }

    let path = output_dir.join(format!("{}.png", name));
    i.save(path).unwrap();
  }
}

pub fn write_rgb(i: &RgbImage, name: &str) {
  if env::var("DEBUG").is_ok() {
    let output_dir = Path::new("./tmp");
    if !output_dir.is_dir() {
      fs::create_dir(output_dir).expect("Failed to create output directory")
    }

    let path = output_dir.join(format!("{}.png", name));
    i.save(path).unwrap();
  }
}
