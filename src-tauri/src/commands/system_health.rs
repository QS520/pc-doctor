use serde::Serialize;
use std::process::Command;

/// 系统损坏类型
#[derive(Serialize, Clone, Debug, Default)]
pub struct SystemIssue {
    pub severity: String,           // critical / warning / info
    pub category: String,            // system_files / registry / boot / updates / activation / integrity
    pub title: String,
    pub description: String,
    pub affected_component: String,  // 具体影响的组件
    pub recommendation: String,
}

/// 损坏的系统文件
#[derive(Serialize, Clone, Debug, Default)]
pub struct CorruptedFile {
    pub file_path: String,
    pub file_name: String,
    pub issue: String,
    pub can_repair: bool,
}

/// 注册表问题
#[derive(Serialize, Clone, Debug, Default)]
pub struct RegistryIssue {
    pub hive: String,                // HKLM / HKCU / HKCR
    pub key_path: String,
    pub issue: String,
    pub severity: String,
}

/// 启动配置项
#[derive(Serialize, Clone, Debug, Default)]
pub struct BootConfig {
    pub last_boot_status: String,    // normal / abnormal / unknown
    pub boot_errors: Vec<String>,
    pub boot_time_seconds: u64,
    pub safe_mode: bool,
    pub description: String,
}

/// Windows 更新失败记录
#[derive(Serialize, Clone, Debug, Default)]
pub struct FailedUpdate {
    pub kb_number: String,
    pub title: String,
    pub error_code: String,
    pub error_description: String,
    pub timestamp: String,
}

/// Windows 激活状态
#[derive(Serialize, Clone, Debug, Default)]
pub struct ActivationStatus {
    pub is_activated: bool,
    pub license_status: String,
    pub description: String,
}

/// 完整系统损坏报告
#[derive(Serialize, Clone, Debug, Default)]
pub struct SystemHealthReport {
    pub overall_status: String,
    pub issues: Vec<SystemIssue>,
    pub corrupted_files: Vec<CorruptedFile>,
    pub registry_issues: Vec<RegistryIssue>,
    pub boot_config: BootConfig,
    pub failed_updates: Vec<FailedUpdate>,
    pub activation: ActivationStatus,
    pub recommendations: Vec<String>,
    pub summary: String,
}

