use std::{error::Error, fs::read_dir};

fn main() -> Result<(), Box<dyn Error>> {
    for file in read_dir("lds")?.filter_map(Result::ok) {
        if file.file_type()?.is_file() {
            println!(
                "cargo:rerun-if-changed={}",
                file.file_name().to_str().unwrap()
            );
        }
    }

    Ok(())
}
