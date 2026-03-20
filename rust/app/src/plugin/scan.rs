use libcommon::prelude::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use walkdir::WalkDir;

/// 读取文件夹中深度为 1 和 2 的 JSON 文件，并解析为 Config 列表
/// - 深度 0：根目录本身（忽略，因为不是文件）
/// - 深度 1：根目录下的直接文件或文件夹
/// - 深度 2：根目录子文件夹内的文件
pub fn scan(dir: impl AsRef<Path>) -> Result<Vec<Config>> {
    let mut configs = Vec::new();

    // 使用 walkdir，最大深度 2（只遍历到第二层子目录）
    for entry in WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        // 只处理文件，且扩展名为 .json
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            let mut config: Config = serde_json::from_reader(fs::File::open(path)?)?;
            config.update_dir(path);
            configs.push(config);
        }
    }

    Ok(configs)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<Files>,
}

impl Config {
    pub fn update_dir(&mut self, dir: &Path) {
        if let Some(dir) = dir.parent()
            && let Some(dir) = dir.to_str()
        {
            self.dir = Some(dir.to_string());
        }
    }
}

impl Config {
    pub fn files(&self) -> Option<&File> {
        #[cfg(debug_assertions)]
        {
            self.files.as_ref()?.dev.as_ref()
        }
        #[cfg(not(debug_assertions))]
        {
            self.files.as_ref()?.release.as_ref()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Files {
    #[serde(skip_serializing_if = "Option::is_none")]
    dev: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    release: Option<File>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lib: Option<String>,
}
