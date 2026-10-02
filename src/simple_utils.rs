use regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    process::Command,
};

const ARIA2_CMD: &str = "./aria2c.exe";
#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
const ARIA2_BIN: &[u8] = include_bytes!("../assets/aria2c_x86.exe");
#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
const ARIA2_BIN: &[u8] = include_bytes!("../assets/aria2c_x64.exe");
const ARIA2_CONFIG: &str = include_str!("../assets/aria2.conf");
const ARIA2_CONF: &str = "./aria2.conf";

pub fn aria2_downloader(
    url: &str,
    output_path: &str,
    config: Option<&str>,
    disable_ipv6: bool,
) -> i32 {
    match config {
        Some(config) => fs::write(ARIA2_CONF, config).unwrap(),
        None => fs::write(ARIA2_CONF, ARIA2_CONFIG).unwrap(),
    }
    fs::write(ARIA2_CMD, ARIA2_BIN).unwrap();
    let conf = format!("--conf-path={}", ARIA2_CONF);
    let mut args = vec![url, &conf, "-o", output_path];
    if disable_ipv6 {
        args.insert(2, "--disable-ipv6");
    }
    let status = simple_spawn(ARIA2_CMD, &args, true, false);
    fs::remove_file(ARIA2_CMD).ok();
    fs::remove_file(ARIA2_CONF).ok();
    status
}

pub fn sha256_file(path: &str) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let hash = hasher.finalize();
    Ok(hash.iter().map(|b| format!("{:02x}", b)).collect())
}

pub fn ends_with_ignore_case(s: &str, suffix: &str) -> bool {
    let (s, suffix) = (s.as_bytes(), suffix.as_bytes());
    s.len() >= suffix.len() && s[s.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
}

pub fn find_file_matching(dir_path: &str, re: &Regex) -> Option<Vec<String>> {
    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("{}", error);
            return None;
        }
    };

    let files: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| re.is_match(name))
        .collect();

    Some(files)
}

pub fn simple_spawn(cmd: &str, args: &[&str], wait: bool, suppress_error: bool) -> i32 {
    let mut cmd_splits = shell_words::split(cmd).unwrap_or_default();
    cmd_splits.extend(args.iter().map(|s| s.to_string()));

    let mut process = Command::new(&cmd_splits[0])
        .args(&cmd_splits[1..])
        .spawn()
        .map_err(|e| eprintln!("ERROR: {} ({})", cmd, e))
        .unwrap();

    if wait {
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
    }
    0
}

pub fn simple_open(path: &str, as_location: bool) {
    if as_location {
        simple_spawn("explorer.exe", &["/select,", path], true, true);
    } else {
        simple_spawn("cmd.exe", &["/c", "start", "/wait", path], true, true);
    }
}
