use rayon::prelude::*;
use regex::Regex;
use std::{fs, process::Command};

const ARIA2_CMD: &str = "./aria2c.exe";
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
const ARIA2_BIN: &[u8] = include_bytes!("../assets/aria2c_x86.exe");
#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
const ARIA2_BIN: &[u8] = include_bytes!("../assets/aria2c_x64.exe");
const ARIA2_CONFIG: &str = include_str!("../assets/aria2.conf");
const ARIA2_CONF: &str = "./aria2.conf";

pub fn aria2_downloader(url: &str, output_path: &str, config: Option<&str>) -> i32 {
    match config {
        Some(config) => fs::write(ARIA2_CONF, config).unwrap(),
        None => fs::write(ARIA2_CONF, ARIA2_CONFIG).unwrap(),
    }
    fs::write(ARIA2_CMD, ARIA2_BIN).unwrap();
    let status = simple_spawn(
        ARIA2_CMD,
        &[
            url,
            &format!("--conf-path={}", ARIA2_CONF),
            "-o",
            output_path,
        ],
        false,
    );
    fs::remove_file(ARIA2_CMD).unwrap();
    fs::remove_file(ARIA2_CONF).unwrap();
    status
}

pub fn regex_replace_all(input: &str, pattern: &str, replacement: &str) -> String {
    let regex = Regex::new(pattern).unwrap();
    regex.replace_all(input, replacement).to_string()
}

pub fn simple_spawn(cmd: &str, args: &[&str], suppress_error: bool) -> i32 {
    let mut cmd_splits = shell_words::split(cmd).unwrap_or_default();
    cmd_splits.extend(args.par_iter().map(|s| s.to_string()).collect::<Vec<_>>());

    let mut process = Command::new(&cmd_splits[0])
        .args(&cmd_splits[1..])
        .spawn()
        .map_err(|e| eprintln!("ERROR: {} ({})", cmd, e))
        .unwrap();

    let status = process.wait().unwrap();
    if !status.success() {
        if !suppress_error {
            eprintln!("{}: failed ({})", cmd, status);
        }
        match status.code() {
            Some(code) => return code,
            None => return 1,
        }
    }
    0
}

pub fn simple_open(path: &str, as_location: bool) {
    if as_location {
        simple_spawn("explorer.exe", &["/select,", path], true);
    } else {
        simple_spawn("cmd.exe", &["/c", "start", "/wait", path], true);
    }
}
