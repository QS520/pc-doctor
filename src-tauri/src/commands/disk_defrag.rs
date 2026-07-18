use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone, Debug)]
pub struct DriveDefragInfo {
    pub drive: String,
    pub drive_type: String,
    pub is_ssd: bool,
    pub fragmentation_percent: f64,
    pub total_fragmented_files: u64,
    pub total_fragments: u64,
    pub last_analysis: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct DefragResult {
    pub success: bool,
    pub output: String,
    pub error: String,
    pub duration_secs: u64,
}

/// 分析磁盘碎片情况 —— 异步执行避免阻塞 UI
#[tauri::command]
pub async fn analyze_defrag() -> Vec<DriveDefragInfo> {
    tokio::task::spawn_blocking(|| {
    let mut drives = Vec::new();

    #[cfg(windows)]
    {
        // 使用 defrag 命令分析所有固定磁盘
        let output = Command::new("defrag")
            .args(["C:", "/A", "/V"])
            .output();

        if let Ok(output) = output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            let (stderr, _, _) = encoding_rs::UTF_8.decode(&output.stderr);
            let combined = format!("{}\n{}", stdout, stderr);

            let mut current_drive = String::new();
            let mut is_ssd = false;
            let mut frag_percent = 0.0;
            let mut frag_files = 0u64;
            let mut total_frags = 0u64;

            for line in combined.lines() {
                let line = line.trim();

                // 检测 SSD
                if line.contains("SSD") || line.contains("固态硬盘") {
                    is_ssd = true;
                }

                // 检测盘符
                if line.contains(":\\") && (line.contains("卷") || line.contains("Volume")) {
                    if !current_drive.is_empty() {
                        drives.push(DriveDefragInfo {
                            drive: current_drive.clone(),
                            drive_type: if is_ssd { "SSD".to_string() } else { "HDD".to_string() },
                            is_ssd,
                            fragmentation_percent: frag_percent,
                            total_fragmented_files: frag_files,
                            total_fragments: total_frags,
                            last_analysis: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                        });
                    }
                    if let Some(colon_pos) = line.find(':') {
                        current_drive = line[..colon_pos].chars().last().unwrap_or('C').to_string();
                    }
                    is_ssd = false;
                    frag_percent = 0.0;
                    frag_files = 0;
                    total_frags = 0;
                }

                // 解析碎片率
                if line.contains("碎片") && line.contains("%") {
                    if let Some(start) = line.find(|c: char| c.is_ascii_digit()) {
                        if let Some(end) = line[start..].find('%') {
                            if let Ok(p) = line[start..start + end].parse::<f64>() {
                                frag_percent = p;
                            }
                        }
                    }
                }

                // 解析碎片文件数
                if line.contains("碎片文件") || line.contains("Fragmented files") {
                    extract_number(line).iter().for_each(|n| frag_files = *n);
                }

                // 解析总碎片数
                if line.contains("总碎片") || line.contains("Total fragments") {
                    extract_number(line).iter().for_each(|n| total_frags = *n);
                }
            }

            if !current_drive.is_empty() {
                drives.push(DriveDefragInfo {
                    drive: current_drive.clone(),
                    drive_type: if is_ssd { "SSD".to_string() } else { "HDD".to_string() },
                    is_ssd,
                    fragmentation_percent: frag_percent,
                    total_fragmented_files: frag_files,
                    total_fragments: total_frags,
                    last_analysis: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
                });
            }
        }

        // 如果解析失败，至少返回 C 盘
        if drives.is_empty() {
            // 通过 PowerShell 判断是否为 SSD
            let ssd_check = Command::new("powershell")
                .args(["-NoProfile", "-Command",
                    "Get-PhysicalDisk | Where-Object { $_.MediaType -eq 'SSD' } | Select-Object -First 1 | ForEach-Object { 'SSD' }"])
                .output();

            let is_ssd = if let Ok(out) = ssd_check {
                String::from_utf8_lossy(&out.stdout).trim() == "SSD"
            } else {
                false
            };

            drives.push(DriveDefragInfo {
                drive: "C".to_string(),
                drive_type: if is_ssd { "SSD".to_string() } else { "HDD".to_string() },
                is_ssd,
                fragmentation_percent: 0.0,
                total_fragmented_files: 0,
                total_fragments: 0,
                last_analysis: "需要管理员权限分析".to_string(),
            });
        }
    }

    drives
    })
    .await
    .unwrap_or_default()
}

/// 执行磁盘碎片整理 (HDD) 或 TRIM 优化 (SSD) —— 异步执行避免阻塞 UI
#[tauri::command]
pub async fn run_defrag(drive: String, optimize_ssd: bool) -> DefragResult {
    tokio::task::spawn_blocking(move || {
    let start = std::time::Instant::now();

    let drive_arg = format!("{}:", drive);

    let args = if optimize_ssd {
        vec![&drive_arg[..], "/O", "/V"]
    } else {
        vec![&drive_arg[..], "/D", "/V"]
    };

    let output = Command::new("defrag")
        .args(&args)
        .output();

    let duration = start.elapsed().as_secs();

    match output {
        Ok(output) => {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            let (stderr, _, _) = encoding_rs::UTF_8.decode(&output.stderr);

            DefragResult {
                success: output.status.success(),
                output: stdout.to_string(),
                error: stderr.to_string(),
                duration_secs: duration,
            }
        }
        Err(e) => DefragResult {
            success: false,
            output: String::new(),
            error: format!("启动碎片整理失败: {}。请以管理员身份运行。", e),
            duration_secs: duration,
        },
    }
    })
    .await
    .unwrap_or_default()
}

/// 对所有磁盘执行 TRIM (SSD 优化) —— 异步执行避免阻塞 UI
#[tauri::command]
pub async fn run_trim_all() -> DefragResult {
    tokio::task::spawn_blocking(|| {
    let start = std::time::Instant::now();

    let output = Command::new("defrag")
        .args(["/C", "/O", "/V"])
        .output();

    let duration = start.elapsed().as_secs();

    match output {
        Ok(output) => {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            let (stderr, _, _) = encoding_rs::UTF_8.decode(&output.stderr);

            DefragResult {
                success: output.status.success(),
                output: stdout.to_string(),
                error: stderr.to_string(),
                duration_secs: duration,
            }
        }
        Err(e) => DefragResult {
            success: false,
            output: String::new(),
            error: format!("启动 TRIM 失败: {}", e),
            duration_secs: duration,
        },
    }
    })
    .await
    .unwrap_or_default()
}

fn extract_number(s: &str) -> Vec<u64> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(n) = current.parse::<u64>() {
                numbers.push(n);
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        if let Ok(n) = current.parse::<u64>() {
            numbers.push(n);
        }
    }
    numbers
}
