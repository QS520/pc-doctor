use serde::Serialize;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Serialize, Clone, Debug)]
pub struct BoostResult {
    pub success: bool,
    pub freed_memory_gb: f64,
    pub killed_processes: u32,
    pub cleaned_temp_mb: f64,
    pub cleaned_cache_mb: f64,
    pub details: Vec<String>,
}

/// 一键加速：结束高占用后台进程 + 清理临时文件 + 释放内存
#[tauri::command]
pub fn one_click_boost() -> BoostResult {
    let mut details = Vec::new();
    let mut freed_memory_bytes: u64 = 0;
    let mut killed_count: u32 = 0;

    let mem_before = {
        let sys = System::new_all();
        sys.used_memory()
    };

    // 1. 结束 CPU 占用 > 5% 且非系统关键进程的后台进程
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let critical_processes = [
        "System", "Registry", "smss.exe", "csrss.exe", "wininit.exe",
        "services.exe", "lsass.exe", "svchost.exe", "fontdrvhost.exe",
        "dwm.exe", "explorer.exe", "RuntimeBroker.exe", "SearchHost.exe",
        "StartMenuExperienceHost.exe", "TextInputHost.exe", "sihost.exe",
        "taskhostw.exe", "ctfmon.exe", "conhost.exe", "spoolsv.exe",
        "winlogon.exe", "fontdrvhost.exe", "SecurityHealthSystray.exe",
        "SecurityHealthService.exe", "SearchIndexer.exe", "MsMpEng.exe",
        "NisSrv.exe", "WmiPrvSE.exe", "dllhost.exe", "audiodg.exe",
        "ApplicationFrameHost.exe", "ShellExperienceHost.exe",
    ];

    let processes_to_kill: Vec<(sysinfo::Pid, String, f32, u64)> = sys
        .processes()
        .iter()
        .filter(|(_, p)| {
            let name = p.name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();
            // 排除系统关键进程
            !critical_processes.iter().any(|c| name_lower == c.to_lowercase())
            // 排除自己的进程
            && !name_lower.contains("pc-doctor") && !name_lower.contains("急诊")
            // CPU 占用 > 5% 或内存 > 500MB
            && (p.cpu_usage() > 5.0 || p.memory() > 500 * 1024 * 1024)
        })
        .map(|(pid, p)| {
            (*pid, p.name().to_string_lossy().to_string(), p.cpu_usage(), p.memory())
        })
        .collect();

    for (pid, name, cpu, mem) in processes_to_kill {
        if let Some(process) = sys.process(pid) {
            if process.kill() {
                freed_memory_bytes += mem;
                killed_count += 1;
                details.push(format!(
                    "已结束: {} (CPU: {:.1}%, 内存: {:.0} MB)",
                    name,
                    cpu,
                    mem as f64 / (1024.0 * 1024.0)
                ));
            }
        }
    }

    // 2. 清理临时文件
    let temp_path = std::env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
    let cleaned_temp = clean_directory_contents(&temp_path);
    details.push(format!("清理临时文件: {:.1} MB", cleaned_temp));

    // 3. 清理系统临时文件
    let sys_temp = "C:\\Windows\\Temp";
    let cleaned_sys_temp = clean_directory_contents(sys_temp);
    details.push(format!("清理系统临时文件: {:.1} MB", cleaned_sys_temp));

    // 4. 清理浏览器缓存
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let mut cleaned_cache = 0.0;

    let chrome_cache = format!("{}\\Google\\Chrome\\User Data\\Default\\Cache", local_app_data);
    cleaned_cache += clean_directory_contents(&chrome_cache);

    let edge_cache = format!("{}\\Microsoft\\Edge\\User Data\\Default\\Cache", local_app_data);
    cleaned_cache += clean_directory_contents(&edge_cache);

    details.push(format!("清理浏览器缓存: {:.1} MB", cleaned_cache));

    // 5. 刷新 DNS 缓存
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("ipconfig")
            .args(["/flushdns"])
            .output();
        details.push("已刷新 DNS 缓存".to_string());
    }

    // 6. 计算释放的内存
    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut sys_after = System::new();
    sys_after.refresh_memory();
    let mem_after = sys_after.used_memory();

    let freed_memory = if mem_before > mem_after {
        (mem_before - mem_after) as f64 / (1024.0 * 1024.0 * 1024.0)
    } else {
        freed_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    };

    BoostResult {
        success: true,
        freed_memory_gb: (freed_memory * 100.0).round() / 100.0,
        killed_processes: killed_count,
        cleaned_temp_mb: (cleaned_temp * 100.0).round() / 100.0,
        cleaned_cache_mb: (cleaned_cache * 100.0).round() / 100.0,
        details,
    }
}

fn clean_directory_contents(path: &str) -> f64 {
    use std::path::Path;
    let path_buf = Path::new(path);
    if !path_buf.exists() {
        return 0.0;
    }

    let mut freed_bytes: u64 = 0;
    let mut stack = vec![path_buf.to_path_buf()];

    while let Some(dir) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path);
                } else {
                    if let Ok(metadata) = std::fs::metadata(&entry_path) {
                        let size = metadata.len();
                        if std::fs::remove_file(&entry_path).is_ok() {
                            freed_bytes += size;
                        }
                    }
                }
            }
        }
    }

    (freed_bytes as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0
}
