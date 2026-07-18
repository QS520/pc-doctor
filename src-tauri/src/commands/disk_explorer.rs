use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

/// 磁盘信息（第一级）
#[derive(Serialize, Clone, Debug)]
pub struct DriveInfo {
    pub drive_letter: String,        // "C:" / "D:"
    pub label: String,               // 磁盘标签
    pub drive_type: String,           // "本地磁盘" / "可移动磁盘" / "网络磁盘"
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub usage_percent: f64,
    pub file_system: String,
}

/// 目录/文件条目（第二级及以下）
#[derive(Serialize, Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub size_display: String,         // "1.23 GB" / "456 MB" / "789 KB"
    pub modified: String,             // "2026-01-15"
    pub extension: Option<String>,    // 文件扩展名（目录为 None）
    pub file_count: u64,              // 目录内的文件数（文件为 0）
}

/// 目录扫描结果
#[derive(Serialize, Clone, Debug)]
pub struct ScanDirResult {
    pub current_path: String,
    pub parent_path: Option<String>,   // 上一级路径（用于返回）
    pub entries: Vec<DirEntry>,
    pub total_size_bytes: u64,
    pub total_size_display: String,
    pub entry_count: usize,
    pub is_root: bool,                  // 是否是磁盘根目录
}

/// 获取所有磁盘列表（第一级）
#[tauri::command]
pub fn list_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    #[cfg(windows)]
    {
        // 通过 PowerShell 查询磁盘信息
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=2 or DriveType=3 or DriveType=4' | ForEach-Object { $total=[math]::Round($_.Size/1GB,2); $free=[math]::Round($_.FreeSpace/1GB,2); $used=$total-$free; $pct=if($total -gt 0){[math]::Round($used/$total*100,1)}else{0}; $label=$_.VolumeName; if(!$label){$label='-'}; Write-Output \"$($_.DeviceID)|$label|$total|$used|$free|$pct|$($_.FileSystem)\" }",
            ])
            .output();

        if let Ok(output) = output {
            let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(7, '|').collect();
                if parts.len() >= 7 {
                    let total: f64 = parts[2].parse().unwrap_or(0.0);
                    let used: f64 = parts[3].parse().unwrap_or(0.0);
                    let free: f64 = parts[4].parse().unwrap_or(0.0);
                    let pct: f64 = parts[5].parse().unwrap_or(0.0);
                    let drive_type = if total > 0.0 {
                        "本地磁盘".to_string()
                    } else {
                        "可移动磁盘".to_string()
                    };
                    drives.push(DriveInfo {
                        drive_letter: parts[0].to_string(),
                        label: parts[1].to_string(),
                        drive_type,
                        total_gb: total,
                        used_gb: used,
                        free_gb: free,
                        usage_percent: pct,
                        file_system: parts[6].to_string(),
                    });
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        // Linux 环境：返回根目录和挂载点
        drives.push(DriveInfo {
            drive_letter: "/".to_string(),
            label: "根目录".to_string(),
            drive_type: "本地磁盘".to_string(),
            total_gb: 100.0,
            used_gb: 50.0,
            free_gb: 50.0,
            usage_percent: 50.0,
            file_system: "ext4".to_string(),
        });
    }

    drives
}

/// 扫描指定目录的内容（第二级及以下）
/// 返回该目录下的所有子目录和文件，按大小降序排列
#[tauri::command]
pub fn scan_directory(path: String) -> ScanDirResult {
    let current_path = path.clone();
    let path_buf = PathBuf::from(&path);

    // 判断是否是磁盘根目录
    let is_root = is_drive_root(&path);

    // 获取父目录
    let parent_path = get_parent_path(&path, is_root);

    let mut entries: Vec<DirEntry> = Vec::new();

    if !path_buf.exists() {
        return ScanDirResult {
            current_path,
            parent_path,
            entries,
            total_size_bytes: 0,
            total_size_display: "0 B".to_string(),
            entry_count: 0,
            is_root,
        };
    }

    // 读取当前目录下的所有条目
    if let Ok(read_dir) = std::fs::read_dir(&path_buf) {
        for entry in read_dir.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let name = entry.file_name().to_string_lossy().to_string();

            // 跳过系统保护目录
            let name_lower = name.to_lowercase();
            if matches!(
                name_lower.as_str(),
                "$recycle.bin" | "system volume information" | "$windows.~bs" | "$windows.~ws"
                | "config.msi"
            ) {
                continue;
            }

            if metadata.is_dir() {
                // 目录：计算目录总大小
                let (size_bytes, file_count) = calculate_dir_size(&entry_path);
                entries.push(DirEntry {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_dir: true,
                    size_bytes,
                    size_display: format_size(size_bytes),
                    modified: get_modified_time(&entry_path),
                    extension: None,
                    file_count,
                });
            } else if metadata.is_file() {
                // 文件
                let size = metadata.len();
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase());
                entries.push(DirEntry {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_dir: false,
                    size_bytes: size,
                    size_display: format_size(size),
                    modified: get_modified_time(&entry_path),
                    extension: ext,
                    file_count: 0,
                });
            }
        }
    }

    // 计算总大小
    let total_size_bytes: u64 = entries.iter().map(|e| e.size_bytes).sum();

    // 按大小降序排列（目录和文件混合排列）
    entries.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    ScanDirResult {
        current_path,
        parent_path,
        entries,
        total_size_bytes,
        total_size_display: format_size(total_size_bytes),
        entry_count: 0, // 后面填充
        is_root,
    }
}

// ========== 内部辅助函数 ==========

fn is_drive_root(path: &str) -> bool {
    #[cfg(windows)]
    {
        // Windows: "C:\" 或 "C:"
        let p = path.trim_end_matches('\\');
        p.len() == 2 && p.ends_with(':')
    }
    #[cfg(not(windows))]
    {
        path == "/"
    }
}

fn get_parent_path(path: &str, is_root: bool) -> Option<String> {
    if is_root {
        return None; // 磁盘根目录没有父级
    }

    let path_buf = PathBuf::from(path);
    path_buf
        .parent()
        .map(|p| {
            let parent = p.to_string_lossy().to_string();
            if parent.is_empty() {
                None
            } else {
                Some(parent)
            }
        })
        .flatten()
}

fn calculate_dir_size(dir_path: &PathBuf) -> (u64, u64) {
    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;

    // 使用 walkdir 遍历子目录，但限制深度避免过慢
    for entry in WalkDir::new(dir_path)
        .max_depth(10)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            !matches!(
                name.as_str(),
                "$recycle.bin" | "system volume information" | "$windows.~bs" | "$windows.~ws"
            )
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                file_count += 1;
            }
        }
    }

    (total_size, file_count)
}

fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

fn get_modified_time(path: &PathBuf) -> String {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                    if let Some(dt) = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    {
                        return dt.format("%Y-%m-%d").to_string();
                    }
                }
            }
        }
        Err(_) => {}
    }
    "-".to_string()
}
