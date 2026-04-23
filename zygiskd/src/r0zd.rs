use crate::constants::{DaemonSocketAction, ProcessFlags};
use crate::utils::{check_unix_socket, LateInit, UnixStreamExt};
use crate::{constants, lp_select, root_impl, utils};
use anyhow::{bail, Result};
use log::{debug, error, info, trace, warn};
use passfd::FdPassingExt;
use rustix::fs::{fcntl_setfd, FdFlags};
use std::fs;
use std::fmt::Write as _;
use std::io::Error;
use std::ops::Deref;
use std::os::fd::{AsFd, OwnedFd, RawFd};
use std::os::unix::process::CommandExt;
use std::os::unix::{
    net::{UnixListener, UnixStream},
    prelude::AsRawFd,
};
use std::path::PathBuf;
use std::process::{exit, Command};
use std::sync::{Arc, Mutex};
use std::thread;

struct Module {
    name: String,
    lib_fd: OwnedFd,
    companion: Mutex<Option<Option<UnixStream>>>,
}

struct Context {
    modules: Vec<Module>,
}

static TMP_PATH: LateInit<String> = LateInit::new();
static PATH_CP_NAME: LateInit<String> = LateInit::new();
static STATUS_PATH: LateInit<String> = LateInit::new();

pub fn main() -> Result<()> {
    info!("Welcome to r0z ({}) !", constants::ZKSU_VERSION);

    TMP_PATH.init(std::env::var("TMP_PATH")?);
    PATH_CP_NAME.init(format!(
        "{}/{}",
        TMP_PATH.deref(),
        lp_select!("/cp32.sock", "/cp64.sock")
    ));
    STATUS_PATH.init(format!("{}/status.json", TMP_PATH.deref()));

    let arch = get_arch()?;
    debug!("Daemon architecture: {arch}");
    let modules = load_modules(arch)?;
    let daemon_info = match root_impl::get_impl() {
        root_impl::RootImpl::KernelSU | root_impl::RootImpl::Magisk => {
            format!("Root: {:?},module_count: {}", root_impl::get_impl(), modules.len())
        }
        _ => format!("Invalid root implementation: {:?}", root_impl::get_impl()),
    };
    update_status_json(false, &daemon_info)?;

    let context = Context { modules };
    let context = Arc::new(context);
    let listener = create_daemon_socket()?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let context = Arc::clone(&context);
        let action = stream.read_u8()?;
        let action = DaemonSocketAction::try_from(action)?;
        trace!("New daemon action {:?}", action);
        match action {
            DaemonSocketAction::PingHeartbeat => {
                update_status_json(true, &daemon_info)?;
            }
            DaemonSocketAction::ZygoteRestart => {
                info!("Zygote restarted, clean up companions");
                for module in &context.modules {
                    let mut companion = module.companion.lock().unwrap();
                    companion.take();
                }
                update_status_json(false, &daemon_info)?;
            }
            DaemonSocketAction::SystemServerStarted => {
                update_status_json(true, &daemon_info)?;
            }
            _ => {
                thread::spawn(move || {
                    if let Err(e) = handle_daemon_action(action, stream, &context) {
                        warn!("Error handling daemon action: {}\n{}", e, e.backtrace());
                    }
                });
            }
        }
    }

    Ok(())
}

