extern crate sysinfo;

use super::FakeUnzip;
use super::super::userinput::get_input;
use serde::Deserialize;
use std::{fs, io, thread, time, process::Command, env};
// use structopt::StructOpt;
use sysinfo::{System, SystemExt};

/*
#[derive(StructOpt)]
struct UnzipArgs {
    #[structopt(parse(from_os_str))]
    _zipfile: PathBuf,
    #[structopt(short = "d", parse(from_os_str))]
    dirpath: PathBuf,
    #[structopt(short = "o")]
    _overwrite: bool,
}
*/

const FAKE_UNZIP_INFO: &str = "/tmp/fake_unzip_info.json";

pub trait FakeUnzipInteractor {
    fn wait_for_fake_unzip_info(&self, system: &mut System) -> Option<FakeUnzipInfo>;
    fn unzip_with_fake_unzip(&self, fake_unzip_info: &FakeUnzipInfo);
    fn interact_with_fake_unzip(&self, fake_unzip_info: &FakeUnzipInfo);
}

#[derive(Debug, Deserialize)]
pub struct FakeUnzipInfo {
    pid: i32,
    pwd: String,
    args: String,
}

impl FakeUnzipInfo {
    fn pid(&self) -> i32 {
        self.pid
    }
    fn pwd(&self) -> String {
        self.pwd.clone()
    }
    fn args(&self) -> String {
        self.args.clone()
    }
    fn get_stop_sig_file(&self) -> String {
        format!("/tmp/unzip_stop_{}", self.pid())
    }
}

pub fn get_current_fake_unzip_info() -> io::Result<FakeUnzipInfo> {
    fs::read_to_string(FAKE_UNZIP_INFO)
        .and_then(|contents| Ok(serde_json::from_str(&contents).unwrap()))
}

impl FakeUnzipInteractor for FakeUnzip {
    fn wait_for_fake_unzip_info(&self, system: &mut System) -> Option<FakeUnzipInfo> {
        system.refresh_all();
        get_current_fake_unzip_info()
            .and_then(|fake_unzip_info| {
                system
                    .process(fake_unzip_info.pid())
                    .and_then(|_| Some(Ok(Some(fake_unzip_info))))
                    .unwrap_or(Ok(None))
            })
            .unwrap_or(None)
    }

    fn unzip_with_fake_unzip(&self, fake_unzip_info: &FakeUnzipInfo) {
        self.backuped_system_unzip
            .clone()
            .into_os_string()
            .into_string()
            .and_then(|mut unzip_cmd| {
                env::set_current_dir(&fake_unzip_info.pwd()).and_then(|_| {
                    unzip_cmd.push_str(" ");
                    unzip_cmd.push_str(&fake_unzip_info.args());
                    println!("unzip {}", fake_unzip_info.args());
                    Command::new("sh")
                        .arg("-c")
                        .arg(&unzip_cmd)
                        .output()
                }).unwrap();
                Ok(())
            })
            .unwrap();
    }

    fn interact_with_fake_unzip(&self, fake_unzip_info: &FakeUnzipInfo) {
        get_input("Press enter key to continue.");
        let fake_unzip_sig_f = fake_unzip_info.get_stop_sig_file();
        fs::File::create(&fake_unzip_sig_f).and_then(|_| {
            thread::sleep(time::Duration::from_secs(3));
            fs::remove_file(&fake_unzip_sig_f)
        }).unwrap();
    }
}

