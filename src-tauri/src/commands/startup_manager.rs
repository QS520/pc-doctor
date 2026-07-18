use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub location: String,
    pub enabled: bool,
    pub source: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct BootDuration {
    pub last_boot_time: String,
    pub boot_duration_seconds: u64,
    pub boot_duration_display: String,
}

/// 获取所有开机启动项
#[tauri::command]
pub fn get_startup_items() -> Vec<StartupItem> {
    let mut items = Vec::new();

    #[cfg(windows)]
    {
        // 1. HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run (所有用户)
        items.extend(read_registry_run_items(
            winreg::enums::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "注册表(系统)",
        ));

        // 2. HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run (当前用户)
        items.extend(read_registry_run_items(
            winreg::enums::HKEY_CURRENT_USER,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
            "注册表(用户)",
        ));

        // 3. HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce
        items.extend(read_registry_run_items(
            winreg::enums::HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
            "注册表(运行一次)",
        ));

        // 4. 启动文件夹 - 当前用户
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        let startup_path = format!(
            "{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup",
            appdata
        );
        items.extend(read_startup_folder_items(&startup_path, "启动文件夹(用户)"));

        // 5. 启动文件夹 - 所有用户
        let program_data = std::env::var("PROGRAMDATA").unwrap_or_default();
        let common_startup_path = format!(
            "{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup",
            program_data
        );
        items.extend(read_startup_folder_items(&common_startup_path, "启动文件夹(系统)"));

        // 6. 计划任务中的启动项 (简化处理)
        items.extend(read_scheduled_tasks());
    }

    items
}

