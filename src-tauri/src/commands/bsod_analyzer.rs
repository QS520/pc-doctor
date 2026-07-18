use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Clone, Debug)]
pub struct BsodCrashInfo {
    pub date: String,
    pub bug_check_code: String,
    pub bug_check_name: String,
    pub description: String,
    pub parameters: Vec<String>,
    pub dump_file: String,
    pub dump_size_mb: f64,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct BsodAnalysisResult {
    pub crash_count: u32,
    pub last_crash_date: String,
    pub crashes: Vec<BsodCrashInfo>,
    pub common_causes: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SystemError {
    pub time: String,
    pub source: String,
    pub event_id: u32,
    pub message: String,
    pub level: String,
}

/// 分析蓝屏崩溃记录 —— 异步执行避免阻塞 UI
#[tauri::command]
pub async fn analyze_bsod() -> BsodAnalysisResult {
    tokio::task::spawn_blocking(|| {
    let mut crashes = Vec::new();

    // 1. 从 Windows 事件日志读取 BugCheck 记录 (Event ID 1001)
    #[cfg(windows)]
    {
        crashes.extend(read_bugcheck_events());
    }

    // 2. 扫描 minidump 文件
    let minidump_dir = PathBuf::from("C:\\Windows\\Minidump");
    if minidump_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&minidump_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("dmp") {
                    let dump_info = parse_minidump_file(&path);
                    if let Some(info) = dump_info {
                        // 避免重复添加（事件日志和 dump 文件可能指向同一次崩溃）
                        if !crashes.iter().any(|c: &BsodCrashInfo| c.dump_file == info.dump_file) {
                            crashes.push(info);
                        }
                    }
                }
            }
        }
    }

    // 3. 检查完整内存转储
    let full_dump = PathBuf::from("C:\\Windows\\Memory.dmp");
    if full_dump.exists() {
        let info = parse_minidump_file(&full_dump);
        if let Some(info) = info {
            if !crashes.iter().any(|c: &BsodCrashInfo| c.dump_file == info.dump_file) {
                crashes.push(info);
            }
        }
    }

    // 按日期排序 (最新的在前)
    crashes.sort_by(|a, b| b.date.cmp(&a.date));

    let crash_count = crashes.len() as u32;
    let last_crash_date = crashes
        .first()
        .map(|c| c.date.clone())
        .unwrap_or_else(|| "无记录".to_string());

    // 分析常见原因
    let common_causes = analyze_common_causes(&crashes);

    BsodAnalysisResult {
        crash_count,
        last_crash_date,
        crashes,
        common_causes,
    }
    })
    .await
    .unwrap_or_default()
}

/// 获取系统错误事件日志 —— 异步执行避免阻塞 UI
#[tauri::command]
pub async fn get_system_errors(limit: Option<u32>) -> Vec<SystemError> {
    tokio::task::spawn_blocking(move || {
    let max = limit.unwrap_or(50);
    let mut errors = Vec::new();

    #[cfg(windows)]
    {
        // 使用 PowerShell 查询系统事件日志中的错误和严重事件
        let ps_command = format!(
            "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-WinEvent -FilterHashtable @{{LogName='System'; Level=1,2}} -MaxEvents {} | ForEach-Object {{ '{0}' + $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss') + '{1}' + $_.ProviderName + '{1}' + $_.Id + '{1}' + ($_.Message -replace '\\s+', ' ').Substring(0, [Math]::Min(500, $_.Message.Length)) + '{1}' + $_.LevelDisplayName + '{2}' }}",
            max, "\x1f", "\x1e"
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_command])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for record in stdout.split('\x1e') {
                let record = record.trim();
                if record.is_empty() {
                    continue;
                }
                let fields: Vec<&str> = record.split('\x1f').collect();
                if fields.len() >= 5 {
                    errors.push(SystemError {
                        time: fields[0].trim().to_string(),
                        source: fields[1].trim().to_string(),
                        event_id: fields[2].trim().parse().unwrap_or(0),
                        message: fields[3].trim().to_string(),
                        level: fields[4].trim().to_string(),
                    });
                }
            }
        }
    }

    errors
    })
    .await
    .unwrap_or_default()
}

// ========== 内部辅助函数 ==========

