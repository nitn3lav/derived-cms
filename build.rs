use std::{env, path::PathBuf};

use walkdir::WalkDir;

fn main() {
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    for e in WalkDir::new(dir.join("i18n")) {
        println!(
            "cargo:rerun-if-changed={}",
            e.unwrap()
                .path()
                .to_owned()
                .strip_prefix(&dir)
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}
