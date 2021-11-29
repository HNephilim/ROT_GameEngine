use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> Result<()> {
    println!("cargo: rerun-if_changed=model/*");

    let out_dir = std::env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("model/");
    copy_items(&paths_to_copy, out_dir, &copy_options);

    Ok(())
}