/// 深度系统损坏分析
/// 定位具体损坏的方面：系统文件、注册表、启动配置、Windows 更新、激活
#[tauri::command]
pub async fn diagnose_system_health() -> SystemHealthReport {
    tokio::task::spawn_blocking(move || {
    let mut issues: Vec<SystemIssue> = Vec::new();
    let mut corrupted_files: Vec<CorruptedFile> = Vec::new();
    let mut registry_issues: Vec<RegistryIssue> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    #[cfg(windows)]
    {
        // 1. 系统文件完整性检查（通过 SFC 的快速验证）
        let sfc_issues = check_sfc_quick();
        for issue in &sfc_issues.corrupted {
            corrupted_files.push(issue.clone());
        }
        if !sfc_issues.corrupted.is_empty() {
            issues.push(SystemIssue {
                severity: "critical".to_string(),
                category: "system_files".to_string(),
                title: format!("检测到 {} 个系统文件损坏", sfc_issues.corrupted.len()),
                description: format!(
                    "以下系统文件损坏或被篡改:\n{}",
                    sfc_issues.corrupted.iter().map(|c| c.file_path.clone()).collect::<Vec<_>>().join("\n")
                ),
                affected_component: "Windows 系统文件".to_string(),
                recommendation: "运行 SFC /scannow 或 DISM /Online /Cleanup-Image /RestoreHealth 修复".to_string(),
            });
            recommendations.push("运行系统修复中的 SFC 扫描修复系统文件".to_string());
        }

        // 2. 关键注册表检查
        let reg_issues = check_critical_registry();
        for ri in &reg_issues {
            registry_issues.push(ri.clone());
        }
        if !reg_issues.is_empty() {
            let critical_reg = reg_issues.iter().filter(|r| r.severity == "critical").count();
            issues.push(SystemIssue {
                severity: if critical_reg > 0 { "critical" } else { "warning" }.to_string(),
                category: "registry".to_string(),
                title: format!("检测到 {} 个注册表问题", reg_issues.len()),
                description: format!(
                    "关键注册表项存在异常:\n{}",
                    reg_issues.iter().map(|r| format!("{}\\{}", r.hive, r.key_path)).collect::<Vec<_>>().join("\n")
                ),
                affected_component: "Windows 注册表".to_string(),
                recommendation: "建议使用系统还原恢复或重新安装相关组件".to_string(),
            });
        }

        // 3. 启动配置检查
        let boot = check_boot_config();
        if boot.last_boot_status == "abnormal" || !boot.boot_errors.is_empty() {
            issues.push(SystemIssue {
                severity: if !boot.boot_errors.is_empty() { "warning" } else { "info" }.to_string(),
                category: "boot".to_string(),
                title: "上次启动异常".to_string(),
                description: boot.description.clone(),
                affected_component: "启动管理器".to_string(),
                recommendation: "若频繁出现，建议运行系统修复中的 CHKDSK 检查磁盘".to_string(),
            });
        }

        // 4. Windows 更新失败记录
        let failed_updates = check_failed_updates();
        if !failed_updates.is_empty() {
            issues.push(SystemIssue {
                severity: "warning".to_string(),
                category: "updates".to_string(),
                title: format!("检测到 {} 个更新安装失败", failed_updates.len()),
                description: format!(
                    "以下 Windows 更新安装失败，可能导致系统组件缺失:\n{}",
                    failed_updates.iter().take(5).map(|u| format!("{} ({})", u.title, u.kb_number)).collect::<Vec<_>>().join("\n")
                ),
                affected_component: "Windows 更新".to_string(),
                recommendation: "在 Windows 更新中重试，或运行 DISM 修复后再更新".to_string(),
            });
            recommendations.push("重试失败的 Windows 更新以确保系统完整".to_string());
        }

        // 5. Windows 激活状态
        let activation = check_activation();
        if !activation.is_activated {
            issues.push(SystemIssue {
                severity: "warning".to_string(),
                category: "activation".to_string(),
                title: "Windows 未激活".to_string(),
                description: activation.description.clone(),
                affected_component: "Windows 许可证".to_string(),
                recommendation: "激活 Windows 以确保系统功能正常".to_string(),
            });
        }

        // 6. 关键系统文件存在性检查
        let missing_files = check_critical_files();
        for mf in missing_files {
            issues.push(SystemIssue {
                severity: "critical".to_string(),
                category: "system_files".to_string(),
                title: format!("关键系统文件缺失: {}", mf.file_name),
                description: format!("文件 {} 不存在或无法访问", mf.file_path),
                affected_component: "Windows 系统文件".to_string(),
                recommendation: "运行 SFC /scannow 修复，或从相同版本的 Windows 复制该文件".to_string(),
            });
            corrupted_files.push(mf);
        }

        // 7. 检查系统还原点是否存在
        let restore_points = check_restore_points();
        if restore_points == 0 {
            issues.push(SystemIssue {
                severity: "info".to_string(),
                category: "integrity".to_string(),
                title: "未检测到系统还原点".to_string(),
                description: "系统没有可用的还原点，出现问题时无法快速恢复".to_string(),
                affected_component: "系统保护".to_string(),
                recommendation: "建议启用系统保护并创建还原点".to_string(),
            });
            recommendations.push("启用系统保护功能，创建系统还原点".to_string());
        }

        // 8. 检查事件日志中的严重错误（最近 24 小时）
        let recent_criticals = count_recent_critical_events();
        if recent_criticals > 0 {
            issues.push(SystemIssue {
                severity: "warning".to_string(),
                category: "integrity".to_string(),
                title: format!("最近 24 小时 {} 个严重系统错误", recent_criticals),
                description: format!("系统事件日志记录了 {} 个严重错误事件", recent_criticals),
                affected_component: "系统稳定性".to_string(),
                recommendation: "查看系统事件日志了解具体错误详情".to_string(),
            });
        }
    }

    let boot_config = check_boot_config();
    let activation = check_activation();
    let failed_updates = check_failed_updates();

    // 计算总体状态
    let has_critical = issues.iter().any(|i| i.severity == "critical");
    let overall_status = if issues.is_empty() {
        "healthy".to_string()
    } else if has_critical {
        "critical".to_string()
    } else {
        "warnings".to_string()
    };

    let summary = if issues.is_empty() {
        "系统完整性检查通过，未发现损坏或异常。".to_string()
    } else {
        let critical = issues.iter().filter(|i| i.severity == "critical").count();
        let warning = issues.iter().filter(|i| i.severity == "warning").count();
        let info = issues.iter().filter(|i| i.severity == "info").count();
        format!(
            "检测到 {} 项问题：{} 项严重，{} 项警告，{} 项提示",
            issues.len(), critical, warning, info
        )
    };

    if recommendations.is_empty() && !issues.is_empty() {
        recommendations.push("建议定期运行系统修复 (SFC + DISM) 维护系统完整性".to_string());
    }

    SystemHealthReport {
        overall_status,
        issues,
        corrupted_files,
        registry_issues,
        boot_config,
        failed_updates,
        activation,
        recommendations,
        summary,
    }
    })
    .await
    .unwrap_or_default()
}

