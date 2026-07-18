use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
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

/// 扫描指定目录的内容（第二级及以下）
/// 使用并行遍历 + 超时保护，大幅提升大目录扫描速度
#[tauri::command]
pub fn scan_directory(path: String) -> ScanDirResult {
    let start_time = Instant::now();
    let current_path = path.clone();
    let path_buf = PathBuf::from(&path);
    let is_root = is_drive_root(&path);
    let parent_path = get_parent_path(&path, is_root);

    let mut entries: Vec<DirEntry> = Vec::new();
    let mut has_partial = false;

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
            has_partial,
        };
    }

    // 先收集所有直接子条目
    let mut subdirs: Vec<(PathBuf, String)> = Vec::new();
    let mut files: Vec<(PathBuf, String, std::fs::Metadata)> = Vec::new();

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
                subdirs.push((entry_path, name));
            } else if metadata.is_file() {
                files.push((entry_path, name, metadata));
            }
        }
    }

    // === 优化1：文件处理极快（只读 metadata） ===
    for (entry_path, name, metadata) in files {
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

    // === 优化2：子目录大小用 rayon 并行计算 ===
    // 为每个子目录分配独立的超时检查，避免某个超大目录卡住整个扫描
    let total_timeout = Arc::new(AtomicBool::new(false));
    let total_deadline = Instant::now() + Duration::from_secs(8); // 总超时 8 秒
    let timeout_clone = total_timeout.clone();
    std::thread::spawn(move || {
        while Instant::now() < total_deadline {
            std::thread::sleep(Duration::from_millis(100));
            if timeout_clone.load(Ordering::Relaxed) {
                return;
            }
        }
        timeout_clone.store(true, Ordering::Relaxed);
    });

    // 使用 rayon 并行计算每个子目录的大小
    use rayon::prelude::*;
    let subdir_results: Vec<(PathBuf, String, (u64, u64, bool))> = subdirs
        .par_iter()
        .map(|(path, name)| {
            let timeout_flag = total_timeout.clone();
            let (size, count, is_est) = calculate_dir_size_fast(path, &timeout_flag);
            (path.clone(), name.clone(), (size, count, is_est))
        })
        .collect();

    // 释放超时控制线程
    total_timeout.store(true, Ordering::Relaxed);

    for (entry_path, name, (size_bytes, file_count, is_estimated)) in subdir_results {
        if is_estimated {
            has_partial = true;
        }
        entries.push(DirEntry {
            name,
            path: entry_path.to_string_lossy().to_string(),
            is_dir: true,
            size_bytes,
            size_display: format_size(size_bytes),
            modified: get_modified_time(&entry_path),
            extension: None,
            file_count,
            is_estimated,
        });
    }

    // 计算总大小
    let total_size_bytes: u64 = entries.iter().map(|e| e.size_bytes).sum();

    // 按大小降序排列
    entries.sort_by(|a, b| {
        b.size_bytes
            .cmp(&a.size_bytes)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    let scan_time_ms = start_time.elapsed().as_millis() as u64;

    ScanDirResult {
        current_path,
        parent_path,
        entries,
        total_size_bytes,
        total_size_display: format_size(total_size_bytes),
        entry_count: 0,
        is_root,
        scan_time_ms,
        has_partial,
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

/// 快速计算目录大小（带超时保护）
/// 返回 (size_bytes, file_count, is_estimated)
fn calculate_dir_size_fast(
    dir_path: &PathBuf,
    timeout_flag: &Arc<AtomicBool>,
) -> (u64, u64, bool) {
    use std::sync::mpsc;
    use std::thread;

    // 用子线程+通道实现，主线程超时检查
    let (tx, rx) = mpsc::channel();
    let path_clone = dir_path.clone();
    let timeout_clone = timeout_flag.clone();

    let handle = thread::spawn(move || {
        let result = walk_with_timeout(&path_clone, &timeout_clone);
        let _ = tx.send(result);
    });

    // 等待最多 3 秒（单目录），或直到全局超时
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match rx.try_recv() {
            Ok(result) => {
                // 线程完成，等待回收
                let _ = handle.join();
                return result;
            }
            Err(mpsc::TryRecvError::Empty) => {
                if Instant::now() >= deadline || timeout_flag.load(Ordering::Relaxed) {
                    // 超时：放弃这个目录的深度扫描，返回 0
                    // 线程仍在运行，但因为我们用的是 scoped thread，它会继续跑完
                    // 这里不 join，让它在后台完成（实际进程退出时会清理）
                    return (0, 0, true);
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                let _ = handle.join();
                return (0, 0, true);
            }
        }
    }
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
