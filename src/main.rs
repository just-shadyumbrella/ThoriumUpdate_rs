use hide_console::hide_console;
use rayon::prelude::*;
use regex::Regex;
use std::{env, fs, process};
use versions::Versioning;

mod simple_utils;
use simple_utils::*;
mod thorium;
use thorium::*;

fn show_help() {
    let mut readme = include_str!("../readme.md")
        .replace("(../../", &format!("({}/", env!("CARGO_PKG_REPOSITORY")));
    readme = regex_replace_all(&readme, r"(?m)^.*```.*$", "");
    readme = regex_replace_all(&readme, r"> \[!(\w+)\]\n> ", "> **$1:** ");
    readme = regex_replace_all(&readme, r"\[(.+)\]\((.+)\)(.)", "$1 [$2]$3");
    readme = regex_replace_all(&readme, r"(?m)^[^\S\n]+>", ">");
    readme = readme.replacen("\n", &format!(" v{}\n", env!("CARGO_PKG_VERSION")), 1);
    println!();
    termimad::print_text(&readme);
}

fn program() -> i32 {
    let tmp = env::var("TMP").ok();
    if tmp.is_some() {
        env::set_current_dir(tmp.unwrap()).unwrap();
    }

    let mut repo = REPO.to_owned();
    let mut sources = RELEASE_SOURCES.to_owned();

    let mut help = false;
    let mut repair = false;
    let mut force = false;
    let mut nocache = false;
    let mut clearuserdata = false;
    let mut simd: Option<String> = None;

    let mut warn_args: Vec<String> = vec![];

    let re_kv = Regex::new(r"^/(?P<key>\w+)=(?P<val>.+)$").unwrap();
    let re_flag = Regex::new(r"^/(?P<flag>.+)$").unwrap();

    let mut args: Vec<String> = env::args().skip(1).collect();

    for i in (0..args.len()).rev() {
        if let Some(capture) = re_flag.captures(&args[i]) {
            let flag = &capture["flag"];
            let matched = match flag {
                "?" => {
                    help = true;
                    true
                }
                "help" => {
                    help = true;
                    true
                }
                "silent" => {
                    hide_console();
                    true
                }
                "repair" => {
                    repair = true;
                    true
                }
                "force" => {
                    if repair {
                        eprintln!("WARNING: Force install...");
                        force = true;
                    } else {
                        eprintln!("WARNING: Ignoring '/force' because '/repair' is not set...");
                    }
                    true
                }
                "nocache" => {
                    nocache = true;
                    true
                }
                "clearuserdata" => {
                    if repair && force {
                        eprintln!("WARNING: Clear user profile...");
                        clearuserdata = true;
                    } else {
                        eprintln!(
                            "WARNING: Ignoring '/clearuserdata' because '/repair' and '/force' are not set..."
                        );
                    }
                    true
                }
                _ => false,
            };
            if help {
                show_help();
                return 0;
            }
            if matched {
                args.remove(i);
                continue;
            }
        }
        if let Some(capture) = re_kv.captures(&args[i]) {
            let key = capture["key"].to_lowercase();
            let val = capture["val"].to_owned();
            let matched = match key.as_str() {
                "repo" => {
                    repo = format!("/{}", val);
                    true
                }
                "simd" => {
                    simd = Some(val);
                    true
                }
                "channel" => {
                    if val == "beta" {
                        eprintln!("WARNING: Using 'beta' channel...");
                        sources = BETA_SOURCES.to_owned();
                    } else if val != "latest" {
                        eprintln!(
                            "WARNING: Channel '{}' is unknown. Currently only knows 'beta', falling back to release...",
                            val
                        )
                    }
                    true
                }
                _ => false,
            };
            if matched {
                args.remove(i);
                continue;
            }
        } else {
            // Won't stay long (dynamic), it's okay
            warn_args.push(args[i].clone());
        }
    }

    if !warn_args.is_empty() {
        let formated_warn_args = warn_args
            .par_iter()
            .map(|a| format!("'{}'", a))
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!(
            "WARNING: {} unknown arguments passed. Passing thru installer arguments...",
            formated_warn_args
        );
    }

    let installer_info = get_package_info(&repo, &sources, simd.as_deref());
    match installer_info {
        Some(info) => {
            let v1 = Versioning::new(uninstall_reg_get_string("Version").unwrap_or_default());
            println!(
                "Currently installed: {}",
                match &v1 {
                    Some(v) => v.to_string(),
                    None => "not installed".to_owned(),
                }
            );
            let v2 = Versioning::new(&info.version);
            println!(
                "Upstream: {}",
                match &v2 {
                    Some(v) => v.to_string(),
                    None => panic!("unexpected version of upstream"),
                }
            );
            if repair || v1.unwrap_or_default() < v2.to_owned().unwrap_or_default() {
                let download_path = "./setup.exe";
                let status = aria2_downloader(&info.url, &download_path, None);
                if status != 0 {
                    return status;
                }

                if force {
                    let uninstall_string = uninstall_reg_get_string("UninstallString");
                    if uninstall_string.is_some() {
                        let mut setup_args: Vec<&str> = ["--force-uninstall"]
                            .into_iter()
                            .chain(args.par_iter().map(|s| s.as_str()).collect::<Vec<_>>())
                            .collect();
                        if clearuserdata {
                            setup_args.push("--delete-profile");
                        }
                        let status = simple_spawn(&uninstall_string.unwrap(), &setup_args, true);
                        if status != 0 && status != 19 {
                            eprintln!("WARNING: Unexpected error code ({})", status)
                        }
                    }
                }

                let setup_args: Vec<&str> = ["--silent"]
                    .into_iter()
                    .chain(args.par_iter().map(|s| s.as_str()).collect::<Vec<_>>())
                    .collect();
                println!("Installing Thorium...");
                let status = simple_spawn(&download_path, &setup_args, false);
                if nocache {
                    fs::remove_file(&download_path).ok();
                }
                if status != 0 {
                    return status;
                }

                let v1 = Versioning::new(uninstall_reg_get_string("Version").unwrap_or_default())
                    .unwrap_or_default();
                if v1 < v2.unwrap_or_default() {
                    eprintln!("ERROR: Thorium isn't updated.");
                    return 1;
                }
                println!("Thorium browser has been updated.");
            } else {
                println!("You are up to date.");
            }
            return 0;
        }
        None => {
            eprintln!("ERROR: Could not get package info.");
            return 2;
        }
    }
}

fn main() -> ! {
    process::exit(program())
}