fn update_status_json(zygote_injected: bool, daemon_info: &str) -> Result<()> {
    let mut raw = String::new();
    let arch_suffix = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
    let _ = write!(
        raw,
        "daemon{}:running,Root: {:?},module_count: {}",
        arch_suffix,
        root_impl::get_impl(),
        daemon_info
            .rsplit("module_count: ")
            .next()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0)
    );
    let zygote_state = if zygote_injected { "injected" } else { "not_injected" };
    let json = if cfg!(target_pointer_width = "64") {
        format!(
            "{{\n  \"monitor\": \"native_bridge\",\n  \"mode\": \"native_bridge\",\n  \"stop_reason\": \"\",\n  \"zygote64\": \"{}\",\n  \"daemon64\": \"running\",\n  \"daemon64_info\": {:?},\n  \"zygote32\": \"unsupported\",\n  \"daemon32\": \"unsupported\",\n  \"daemon32_info\": \"\",\n  \"raw\": {:?}\n}}\n",
            zygote_state, daemon_info, raw
        )
    } else {
        format!(
            "{{\n  \"monitor\": \"native_bridge\",\n  \"mode\": \"native_bridge\",\n  \"stop_reason\": \"\",\n  \"zygote64\": \"unsupported\",\n  \"daemon64\": \"unsupported\",\n  \"daemon64_info\": \"\",\n  \"zygote32\": \"{}\",\n  \"daemon32\": \"running\",\n  \"daemon32_info\": {:?},\n  \"raw\": {:?}\n}}\n",
            zygote_state, daemon_info, raw
        )
    };
    fs::write(STATUS_PATH.deref(), json)?;
    Ok(())
}

fn get_arch() -> Result<&'static str> {
    let system_arch = utils::get_property("ro.product.cpu.abi")?;
    if system_arch.contains("arm") {
        return Ok(lp_select!("armeabi-v7a", "arm64-v8a"));
    }
    if system_arch.contains("x86") {
        return Ok(lp_select!("x86", "x86_64"));
    }
    bail!("Unsupported system architecture: {}", system_arch);
}

fn load_modules(arch: &str) -> Result<Vec<Module>> {
    let mut modules = Vec::new();
    let dir = match fs::read_dir(constants::PATH_MODULES_DIR) {
        Ok(dir) => dir,
        Err(e) => {
            warn!("Failed reading modules directory: {}", e);
            return Ok(modules);
        }
    };
    for entry in dir.into_iter() {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap();
        let so_path = entry.path().join(format!("zygisk/{arch}.so"));
        let disabled = entry.path().join("disable");
        if !so_path.exists() || disabled.exists() {
            continue;
        }
        info!("  Loading module `{name}`...");
        let lib_fd = match create_library_fd(&so_path) {
            Ok(fd) => fd,
            Err(e) => {
                warn!("  Failed to create memfd for `{name}`: {e}");
                continue;
            }
        };
        let companion = Mutex::new(None);
        let module = Module {
            name,
            lib_fd,
            companion,
        };
        modules.push(module);
    }

    Ok(modules)
}

