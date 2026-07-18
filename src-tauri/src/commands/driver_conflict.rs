use serde::Serialize;
use std::process::Command;

/// 驱动冲突项
#[derive(Serialize, Clone, Debug)]
pub struct DriverConflict {
    pub device_name: String,
    pub device_id: String,
    pub problem_code: u32,
    pub problem_description: String,
    pub probable_cause: String,
    pub fix_suggestion: String,
    pub driver_name: String,
    pub driver_version: String,
    pub driver_date: String,
    pub is_signed: bool,
    pub conflict_type: String,   // problem_device / unsigned / duplicate / version_mismatch
}

/// 驱动版本冲突
#[derive(Serialize, Clone, Debug)]
pub struct DriverVersionConflict {
    pub driver_name: String,
    pub devices: Vec<String>,
    pub versions: Vec<String>,
    pub description: String,
}

/// 驱动加载失败记录
#[derive(Serialize, Clone, Debug)]
pub struct DriverLoadFailure {
    pub timestamp: String,
    pub driver_name: String,
    pub failure_reason: String,
    pub event_id: u32,
}

/// 驱动诊断完整报告
#[derive(Serialize, Clone, Debug)]
pub struct DriverConflictReport {
    pub overall_status: String,   // healthy / warnings / critical
    pub conflicts: Vec<DriverConflict>,
    pub version_conflicts: Vec<DriverVersionConflict>,
    pub load_failures: Vec<DriverLoadFailure>,
    pub unsigned_drivers: Vec<DriverConflict>,
    pub recommendations: Vec<String>,
    pub summary: String,
}

