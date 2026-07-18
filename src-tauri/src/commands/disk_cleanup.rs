use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

/// 垃圾文件分类
#[derive(Serialize, Clone, Debug)]
pub struct JunkCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub file_count: u64,
    pub size_mb: f64,
    pub safe_to_delete: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct ScanResult {
    pub categories: Vec<JunkCategory>,
    pub total_size_mb: f64,
    pub total_files: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct CleanResult {
    pub success: bool,
    pub deleted_files: u64,
    pub freed_mb: f64,
    pub errors: Vec<String>,
    pub skipped: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiskSpace {
    pub drive: String,
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f64,
}

/// 扫描所有可清理的垃圾文件
#[tauri::command]
pub fn scan_junk_files() -> ScanResult {
    let mut categories = Vec::new();

    // 1. 用户临时文件 %TEMP%
    let user_temp = std::env::var("TEMP").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Local\\Temp".to_string());
    categories.push(scan_directory(
        "user_temp",
        "用户临时文件",
        "应用程序运行产生的临时文件，可安全删除",
        &user_temp,
        true,
    ));

    // 2. 系统临时文件
    categories.push(scan_directory(
        "system_temp",
        "系统临时文件",
        "Windows 系统临时文件",
        "C:\\Windows\\Temp",
        true,
    ));

    // 3. Windows 更新缓存
    categories.push(scan_directory(
        "windows_update",
        "Windows 更新缓存",
        "已下载的 Windows 更新安装包",
        "C:\\Windows\\SoftwareDistribution\\Download",
        true,
    ));

    // 4. 预读取文件
    categories.push(scan_directory(
        "prefetch",
        "预读取缓存",
        "应用程序启动预读取数据，删除后首次启动略慢",
        "C:\\Windows\\Prefetch",
        true,
    ));

    // 5. Windows 错误报告
    let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    let wer_path = format!("{}\\Microsoft\\Windows\\WER", program_data);
    categories.push(scan_directory(
        "error_reports",
        "Windows 错误报告",
        "程序崩溃和错误报告数据",
        &wer_path,
        true,
    ));

    // 6. Windows 日志
    categories.push(scan_directory(
        "windows_logs",
        "Windows 日志文件",
        "系统事件日志和安装日志",
        "C:\\Windows\\Logs",
        true,
    ));

    // 7. 缩略图缓存
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
        format!("C:\\Users\\{}\\AppData\\Local", std::env::var("USERNAME").unwrap_or_default())
    });
    let thumb_path = format!("{}\\Microsoft\\Windows\\Explorer", local_app_data);
    categories.push(scan_files_pattern(
        "thumbnail_cache",
        "缩略图缓存",
        "资源管理器缩略图缓存，删除后自动重建",
        &thumb_path,
        "thumbcache_*.db",
        true,
    ));

    // 8. Chrome 浏览器缓存
    let chrome_cache = format!(
        "{}\\Google\\Chrome\\User Data\\Default\\Cache",
        local_app_data
    );
    categories.push(scan_directory(
        "chrome_cache",
        "Chrome 浏览器缓存",
        "Google Chrome 浏览器缓存文件",
        &chrome_cache,
        true,
    ));

    // 9. Edge 浏览器缓存
    let edge_cache = format!(
        "{}\\Microsoft\\Edge\\User Data\\Default\\Cache",
        local_app_data
    );
    categories.push(scan_directory(
        "edge_cache",
        "Edge 浏览器缓存",
        "Microsoft Edge 浏览器缓存文件",
        &edge_cache,
        true,
    ));

    // 10. 临时互联网文件
    let ie_cache = format!(
        "{}\\Microsoft\\Windows\\INetCache",
        local_app_data
    );
    categories.push(scan_directory(
        "inet_cache",
        "临时互联网文件",
        "Internet Explorer/Edge 临时文件",
        &ie_cache,
        true,
    ));

    // 11. 交付优化文件
    categories.push(scan_directory(
        "delivery_optimization",
        "交付优化缓存",
        "Windows 交付优化下载缓存",
        "C:\\Windows\\ServiceProfiles\\NetworkService\\AppData\\Local\\Microsoft\\Windows\\DeliveryOptimization",
        true,
    ));

    // 12. 内存转储文件
    categories.push(scan_single_file(
        "memory_dumps",
        "内存转储文件",
        "系统崩溃时的完整内存转储",
        "C:\\Windows\\Memory.dmp",
        true,
    ));

    // 13. 小型内存转储
    categories.push(scan_directory(
        "minidumps",
        "小型内存转储",
        "蓝屏崩溃时生成的小型转储文件",
        "C:\\Windows\\Minidump",
        true,
    ));

    // 14. 旧版 Windows 安装文件
    categories.push(scan_directory(
        "windows_old",
        "旧版 Windows 安装",
        "Windows 升级后保留的旧系统文件（删除后不可回退）",
        "C:\\Windows.old",
        false,
    ));

    // 15. 回收站
    #[cfg(windows)]
    {
        let recycle_size = get_recycle_bin_size();
        categories.push(JunkCategory {
            id: "recycle_bin".to_string(),
            name: "回收站".to_string(),
            description: "已删除文件的回收站内容".to_string(),
            path: "回收站".to_string(),
            file_count: 0,
            size_mb: recycle_size,
            safe_to_delete: true,
        });
    }

    let total_size: f64 = categories.iter().map(|c| c.size_mb).sum();
    let total_files: u64 = categories.iter().map(|c| c.file_count).sum();

    ScanResult {
        categories,
        total_size_mb: (total_size * 100.0).round() / 100.0,
        total_files,
    }
}

