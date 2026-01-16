#[cfg(target_os = "windows")]
fn main() {
    use libcommon::curr_dir;
    use libcommon::ext::{Command, CommandInExt};
    use std::time::Instant;
    {
        let file = curr_dir!("plugins", "plugin-adb-connect").unwrap();

        let target_dir = file.join("target\\release");
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir).unwrap();
        }

        let start = Instant::now();
        let str = format!("cmd /c cd {} | cargo build --release", file.display());
        let cmd = Command::from_str(&str).output().unwrap();
        let cost = start.elapsed().as_millis();
        println!("execute: {str} => {:?} ({cost}s)", cmd.out_or_err());
    }

    {
        let target_dir = curr_dir!("target", "release").unwrap();
        let name = format!("plugin_adb_connect.{}", get_library_extension());
        let dll = target_dir.join(&name);

        if dll.exists() {
            use std::fs;

            let dist_dir = curr_dir!("plugins", "dist", "plugin_adb_connect")
                .unwrap()
                .join(name);
            fs::copy(dll, dist_dir).unwrap();
        }
    }
}

fn get_library_extension() -> &'static str {
    #[cfg(target_os = "linux")]
    return "so";
    #[cfg(target_os = "macos")]
    return "dylib";
    #[cfg(target_os = "windows")]
    return "dll";
}