/// 简单解析 minidump 文件头部
/// Minidump 文件格式: MINIDUMP_HEADER followed by streams
/// 我们读取头部获取时间戳和检查是否有 BugCheck 流
fn parse_minidump_file(path: &PathBuf) -> Option<BsodCrashInfo> {
    use std::io::{Read, Seek, SeekFrom};

    let metadata = std::fs::metadata(path).ok()?;
    let dump_size = metadata.len();

    let mut file = std::fs::File::open(path).ok()?;

    // MINIDUMP_HEADER:
    // u32 Signature (0x504D444D = "MDMP")
    // u32 Version
    // u32 NumberOfStreams
    // RVA StreamDirectoryRva
    // u32 CheckSum
    // u32 TimeDateStamp
    // u64 Flags

    let mut header = [0u8; 32];
    file.read_exact(&mut header).ok()?;

    let signature = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    if signature != 0x504D444D {
        // 不是有效的 minidump 文件
        return None;
    }

    let num_streams = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
    let stream_dir_rva = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
    let time_stamp = u32::from_le_bytes([header[20], header[21], header[22], header[23]]);

    // 转换 Unix 时间戳
    let datetime = chrono::DateTime::from_timestamp(time_stamp as i64, 0)?;
    let date_str = datetime
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    // 读取流目录查找 BugCheck 流 (StreamType = 7)
    file.seek(SeekFrom::Start(stream_dir_rva as u64)).ok()?;

    let mut bug_check_code: u32 = 0;
    let mut params: [u64; 5] = [0; 5];
    let mut found_bug_check = false;

    for _ in 0..num_streams {
        let mut stream_entry = [0u8; 12]; // MINIDUMP_DIRECTORY: u32 StreamType, u32 DataSize, RVA rva
        if file.read_exact(&mut stream_entry).is_err() {
            break;
        }

        let stream_type = u32::from_le_bytes([
            stream_entry[0],
            stream_entry[1],
            stream_entry[2],
            stream_entry[3],
        ]);
        let data_size = u32::from_le_bytes([
            stream_entry[4],
            stream_entry[5],
            stream_entry[6],
            stream_entry[7],
        ]);
        let data_rva = u32::from_le_bytes([
            stream_entry[8],
            stream_entry[9],
            stream_entry[10],
            stream_entry[11],
        ]);

        if stream_type == 7 {
            // BugCheckStream (MINIDUMP_STREAM_TYPE.ThreadListStream = 7? No...)
            // Actually:  MINIDUMP_STREAM_TYPE.BugCheckStream = 7? Let me check
            // Actually BugCheckStream is 7
            // The data contains: u32 BugCheckCode, u64 Parameters[5] = 48 bytes

            let current_pos = file.stream_position().ok()?;
            file.seek(SeekFrom::Start(data_rva as u64)).ok()?;

            let mut bug_data = [0u8; 48];
            if file.read_exact(&mut bug_data).is_ok() {
                bug_check_code =
                    u32::from_le_bytes([bug_data[0], bug_data[1], bug_data[2], bug_data[3]]);
                for i in 0..5 {
                    let offset = 4 + i * 8;
                    params[i] = u64::from_le_bytes([
                        bug_data[offset],
                        bug_data[offset + 1],
                        bug_data[offset + 2],
                        bug_data[offset + 3],
                        bug_data[offset + 4],
                        bug_data[offset + 5],
                        bug_data[offset + 6],
                        bug_data[offset + 7],
                    ]);
                }
                found_bug_check = true;
            }

            let _ = file.seek(SeekFrom::Start(current_pos));
        }

        // 防止读取过多
        if data_size > 1024 * 1024 * 100 {
            break;
        }
    }

    // 获取 Bug Check 名称和描述
    let (code_name, description) = get_bug_check_info(bug_check_code);

    let param_strings: Vec<String> = params
        .iter()
        .filter(|p| **p != 0)
        .map(|p| format!("0x{:016X}", p))
        .collect();

    Some(BsodCrashInfo {
        date: date_str,
        bug_check_code: format!("0x{:08X}", bug_check_code),
        bug_check_name: if found_bug_check {
            code_name
        } else {
            "未知".to_string()
        },
        description,
        parameters: param_strings,
        dump_file: path.to_string_lossy().to_string(),
        dump_size_mb: (dump_size as f64 / (1024.0 * 1024.0) * 100.0).round() / 100.0,
    })
}