/// 深度驱动冲突诊断
/// 定位具体哪个驱动冲突、原因、修复方法
#[tauri::command]
#[allow(unused_assignments)]
pub fn diagnose_driver_conflicts() -> DriverConflictReport {
    let mut conflicts: Vec<DriverConflict> = Vec::new();
    let mut version_conflicts: Vec<DriverVersionConflict> = Vec::new();
    let mut load_failures: Vec<DriverLoadFailure> = Vec::new();
    let mut unsigned: Vec<DriverConflict> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    #[cfg(windows)]
    {
        // 1. 查询有问题的设备（驱动冲突）
        let problem_devices = query_problem_driver_devices();
        for pd in problem_devices {
            // 尝试找到对应的驱动信息
            let driver_info = query_driver_for_device(&pd.device_id);
            let conflict_type = match pd.problem_code {
                1 | 28 => "missing_driver",
                3 | 10 | 38 | 41 => "driver_corrupt",
                12 => "resource_conflict",
                31 => "version_mismatch",
                43 => "hardware_failure",
                52 => "unsigned_driver",
                _ => "problem_device",
            };
            let conflict = DriverConflict {
                device_name: pd.device_name.clone(),
                device_id: pd.device_id.clone(),
                problem_code: pd.problem_code,
                problem_description: pd.problem_description.clone(),
                probable_cause: pd.probable_cause.clone(),
                fix_suggestion: pd.fix_suggestion.clone(),
                driver_name: driver_info.name,
                driver_version: driver_info.version,
                driver_date: driver_info.date,
                is_signed: driver_info.is_signed,
                conflict_type: conflict_type.to_string(),
            };

            // 未签名驱动单独归类
            if conflict_type == "unsigned_driver" || !conflict.is_signed {
                unsigned.push(conflict.clone());
            }
            conflicts.push(conflict);
        }

        // 2. 检测同一驱动的版本冲突（多个设备加载同一驱动但版本不同）
        let all_drivers = query_all_drivers();
        version_conflicts = detect_version_conflicts(&all_drivers);

        for vc in &version_conflicts {
            recommendations.push(format!(
                "驱动 \"{}\" 存在 {} 个不同版本（{}），建议统一为最新版本",
                vc.driver_name,
                vc.versions.len(),
                vc.versions.join(", ")
            ));
        }

        // 3. 查询驱动加载失败事件（最近 7 天）
        load_failures = query_driver_load_failures();
        if !load_failures.is_empty() {
            recommendations.push(format!(
                "检测到 {} 次驱动加载失败，建议更新相关驱动",
                load_failures.len()
            ));
        }

        // 4. 检查未签名驱动
        let unsigned_drivers_list = query_unsigned_drivers();
        for ud in unsigned_drivers_list {
            unsigned.push(ud);
        }

        // 5. 生成建议
        if !conflicts.is_empty() {
            let corrupt_count = conflicts.iter().filter(|c| c.conflict_type == "driver_corrupt").count();
            let missing_count = conflicts.iter().filter(|c| c.conflict_type == "missing_driver").count();
            let hw_fail_count = conflicts.iter().filter(|c| c.conflict_type == "hardware_failure").count();

            if corrupt_count > 0 {
                recommendations.push(format!(
                    "检测到 {} 个驱动可能已损坏，建议卸载后从制造商官网重新下载安装",
                    corrupt_count
                ));
            }
            if missing_count > 0 {
                recommendations.push(format!(
                    "检测到 {} 个设备缺少驱动，建议从设备制造商官网下载对应驱动",
                    missing_count
                ));
            }
            if hw_fail_count > 0 {
                recommendations.push(format!(
                    "检测到 {} 个设备报告硬件故障（错误码 43），设备可能已损坏",
                    hw_fail_count
                ));
            }
        }

        // 6. 检查显示驱动（GPU）冲突 - 最常见的问题
        let gpu_issues = check_gpu_driver_issues();
        for issue in gpu_issues {
            recommendations.push(issue);
        }
    }

    // 总体状态
    let has_critical = conflicts.iter().any(|c| {
        c.problem_code == 10 || c.problem_code == 43 || c.problem_code == 12
    });
    let overall_status = if conflicts.is_empty() && version_conflicts.is_empty() && load_failures.is_empty() {
        "healthy".to_string()
    } else if has_critical {
        "critical".to_string()
    } else {
        "warnings".to_string()
    };

    let summary = if conflicts.is_empty() && version_conflicts.is_empty() {
        "未检测到驱动冲突，所有设备工作正常。".to_string()
    } else {
        let mut parts = Vec::new();
        if !conflicts.is_empty() {
            parts.push(format!("{} 个设备问题", conflicts.len()));
        }
        if !version_conflicts.is_empty() {
            parts.push(format!("{} 个版本冲突", version_conflicts.len()));
        }
        if !load_failures.is_empty() {
            parts.push(format!("{} 次加载失败", load_failures.len()));
        }
        if !unsigned.is_empty() {
            parts.push(format!("{} 个未签名驱动", unsigned.len()));
        }
        format!("检测到: {}", parts.join("，"))
    };

    DriverConflictReport {
        overall_status,
        conflicts,
        version_conflicts,
        load_failures,
        unsigned_drivers: unsigned,
        recommendations,
        summary,
    }
}

// ========== 内部辅助结构 ==========

#[cfg(windows)]
struct ProblemDeviceRaw {
    device_name: String,
    device_id: String,
    problem_code: u32,
    problem_description: String,
    probable_cause: String,
    fix_suggestion: String,
}

#[cfg(windows)]
struct DriverInfoRaw {
    name: String,
    version: String,
    date: String,
    is_signed: bool,
}

#[cfg(windows)]
#[allow(dead_code)]
struct DriverRecord {
    device_name: String,
    driver_name: String,
    driver_version: String,
    driver_date: String,
    device_id: String,
    is_signed: bool,
}

// ========== 内部实现函数 ==========

#[cfg(windows)]
fn query_problem_driver_devices() -> Vec<ProblemDeviceRaw> {
    let ps_command = r#"
Get-WmiObject Win32_PnPEntity | Where-Object { $_.ConfigManagerErrorCode -ne 0 } | ForEach-Object {
    $name = $_.Name
    if (!$name) { $name = $_.DeviceID }
    $deviceId = $_.DeviceID
    $code = $_.ConfigManagerErrorCode
    Write-Output "$name|$deviceId|$code"
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    let mut devices = Vec::new();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                let code: u32 = parts[2].parse().unwrap_or(0);
                let (desc, cause, fix) = explain_problem_code(code);
                devices.push(ProblemDeviceRaw {
                    device_name: parts[0].to_string(),
                    device_id: parts[1].to_string(),
                    problem_code: code,
                    problem_description: desc,
                    probable_cause: cause,
                    fix_suggestion: fix,
                });
            }
        }
    }
    devices
}

