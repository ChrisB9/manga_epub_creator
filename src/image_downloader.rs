use std::fs::File;
use std::io;
use image::{DynamicImage, ImageBuffer, Rgba};

use crate::request_handler::get_request_builder_for_url;
use crate::utils;

pub struct DownloadImage {
    pub url: String,
    pub target_file: String,
    pub drm: bool,
}

impl DownloadImage {
    pub fn download_image(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut response = get_request_builder_for_url(&self.url)?;

        let path = &self.target_file.as_str();
        let mut file = File::create(path)?;
        io::copy(&mut response, &mut file)?;

        if self.drm {
            remove_drm(self.target_file.as_str())?;
        }

        Ok(())
    }
}

pub fn remove_drm(input_filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = match image::open(input_filename) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Failed to open image {}: {:?}", input_filename, e);
            return Err(Box::new(e));
        }
    };

    let img = img.into_rgba8();
    let (width, height) = img.dimensions();

    let divide_num = 4;
    let multiple = 8;
    let cell_width = (width / (divide_num * multiple)) * multiple;
    let cell_height = (height / (divide_num * multiple)) * multiple;

    let mut new_img = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    for e in 0..(divide_num * divide_num) {
        let t = (e / divide_num) * cell_height;
        let n = (e % divide_num) * cell_width;
        let r = e / divide_num;
        let i = e % divide_num;
        let u = i * divide_num + r;
        let s = (u % divide_num) * cell_width;
        let c = (u / divide_num) * cell_height;

        for y in 0..cell_height {
            for x in 0..cell_width {
                let src_pixel = img.get_pixel(n + x, t + y);
                new_img.put_pixel(s + x, c + y, *src_pixel);
            }
        }
    }

    // Convert the image to RGB before saving as JPEG
    let new_img = DynamicImage::ImageRgba8(new_img).into_rgb8();
    new_img.save(input_filename)?;

    Ok(())
}

pub fn remove_drm_on_all_images_in_directory(directory: &str, exclude_filenames: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in utils::collect_images(directory, exclude_filenames) ? {
        if entry.ends_with(".jpg") || entry.ends_with(".png") {
            remove_drm(entry.as_str())?;
        }
    }
    Ok(())
}