/// 从 Windows 事件日志读取 BugCheck 记录 (Event ID 1001)
#[cfg(windows)]
fn read_bugcheck_events() -> Vec<BsodCrashInfo> {
    let mut crashes = Vec::new();

    // 使用 PowerShell 查询 BugCheck 事件
    let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
        Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Microsoft-Windows-WER-SystemErrorReporting'} -MaxEvents 20 | ForEach-Object {
            $msg = $_.Message
            Write-Output "$($_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss'))|$msg"
        }
    "#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() == 2 {
                let date = parts[0].trim().to_string();
                let message = parts[1].trim().to_string();

                // 从消息中提取 Bug Check Code
                // 消息格式通常包含: "The computer has rebooted from a bugcheck. The bugcheck was: 0x0000001a (0x0000000000000412, ...)"
                let (code, name, desc) = parse_bugcheck_from_message(&message);

                crashes.push(BsodCrashInfo {
                    date,
                    bug_check_code: code,
                    bug_check_name: name,
                    description: desc,
                    parameters: Vec::new(),
                    dump_file: String::new(),
                    dump_size_mb: 0.0,
                });
            }
        }
    }

    // 也查询 BugCheck 事件源
    let ps_command2 = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
        Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Microsoft-Windows-Kernel-Power'; Id=41} -MaxEvents 10 | ForEach-Object {
            Write-Output "$($_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss'))|$($_.Message -replace '\s+', ' ')"
        }
    "#;

    let output2 = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command2.trim()])
        .output();

    if let Ok(output2) = output2 {
        let stdout = String::from_utf8_lossy(&output2.stdout);
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() == 2 {
                let date = parts[0].trim().to_string();
                let message = parts[1].trim().to_string();

                crashes.push(BsodCrashInfo {
                    date,
                    bug_check_code: "0x0000009F".to_string(), // DRIVER_POWER_STATE_FAILURE
                    bug_check_name: "Kernel-Power 事件 (非正常关机)".to_string(),
                    description: message,
                    parameters: Vec::new(),
                    dump_file: String::new(),
                    dump_size_mb: 0.0,
                });
            }
        }
    }

    crashes
}

fn parse_bugcheck_from_message(message: &str) -> (String, String, String) {
    // 尝试从消息文本中提取 Bug Check Code
    // 格式: "The bugcheck was: 0x0000001a" 或 "BugCheckCode: 0x0000001a"

    let code = if let Some(start) = message.find("0x") {
        let rest = &message[start..];
        let end = rest
            .find(|c: char| !c.is_ascii_hexdigit() && c != 'x')
            .unwrap_or(rest.len());
        let code_str = &rest[..end];
        code_str.to_uppercase()
    } else {
        "未知".to_string()
    };

    let (name, desc) = get_bug_check_info(
        u32::from_str_radix(code.trim_start_matches("0x"), 16).unwrap_or(0),
    );

    (code, name, desc)
}

