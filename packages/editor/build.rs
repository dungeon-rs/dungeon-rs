use std::{env, path::PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo::rerun-if-changed=../../assets/logo.png");

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let ico_path = out_dir.join("logo.ico");

        let img = image::open("../../assets/logo.png").expect("Failed to open PNG");
        img.save_with_format(&ico_path, image::ImageFormat::Ico)
            .expect("Failed to save ICO");

        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico_path.to_str().unwrap());
        res.compile().unwrap();
    }
}
