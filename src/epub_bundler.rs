use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use chrono::{DateTime, FixedOffset, Utc};
use epub_builder::{EpubBuilder, EpubContent, MetadataOpf, ZipLibrary};

use crate::configuration::Settings;
use crate::utils;

pub fn convert_to_epub(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    // Process all images in the directory
    let images = utils::collect_images(config.destination.as_str(), vec![config.epub_configuration.cover_image.to_string()]);

    // Create EPUB
    let epub_path = Path::new(&config.destination).join(&config.epub_file_name);
    let zip_library = ZipLibrary::new()?;
    let mut epub = EpubBuilder::new(zip_library)?;

    epub.set_title(&config.epub_configuration.title);
    /*let date_time: DateTime<FixedOffset> = DateTime::parse_from_str(&config.epub_configuration.published_at.as_str(), "%Y-%m-%dT%H:%M:%S%z")
        .expect("Failed to parse date-time");
    let date_time_utc: DateTime<Utc> = date_time.with_timezone(&Utc);
    epub.set_publication_date(date_time_utc);*/
    epub.set_authors(vec!["Shonenmagazine".to_string()]);
    epub.add_metadata_opf(MetadataOpf { name: "primary-writing-mode".to_string(), content: "vertical-rl".to_string() });
    let path_buf = PathBuf::from(format!("{}/{}", config.destination, config.epub_configuration.cover_image));
    let kind = infer::get_from_path(path_buf.as_path())
        .expect("file read successfully")
        .expect("file type is known");

    // Add image to EPUB
    let image_data = fs::read(path_buf.as_path())?;
    epub.add_cover_image(&config.epub_configuration.cover_image, &image_data[..], kind.mime_type().to_string())?;

    let css = ""; // maybe we should add some styling
    epub.stylesheet(css.as_bytes())?;

    let mut images = images.expect("Images need to be available");
    images.sort_by_key(|path| {
        // Extract the filename from the path
        let filename = Path::new(path).file_stem().unwrap().to_str().unwrap();
        // Parse the numeric part of the filename
        filename.trim_start_matches('0').parse::<u32>().unwrap()
    });

    for (index, image_path) in images.iter().enumerate() {
        let path_buf = PathBuf::from(image_path);
        let kind = infer::get_from_path(path_buf.as_path())
            .expect("file read successfully")
            .expect("file type is known");

        // Add image to EPUB
        let image_data = fs::read(path_buf.as_path())?;
        let image_name = format!("image_{}.{}", index + 1, kind.extension());
        epub.add_resource(&image_name, &image_data[..], kind.mime_type().to_string())?;

        // Add chapter with image
        let chapter_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
        <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en" lang="en">
            <img src="{}" alt="Page {}"/>
        </html>"#, image_name, index + 1);
        epub.add_content(EpubContent::new(
            format!("Page {}", index + 1),
            chapter_content.as_bytes(),
        ))?;
    }

    // Finalize EPUB
    epub.generate(File::create(epub_path)?)?;

    Ok(())
}