/// 清理选定的垃圾文件
#[tauri::command]
pub fn clean_junk_files(category_ids: Vec<String>) -> CleanResult {
    let mut deleted_files: u64 = 0;
    let mut freed_bytes: u64 = 0;
    let mut errors = Vec::new();
    let mut skipped: u64 = 0;

    let all_categories = scan_junk_files();

    for cat_id in &category_ids {
        let category = match all_categories.categories.iter().find(|c| &c.id == cat_id) {
            Some(c) => c,
            None => continue,
        };

        match category.id.as_str() {
            "recycle_bin" => {
                #[cfg(windows)]
                {
                    match empty_recycle_bin_internal() {
                        Ok(size) => freed_bytes += size,
                        Err(e) => errors.push(format!("清空回收站失败: {}", e)),
                    }
                }
            }
            "thumbnail_cache" => {
                let (deleted, freed, errs) = delete_files_pattern(&category.path, "thumbcache_*.db");
                deleted_files += deleted;
                freed_bytes += freed;
                errors.extend(errs);
            }
            "memory_dumps" => {
                let path = PathBuf::from(&category.path);
                if path.exists() {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        freed_bytes += metadata.len();
                        if let Err(e) = std::fs::remove_file(&path) {
                            errors.push(format!("删除 {} 失败: {}", path.display(), e));
                        } else {
                            deleted_files += 1;
                        }
                    }
                }
            }
            _ => {
                let (deleted, freed, errs, skip) = delete_directory_contents(&category.path);
                deleted_files += deleted;
                freed_bytes += freed;
                errors.extend(errs);
                skipped += skip;
            }
        }
    }

    CleanResult {
        success: errors.is_empty(),
        deleted_files,
        freed_mb: (freed_bytes as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
        errors,
        skipped,
    }
}

/// 清空回收站
#[tauri::command]
pub fn empty_recycle_bin() -> Result<bool, String> {
    #[cfg(windows)]
    {
        empty_recycle_bin_internal().map(|_| true).map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows 平台".to_string())
    }
}

/// 刷新 DNS 缓存
#[tauri::command]
pub fn flush_dns_cache() -> Result<String, String> {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("ipconfig")
            .args(["/flushdns"])
            .output()
            .map_err(|e| format!("执行失败: {}", e))?;

        if output.status.success() {
            Ok("DNS 缓存已成功刷新".to_string())
        } else {
            Err(format!(
                "刷新失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows 平台".to_string())
    }
}

/// 获取磁盘剩余空间
#[tauri::command]
pub fn get_disk_space(drive: Option<String>) -> DiskSpace {
    #[cfg(windows)]
    {
        let drive_letter = drive.unwrap_or_else(|| "C:\\".to_string());
        let drive_path = if drive_letter.ends_with('\\') {
            drive_letter
        } else {
            format!("{}\\", drive_letter)
        };

        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        use windows::core::PCWSTR;

        let wide: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let mut free_bytes: u64 = 0;
            let mut total_bytes: u64 = 0;
            let mut available_bytes: u64 = 0;

            let _ = GetDiskFreeSpaceExW(
                PCWSTR(wide.as_ptr()),
                Some(&mut available_bytes),
                Some(&mut total_bytes),
                Some(&mut free_bytes),
            );

            let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let free_gb = free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = total_gb - free_gb;
            let percent = if total_gb > 0.0 {
                (used_gb / total_gb) * 100.0
            } else {
                0.0
            };

            DiskSpace {
                drive: drive_letter.chars().next().unwrap_or('C').to_string(),
                total_gb: (total_gb * 100.0).round() / 100.0,
                free_gb: (free_gb * 100.0).round() / 100.0,
                used_gb: (used_gb * 100.0).round() / 100.0,
                usage_percent: (percent * 10.0).round() / 10.0,
            }
        }
    }
    #[cfg(not(windows))]
    {
        DiskSpace {
            drive: "C".to_string(),
            total_gb: 0.0,
            free_gb: 0.0,
            used_gb: 0.0,
            usage_percent: 0.0,
        }
    }
}

// ========== 内部辅助函数 ==========

fn scan_directory(id: &str, name: &str, desc: &str, path: &str, safe: bool) -> JunkCategory {
    let path_buf = PathBuf::from(path);

    if !path_buf.exists() {
        return JunkCategory {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            path: path.to_string(),
            file_count: 0,
            size_mb: 0.0,
            safe_to_delete: safe,
        };
    }

    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;

    for entry in WalkDir::new(&path_buf)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
                file_count += 1;
            }
        }
    }

    JunkCategory {
        id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        path: path.to_string(),
        file_count,
        size_mb: (total_size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
        safe_to_delete: safe,
    }
}