// ========== 内部辅助结构 ==========

#[cfg(windows)]
struct SfcCheckResult {
    corrupted: Vec<CorruptedFile>,
}

// ========== 内部实现函数 ==========

#[cfg(windows)]
fn check_sfc_quick() -> SfcCheckResult {
    // 注意：SFC 完整扫描需要 10-30 分钟且需要管理员权限
    // 这里我们检查 CBS.log 中最近的损坏记录，作为快速检测
    let mut corrupted = Vec::new();

    let cbs_log = std::path::Path::new("C:\\Windows\\Logs\\CBS\\CBS.log");
    if cbs_log.exists() {
        // 读取 CBS.log 中最近的损坏记录
        let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$logFile = "C:\Windows\Logs\CBS\CBS.log"
if (Test-Path $logFile) {
    $lines = Get-Content $logFile -Tail 2000 -ErrorAction SilentlyContinue
    $corruptLines = $lines | Where-Object { $_ -match 'Cannot repair|corrupt|repair failed|not fix' -and $_ -match 'SR' }
    $corruptLines | Select-Object -Last 10 | ForEach-Object {
        if ($_ -match '`(.+?)`') {
            $filePath = $matches[1]
            Write-Output $filePath
        } elseif ($_ -match '(\w:\\[^\s]+\.\w+)') {
            Write-Output $matches[1]
        }
    }
}
"#;
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_command.trim()])
            .output();
        if let Ok(output) = output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let file_name = std::path::Path::new(line)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| line.to_string());
                corrupted.push(CorruptedFile {
                    file_path: line.to_string(),
                    file_name,
                    issue: "SFC 检测到损坏且无法自动修复".to_string(),
                    can_repair: true,
                });
            }
        }
    }

    SfcCheckResult { corrupted }
}