#[cfg(windows)]
fn explain_problem_code(code: u32) -> (String, String, String) {
    match code {
        1 => (
            "设备未正确配置".to_string(),
            "驱动程序缺失或损坏".to_string(),
            "更新或重新安装该设备的驱动程序".to_string(),
        ),
        3 => (
            "驱动程序可能已损坏".to_string(),
            "驱动程序文件损坏或版本不兼容".to_string(),
            "卸载后重新安装驱动，或回滚到之前版本".to_string(),
        ),
        10 => (
            "设备无法启动".to_string(),
            "设备硬件故障或驱动加载失败".to_string(),
            "检查设备连接，更新驱动；若仍失败可能硬件损坏".to_string(),
        ),
        12 => (
            "没有足够的可用资源".to_string(),
            "资源冲突（IRQ/DMA）或资源不足，多个设备争用同一资源".to_string(),
            "在设备管理器中查看资源冲突，禁用冲突设备或更换插槽/端口".to_string(),
        ),
        18 => (
            "需要重新安装驱动程序".to_string(),
            "驱动程序配置不完整".to_string(),
            "重新安装驱动程序".to_string(),
        ),
        19 => (
            "注册表可能已损坏".to_string(),
            "注册表中的设备配置信息损坏".to_string(),
            "卸载设备并重启让系统重新识别".to_string(),
        ),
        22 => (
            "设备已被禁用".to_string(),
            "用户或系统禁用了该设备".to_string(),
            "在设备管理器中启用该设备".to_string(),
        ),
        28 => (
            "未安装驱动程序".to_string(),
            "设备没有匹配的驱动".to_string(),
            "安装制造商提供的最新驱动程序".to_string(),
        ),
        31 => (
            "驱动程序不兼容当前 Windows 版本".to_string(),
            "驱动版本与 Windows 版本不匹配".to_string(),
            "安装适用于当前 Windows 版本的驱动".to_string(),
        ),
        38 => (
            "驱动程序无法加载（上次启动失败）".to_string(),
            "驱动文件损坏或依赖缺失".to_string(),
            "重启电脑；若仍失败则重新安装驱动".to_string(),
        ),
        41 => (
            "驱动加载失败".to_string(),
            "驱动程序加载过程中失败".to_string(),
            "更新或重新安装驱动程序".to_string(),
        ),
        42 => (
            "设备驱动重复加载".to_string(),
            "系统中存在重复的设备实例".to_string(),
            "在设备管理器中卸载重复设备".to_string(),
        ),
        43 => (
            "设备已停止（Windows 收到故障报告）".to_string(),
            "设备硬件可能已损坏".to_string(),
            "检查设备连接；若仍失败，硬件可能已损坏需更换".to_string(),
        ),
        45 => (
            "设备未就绪（休眠中）".to_string(),
            "设备正从节能状态恢复".to_string(),
            "重启电脑或重新插拔设备".to_string(),
        ),
        46 => (
            "设备不可用".to_string(),
            "设备正在被其他程序占用或已断开".to_string(),
            "关闭占用该设备的程序，重新连接设备".to_string(),
        ),
        48 => (
            "设备驱动已被禁用（未知原因）".to_string(),
            "驱动程序被系统强制禁用".to_string(),
            "更新驱动程序或联系制造商".to_string(),
        ),
        52 => (
            "无法验证数字签名".to_string(),
            "驱动程序未经签名或签名无效".to_string(),
            "安装已签名的驱动程序".to_string(),
        ),
        _ => (
            format!("设备管理器错误码: {}", code),
            "未知错误类型".to_string(),
            "建议搜索该错误码或联系技术支持".to_string(),
        ),
    }
}

