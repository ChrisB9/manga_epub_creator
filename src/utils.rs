use std::fs;

pub fn collect_images(directory: &str, exclude_filenames: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut images = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if exclude_filenames.contains(&path.file_name().expect("TODO").to_str().unwrap().to_string()) {
                continue;
            }
            if let Some(extension) = &path.extension() {
                if extension == &"jpg" || extension == &"png" {
                    images.push(path.to_str().unwrap().to_string());
                }
            }
        }
    }
    Ok(images)
}
