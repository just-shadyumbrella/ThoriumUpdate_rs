use crate::simple_utils::*;
use const_format::concatcp;
use rayon::prelude::*;
use simd_json::prelude::*;
use std::{fs, process::exit};
use windows_registry::*;

pub const API: &str = "https://api.github.com/repos";
pub const REPO: &str = "/gz83/thorium";
pub const BETA_SOURCES: &str = "/releases";
pub const RELEASE_SOURCES: &str = concatcp!(BETA_SOURCES, "/latest");
pub const TARGET_QUERY: &str = "mini_installer.exe";

#[derive(Debug, Clone)]
pub struct InstallerInfo {
    pub version: String,
    pub url: String,
}

pub fn get_package_info(
    repo: &str,
    sources: &str,
    mut simd: Option<&str>,
) -> Option<InstallerInfo> {
    let api_url = format!("{}{}{}", API, repo, sources);
    let beta = api_url.ends_with(BETA_SOURCES);
    let sources = "sources.json";

    let status = simple_spawn("curl", &["-#L", &api_url, "-o", &sources], false);
    if status != 0 {
        exit(status);
    }

    let mut json = fs::read(&sources).unwrap();
    fs::remove_file(&sources).unwrap();
    let root = simd_json::to_owned_value(&mut json).unwrap();
    let version = root["tag_name"].as_str().unwrap();
    let value = if beta {
        &root[0]["assets"]
    } else {
        &root["assets"]
    };
    let mut assets = value
        .as_array()
        .unwrap()
        .par_iter()
        .filter_map(|asset| {
            let url = asset["browser_download_url"].as_str()?;
            if url.to_lowercase().ends_with(TARGET_QUERY) {
                Some(url.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut simd_list = assets
        .par_iter()
        .map(|url| {
            let _t = url.replace(&format!("_{}", TARGET_QUERY), "");
            let _i = _t.rfind("_").unwrap();
            _t[_i + 1..].to_owned()
        })
        .collect::<Vec<_>>();
    simd_list.sort_by(|a, b| natord::compare(a, b));

    let show = simd.as_deref().unwrap_or_default().to_lowercase() == "show";
    if show {
        simd = None;
    }

    let max_simd = if let Some(simd) = simd {
        simd
    } else if is_x86_feature_detected!("avx512f") {
        "avx512"
    } else if is_x86_feature_detected!("avx2") {
        "avx2"
    } else if is_x86_feature_detected!("avx") {
        "avx"
    } else if is_x86_feature_detected!("sse4.2") {
        "sse4"
    } else if is_x86_feature_detected!("sse3") {
        "sse3"
    } else if is_x86_feature_detected!("sse2") {
        "sse2"
    } else {
        ""
    };

    println!(
        "Currently supported CPU instructions sets: {:?}. {}: {}",
        simd_list,
        if simd.is_none() {
            "Detected CPU maximum capability"
        } else {
            "You selected"
        },
        max_simd.to_uppercase()
    );
    if show {
        exit(0);
    }

    assets.retain(|url| {
        if max_simd.is_empty() {
            false
        } else {
            url.to_lowercase()
                .ends_with(&format!("{}_{}", max_simd, TARGET_QUERY).to_lowercase())
        }
    });

    if assets.is_empty() {
        eprintln!("ERROR: No match or your CPU is not supported.");
        return None;
    }
    Some(InstallerInfo {
        version: version.replace("M", ""),
        url: assets.first().unwrap().to_owned(),
    })
}

pub fn uninstall_reg_get_string(name: &str) -> Option<String> {
    let paths = [
        (
            &CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Thorium",
        ),
        (
            &LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Thorium",
        ),
        (
            &CURRENT_USER,
            r"Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Thorium",
        ),
    ];

    for (root, path) in paths {
        if let Ok(key) = root.open(path) {
            if let Ok(version) = key.get_string(name) {
                return Some(version);
            }
        }
    }
    None
}
