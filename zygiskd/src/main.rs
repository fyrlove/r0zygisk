mod companion;
mod constants;
mod dl;
mod root_impl;
mod utils;
mod r0zd;

use crate::constants::ZKSU_VERSION;
use std::ffi::CString;

fn mask_process_name(name: &str) {
    if let Ok(cname) = CString::new(name) {
        unsafe {
            libc::prctl(libc::PR_SET_NAME, cname.as_ptr() as usize, 0, 0, 0);
        }
    }
}

fn init_android_logger(tag: &str) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(constants::MAX_LOG_LEVEL)
            .with_tag(tag),
    );
}

fn start() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "companion" {
        mask_process_name("system_server");
        let fd: i32 = args[2].parse().unwrap();
        companion::entry(fd);
        return;
    } else if args.len() == 2 && args[1] == "service-stage" {
        mask_process_name("adbd");
        return;
    } else if args.len() == 2 && args[1] == "daemon" {
        mask_process_name(if cfg!(target_pointer_width = "64") { "netd" } else { "logd" });
        utils::switch_mount_namespace(1).expect("switch mnt ns");
        root_impl::setup();
        log::info!("current root impl: {:?}", root_impl::get_impl());
        r0zd::main().expect("r0zd main");
        return;
    } else if args.len() == 2 && args[1] == "version" {
        mask_process_name("statsd");
        println!("r0zd {}", ZKSU_VERSION);
        return;
    } else if args.len() == 2 && args[1] == "root" {
        mask_process_name("adbd");
        root_impl::setup();
        println!("root impl: {:?}", root_impl::get_impl());
        return;
    }

    mask_process_name(if cfg!(target_pointer_width = "64") { "netd" } else { "logd" });
    utils::switch_mount_namespace(1).expect("switch mnt ns");
    root_impl::setup();
    log::info!("current root impl: {:?}", root_impl::get_impl());
    r0zd::main().expect("r0zd main");
}

fn main() {
    let process = std::env::args().next().unwrap();
    let nice_name = process.split('/').last().unwrap();
    init_android_logger(nice_name);

    start();
}
