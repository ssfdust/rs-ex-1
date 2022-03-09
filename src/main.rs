extern crate ctrlc;
extern crate sysinfo;

mod fakeunzip;
mod userinput;

use fakeunzip::{FakeUnzip, FakeUnzipCreator, FakeUnzipInteractor};
use std::process::{exit};
use std::thread;
use std::time;
use sysinfo::{System, SystemExt};

fn main() {
    // Init
    let mut system = System::new_all();
    let mut fake_unzip_inst = FakeUnzip::default();
    fake_unzip_inst.create_fake_unzip();

    // Handle Ctrl-C
    let cloned = fake_unzip_inst.clone();
    ctrlc::set_handler(move || {
        cloned.recovery_unzip();
        exit(2);
    })
    .expect("Error setting Ctrl-C handler");

    // Start Hook Loop
    println!("waiting for hook signal. Use Ctrl-C to exit.");
    loop {
        match fake_unzip_inst.wait_for_fake_unzip_info(&mut system) {
            None => thread::sleep(time::Duration::from_secs(3)),
            Some(fake_unzip_info) => {
                fake_unzip_inst.unzip_with_fake_unzip(&fake_unzip_info);
                fake_unzip_inst.interact_with_fake_unzip(&fake_unzip_info);
            }
        }
    }
}
