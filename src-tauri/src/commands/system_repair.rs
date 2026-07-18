use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone, Debug, Default)]
pub struct RepairResult {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub duration_secs: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiskHealthInfo {
    pub drive: String,
    pub model: String,
    pub status: String,
    pub smart_ok: bool,
    pub temperature: f32,
    pub total_size_gb: f64,
    pub bad_sectors: u32,
    pub power_on_hours: u32,
    pub details: Vec<(String, String)>,
}

/// 运行 SFC (系统文件检查器)
/// sfc /scannow - 扫描所有受保护的系统文件，并用正确的版本替换损坏的文件
#[tauri::command]
pub async fn run_sfc() -> RepairResult {
    tokio::task::spawn_blocking(|| {
    let start = std::time::Instant::now();

    let output = Command::new("sfc")
        .args(["/scannow"])
        .output();

    let duration = start.elapsed().as_secs();

    match output {
        Ok(output) => {
            // sfc 的输出是 GBK 编码，需要用 encoding_rs 解码
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            let (stderr, _, _) = encoding_rs::UTF_8.decode(&output.stderr);

            RepairResult {
                success: output.status.success(),
                output: stdout.to_string(),
                error: stderr.to_string(),
                duration_secs: duration,
            }
        }
        Err(e) => RepairResult {
            success: false,
            output: String::new(),
            error: format!("启动 SFC 失败: {}。请以管理员身份运行本程序。", e),
            duration_secs: duration,
        },
    }
    })
    .await
    .unwrap_or_default()
}

/// 运行 DISM (部署映像服务和管理)
/// DISM /Online /Cleanup-Image /RestoreHealth - 修复 Windows 组件存储
#[tauri::command]
pub async fn run_dism() -> RepairResult {
    tokio::task::spawn_blocking(|| {
    let start = std::time::Instant::now();

    let output = Command::new("DISM")
        .args(["/Online", "/Cleanup-Image", "/RestoreHealth"])
        .output();

    let duration = start.elapsed().as_secs();

    match output {
        Ok(output) => {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            let (stderr, _, _) = encoding_rs::UTF_8.decode(&output.stderr);

            RepairResult {
                success: output.status.success(),
                output: stdout.to_string(),
                error: stderr.to_string(),
                duration_secs: duration,
            }
        }
        Err(e) => RepairResult {
            success: false,
            output: String::new(),
            error: format!("启动 DISM 失败: {}。请以管理员身份运行本程序。", e),
            duration_secs: duration,
        },
    }
    })
    .await
    .unwrap_or_default()
}

/// 运行 CHKDSK (磁盘检查)
/// chkdsk C: /scan - 在线扫描磁盘错误 (不需要重启)
#[tauri::command]
pub async fn run_chkdsk(drive: Option<String>) -> RepairResult {
    tokio::task::spawn_blocking(move || {
    let drive_letter = drive.unwrap_or_else(|| "C:".to_string());
    let start = std::time::Instant::now();

    let output = Command::new("chkdsk")
        .args([&drive_letter, "/scan"])
        .output();

    let duration = start.elapsed().as_secs();

    match output {
        Ok(output) => {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            let (stderr, _, _) = encoding_rs::UTF_8.decode(&output.stderr);

            RepairResult {
                success: output.status.success(),
                output: stdout.to_string(),
                error: stderr.to_string(),
                duration_secs: duration,
            }
        }
        Err(e) => RepairResult {
            success: false,
            output: String::new(),
            error: format!("启动 CHKDSK 失败: {}。请以管理员身份运行本程序。", e),
            duration_secs: duration,
        },
    }
    })
    .await
    .unwrap_or_default()
}

/// 检查磁盘健康状态 (通过 WMI/S.M.A.R.T) —— 异步执行避免阻塞 UI
#[tauri::command]
pub async fn check_disk_health() -> Vec<DiskHealthInfo> {
    tokio::task::spawn_blocking(|| {
    let mut disks = Vec::new();

    #[cfg(windows)]
    {
        // 使用 PowerShell 查询磁盘 S.M.A.R.T 状态
        let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Get-PhysicalDisk | ForEach-Object {
    $disk = $_
    $details = @()
    $details += "型号: $($disk.FriendlyName)"
    $details += "介质类型: $($disk.MediaType)"
    $details += "总线类型: $($disk.BusType)"
    $details += "大小: $([math]::Round($disk.Size / 1GB, 2)) GB"
    $details += "健康状态: $($disk.HealthStatus)"
    $details += "运行状态: $($disk.OperationalStatus)"
    $details += "设备ID: $($disk.DeviceId)"
    $details += "序号: $($disk.SerialNumber)"

    $healthStr = $disk.HealthStatus.ToString()
    $smartOk = ($healthStr -eq 'Healthy')
    $temp = 0
    try {
        $tempInfo = Get-StorageReliabilityCounter -PhysicalDisk $disk -ErrorAction SilentlyContinue
        if ($tempInfo -and $tempInfo.Temperature -gt 0) {
            $temp = $tempInfo.Temperature
        }
    } catch {}

    $line = "DRIVE|$($disk.DeviceId)|$($disk.FriendlyName)|$healthStr|$smartOk|$temp|$([math]::Round($disk.Size / 1GB, 2))"
    $line += "|DETAILS:" + ($details -join ';')
    $line += "|END"
    Write-Output $line
}
"#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_command.trim()])
            .output();

        if let Ok(output) = output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);

            for line in stdout.lines() {
                let line = line.trim();
                if !line.starts_with("DRIVE|") {
                    continue;
                }

                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() < 8 {
                    continue;
                }

                let mut details = Vec::new();
                if let Some(details_str) = parts.get(7) {
                    let details_str = details_str.trim_start_matches("DETAILS:");
                    for d in details_str.split(';') {
                        if !d.is_empty() {
                            if let Some(idx) = d.find(':') {
                                details.push((
                                    d[..idx].trim().to_string(),
                                    d[idx + 1..].trim().to_string(),
                                ));
                            }
                        }
                    }
                }

                disks.push(DiskHealthInfo {
                    drive: parts.get(1).unwrap_or(&"?").to_string(),
                    model: parts.get(2).unwrap_or(&"Unknown").to_string(),
                    status: parts.get(3).unwrap_or(&"Unknown").to_string(),
                    smart_ok: parts.get(4).unwrap_or(&"false").parse().unwrap_or(false),
                    temperature: parts.get(5).unwrap_or(&"0").parse().unwrap_or(0.0),
                    total_size_gb: parts.get(6).unwrap_or(&"0").parse().unwrap_or(0.0),
                    bad_sectors: 0,
                    power_on_hours: 0,
                    details,
                });
            }
        }

        // 如果 WMI 查询失败，尝试使用 wmic 命令
        if disks.is_empty() {
            let wmic_output = Command::new("wmic")
                .args(["diskdrive", "get", "Model,Status,Size,InterfaceType"])
                .output();

            if let Ok(output) = wmic_output {
                let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
                let lines: Vec<&str> = stdout.lines().collect();

                for line in lines.iter().skip(1) {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 3 {
                        let model = fields[0..fields.len() - 2].join(" ");
                        let status = fields[fields.len() - 2];
                        let size_str = fields[fields.len() - 1];
                        let size_gb = size_str.parse::<u64>().unwrap_or(0) as f64 / (1024.0 * 1024.0 * 1024.0);

                        disks.push(DiskHealthInfo {
                            drive: "0".to_string(),
                            model,
                            status: status.to_string(),
                            smart_ok: status.eq_ignore_ascii_case("ok"),
                            temperature: 0.0,
                            total_size_gb: (size_gb * 100.0).round() / 100.0,
                            bad_sectors: 0,
                            power_on_hours: 0,
                            details: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    disks
    })
    .await
    .unwrap_or_default()
}
