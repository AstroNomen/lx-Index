fn main() {
    #[cfg(target_os = "windows")]
    let ui_file = "ui/appwindow_windows.slint";

    #[cfg(not(target_os = "windows"))]
    let ui_file = "ui/appwindow.slint";

    slint_build::compile(ui_file).expect("Slint build failed");

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app_icon.ico");
        res.set("ProductName", "lx Index");
        res.set("FileDescription", "Literature Management HUD");
        res.set("LegalCopyright", "© 2026 Lucentia");
        if let Err(e) = res.compile() {
            eprintln!("Warning: failed to compile windows resources: {}", e);
        }
    }
}
