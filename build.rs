fn main() {
    println!("cargo:rerun-if-changed=assets/packaging/carriage_run.ico");

    let is_windows_target = std::env::var("TARGET")
        .map(|target| target.contains("windows"))
        .unwrap_or(false);
    if cfg!(windows) && is_windows_target {
        let mut resource = winres::WindowsResource::new();
        resource
            .set_icon("assets/packaging/carriage_run.ico")
            .set_language(0x0409)
            .set_version_info(winres::VersionInfo::FILEVERSION, 0x0001_0000_0000_0000)
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001_0000_0000_0000)
            .set("FileDescription", "Carriage Run")
            .set("ProductName", "Carriage Run")
            .set("LegalCopyright", "WebHatchery");
        resource
            .compile()
            .expect("Windows application resource compilation failed");
    }
}