/// 禁用启动项 (将其从 Run 移到 Run- 下，或从启动文件夹删除快捷方式)
#[tauri::command]
pub fn disable_startup_item(name: String, source: String) -> Result<bool, String> {
    #[cfg(windows)]
    {
        match source.as_str() {
            "注册表(系统)" => {
                move_registry_value(
                    winreg::enums::HKEY_LOCAL_MACHINE,
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run-",
                    &name,
                )
            }
            "注册表(用户)" => {
                move_registry_value(
                    winreg::enums::HKEY_CURRENT_USER,
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run-",
                    &name,
                )
            }
            "启动文件夹(用户)" | "启动文件夹(系统)" => {
                // 重命名快捷方式添加 .disabled 后缀
                let appdata = std::env::var("APPDATA").unwrap_or_default();
                let program_data = std::env::var("PROGRAMDATA").unwrap_or_default();
                let base_path = if source.contains("用户") {
                    format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup", appdata)
                } else {
                    format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup", program_data)
                };

                // 尝试 .lnk 和 .bat
                for ext in &[".lnk", ".bat", ".cmd", ".vbs"] {
                    let file_path = format!("{}\\{}{}", base_path, name, ext);
                    let disabled_path = format!("{}\\{}{}.disabled", base_path, name, ext);
                    if std::path::Path::new(&file_path).exists() {
                        std::fs::rename(&file_path, &disabled_path)
                            .map_err(|e| format!("禁用失败: {}", e))?;
                        return Ok(true);
                    }
                }
                Err("未找到启动文件".to_string())
            }
            "计划任务" => {
                let output = std::process::Command::new("schtasks")
                    .args(["/Change", "/TN", &name, "/DISABLE"])
                    .output()
                    .map_err(|e| format!("执行失败: {}", e))?;
                if output.status.success() {
                    Ok(true)
                } else {
                    Err(format!("禁用失败: {}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            _ => Err(format!("未知启动项来源: {}", source)),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (name, source);
        Err("仅支持 Windows 平台".to_string())
    }
}

/// 启用启动项 (从 Run- 恢复到 Run)
#[tauri::command]
pub fn enable_startup_item(name: String, source: String) -> Result<bool, String> {
    #[cfg(windows)]
    {
        match source.as_str() {
            "注册表(系统)" => {
                move_registry_value(
                    winreg::enums::HKEY_LOCAL_MACHINE,
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run-",
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                    &name,
                )
            }
            "注册表(用户)" => {
                move_registry_value(
                    winreg::enums::HKEY_CURRENT_USER,
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run-",
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run",
                    &name,
                )
            }
            "启动文件夹(用户)" | "启动文件夹(系统)" => {
                let appdata = std::env::var("APPDATA").unwrap_or_default();
                let program_data = std::env::var("PROGRAMDATA").unwrap_or_default();
                let base_path = if source.contains("用户") {
                    format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup", appdata)
                } else {
                    format!("{}\\Microsoft\\Windows\\Start Menu\\Programs\\Startup", program_data)
                };

                for ext in &[".lnk", ".bat", ".cmd", ".vbs"] {
                    let disabled_path = format!("{}\\{}{}.disabled", base_path, name, ext);
                    let file_path = format!("{}\\{}{}", base_path, name, ext);
                    if std::path::Path::new(&disabled_path).exists() {
                        std::fs::rename(&disabled_path, &file_path)
                            .map_err(|e| format!("启用失败: {}", e))?;
                        return Ok(true);
                    }
                }
                Err("未找到已禁用的启动文件".to_string())
            }
            "计划任务" => {
                let output = std::process::Command::new("schtasks")
                    .args(["/Change", "/TN", &name, "/ENABLE"])
                    .output()
                    .map_err(|e| format!("执行失败: {}", e))?;
                if output.status.success() {
                    Ok(true)
                } else {
                    Err(format!("启用失败: {}", String::from_utf8_lossy(&output.stderr)))
                }
            }
            _ => Err(format!("未知启动项来源: {}", source)),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (name, source);
        Err("仅支持 Windows 平台".to_string())
    }
}

/// 获取上次开机时长 (通过事件日志 100-109 启动事件)
#[tauri::command]
pub fn get_boot_duration() -> BootDuration {
    #[cfg(windows)]
    {
        // 使用 PowerShell 查询最近一次开机启动时间
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; (Get-CimInstance -ClassName Win32_OperatingSystem).LastBootUpTime.ToString('yyyy-MM-dd HH:mm:ss')",
            ])
            .output();

        let boot_time = match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => "未知".to_string(),
        };

        // 获取开机时长: 从启动到用户登录的时间
        // 使用事件 ID 100 (Kernel-Boot) 获取启动性能信息
        let boot_duration_output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-WinEvent -FilterHashtable @{LogName='Microsoft-Windows-Diagnostics-Performance/Operational'; Id=100} -MaxEvents 1 | ForEach-Object { $_.Properties[1].Value }",
            ])
            .output();

        let boot_secs = match boot_duration_output {
            Ok(o) => {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                s.parse::<u64>().unwrap_or(0)
            }
            Err(_) => 0,
        };

        let display = if boot_secs > 0 {
            let mins = boot_secs / 60;
            let secs = boot_secs % 60;
            format!("{}分{}秒", mins, secs)
        } else {
            "未知".to_string()
        };

        BootDuration {
            last_boot_time: boot_time,
            boot_duration_seconds: boot_secs,
            boot_duration_display: display,
        }
    }
    #[cfg(not(windows))]
    {
        BootDuration {
            last_boot_time: "未知".to_string(),
            boot_duration_seconds: 0,
            boot_duration_display: "未知".to_string(),
        }
    }
}

// ========== 内部辅助函数 ==========

#[cfg(windows)]
fn read_registry_run_items(
    hive: isize,
    path: &str,
    source_name: &str,
) -> Vec<StartupItem> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hive_key = RegKey::predef(hive);
    let mut items = Vec::new();

    let key = match hive_key.open_subkey_with_flags(path, KEY_READ) {
        Ok(k) => k,
        Err(_) => return items,
    };

    // 同时检查 Run- 下被禁用的项目
    let disabled_path = path.replace("\\Run", "\\Run-");
    let disabled_key = hive_key.open_subkey_with_flags(&disabled_path, KEY_READ).ok();

    for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
        let command = value.to_string();
        let enabled = true;

        items.push(StartupItem {
            name: name.clone(),
            command,
            location: format!("{}\\{}", path, name),
            enabled,
            source: source_name.to_string(),
        });
    }

    // 添加被禁用的项
    if let Some(dk) = disabled_key {
        for (name, value) in dk.enum_values().filter_map(|r| r.ok()) {
            let command = value.to_string();
            items.push(StartupItem {
                name: name.clone(),
                command,
                location: format!("{}\\{}", disabled_path, name),
                enabled: false,
                source: source_name.to_string(),
            });
        }
    }

    items
}

