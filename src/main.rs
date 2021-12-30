extern crate structopt;
extern crate sysinfo;
extern crate which;
extern crate libc;
extern crate ctrlc;
use simple_user_input::get_input;
use std::fs;
use std::fs::rename;
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::thread;
use std::time;
use structopt::StructOpt;
use sysinfo::{Process, System, SystemExt};
use which::which;
use libc::c_int;

#[derive(StructOpt)]
struct UnzipArgs {
    #[structopt(parse(from_os_str))]
    _zipfile: PathBuf,
    #[structopt(short = "d", parse(from_os_str))]
    dirpath: PathBuf,
    #[structopt(short = "o")]
    _overwrite: bool
}

fn get_user_confirm(decompressed_dir: Option<PathBuf>) -> bool {
    match decompressed_dir {
        Some(upgrade_sh) => {
            println!("Upgrade.sh found at {}", upgrade_sh.to_string_lossy());
            let input: String = get_input("continue to next with yes: ");
            if input.contains("n") || input.contains("N") {
                return true;
            } else {
                return false;
            }
        }
        _ => return false,
    }
}

extern "C" fn clean_up() {
    let unzip_path = get_unzip_path();
    remove_fake_unzip(&unzip_path);
    recovery_unzip(&unzip_path);
}

#[link(name="c")]
extern {
    fn atexit(cb: extern "C" fn()) -> c_int;
}

fn main() {
    let unzip_path = get_unzip_path();
    let unzip_pid_path = PathBuf::from("/tmp/unzip_pid");
    let unzip_args_path = PathBuf::from("/tmp/unzip_args");
    let fake_unzip_sig_f = PathBuf::from("/tmp/unzip_stop");
    let mut system = System::new_all();
    let renamed_unzip = rename_unzip(&unzip_path).expect("the unzip not existed");
    create_fake_unzip(&unzip_path).unwrap();
    ctrlc::set_handler(move || {
        remove_fake_unzip(&unzip_path);
        recovery_unzip(&unzip_path);
        exit(2);
    }).expect("Error setting Ctrl-C handler");
    unsafe {
        atexit(clean_up);
    }
    loop {
        match wait_for_unzip(&mut system, &unzip_pid_path) {
            Some(_) => {
                let mut decompressed_dir: Option<PathBuf> = None;
                if let Ok(unzip_args) = fs::read_to_string(&unzip_args_path) {
                    let unzip_args = unzip_args.trim().to_owned();
                    let unzip_cmd = unzip_with_args(&renamed_unzip, &unzip_args);
                    decompressed_dir = get_decompress_dir_from_args(&unzip_cmd);
                }
                if decompressed_dir != None && get_user_confirm(decompressed_dir) {
                    interact_with_fake_unzip(&fake_unzip_sig_f);
                    break;
                } else {
                    interact_with_fake_unzip(&fake_unzip_sig_f);
                }
            }
            None => thread::sleep(time::Duration::from_secs(1)),
        }
    }
}

fn clean_upgrade_sh(upgrade_sh: &PathBuf) -> PathBuf {
    let upgrade_sh_str = upgrade_sh.clone().into_os_string().into_string().unwrap();
    PathBuf::from(upgrade_sh_str.replace("//", "/"))
}

fn get_decompress_dir_from_args(args: &String) -> Option<PathBuf> {
    let unzip_args = UnzipArgs::from_iter(args.split(" "));
    let upgrade_sh = clean_upgrade_sh(&unzip_args.dirpath.join("Upgrade.sh"));
    if upgrade_sh.exists() {
        return Some(upgrade_sh);
    }
    None
}

fn interact_with_fake_unzip(fake_unzip_sig_f: &PathBuf) {
    get_input("Press enter key to continue.");
    fs::File::create(fake_unzip_sig_f).expect("failed to stop fake unzip");
    thread::sleep(time::Duration::from_secs(3));
    fs::remove_file(fake_unzip_sig_f).expect("faile to delete fake stop file");
}

fn unzip_with_args(unzip_path: &PathBuf, args: &String) -> String {
    let mut unzip_cmd = unzip_path.clone().into_os_string().into_string().unwrap();
    unzip_cmd.push_str(" ");
    unzip_cmd.push_str(args);
    println!("execute {}", unzip_cmd);
    Command::new("sh")
        .arg("-c")
        .arg(&unzip_cmd)
        .output()
        .expect("failed to execute process");
    unzip_cmd
}

fn wait_for_unzip<'a>(system: &'a mut System, unzip_pid_path: &PathBuf) -> Option<&'a Process> {
    system.refresh_all();
    println!("waiting for unzip command");
    if let Ok(pid_str) = fs::read_to_string(unzip_pid_path) {
        let pid_str = pid_str.lines().collect::<Vec<&str>>()[0];
        if let Ok(pid) = pid_str.parse::<i32>() {
            return system.process(pid);
        }
    }
    None
}

fn remove_fake_unzip(src_unzip_path: &PathBuf) -> () {
    if src_unzip_path.exists() && src_unzip_path.with_extension("backup").exists() {
        fs::remove_file(src_unzip_path).unwrap();
    }
}

fn create_fake_unzip(src_unzip_path: &PathBuf) -> std::io::Result<()> {
    println!("create fake unzip file");
    let scripts = String::from(
        "#!/bin/bash\necho $@ > /tmp/unzip_args\necho $$ >/tmp/unzip_pid\nwhile [ ! -f /tmp/unzip_stop ]; do sleep 1;done",
    );
    if !src_unzip_path.exists() {
        let mut fake_unzip = fs::File::create(src_unzip_path).unwrap();
        fake_unzip.write_all(scripts.as_bytes()).unwrap();
        fs::set_permissions(src_unzip_path, fs::Permissions::from_mode(0o755))?;
        ()
    }
    Ok(())
}

fn recovery_unzip(src_unzip_path: &PathBuf) -> bool {
    println!("recover unzip file");
    if src_unzip_path.with_extension("backup").exists() {
        rename(src_unzip_path.with_extension("backup"), src_unzip_path).unwrap();
        true
    } else {
        false
    }
}

fn get_unzip_path() -> PathBuf {
    which("unzip").unwrap()
}

fn rename_unzip(src_unzip_path: &PathBuf) -> Option<PathBuf> {
    if src_unzip_path.exists() {
        let src_backup_unzip_path = src_unzip_path.with_extension("backup");
        rename(src_unzip_path, src_backup_unzip_path).unwrap();
        return Some(src_unzip_path.with_extension("backup"));
    } else {
        None
    }
}

mod simple_user_input {
    use std::io;
    pub fn get_input(prompt: &str) -> String {
        println!("{}", prompt);
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {}
            Err(_no_updates_is_fine) => {}
        }
        input.trim().to_string()
    }
}
