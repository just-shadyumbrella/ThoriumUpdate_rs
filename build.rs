use chrono::{Datelike, Local};
use std::fs;
use std::process::{Command, Stdio};

#[cfg(windows)]
fn main() -> std::io::Result<()> {
    // Fix this if you want to use a custom icon or manifest
    let app_icon = "app.ico";
    let manifest = "app.manifest";

    let mut res = winresource::WindowsResource::new();
    let icon_is_file = fs::metadata(app_icon)
        .map(|m| m.is_file())
        .unwrap_or_else(|_| false);
    if icon_is_file {
        res.set_icon(app_icon);
    }
    let manifest_is_file = fs::metadata(manifest)
        .map(|m| m.is_file())
        .unwrap_or_else(|_| false);
    if manifest_is_file {
        res.set_manifest_file(manifest);
    }

    // Git rev-parse
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let rev =
        String::from_utf8(output.stdout).unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
    let year = Local::now().year();
    let origin = "thrup.exe";

    res.set("CompanyName", env!("CARGO_PKG_AUTHORS"))
        .set("FileDescription", env!("CARGO_PKG_DESCRIPTION"))
        // .set("FileVersion", "10.0.26100.1 (WinBuild.160101.0800)")z
        .set("InternalName", origin)
        .set(
            "LegalCopyright",
            &format!(
                "© {} {}. All rights reserved.",
                year,
                env!("CARGO_PKG_AUTHORS")
            ),
        )
        .set("OriginalFilename", origin)
        .set("ProductName", env!("CARGO_PKG_DESCRIPTION"))
        .set(
            "ProductVersion",
            &format!("{} ({})", env!("CARGO_PKG_VERSION"), rev),
        );
    res.compile()?;
    Ok(())
}