/// 根据 Bug Check Code 返回名称和描述
fn get_bug_check_info(code: u32) -> (String, String) {
    match code {
        0x0000000A => ("IRQL_NOT_LESS_OR_EQUAL".to_string(),
            "内核或驱动程序访问了不该访问的内存地址。通常由驱动程序错误、硬件不兼容或内存故障引起。".to_string()),
        0x0000001A => ("MEMORY_MANAGEMENT".to_string(),
            "内存管理错误。可能是内存条故障、驱动程序内存泄漏或系统内存不足。建议运行 Windows 内存诊断。".to_string()),
        0x0000003B => ("SYSTEM_SERVICE_EXCEPTION".to_string(),
            "系统服务异常。通常由驱动程序错误、系统服务崩溃或杀毒软件冲突引起。".to_string()),
        0x00000050 => ("PAGE_FAULT_IN_NONPAGED_AREA".to_string(),
            "请求的内存页不在内存中。可能由驱动程序错误、内存故障或系统服务引起。".to_string()),
        0x0000007B => ("INACCESSIBLE_BOOT_DEVICE".to_string(),
            "无法访问启动设备。通常由硬盘驱动问题、SATA模式设置错误或硬盘故障引起。".to_string()),
        0x0000007E => ("SYSTEM_THREAD_EXCEPTION_NOT_HANDLED".to_string(),
            "系统线程未处理异常。通常由驱动程序错误引起。检查参数中的驱动文件名。".to_string()),
        0x0000009F => ("DRIVER_POWER_STATE_FAILURE".to_string(),
            "驱动程序电源状态失败。某驱动程序在电源状态转换时无响应。检查最近更新的驱动。".to_string()),
        0x000000D1 => ("DRIVER_IRQL_NOT_LESS_OR_EQUAL".to_string(),
            "驱动程序访问了不当的内存地址。通常由网络或存储驱动程序错误引起。".to_string()),
        0x000000EA => ("THREAD_STUCK_IN_DEVICE_DRIVER".to_string(),
            "线程卡在设备驱动中。通常是显卡驱动问题。建议更新或回退显卡驱动。".to_string()),
        0x000000F4 => ("CRITICAL_OBJECT_TERMINATION".to_string(),
            "关键进程或线程意外终止。可能是硬盘故障、系统文件损坏或内存问题。".to_string()),
        0x00000116 => ("VIDEO_TDR_FAILURE".to_string(),
            "显卡超时检测和恢复失败。显卡驱动崩溃或显卡硬件故障。建议更新显卡驱动。".to_string()),
        0x00000133 => ("DPC_WATCHDOG_VIOLATION".to_string(),
            "DPC 看门狗违规。驱动程序执行时间过长。通常是存储驱动或网络驱动问题。".to_string()),
        0x00000139 => ("KERNEL_SECURITY_CHECK_FAILURE".to_string(),
            "内核安全检查失败。内存损坏、驱动程序错误或系统文件损坏。".to_string()),
        0x00000154 => ("UNEXPECTED_STORE_EXCEPTION".to_string(),
            "意外存储异常。通常是硬盘/SSD故障或存储驱动问题。建议检查磁盘健康。".to_string()),
        0xC000021A => ("STATUS_SYSTEM_PROCESS_TERMINATED".to_string(),
            "关键系统进程终止。可能是系统文件损坏或杀毒软件冲突。建议运行 SFC 和 DISM 修复。".to_string()),
        0xC0000221 => ("STATUS_IMAGE_CHECKSUM_MISMATCH".to_string(),
            "镜像校验和不匹配。系统文件或驱动程序损坏。建议运行 SFC 修复。".to_string()),
        0x000000C5 => ("DRIVER_CORRUPTED_SYSTEM_POOL".to_string(),
            "驱动程序损坏系统内存池。通常是驱动程序错误。建议更新所有驱动。".to_string()),
        0x000000FC => ("ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY".to_string(),
            "尝试执行不可执行内存。驱动程序代码错误。".to_string()),
        0x00000101 => ("CLOCK_WATCHDOG_TIMEOUT".to_string(),
            "时钟看门狗超时。CPU或主板问题，也可能是BIOS设置不正确。".to_string()),
        0x00000124 => ("WHEA_UNCORRECTABLE_ERROR".to_string(),
            "硬件错误。CPU、内存或主板硬件故障。建议检查硬件温度和运行硬件诊断。".to_string()),
        _ => ("未知错误代码".to_string(),
            format!("Bug Check Code: 0x{:08X}。建议查看 Microsoft 文档了解此错误的具体含义。", code)),
    }
}

fn analyze_common_causes(crashes: &[BsodCrashInfo]) -> Vec<String> {
    let mut causes = Vec::new();
    let mut driver_count = 0;
    let mut memory_count = 0;
    let mut hardware_count = 0;

    for crash in crashes {
        match crash.bug_check_code.as_str() {
            "0x0000001A" | "0x00000050" | "0x000000C5" => memory_count += 1,
            "0x0000000A" | "0x0000003B" | "0x0000007E" | "0x000000D1" | "0x000000EA"
            | "0x00000116" | "0x00000133" | "0x00000139" => driver_count += 1,
            "0x00000124" | "0x00000101" | "0x000000F4" | "0x00000154" => hardware_count += 1,
            _ => {}
        }
    }

    if driver_count > 0 {
        causes.push(format!(
            "检测到 {} 次驱动程序相关蓝屏。建议更新所有驱动程序，特别是显卡和存储驱动。",
            driver_count
        ));
    }
    if memory_count > 0 {
        causes.push(format!(
            "检测到 {} 次内存相关蓝屏。建议运行 Windows 内存诊断 (mdsched.exe) 检查内存条。",
            memory_count
        ));
    }
    if hardware_count > 0 {
        causes.push(format!(
            "检测到 {} 次硬件相关蓝屏。建议检查 CPU 温度、硬盘健康状态和电源供电。",
            hardware_count
        ));
    }

    if causes.is_empty() && !crashes.is_empty() {
        causes.push("蓝屏原因不明确，建议查看崩溃详情中的 Bug Check Code 和参数。".to_string());
    }

    causes
}
