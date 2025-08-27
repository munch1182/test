use std::{path::PathBuf, process::Command};

use libcommon::{
    curr_dir,
    ext::{FileDirCreateExt, PrettyStringExt},
    prelude::*,
};
use loading::{Loader, Symbol};

#[timer]
#[logsetup]
fn main() -> Result<()> {
    let dllfile = generate_dll()?;
    let loader = Loader::new(dllfile)?;

    let curr: Symbol<fn(&str) -> String> = loader.get_fn("currdir")?;
    let result = curr("path_from_call");
    info!("result: {}", result);
    Ok(())
}

fn generate_dll() -> Result<PathBuf> {
    let dir = curr_dir!("rust", "loading")?;
    let curr_path = curr_dir!(&dir, "examples", "currdir.rs")?;
    let output_path = curr_dir!(dir, "test_dll_dir", "curr.dll")?.create_parent()?;

    let mut cmd = Command::new("rustc");
    let out = cmd
        .args([
            "--crate-type",
            "cdylib",
            curr_path.as_os_str().to_str().unwrap_or_default(),
            "-o",
            output_path.as_os_str().to_str().unwrap_or_default(),
        ])
        .output()?;

    info!(
        "cmd: {:?}: {:?}",
        cmd.to_string_pretty(),
        out.status.success()
    );

    Ok(output_path)
}