#[cfg(windows)]
fn check_critical_registry() -> Vec<RegistryIssue> {
    let mut issues = Vec::new();

    // 检查 Winlogon 关键键
    let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
# 检查关键注册表项
$results = @()

# 1. Winlogon - userinit 和 shell
$userinit = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon' -ErrorAction SilentlyContinue).Userinit
if ($userinit -ne 'C:\Windows\system32\userinit.exe,') {
    $results += "HKLM|SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Userinit|值异常: '$userinit'|critical"
}

$shell = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon' -ErrorAction SilentlyContinue).Shell
if ($shell -ne 'explorer.exe' -and $shell -ne 'explorer.exe,') {
    $results += "HKLM|SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Shell|值异常: '$shell'|critical"
}

# 2. 检查 BootExecute
$bootExec = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager' -ErrorAction SilentlyContinue).BootExecute
if ($bootExec -and $bootExec -ne 'autocheck autochk *') {
    $results += "HKLM|SYSTEM\CurrentControlSet\Control\Session Manager\BootExecute|值异常: '$bootExec'|warning"
}

# 3. 检查 Image File Execution Options（可能被恶意篡改）
$ifeoPath = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options'
if (Test-Path $ifeoPath) {
    Get-ChildItem $ifeoPath -ErrorAction SilentlyContinue | ForEach-Object {
        $dbg = (Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue).Debugger
        if ($dbg) {
            $results += "HKLM|SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\$($_.PSChildName)|Debugger 值存在: '$dbg' - 可能阻止程序运行|critical"
        }
    }
}

# 4. 检查 Run 键是否为空或被清空（重要项缺失）
$runKey = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue)
if (-not $runKey) {
    $results += "HKLM|SOFTWARE\Microsoft\Windows\CurrentVersion\Run|注册表项缺失或无法访问|warning"
}

# 5. 检查 Safe Mode 最小驱动
$safeBoot = Get-ChildItem 'HKLM:\SYSTEM\CurrentControlSet\Control\SafeBoot\Minimal' -ErrorAction SilentlyContinue
if (-not $safeBoot) {
    $results += "HKLM|SYSTEM\CurrentControlSet\Control\SafeBoot\Minimal|安全模式配置缺失|critical"
}

# 输出结果
$results | ForEach-Object { Write-Output $_ }
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 4 {
                issues.push(RegistryIssue {
                    hive: parts[0].to_string(),
                    key_path: parts[1].to_string(),
                    issue: parts[2].to_string(),
                    severity: parts[3].to_string(),
                });
            }
        }
    }

    issues
}

#[cfg(windows)]
fn check_boot_config() -> BootConfig {
    let mut boot_errors = Vec::new();
    let description;
    let mut safe_mode = false;
    let mut boot_time: u64 = 0;

    // 检查是否在安全模式
    let safe_output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; if (Test-Path 'HKLM:\SYSTEM\CurrentControlSet\Control\SafeBoot\Option') { (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\SafeBoot\Option' -ErrorAction SilentlyContinue).OptionValue } else { 'normal' }"#,
        ])
        .output();
    if let Ok(output) = safe_output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        let val = stdout.trim();
        if !val.is_empty() && val != "normal" && val != "0" {
            safe_mode = true;
        }
    }

    // 检查上次启动是否正常
    let boot_output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; (Get-CimInstance Win32_OperatingSystem).LastBootUpTime.ToString('yyyy-MM-dd HH:mm:ss')",
        ])
        .output();
    let last_boot_time = if let Ok(output) = boot_output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        stdout.trim().to_string()
    } else {
        "未知".to_string()
    };

    // 检查启动事件中的错误
    let err_output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Microsoft-Windows-Kernel-Boot','Microsoft-Windows-Kernel-General'; Level=1,2; StartTime=(Get-Date).AddDays(-7)} -MaxEvents 10 -ErrorAction SilentlyContinue | ForEach-Object { Write-Output "$($_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss'))|$($_.Id)|$(($_.Message -replace '[\r\n]+', ' ').Trim())" }"#,
        ])
        .output();
    if let Ok(output) = err_output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() >= 3 {
                    let event_id: u32 = parts[1].parse().unwrap_or(0);
                    // Event ID 41 = 系统意外重启
                    if event_id == 41 {
                        boot_errors.push(format!("系统意外重启 ({})", parts[0]));
                    } else if event_id == 6008 {
                        boot_errors.push(format!("上次关机异常 ({})", parts[0]));
                    }
                }
            }
        }
    }

    // 获取启动耗时
    let dur_output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-WinEvent -FilterHashtable @{LogName='Microsoft-Windows-Diagnostics-Performance/Operational'; Id=100} -MaxEvents 1 -ErrorAction SilentlyContinue | ForEach-Object { [math]::Round($_.Properties[1].Value) }"#,
        ])
        .output();
    if let Ok(output) = dur_output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        boot_time = stdout.trim().parse().unwrap_or(0);
    }

    let last_boot_status = if boot_errors.is_empty() {
        "normal".to_string()
    } else {
        "abnormal".to_string()
    };

    if boot_errors.is_empty() {
        description = format!("系统启动正常，上次启动于 {}", last_boot_time);
    } else {
        description = format!(
            "检测到启动异常：{}。上次启动于 {}",
            boot_errors.join("；"),
            last_boot_time
        );
    }

    BootConfig {
        last_boot_status,
        boot_errors,
        boot_time_seconds: boot_time,
        safe_mode,
        description,
    }
}

