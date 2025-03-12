pub struct Settings {
    pub source: String,
    pub destination: String,
    pub epub_file_name: String,
    pub process_only: bool,
    pub convert_only: bool,
    pub epub_configuration: EpubConfiguration,
}

pub struct EpubConfiguration {
    pub title: String,
    pub published_at: String,
    pub cover_image: String,
}