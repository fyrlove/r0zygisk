use crate::utils::LateInit;
use std::collections::BTreeSet;
use std::fs;
use std::process::{Command, Stdio};

static HIDE_LIST_PATH: LateInit<String> = LateInit::new();

pub fn init(base_dir: &str) {
    if !HIDE_LIST_PATH.initiated() {
        HIDE_LIST_PATH.init(format!("{}/hide_list.conf", base_dir));
    }
}

fn read_hidden_packages() -> BTreeSet<String> {
    fs::read_to_string(&*HIDE_LIST_PATH)
        .ok()
        .map(|content| {
            content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| line.to_string())
                .collect()
        })
        .unwrap_or_default()
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
                .filter_map(|line| {
                    let pkg = line.strip_prefix("package:")?;
                    Some(pkg.split(' ').next().unwrap_or(pkg).trim().to_string())
                })
                .filter(|pkg| !pkg.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn uid_should_hide(uid: i32) -> bool {
    let hidden = read_hidden_packages();
    if hidden.is_empty() {
        return false;
    }
    packages_for_uid(uid)
        .into_iter()
        .any(|pkg| hidden.contains(pkg.as_str()))
}
