mod creator;
mod interactor;
use std::path::PathBuf;
pub use creator::FakeUnzipCreator;
pub use interactor::{get_current_fake_unzip_info, FakeUnzipInteractor, FakeUnzipInfo};

#[derive(Default, Clone)]
pub struct FakeUnzip {
    system_unzip: PathBuf,
    backuped_system_unzip: PathBuf,
}
