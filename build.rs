use std::{fs, path::Path, process::Command};

use walkdir::WalkDir;

fn main() {
    Command::new("npm")
        .arg("install")
        .status()
        .expect("Failed to run `npm install`")
        .success()
        .then_some(())
        .unwrap();

    include_file("node_modules/easymde/dist/easymde.min.js");
    include_file("node_modules/easymde/dist/easymde.min.css");
    include_file("node_modules/sortablejs/Sortable.min.js");

    for e in WalkDir::new("i18n") {
        println!(
            "cargo:rerun-if-changed={}",
            e.unwrap().path().to_owned().to_str().unwrap()
        );
    }
}

fn include_file(path: impl AsRef<Path>) {
    let dest = Path::new("static").join(&path);
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).expect("Failed to create dir");
    }
    fs::copy(path.as_ref(), dest).expect("Failed to copy file");
    println!("cargo:rerun-if-changed={}", path.as_ref().to_str().unwrap());
}