#[cfg(windows)]
fn query_driver_for_device(device_id: &str) -> DriverInfoRaw {
    let escaped_id = device_id.replace('\'', "''").replace('\\', "\\\\");
    let ps_command = format!(
        r#"
Get-CimInstance Win32_PnPSignedDriver -Filter "DeviceID='{}'" -ErrorAction SilentlyContinue | Select-Object -First 1 | ForEach-Object {{
    $signed = if ($_.DriverSignatureProvider -ne $null -and $_.DriverSignatureProvider -ne '') {{ 'true' }} else {{ 'false' }}
    $date = ''
    if ($_.DriverDate) {{ $date = $_.DriverDate.ToString('yyyy-MM-dd') }}
    Write-Output "$($_.DriverName)|$($_.DriverVersion)|$date|$signed"
}}
"#,
        escaped_id
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_command])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        let line = stdout.trim();
        if !line.is_empty() {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 4 {
                return DriverInfoRaw {
                    name: parts[0].to_string(),
                    version: parts[1].to_string(),
                    date: parts[2].to_string(),
                    is_signed: parts[3] == "true",
                };
            }
        }
    }
    DriverInfoRaw {
        name: "未知".to_string(),
        version: "未知".to_string(),
        date: "未知".to_string(),
        is_signed: false,
    }
}

#[cfg(windows)]
fn query_all_drivers() -> Vec<DriverRecord> {
    let ps_command = r#"
Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceName -and $_.DriverName } | ForEach-Object {
    $date = ''
    if ($_.DriverDate) { $date = $_.DriverDate.ToString('yyyy-MM-dd') }
    $signed = if ($_.DriverSignatureProvider -ne $null -and $_.DriverSignatureProvider -ne '') { 'true' } else { 'false' }
    $devName = ($_.DeviceName -replace '[\|]', ' ').Trim()
    $drvName = ($_.DriverName -replace '[\|]', ' ').Trim()
    Write-Output "$devName|$drvName|$($_.DriverVersion)|$date|$($_.DeviceID -replace '[\|]', ' ')|$signed"
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    let mut drivers = Vec::new();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(6, '|').collect();
            if parts.len() >= 6 {
                drivers.push(DriverRecord {
                    device_name: parts[0].to_string(),
                    driver_name: parts[1].to_string(),
                    driver_version: parts[2].to_string(),
                    driver_date: parts[3].to_string(),
                    device_id: parts[4].to_string(),
                    is_signed: parts[5] == "true",
                });
            }
        }
    }
    drivers
}

#[cfg(windows)]
fn detect_version_conflicts(drivers: &[DriverRecord]) -> Vec<DriverVersionConflict> {
    use std::collections::HashMap;
    let mut driver_versions: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    for d in drivers {
        let versions = driver_versions
            .entry(d.driver_name.clone())
            .or_insert_with(HashMap::new);
        let devices = versions
            .entry(d.driver_version.clone())
            .or_insert_with(Vec::new);
        if !devices.contains(&d.device_name) {
            devices.push(d.device_name.clone());
        }
    }

    let mut conflicts = Vec::new();
    for (driver_name, versions) in driver_versions {
        if versions.len() > 1 {
            let mut all_versions: Vec<String> = versions.keys().cloned().collect();
            all_versions.sort();
            let mut all_devices: Vec<String> = Vec::new();
            for (_, devs) in &versions {
                for d in devs {
                    if !all_devices.contains(d) {
                        all_devices.push(d.clone());
                    }
                }
            }
            conflicts.push(DriverVersionConflict {
                driver_name: driver_name.clone(),
                devices: all_devices,
                versions: all_versions.clone(),
                description: format!(
                    "驱动 {} 在不同设备上存在 {} 个版本: {}",
                    driver_name,
                    all_versions.len(),
                    all_versions.join(", ")
                ),
            });
        }
    }
    conflicts.sort_by(|a, b| b.versions.len().cmp(&a.versions.len()));
    conflicts
}

#[cfg(windows)]
fn query_driver_load_failures() -> Vec<DriverLoadFailure> {
    // 查询驱动加载失败事件（Service Control Manager 错误）
    let ps_command = r#"
try {
    $events = Get-WinEvent -FilterHashtable @{
        LogName='System'
        ProviderName='Service Control Manager'
        Level=2
        StartTime=(Get-Date).AddDays(-7)
    } -MaxEvents 200 -ErrorAction SilentlyContinue
    if ($events) {
        $drvFailures = $events | Where-Object { $_.Message -match 'driver|驱动' }
        $drvFailures | Select-Object -First 20 | ForEach-Object {
            $msg = ($_.Message -replace '[\r\n]+', ' ').Trim()
            $time = $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss')
            Write-Output "$time|$($_.Id)|$msg"
        }
    }
} catch {}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    let mut failures = Vec::new();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                let driver_name = extract_driver_name_from_message(parts[2]);
                failures.push(DriverLoadFailure {
                    timestamp: parts[0].to_string(),
                    driver_name,
                    failure_reason: parts[2].to_string(),
                    event_id: parts[1].parse().unwrap_or(0),
                });
            }
        }
    }
    failures
}

