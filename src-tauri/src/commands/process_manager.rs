use serde::Serialize;
use sysinfo::System;

#[derive(Serialize, Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub memory_percent: f64,
    pub status: String,
    pub command: String,
    pub user: String,
}

/// 获取所有运行中的进程 (按 CPU 使用率排序)
#[tauri::command]
pub fn get_processes(sort_by: Option<String>) -> Vec<ProcessInfo> {
    let sort_field = sort_by.unwrap_or_else(|| "cpu".to_string());

    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let total_mem = sys.total_memory() as f64;

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let mem_mb = process.memory() as f64 / (1024.0 * 1024.0);
            let mem_percent = if total_mem > 0.0 {
                (process.memory() as f64 / total_mem) * 100.0
            } else {
                0.0
            };

            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_mb: (mem_mb * 100.0).round() / 100.0,
                memory_percent: (mem_percent * 10.0).round() / 10.0,
                status: format!("{:?}", process.status()),
                command: process
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                user: process
                    .user_id()
                    .map(|uid| uid.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
            }
        })
        .collect();

    match sort_field.as_str() {
        "memory" => processes.sort_by(|a, b| {
            b.memory_mb
                .partial_cmp(&a.memory_mb)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "name" => processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
        _ => processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }

    // 只返回前 100 个进程
    processes.truncate(100);
    processes
}

/// 结束指定进程
#[tauri::command]
pub fn kill_process(pid: u32) -> Result<bool, String> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let pid = sysinfo::Pid::from_u32(pid);
    if let Some(process) = sys.process(pid) {
        if process.kill() {
            Ok(true)
        } else {
            Err(format!("无法结束进程 PID: {}，可能需要管理员权限", pid))
        }
    } else {
        Err(format!("未找到进程 PID: {}", pid))
    }
}

/// 结束进程树 (包括所有子进程)
#[tauri::command]
pub fn kill_process_tree(pid: u32) -> Result<u32, String> {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let target_pid = sysinfo::Pid::from_u32(pid);
    let mut killed = 0u32;

    // 查找所有子进程
    let children: Vec<sysinfo::Pid> = sys
        .processes()
        .iter()
        .filter(|(_, p)| p.parent() == Some(target_pid))
        .map(|(pid, _)| *pid)
        .collect();

    // 递归结束子进程
    for child_pid in children {
        if let Some(child) = sys.process(child_pid) {
            if child.kill() {
                killed += 1;
            }
        }
    }

    // 结束主进程
    if let Some(process) = sys.process(target_pid) {
        if process.kill() {
            killed += 1;
        } else {
            return Err(format!("无法结束进程 PID: {}，可能需要管理员权限", pid));
        }
    } else {
        return Err(format!("未找到进程 PID: {}", pid));
    }

    Ok(killed)
}
