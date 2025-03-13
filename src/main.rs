#![allow(non_snake_case)]

mod image_downloader;
mod epub_bundler;
mod configuration;
mod request_handler;
mod utils;

use dioxus::prelude::*;

use std::fs::{self};
use std::path::Path;
use std::env;
use std::thread::Scope;
use infer;
use tokio::task;
use tokio::task::{spawn_blocking, LocalSet};
use tokio::runtime::Runtime;
use crate::configuration::{EpubConfiguration, Settings};
use crate::image_downloader::{remove_drm_on_all_images_in_directory, DownloadImage};
use crate::request_handler::{JsonResult, Page};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(App);

    return Ok(());


    // let args: Vec<String> = env::args().collect();
    let args: Vec<String> = vec!["untitled".to_string(), "https://pocket.shonenmagazine.com/episode/13932016480029113131".to_string(), "/Users/mpb-crb/Documents/shonenmagazine".to_string()];
    // let args: Vec<String> = vec!["untitled".to_string(), "https://pocket.shonenmagazine.com/episode/13932016480029113131".to_string(), "~/Documents/shonenmagazine".to_string(), "--process-only".to_string()];

    let url = &args[1];
    let destination = &args[2];
    let process_only = args.iter().any(|arg| arg == "--process-only");
    let convert_only = args.iter().any(|arg| arg == "--convert-only");

    // run(url.to_string(), destination.to_string(), process_only, convert_only)?;

    Ok(())
}

fn run(url: String, destination: String, process_only: bool, convert_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = url.split('/').collect();
    let mut config = Settings {
        source: url.to_string(),
        destination: format!("{}/{}", destination, parts.last().unwrap_or(&"")),
        epub_file_name: "output.epub".to_string(),
        process_only,
        convert_only,
        epub_configuration: EpubConfiguration {
            title: "".to_string(),
            published_at: "".to_string(),
            cover_image: "cover.jpg".to_string(),
        },
    };

    if !Path::new(&config.destination).exists() {
        fs::create_dir_all(&config.destination)?;
    }

    if config.process_only {
        remove_drm_on_all_images_in_directory(&config.destination, vec!["cover.jpg".to_string()])?;
        if !config.convert_only {
            return Ok(())
        }
    }

    if config.convert_only {
        epub_bundler::convert_to_epub(&config)?;
        return Ok(())
    }

    let json = request_handler::extract_json_from_website(&config)?;
    let data = JsonResult::from_json_str(json.as_str()).unwrap().readable_product;

    config.epub_configuration.title = data.title;
    config.epub_configuration.published_at = data.published_at;

    DownloadImage {
        url: data.series.thumbnail_uri.to_string(),
        target_file: format!("{}/{}", &config.destination, &config.epub_configuration.cover_image),
        drm: false,
    }.download_image()?;

    for (index, page) in data.page_structure.pages.iter().enumerate() {
        match page {
            Page::Main{ src, .. } => if let Some(url) = src {
                let target = format!("{}/{:04}.jpg", &config.destination, index + 1);
                if Path::new(&target).exists() {
                    continue;
                }
                DownloadImage {
                    url: url.to_string(),
                    target_file: target,
                    drm: data.page_structure.cho_ju_giga != "usagi".to_string(),
                }.download_image()?;
            },
            _ => (),
        }
    }

    epub_bundler::convert_to_epub(&config)?;

    Ok(())
}

fn App() -> Element {
    let mut url = use_signal(|| "".to_string());
    let mut destination = use_signal(|| "".to_string());
    let mut process_only = use_signal(|| false);
    let mut convert_only = use_signal(|| false);

    let set_url = move |e: Event<FormData>| url.set(e.value());
    let set_destination = move |e: Event<FormData>| {
        let path = e.files().expect("").files()[0].clone();
        destination.set(path)
    };
    let p_o = move |e: Event<FormData>| process_only.set(e.checked());
    let c_o = move |e: Event<FormData>| convert_only.set(e.checked());

    static CSS: Asset = asset!("/dist/output.css");

    let download_data = move |_| {
        let url = url();
        let destination = destination();
        let process_only = process_only();
        let convert_only = convert_only();

        spawn_blocking(move || {
            run(url, destination, process_only, convert_only).expect("");
        });
    };

    rsx! {
        document::Stylesheet { href: CSS },
        div {
            class: "h-screen bg-manga-paper flex items-center justify-center flex-col",
            h1 {
                class: "text-5xl font-manga-headline text-manga-black drop-shadow-md",
                "エプブ・ダウンローダー"
            },
            div {
                class: "mt-4 trapezoid p-4 bg-manga-white rounded-lg shadow-md border-2 border-manga-black",
                input {
                    class: "mt-4 p-2 border border-manga-black rounded w-full focus:outline-none focus:ring-2 focus:ring-manga-red",
                    placeholder: "URL",
                    oninput: set_url,
                },
                div {
                    class: "flex items-center justify-center w-full mt-4",
                    label {
                        class: "flex flex-col items-center justify-center w-full h-32 border-2 border-dashed border-manga-black rounded-lg cursor-pointer bg-manga-light hover:bg-manga-lighter",
                        div {
                            class: "flex flex-col items-center justify-center pt-5 pb-6",
                            svg {
                                class: "w-8 h-8 mb-4 text-manga-black",
                                fill: "none",
                                stroke: "currentColor",
                                "viewBox": "0 0 24 24",
                                xmlns: "http://www.w3.org/2000/svg",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                                }
                            }
                            p {
                                class: "mb-2 text-sm text-manga-black",
                                span {
                                    class: "font-semibold",
                                    if destination() != "".to_string() {
                                        "{destination()}"
                                    } else {
                                        "クリックしてディレクトリを選択"
                                    }
                                }
                            }
                        }
                        input {
                            id: "directory-selector",
                            r#type: "file",
                            directory: "true",
                            class: "hidden",
                            oninput: set_destination,
                        }
                    }
                }
            },
            div {
                class: "mt-4 flex gap-4",
                label {
                    input {
                        class: "mr-2",
                        r#type: "checkbox",
                        checked: process_only(),
                        oninput: p_o,
                    },
                    "プロセスのみ"
                },
                label {
                    input {
                        class: "mr-2",
                        r#type: "checkbox",
                        checked: convert_only(),
                        oninput: c_o,
                    },
                    "コンバートのみ"
                },
            },
            button {
                class: "mt-4 button-manga",
                onclick: download_data,
                "次へ"
            },
        },
    }
}

