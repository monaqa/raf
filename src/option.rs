//! Configurations for raf

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// toml configuration for raf
pub struct RafConfig {
    /// path 関連の設定
    pub path: RafConfigPath,
}

#[derive(Debug, Deserialize)]
/// path 関連の設定。
pub struct RafConfigPath {
    /// raf のルートディレクトリ。
    pub root: PathBuf,
    /// template 置き場。
    pub template: PathBuf,
}