#[cfg(windows)]
fn extract_driver_name_from_message(msg: &str) -> String {
    // 尝试从消息中提取驱动名称
    if let Some(start) = msg.find('"') {
        if let Some(end) = msg[start + 1..].find('"') {
            return msg[start + 1..start + 1 + end].to_string();
        }
    }
    // 尝试匹配 .sys 文件名
    let words: Vec<&str> = msg.split_whitespace().collect();
    for w in words {
        if w.to_lowercase().ends_with(".sys") {
            return w.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_').to_string();
        }
    }
    "未知驱动".to_string()
}

#[cfg(windows)]
fn query_unsigned_drivers() -> Vec<DriverConflict> {
    let ps_command = r#"
Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DriverSignatureProvider -eq $null -or $_.DriverSignatureProvider -eq '' } | Select-Object -First 20 | ForEach-Object {
    $devName = $_.DeviceName
    if (!$devName) { $devName = $_.DeviceID }
    $drvName = if ($_.DriverName) { $_.DriverName } else { 'Unknown' }
    $drvVer = if ($_.DriverVersion) { $_.DriverVersion } else { 'Unknown' }
    $date = if ($_.DriverDate) { $_.DriverDate.ToString('yyyy-MM-dd') } else { 'Unknown' }
    Write-Output "$devName|$($_.DeviceID)|$drvName|$drvVer|$date"
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    let mut drivers = Vec::new();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(5, '|').collect();
            if parts.len() >= 5 {
                drivers.push(DriverConflict {
                    device_name: parts[0].to_string(),
                    device_id: parts[1].to_string(),
                    problem_code: 52,
                    problem_description: "驱动程序未经数字签名".to_string(),
                    probable_cause: "驱动来自未知来源或已损坏".to_string(),
                    fix_suggestion: "从设备制造商官网下载已签名的驱动程序".to_string(),
                    driver_name: parts[2].to_string(),
                    driver_version: parts[3].to_string(),
                    driver_date: parts[4].to_string(),
                    is_signed: false,
                    conflict_type: "unsigned".to_string(),
                });
            }
        }
    }
    drivers
}

#[cfg(windows)]
fn check_gpu_driver_issues() -> Vec<String> {
    let mut issues = Vec::new();
    let ps_command = r#"
$gpuDrivers = Get-CimInstance Win32_PnPSignedDriver | Where-Object { $_.DeviceClass -eq 'DISPLAY' -or $_.DeviceName -match 'NVIDIA|AMD|Radeon|GeForce|Intel.*Graphics' }
foreach ($drv in $gpuDrivers) {
    $ageDays = 0
    if ($drv.DriverDate) {
        $ageDays = ((Get-Date) - $drv.DriverDate).Days
    }
    $devName = $drv.DeviceName
    $drvVer = $drv.DriverVersion
    $drvDate = if ($drv.DriverDate) { $drv.DriverDate.ToString('yyyy-MM-dd') } else { 'Unknown' }
    Write-Output "$devName|$drvVer|$drvDate|$ageDays"
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 4 {
                let gpu_name = parts[0];
                let version = parts[1];
                let date = parts[2];
                let age_days: i64 = parts[3].parse().unwrap_or(0);
                if age_days > 730 {
                    issues.push(format!(
                        "显卡驱动 \"{}\"（版本 {}，日期 {}）已超过 2 年未更新，建议更新到最新版本以避免兼容性问题",
                        gpu_name, version, date
                    ));
                } else if version.is_empty() || version == "Unknown" {
                    issues.push(format!("显卡 \"{}\" 的驱动信息异常，建议重新安装驱动", gpu_name));
                }
            }
        }
    }
    issues
}