fn create_library_fd(so_path: &PathBuf) -> Result<OwnedFd> {
    let opts = memfd::MemfdOptions::default().allow_sealing(true);
    let memfd = opts.create("jit-cache")?;
    let file = fs::File::open(so_path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut writer = memfd.as_file();
    std::io::copy(&mut reader, &mut writer)?;

    let mut seals = memfd::SealsHashSet::new();
    seals.insert(memfd::FileSeal::SealShrink);
    seals.insert(memfd::FileSeal::SealGrow);
    seals.insert(memfd::FileSeal::SealWrite);
    seals.insert(memfd::FileSeal::SealSeal);
    memfd.add_seals(&seals)?;

    Ok(OwnedFd::from(memfd.into_file()))
}

fn create_daemon_socket() -> Result<UnixListener> {
    utils::set_socket_create_context("u:r:zygote:s0")?;
    let listener = utils::unix_listener_from_path(&PATH_CP_NAME)?;
    Ok(listener)
}

fn spawn_companion(name: &str, lib_fd: RawFd) -> Result<Option<UnixStream>> {
    let (mut daemon, companion) = UnixStream::pair()?;

    // FIXME: avoid getting self path from arg0
    let process = std::env::args().next().unwrap();

    unsafe {
        let pid = libc::fork();
        if pid < 0 {
            bail!(Error::last_os_error());
        } else if pid > 0 {
            drop(companion);
            let mut status: libc::c_int = 0;
            libc::waitpid(pid, &mut status, 0);
            if libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0 {
                daemon.write_string(name)?;
                daemon.send_fd(lib_fd)?;
                return match daemon.read_u8()? {
                    0 => Ok(None),
                    1 => Ok(Some(daemon)),
                    _ => bail!("Invalid companion response"),
                };
            } else {
                bail!("exited with status {}", status);
            }
        } else {
            // Remove FD_CLOEXEC flag
            fcntl_setfd(companion.as_fd(), FdFlags::empty())?;
        }
    }

    Command::new(&process)
        .arg0("system_server")
        .arg("companion")
        .arg(format!("{}", companion.as_raw_fd()))
        .spawn()?;
    exit(0)
}

fn handle_daemon_action(
    action: DaemonSocketAction,
    mut stream: UnixStream,
    context: &Context,
) -> Result<()> {
    match action {
        DaemonSocketAction::RequestLogcatFd => loop {
            let level = match stream.read_u8() {
                Ok(level) => level,
                Err(_) => break,
            };
            let tag = stream.read_string()?;
            let message = stream.read_string()?;
            utils::log_raw(level as i32, &tag, &message)?;
        },
        DaemonSocketAction::GetProcessFlags => {
            let uid = stream.read_u32()? as i32;
            let mut flags = ProcessFlags::empty();
            if root_impl::uid_is_manager(uid) {
                flags |= ProcessFlags::PROCESS_IS_MANAGER;
            } else {
                if root_impl::uid_granted_root(uid) {
                    flags |= ProcessFlags::PROCESS_GRANTED_ROOT;
                }
                if root_impl::uid_should_umount(uid) {
                    flags |= ProcessFlags::PROCESS_ON_DENYLIST;
                }
            }
            match root_impl::get_impl() {
                root_impl::RootImpl::KernelSU => flags |= ProcessFlags::PROCESS_ROOT_IS_KSU,
                root_impl::RootImpl::Magisk => flags |= ProcessFlags::PROCESS_ROOT_IS_MAGISK,
                _ => panic!("wrong root impl: {:?}", root_impl::get_impl()),
            }
            trace!(
                "Uid {} granted root: {}",
                uid,
                flags.contains(ProcessFlags::PROCESS_GRANTED_ROOT)
            );
            trace!(
                "Uid {} on denylist: {}",
                uid,
                flags.contains(ProcessFlags::PROCESS_ON_DENYLIST)
            );
            stream.write_u32(flags.bits())?;
        }
        DaemonSocketAction::ReadModules => {
            stream.write_usize(context.modules.len())?;
            for module in context.modules.iter() {
                stream.write_string(&module.name)?;
                stream.send_fd(module.lib_fd.as_raw_fd())?;
            }
        }
        DaemonSocketAction::RequestCompanionSocket => {
            let index = stream.read_usize()?;
            let module = &context.modules[index];
            let mut companion = module.companion.lock().unwrap();
            if let Some(Some(sock)) = companion.as_ref() {
                if !check_unix_socket(sock, false) {
                    error!("Poll companion for module `{}` crashed", module.name);
                    companion.take();
                }
            }
            if companion.is_none() {
                match spawn_companion(&module.name, module.lib_fd.as_raw_fd()) {
                    Ok(c) => {
                        if c.is_some() {
                            trace!("  Spawned companion for `{}`", module.name);
                        } else {
                            trace!(
                                "  No companion spawned for `{}` because it has not entry",
                                module.name
                            );
                        }
                        *companion = Some(c);
                    }
                    Err(e) => {
                        warn!("  Failed to spawn companion for `{}`: {}", module.name, e);
                    }
                };
            }
            match companion.as_ref() {
                Some(Some(sock)) => {
                    if let Err(e) = sock.send_fd(stream.as_raw_fd()) {
                        error!(
                            "Failed to send companion fd socket of module `{}`: {}",
                            module.name, e
                        );
                        stream.write_u8(0)?;
                    }
                    // Ok: Send by companion
                }
                _ => {
                    stream.write_u8(0)?;
                }
            }
        }
        DaemonSocketAction::GetModuleDir => {
            let index = stream.read_usize()?;
            let module = &context.modules[index];
            let dir = format!("{}/{}", constants::PATH_MODULES_DIR, module.name);
            let dir = fs::File::open(dir)?;
            stream.send_fd(dir.as_raw_fd())?;
        }
        _ => {}
    }
    Ok(())
}
