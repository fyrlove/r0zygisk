use std::fs;
use std::os::android::fs::MetadataExt;
use std::path::Path;
use std::process::{Command, Stdio};

const APATCH_WORK_DIR: &str = "/data/adb/ap";
const APATCH_DAEMON_PATH: &str = "/data/adb/apd";
const APATCH_PACKAGE_CONFIG: &str = "/data/adb/ap/package_config";
const APATCH_MANAGER_PACKAGE: &str = "me.bmax.apatch";

pub enum Version {
    Supported,
}

#[derive(Debug, Clone)]
struct PackageConfig {
    pkg: String,
    exclude: i32,
    allow: i32,
    uid: i32,
}

pub fn get_apatch() -> Option<Version> {
    if Path::new(APATCH_DAEMON_PATH).exists() || Path::new(APATCH_WORK_DIR).exists() {
        Some(Version::Supported)
    } else {
        None
    }
}

pub fn uid_granted_root(uid: i32) -> bool {
    let packages = packages_for_uid(uid);
    if packages.is_empty() {
        return false;
    }
    read_package_configs().into_iter().any(|config| {
        packages.iter().any(|pkg| pkg == &config.pkg)
            && config.allow != 0
            && same_app_id(config.uid, uid)
    })
}

pub fn uid_should_umount(uid: i32) -> bool {
    let packages = packages_for_uid(uid);
    if packages.is_empty() {
        return false;
    }
    read_package_configs().into_iter().any(|config| {
        packages.iter().any(|pkg| pkg == &config.pkg)
            && config.exclude != 0
            && same_app_id(config.uid, uid)
    })
}

pub fn uid_is_manager(uid: i32) -> bool {
    [
        format!("/data/user_de/0/{APATCH_MANAGER_PACKAGE}"),
        format!("/data/user/0/{APATCH_MANAGER_PACKAGE}"),
    ]
    .into_iter()
    .any(|path| {
        fs::metadata(path)
            .map(|meta| meta.st_uid() == uid as u32)
            .unwrap_or(false)
    })
}

fn packages_for_uid(uid: i32) -> Vec<String> {
    Command::new("pm")
        .args(["list", "packages", "--uid", &uid.to_string()])
        .stdout(Stdio::piped())
        .spawn()
        .ok()
        .and_then(|child| child.wait_with_output().ok())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| {
            output
                .lines()
                .filter_map(|line| line.strip_prefix("package:"))
                .filter_map(|line| line.split_whitespace().next())
                .map(|pkg| pkg.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn read_package_configs() -> Vec<PackageConfig> {
    fs::read_to_string(APATCH_PACKAGE_CONFIG)
        .ok()
        .map(|raw| raw.lines().filter_map(parse_package_config).collect())
        .unwrap_or_default()
}

fn parse_package_config(line: &str) -> Option<PackageConfig> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let mut parts = line.split(',');
    let pkg = parts.next()?.trim().to_string();
    let exclude = parts.next()?.trim().parse().ok()?;
    let allow = parts.next()?.trim().parse().ok()?;
    let uid = parts.next()?.trim().parse().ok()?;
    let _to_uid = parts.next()?.trim().parse::<i32>().ok()?;
    let _sctx = parts.next()?.trim();
    Some(PackageConfig {
        pkg,
        exclude,
        allow,
        uid,
    })
}

fn same_app_id(lhs: i32, rhs: i32) -> bool {
    lhs.rem_euclid(100_000) == rhs.rem_euclid(100_000)
}