#[cfg(windows)]
fn check_failed_updates() -> Vec<FailedUpdate> {
    let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$session = New-Object -ComObject Microsoft.Update.Session
$searcher = $session.CreateUpdateSearcher()
$count = $searcher.GetTotalHistoryCount()
if ($count -gt 0) {
    $history = $searcher.QueryHistory(0, [math]::Min($count, 50))
    $failed = $history | Where-Object { $_.ResultCode -eq 4 -or $_.ResultCode -eq 5 }
    $failed | ForEach-Object {
        $kb = ''
        if ($_.Title -match 'KB(\d+)') { $kb = 'KB' + $matches[1] }
        $errCode = if ($_.HResult) { '0x{0:X8}' -f $_.HResult } else { '未知' }
        $title = ($_.Title -replace '[\|]', ' ').Trim()
        $time = if ($_.Date) { $_.Date.ToString('yyyy-MM-dd HH:mm') } else { '未知' }
        Write-Output "$kb|$title|$errCode|$time"
    }
}
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    let mut updates = Vec::new();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 4 {
                let (err_desc, _) = interpret_update_error(&parts[2]);
                updates.push(FailedUpdate {
                    kb_number: parts[0].to_string(),
                    title: parts[1].to_string(),
                    error_code: parts[2].to_string(),
                    error_description: err_desc,
                    timestamp: parts[3].to_string(),
                });
            }
        }
    }
    updates
}

#[cfg(windows)]
fn interpret_update_error(code: &str) -> (String, String) {
    if code.contains("800F0922") {
        ("系统保留分区空间不足".to_string(), "扩展系统保留分区或清理空间后重试".to_string())
    } else if code.contains("800F081F") {
        ("CBS 文件丢失或损坏".to_string(), "运行 DISM 修复后再重试更新".to_string())
    } else if code.contains("800F0902") {
        ("网络连接问题".to_string(), "检查网络连接后重试".to_string())
    } else if code.contains("8024004C") {
        ("更新被其他安装阻止".to_string(), "重启电脑后重试".to_string())
    } else if code.contains("8024200D") {
        ("更新下载不完整".to_string(), "清理 Windows\\SoftwareDistribution\\Download 后重试".to_string())
    } else {
        (format!("更新错误码 {}", code), "建议搜索该错误码或重试更新".to_string())
    }
}

