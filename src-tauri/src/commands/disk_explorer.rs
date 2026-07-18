use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;
use walkdir::WalkDir;

/// 磁盘信息（第一级）
#[derive(Serialize, Clone, Debug)]
pub struct DriveInfo {
    pub drive_letter: String,
    pub label: String,
    pub drive_type: String,
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
    pub size_display: String,
    pub modified: String,
    pub extension: Option<String>,
    pub file_count: u64,
    /// 是否为估算大小（true 表示因超时或深度限制未完整扫描）
    pub is_estimated: bool,
}

/// 目录扫描结果
#[derive(Serialize, Clone, Debug)]
pub struct ScanDirResult {
    pub current_path: String,
    pub parent_path: Option<String>,
    pub entries: Vec<DirEntry>,
    pub total_size_bytes: u64,
    pub total_size_display: String,
    pub entry_count: usize,
    pub is_root: bool,
    /// 扫描耗时（毫秒）
    pub scan_time_ms: u64,
    /// 是否部分目录因超时被跳过深度扫描
    pub has_partial: bool,
}

/// 获取所有磁盘列表（第一级）
#[tauri::command]
pub fn list_drives() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    #[cfg(windows)]
    {
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

/// 扫描指定目录的内容（快速模式）
/// 立即返回当前层级的所有条目：文件大小即时获取，子目录大小先返回 0（待异步计算）
/// 配合 calculate_dir_sizes 命令异步推送子目录大小，实现"秒开"体验
#[tauri::command]
pub fn scan_directory(path: String) -> ScanDirResult {
    let start_time = Instant::now();
    let current_path = path.clone();
    let path_buf = PathBuf::from(&path);
    let is_root = is_drive_root(&path);
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
            scan_time_ms: 0,
            has_partial: false,
        };
    }

    // 只读取当前层级（read_dir 本身极快，不递归）
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
                // 子目录：大小先返回 0，标记为待计算
                entries.push(DirEntry {
                    name,
                    path: entry_path.to_string_lossy().to_string(),
                    is_dir: true,
                    size_bytes: 0,
                    size_display: "计算中...".to_string(),
                    modified: get_modified_time(&entry_path),
                    extension: None,
                    file_count: 0,
                    is_estimated: true,
                });
            } else if metadata.is_file() {
                // 文件：大小即时获取
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
                    is_estimated: false,
                });
            }
        }
    }

    // 文件按大小降序，子目录先排在前面（待计算）
    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            b.size_bytes
                .cmp(&a.size_bytes)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
    });

    let scan_time_ms = start_time.elapsed().as_millis() as u64;
    let total_size_bytes: u64 = entries.iter().map(|e| e.size_bytes).sum();

    ScanDirResult {
        current_path,
        parent_path,
        entries,
        total_size_bytes,
        total_size_display: format_size(total_size_bytes),
        entry_count: 0,
        is_root,
        scan_time_ms,
        has_partial: false,
    }
}

/// 异步计算多个目录的大小，通过事件逐步推送结果
/// 前端监听 "dir-size-update" 事件，实时更新 UI
#[tauri::command]
pub async fn calculate_dir_sizes(paths: Vec<String>, app: tauri::AppHandle) {
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    if paths.is_empty() {
        return;
    }

    // 全局超时保护：最多 30 秒
    let timeout = Arc::new(AtomicBool::new(false));
    let timeout_clone = timeout.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(30)).await;
        timeout_clone.store(true, Ordering::Relaxed);
    });

    // 并行计算每个目录大小，完成后立即推送事件
    let results: Vec<(String, u64, u64, bool)> = paths
        .par_iter()
        .map(|path| {
            let path_buf = PathBuf::from(path);
            let timeout_flag = timeout.clone();
            let (size, count, is_est) = walk_with_timeout(&path_buf, &timeout_flag);
            (path.clone(), size, count, is_est)
        })
        .collect();

    // 推送每个结果
    for (path, size, count, is_estimated) in results {
        let _ = app.emit(
            "dir-size-update",
            serde_json::json!({
                "path": path,
                "size_bytes": size,
                "size_display": format_size(size),
                "file_count": count,
                "is_estimated": is_estimated,
            }),
        );
    }
}

// ========== 内部辅助函数 ==========

fn is_drive_root(path: &str) -> bool {
    #[cfg(windows)]
    {
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
        return None;
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

/// 带超时检查的目录遍历
fn walk_with_timeout(dir_path: &PathBuf, timeout_flag: &Arc<AtomicBool>) -> (u64, u64, bool) {
    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;
    let mut is_estimated = false;
    let mut iter_count: u64 = 0;

    // 根据目录层级动态调整最大深度
    // 磁盘根目录：max_depth=8（覆盖大部分内容）
    // 二级及以下：max_depth=15（更深的完整扫描）
    let max_depth = if is_drive_root(&dir_path.to_string_lossy()) {
        8
    } else {
        15
    };

    for entry in WalkDir::new(dir_path)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            !matches!(
                name.as_str(),
                "$recycle.bin" | "system volume information" | "$windows.~bs" | "$windows.~ws"
                    | "node_modules" | ".git" | "__pycache__" | "target"
            )
        })
        .filter_map(|e| e.ok())
    {
        // 每 500 个文件检查一次超时
        iter_count += 1;
        if iter_count % 500 == 0 && timeout_flag.load(Ordering::Relaxed) {
            is_estimated = true;
            break;
        }

        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                file_count += 1;
            }
        }
    }

    (total_size, file_count, is_estimated)
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
