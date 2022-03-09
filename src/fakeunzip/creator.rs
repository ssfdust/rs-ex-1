extern crate which;
use super::FakeUnzip;
use std::fs::{self, rename};
use std::io::prelude::*;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use which::which;

const FAKE_ZIP_SCRIPT: &str = r#"
#!/bin/bash
args=$(echo $@ | tr -s /)
echo '{"args":"'$args'","pid":'$$',"pwd":"'$PWD'"}' >/tmp/fake_unzip_info.json
while [ ! -f "/tmp/unzip_stop_$$" ]; do sleep 1;done
rm -f /tmp/fake_unzip_info.json"#;

const UNZIP_COMMAND: &str = "unzip";

pub trait FakeUnzipCreator {
    fn get_system_unzip(&self) -> which::Result<PathBuf>;
    fn system_unzip(&self) -> PathBuf;
    fn backuped_system_unzip(&self) -> PathBuf;
    fn create_fake_unzip(&mut self);
    fn recovery_unzip(&self);
}


impl FakeUnzipCreator for FakeUnzip {
    fn get_system_unzip(&self) -> which::Result<PathBuf> {
        which(UNZIP_COMMAND)
    }

    fn system_unzip(&self) -> PathBuf {
        self.system_unzip.clone()
    }

    fn backuped_system_unzip(&self) -> PathBuf {
        self.backuped_system_unzip.clone()
    }

    fn create_fake_unzip(&mut self) {
        println!("{}", "start create upgrade hook.");
        self.get_system_unzip()
            .map_err(|_| eprintln!("No unzip file found."))
            .and_then(|sys_unzip_path| {
                self.system_unzip = sys_unzip_path.clone();
                let backup_sys_unzip_path = sys_unzip_path.with_extension("backup");
                self.backuped_system_unzip = backup_sys_unzip_path.clone();
                Ok(rename(sys_unzip_path, backup_sys_unzip_path)
                    .and_then(|_| fs::File::create(&self.system_unzip()))
                    .and_then(|mut fake_unzip_handler| {
                        let scripts = String::from(FAKE_ZIP_SCRIPT);
                        fake_unzip_handler.write_all(scripts.as_bytes())
                    })
                    .and_then(|_| {
                        fs::set_permissions(&self.system_unzip(), fs::Permissions::from_mode(0o755))
                    })
                    .unwrap_or_else(|_| eprintln!("Failed to create fake unzip file.")))
            })
            .unwrap_or_else(|_| std::process::exit(1));
    }

    fn recovery_unzip(&self) {
        if self.system_unzip().exists() && self.backuped_system_unzip().exists() {
            println!("Clean up upgrade hook");
            fs::remove_file(self.system_unzip())
                .and_then(|_| rename(self.backuped_system_unzip(), self.system_unzip()))
                .unwrap_or_else(|_| {
                    eprintln!("Failed to clean upgrade hook. Please check whether unzip is usable.")
                });
        }
    }
}