#[cfg(windows)]
fn check_activation() -> ActivationStatus {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $slmgr = Get-CimInstance -ClassName SoftwareLicensingProduct -Filter "ApplicationId='55c92734-d682-4d71-983e-d6ec3f16059f' AND PartialProductKey != NULL" -ErrorAction SilentlyContinue
            if ($slmgr) {
                $licenseStatus = $slmgr.LicenseStatus
                $name = $slmgr.Name
                if ($licenseStatus -eq 1) {
                    Write-Output "activated|$name"
                } else {
                    Write-Output "not_activated|$name (status=$licenseStatus)"
                }
            } else {
                Write-Output "unknown|无法查询激活状态"
            }"#,
        ])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        let line = stdout.trim();
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() >= 2 {
            let status = parts[0];
            let info = parts[1];
            return match status {
                "activated" => ActivationStatus {
                    is_activated: true,
                    license_status: "已激活".to_string(),
                    description: format!("Windows 已激活 ({})", info),
                },
                "not_activated" => ActivationStatus {
                    is_activated: false,
                    license_status: "未激活".to_string(),
                    description: format!("Windows 未激活: {}", info),
                },
                _ => ActivationStatus {
                    is_activated: false,
                    license_status: "未知".to_string(),
                    description: info.to_string(),
                },
            };
        }
    }
    ActivationStatus {
        is_activated: false,
        license_status: "未知".to_string(),
        description: "无法查询激活状态".to_string(),
    }
}

#[cfg(windows)]
fn check_critical_files() -> Vec<CorruptedFile> {
    let mut missing = Vec::new();
    let system32 = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let critical_files = vec![
        "system32\\kernel32.dll",
        "system32\\ntdll.dll",
        "system32\\user32.dll",
        "system32\\gdi32.dll",
        "system32\\advapi32.dll",
        "system32\\rpcrt4.dll",
        "system32\\oleaut32.dll",
        "system32\\shell32.dll",
        "system32\\combase.dll",
        "system32\\wininet.dll",
        "system32\\ws2_32.dll",
        "system32\\drvstore.dll",
        "system32\\dism.exe",
        "system32\\sfc.exe",
        "explorer.exe",
        "regedit.exe",
        "cmd.exe",
        "notepad.exe",
    ];

    for f in critical_files {
        let full_path = format!("{}\\{}", system32, f);
        if !std::path::Path::new(&full_path).exists() {
            let file_name = std::path::Path::new(f)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| f.to_string());
            missing.push(CorruptedFile {
                file_path: full_path,
                file_name,
                issue: "关键系统文件缺失".to_string(),
                can_repair: true,
            });
        }
    }
    missing
}

#[cfg(windows)]
fn check_restore_points() -> u32 {
    let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
try {
    $sr = Get-CimInstance -Namespace 'root\default' -Class SystemRestore -ErrorAction SilentlyContinue
    if ($sr) {
        Write-Output $sr.Count
    } else {
        Write-Output 0
    }
} catch {
    Write-Output 0
}
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        stdout.trim().parse().unwrap_or(0)
    } else {
        0
    }
}

#[cfg(windows)]
fn count_recent_critical_events() -> u32 {
    let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
try {
    $events = Get-WinEvent -FilterHashtable @{
        LogName='System'
        Level=1
        StartTime=(Get-Date).AddHours(-24)
    } -ErrorAction SilentlyContinue
    if ($events) {
        Write-Output @($events).Count
    } else {
        Write-Output 0
    }
} catch {
    Write-Output 0
}
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        stdout.trim().parse().unwrap_or(0)
    } else {
        0
    }
}

// 非 Windows 平台的占位实现
#[cfg(not(windows))]
fn check_boot_config() -> BootConfig {
    BootConfig {
        last_boot_status: "unknown".to_string(),
        boot_errors: vec![],
        boot_time_seconds: 0,
        safe_mode: false,
        description: "仅支持 Windows".to_string(),
    }
}

#[cfg(not(windows))]
fn check_activation() -> ActivationStatus {
    ActivationStatus {
        is_activated: false,
        license_status: "未知".to_string(),
        description: "仅支持 Windows".to_string(),
    }
}

#[cfg(not(windows))]
fn check_failed_updates() -> Vec<FailedUpdate> {
    Vec::new()
}
