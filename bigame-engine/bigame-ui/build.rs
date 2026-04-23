use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    let data_dir = workspace.join("data");
    let style_dir = workspace.join("style");
    let gresource_xml = data_dir.join("resources.gresource.xml");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let output = format!("{out_dir}/resources.gresource");

    // Copy style.css into data/ so glib-compile-resources can find it
    let css_src = style_dir.join("style.css");
    let css_dst = data_dir.join("style.css");
    if css_src.exists() {
        std::fs::copy(&css_src, &css_dst).expect("copy style.css");
    }

    // Temporarily copy icons from usr/share/icons/ into data/
    let icons_src = workspace.join("usr/share/icons");
    let icons_dst = data_dir.join("icons");
    if icons_src.exists() {
        let _ = std::process::Command::new("cp")
            .arg("-r")
            .arg(&icons_src)
            .arg(&data_dir)
            .status();
    }

    let status = Command::new("glib-compile-resources")
        .arg("--sourcedir")
        .arg(&data_dir)
        .arg("--target")
        .arg(&output)
        .arg(&gresource_xml)
        .status()
        .expect("glib-compile-resources");

    assert!(status.success(), "glib-compile-resources failed");

    // Cleanup copied CSS and icons
    let _ = std::fs::remove_file(&css_dst);
    let _ = std::process::Command::new("rm")
        .arg("-rf")
        .arg(&icons_dst)
        .status();

    println!("cargo::rerun-if-changed={}", gresource_xml.display());
    println!(
        "cargo::rerun-if-changed={}",
        style_dir.join("style.css").display()
    );
    println!(
        "cargo::rerun-if-changed={}",
        workspace
            .join("usr/share/icons/hicolor/scalable/apps/com.biglinux.BiGameMode.svg")
            .display()
    );
}
