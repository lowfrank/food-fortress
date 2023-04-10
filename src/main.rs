#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release
#![cfg(all(target_arch = "x86_64", target_os = "windows"))] // Set target os as Windows
#![allow(non_snake_case)]

mod app;

use app::frontend::App;
use app::log;

fn main() {
    if cfg!(target_os = "windows") {
        eframe::run_native(
            "Fridge",
            eframe::NativeOptions {
                // initial_window_size has been hardcoded but I like it that way
                initial_window_size: Some((660.0, 550.0).into()),
                icon_data: load_image("images\\refrigerator.png"),
                ..Default::default()
            },
            Box::new(|cc| Box::new(App::new(cc))),
        );
    }
}

/// Load an image using the [`image`] crate. Return [`None`] if the image cannot be opened.
fn load_image(path: &str) -> Option<eframe::IconData> {
    let Some(img) = image::open(path).ok() else {
        log::warning(format!("App icon '{}' could not be found", path));
        return None;
    };

    let img = img.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    Some(eframe::IconData {
        rgba,
        width,
        height,
    })
}
