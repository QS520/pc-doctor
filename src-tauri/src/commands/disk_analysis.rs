use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Serialize, Clone, Debug)]
pub struct LargeFile {
    pub path: String,
    pub name: String,
    pub size_mb: f64,
    pub modified: String,
    pub extension: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DirectorySize {
    pub path: String,
    pub name: String,
    pub size_mb: f64,
    pub file_count: u64,
    pub subdirectory_count: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct ExtensionStat {
    pub extension: String,
    pub file_count: u64,
    pub total_size_mb: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct ScanResult {
    pub large_files: Vec<LargeFile>,
    pub directory_sizes: Vec<DirectorySize>,
    pub extension_stats: Vec<ExtensionStat>,
    pub total_scanned: u64,
    pub scan_path: String,
}

/// 扫描指定目录下的大文件 (>100MB)
#[tauri::command]
pub fn scan_large_files(path: Option<String>, min_size_mb: Option<f64>) -> ScanResult {
    let scan_path = path.unwrap_or_else(|| "C:\\".to_string());
    let min_size = min_size_mb.unwrap_or(100.0);
    let min_size_bytes = (min_size * 1024.0 * 1024.0) as u64;

    let mut large_files = Vec::new();
    let mut extension_map: std::collections::HashMap<String, (u64, u64)> = std::collections::HashMap::new();
    let mut total_scanned: u64 = 0;

    for entry in WalkDir::new(&scan_path)
        .max_depth(8)
        .into_iter()
        .filter_entry(|e| {
            // 跳过一些系统目录避免卡顿
            let name = e.file_name().to_string_lossy().to_lowercase();
            !matches!(
                name.as_str(),
                "$recycle.bin" | "system volume information" | "$windows.~bs" | "$windows.~ws"
            )
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total_scanned += 1;

            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();

                // 统计扩展名
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("无扩展名")
                    .to_lowercase();
                let ext_stat = extension_map.entry(ext.clone()).or_insert((0, 0));
                ext_stat.0 += 1;
                ext_stat.1 += size;

                // 如果是大文件，加入列表
                if size >= min_size_bytes {
                    let modified = metadata
                        .modified()
                        .ok()
                        .and_then(|t| {
                            t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .ok()
                        })
                        .map(|d| {
                            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();

                    large_files.push(LargeFile {
                        path: entry.path().to_string_lossy().to_string(),
                        name: entry
                            .file_name()
                            .to_string_lossy()
                            .to_string(),
                        size_mb: (size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
                        modified,
                        extension: ext,
                    });
                }
            }
        }
    }

    // 按大小排序
    large_files.sort_by(|a, b| {
        b.size_mb
            .partial_cmp(&a.size_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 限制返回数量
    large_files.truncate(200);

    // 转换扩展名统计
    let mut extension_stats: Vec<ExtensionStat> = extension_map
        .into_iter()
        .map(|(ext, (count, size))| ExtensionStat {
            extension: format!(".{}", ext),
            file_count: count,
            total_size_mb: (size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
        })
        .collect();
    extension_stats.sort_by(|a, b| {
        b.total_size_mb
            .partial_cmp(&a.total_size_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    extension_stats.truncate(20);

    // 扫描一级子目录大小
    let directory_sizes = scan_top_level_dirs(&scan_path);

    ScanResult {
        large_files,
        directory_sizes,
        extension_stats,
        total_scanned,
        scan_path,
    }
}

/// 扫描指定目录下各子目录的大小
#[tauri::command]
pub fn scan_directory_sizes(path: Option<String>) -> Vec<DirectorySize> {
    let scan_path = path.unwrap_or_else(|| "C:\\".to_string());
    scan_top_level_dirs(&scan_path)
}

/// 删除指定文件
#[tauri::command]
pub fn delete_file(path: String) -> Result<bool, String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("文件不存在: {}", path));
    }

    std::fs::remove_file(&path_buf)
        .map_err(|e| format!("删除文件失败: {} - {}", path, e))?;

    Ok(true)
}

/// 删除指定目录
#[tauri::command]
pub fn delete_directory(path: String) -> Result<bool, String> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("目录不存在: {}", path));
    }

    std::fs::remove_dir_all(&path_buf)
        .map_err(|e| format!("删除目录失败: {} - {}", path, e))?;

    Ok(true)
}

// ========== 内部辅助函数 ==========

fn scan_top_level_dirs(root_path: &str) -> Vec<DirectorySize> {
    let mut dirs = Vec::new();
    let root = PathBuf::from(root_path);

    if !root.exists() {
        return dirs;
    }

    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();

            let mut total_size: u64 = 0;
            let mut file_count: u64 = 0;
            let mut subdir_count: u64 = 0;

            for walk_entry in WalkDir::new(&path)
                .max_depth(6)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if walk_entry.file_type().is_file() {
                    if let Ok(metadata) = walk_entry.metadata() {
                        total_size += metadata.len();
                        file_count += 1;
                    }
                } else if walk_entry.file_type().is_dir() && walk_entry.depth() == 1 {
                    subdir_count += 1;
                }
            }

            if total_size > 0 {
                dirs.push(DirectorySize {
                    path: path.to_string_lossy().to_string(),
                    name,
                    size_mb: (total_size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
                    file_count,
                    subdirectory_count: subdir_count,
                });
            }
        }
    }

    dirs.sort_by(|a, b| {
        b.size_mb
            .partial_cmp(&a.size_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    dirs
}