fn scan_files_pattern(id: &str, name: &str, desc: &str, dir: &str, pattern: &str, safe: bool) -> JunkCategory {
    let path_buf = PathBuf::from(dir);

    if !path_buf.exists() {
        return JunkCategory {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            path: dir.to_string(),
            file_count: 0,
            size_mb: 0.0,
            safe_to_delete: safe,
        };
    }

    let mut total_size: u64 = 0;
    let mut file_count: u64 = 0;

    let glob_pattern = format!("{}\\{}", dir, pattern);
    if let Ok(paths) = glob::glob(&glob_pattern) {
        for entry in paths.filter_map(|e| e.ok()) {
            if let Ok(metadata) = std::fs::metadata(&entry) {
                if metadata.is_file() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }
    }

    JunkCategory {
        id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        path: dir.to_string(),
        file_count,
        size_mb: (total_size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
        safe_to_delete: safe,
    }
}

fn scan_single_file(id: &str, name: &str, desc: &str, path: &str, safe: bool) -> JunkCategory {
    let path_buf = PathBuf::from(path);

    if !path_buf.exists() {
        return JunkCategory {
            id: id.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            path: path.to_string(),
            file_count: 0,
            size_mb: 0.0,
            safe_to_delete: safe,
        };
    }

    let size = std::fs::metadata(&path_buf)
        .map(|m| m.len())
        .unwrap_or(0);

    JunkCategory {
        id: id.to_string(),
        name: name.to_string(),
        description: desc.to_string(),
        path: path.to_string(),
        file_count: 1,
        size_mb: (size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
        safe_to_delete: safe,
    }
}

fn delete_directory_contents(path: &str) -> (u64, u64, Vec<String>, u64) {
    let mut deleted: u64 = 0;
    let mut freed: u64 = 0;
    let mut errors = Vec::new();
    let mut skipped: u64 = 0;

    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return (0, 0, errors, 0);
    }

    for entry in WalkDir::new(&path_buf)
        .max_depth(5)
        .contents_first(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let file_path = entry.path();
            match std::fs::metadata(file_path) {
                Ok(meta) => {
                    let size = meta.len();
                    match std::fs::remove_file(file_path) {
                        Ok(_) => {
                            deleted += 1;
                            freed += size;
                        }
                        Err(e) => {
                            // 某些文件被占用是正常的，不报错只跳过
                            if e.kind() != std::io::ErrorKind::PermissionDenied {
                                errors.push(format!("删除失败: {} - {}", file_path.display(), e));
                            } else {
                                skipped += 1;
                            }
                        }
                    }
                }
                Err(_) => {
                    skipped += 1;
                }
            }
        }
    }

    (deleted, freed, errors, skipped)
}

fn delete_files_pattern(dir: &str, pattern: &str) -> (u64, u64, Vec<String>) {
    let mut deleted: u64 = 0;
    let mut freed: u64 = 0;
    let mut errors = Vec::new();

    let glob_pattern = format!("{}\\{}", dir, pattern);
    if let Ok(paths) = glob::glob(&glob_pattern) {
        for entry in paths.filter_map(|e| e.ok()) {
            if let Ok(metadata) = std::fs::metadata(&entry) {
                let size = metadata.len();
                match std::fs::remove_file(&entry) {
                    Ok(_) => {
                        deleted += 1;
                        freed += size;
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::PermissionDenied {
                            errors.push(format!("删除失败: {} - {}", entry.display(), e));
                        }
                    }
                }
            }
        }
    }

    (deleted, freed, errors)
}

#[cfg(windows)]
fn get_recycle_bin_size() -> f64 {
    use windows::Win32::UI::Shell::{
        SHQueryRecycleBinW, SHQUERYRBINFO,
    };

    unsafe {
        let mut info: SHQUERYRBINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<SHQUERYRBINFO>() as u32;

        let drive_wide: Vec<u16> = "C:\\".encode_utf16().chain(std::iter::once(0)).collect();
        let _ = SHQueryRecycleBinW(
            windows::core::PCWSTR(drive_wide.as_ptr()),
            &mut info,
        );

        (info.i64Size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0
    }
}

#[cfg(windows)]
fn empty_recycle_bin_internal() -> Result<u64, String> {
    use windows::Win32::UI::Shell::SHEmptyRecycleBinW;
    use windows::Win32::UI::Shell::SHERB_NOCONFIRMATION;
    use windows::Win32::UI::Shell::SHERB_NOPROGRESSUI;
    use windows::Win32::UI::Shell::SHERB_NOSOUND;

    let recycle_size = get_recycle_bin_size() as u64 * 1024 * 1024;

    unsafe {
        let result = SHEmptyRecycleBinW(
            None,
            windows::core::PCWSTR::null(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
        );

        if result.is_ok() {
            Ok(recycle_size)
        } else {
            Err("清空回收站失败，可能需要管理员权限".to_string())
        }
    }
}
