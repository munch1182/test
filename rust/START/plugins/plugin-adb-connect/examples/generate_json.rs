use libcommon::ext::FileDirCreateExt;
use plugin::PluginInterface;
use plugin_adb_connect::PluginAdbConnect;
use std::{fs, path::Path};

fn main() {
    let meta = PluginAdbConnect.get_metadata();

    let file = Path::new("plugins/dist/plugin_adb_connect/plugin.json");
    file.create_parent().unwrap();
    println!("{:?}: {}", &file, file.exists());

    serde_json::to_writer_pretty(fs::File::create(file).unwrap(), &meta).unwrap();
}