#[cfg(windows)]
fn read_startup_folder_items(path: &str, source_name: &str) -> Vec<StartupItem> {
    let mut items = Vec::new();
    let path_buf = std::path::PathBuf::from(path);

    if !path_buf.exists() {
        return items;
    }

    if let Ok(entries) = std::fs::read_dir(&path_buf) {
        for entry in entries.filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();

            // 检查是否被禁用 (.disabled 后缀)
            let (display_name, enabled) = if file_name.ends_with(".disabled") {
                (file_name.trim_end_matches(".disabled").to_string(), false)
            } else {
                (file_name, true)
            };

            // 去掉扩展名作为显示名
            let display_name = display_name
                .trim_end_matches(".lnk")
                .trim_end_matches(".bat")
                .trim_end_matches(".cmd")
                .trim_end_matches(".vbs")
                .to_string();

            items.push(StartupItem {
                name: display_name,
                command: file_path.to_string_lossy().to_string(),
                location: file_path.to_string_lossy().to_string(),
                enabled,
                source: source_name.to_string(),
            });
        }
    }

    items
}

#[cfg(windows)]
fn read_scheduled_tasks() -> Vec<StartupItem> {
    let mut items = Vec::new();

    let output = std::process::Command::new("schtasks")
        .args(["/Query", "/FO", "CSV", "/V"])
        .output();

    if let Ok(output) = output {
        let csv = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = csv.lines().collect();

        for line in lines.iter().skip(1) {
            // 解析 CSV 行
            let fields: Vec<String> = parse_csv_line(line);
            if fields.len() < 8 {
                continue;
            }

            let task_name = fields[0].trim_matches('"').to_string();
            let status = fields[3].trim_matches('"').to_string();

            // 只显示正在运行或已准备就绪的任务
            if status == "就绪" || status == "Ready" || status == "正在运行" || status == "Running" {
                // 过滤掉系统任务
                if task_name.starts_with("\\Microsoft\\") || task_name.starts_with("\\Google\\") {
                    continue;
                }

                items.push(StartupItem {
                    name: task_name,
                    command: fields[7].trim_matches('"').to_string(),
                    location: "计划任务".to_string(),
                    enabled: status == "就绪" || status == "Ready" || status == "正在运行" || status == "Running",
                    source: "计划任务".to_string(),
                });
            }
        }
    }

    items
}

#[cfg(windows)]
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }
    fields.push(current);
    fields
}

#[cfg(windows)]
fn move_registry_value(
    hive: isize,
    from_path: &str,
    to_path: &str,
    name: &str,
) -> Result<bool, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hive_key = RegKey::predef(hive);

    // 读取源值
    let from_key = hive_key
        .open_subkey_with_flags(from_path, KEY_READ)
        .map_err(|e| format!("读取注册表失败: {}", e))?;

    let value: String = from_key
        .get_value(name)
        .map_err(|e| format!("读取值失败: {}", e))?;

    // 写入目标键
    let to_key = hive_key
        .create_subkey_with_flags(to_path, KEY_WRITE)
        .map_err(|e| format!("创建注册表键失败: {}", e))?
        .0;

    to_key
        .set_value(name, &value)
        .map_err(|e| format!("写入注册表失败: {}", e))?;

    // 从源键删除
    let from_key_write = hive_key
        .open_subkey_with_flags(from_path, KEY_WRITE)
        .map_err(|e| format!("打开注册表失败: {}", e))?;

    from_key_write
        .delete_value(name)
        .map_err(|e| format!("删除注册表值失败: {}", e))?;

    Ok(true)
}
