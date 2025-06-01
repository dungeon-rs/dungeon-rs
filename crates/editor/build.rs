#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

fn main() {
    #[cfg(target_os = "windows")]
    generate_exe()
}

#[cfg(target_os = "windows")]
fn generate_exe() {
    println!("cargo::rerun-if-changed=../../assets/logo.png");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico_path = out_dir.join("logo.ico");

    let img = image::open("../../assets/logo.png").expect("Failed to open PNG");
    img.save_with_format(&ico_path, image::ImageFormat::Ico)
        .expect("Failed to save ICO");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico_path.to_str().unwrap());
    res.compile().unwrap();
}
