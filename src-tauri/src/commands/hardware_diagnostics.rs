use serde::Serialize;
use std::process::Command;

/// 诊断发现项
#[derive(Serialize, Clone, Debug)]
pub struct Finding {
    pub severity: String,       // critical / warning / info
    pub component: String,      // CPU / Memory / Disk / GPU / Battery / USB
    pub title: String,
    pub description: String,
    pub recommendation: String,
}

/// 硬件问题设备
#[derive(Serialize, Clone, Debug)]
pub struct ProblemDevice {
    pub name: String,
    pub device_id: String,
    pub problem_code: u32,
    pub problem_description: String,
    pub probable_cause: String,
    pub fix_suggestion: String,
}

/// WHEA 硬件错误记录
#[derive(Serialize, Clone, Debug)]
pub struct WheaError {
    pub time_created: String,
    pub error_source: String,   // CPU / Memory / PCIe / USB / Disk
    pub error_type: String,
    pub severity: String,
    pub description: String,
}

/// 磁盘 SMART 详细属性
#[derive(Serialize, Clone, Debug)]
pub struct SmartAttribute {
    pub drive: String,
    pub attribute_id: u32,
    pub attribute_name: String,
    pub raw_value: u64,
    pub threshold: u64,
    pub status: String,         // ok / warning / critical
    pub interpretation: String,
}

/// 电池健康信息
#[derive(Serialize, Clone, Debug)]
pub struct BatteryHealth {
    pub designed_capacity: u64,
    pub full_charge_capacity: u64,
    pub cycle_count: u32,
    pub health_percent: f32,
    pub status: String,         // good / fair / poor
    pub interpretation: String,
}

/// 完整的硬件诊断报告
#[derive(Serialize, Clone, Debug)]
pub struct HardwareDiagnostics {
    pub overall_status: String,    // healthy / warnings / critical
    pub findings: Vec<Finding>,
    pub problem_devices: Vec<ProblemDevice>,
    pub whea_errors: Vec<WheaError>,
    pub smart_attributes: Vec<SmartAttribute>,
    pub battery: Option<BatteryHealth>,
    pub memory_errors_detected: bool,
    pub summary: String,
}

/// 深度硬件故障诊断
/// 通过 WHEA 错误日志、设备管理器错误码、SMART 属性、电池健康综合判断
#[tauri::command]
pub fn diagnose_hardware() -> HardwareDiagnostics {
    let mut findings: Vec<Finding> = Vec::new();

    #[cfg(windows)]
    let mut problem_devices: Vec<ProblemDevice>;
    #[cfg(not(windows))]
    let problem_devices: Vec<ProblemDevice> = Vec::new();

    #[cfg(windows)]
    let mut whea_errors: Vec<WheaError>;
    #[cfg(not(windows))]
    let whea_errors: Vec<WheaError> = Vec::new();

    #[cfg(windows)]
    let mut smart_attrs: Vec<SmartAttribute>;
    #[cfg(not(windows))]
    let smart_attrs: Vec<SmartAttribute> = Vec::new();

    let mut battery: Option<BatteryHealth> = None;
    let mut memory_errors = false;

    #[cfg(windows)]
    {
        // 1. 查询设备管理器中有问题的设备 (ConfigManagerErrorCode != 0)
        let detected_devices = query_problem_devices();
        for pd in &detected_devices {
            let severity = match pd.problem_code {
                10 | 12 | 18 | 22 | 28 | 31 | 38 | 41 | 42 | 43 | 45 | 46 | 48 => "critical",
                _ => "warning",
            };
            findings.push(Finding {
                severity: severity.to_string(),
                component: detect_component_type(&pd.name, &pd.device_id),
                title: format!("设备异常: {}", pd.name),
                description: pd.problem_description.clone(),
                recommendation: pd.fix_suggestion.clone(),
            });
        }
        problem_devices = detected_devices;

        // 2. 查询 WHEA 硬件错误日志 (最近 7 天)
        let whea = query_whea_errors();
        if !whea.is_empty() {
            let recent_count = whea.len();
            let critical_count = whea.iter().filter(|w| w.severity == "critical").count();
            let source_summary = summarize_whea_sources(&whea);

            findings.push(Finding {
                severity: if critical_count > 5 { "critical" } else { "warning" }.to_string(),
                component: source_summary.clone(),
                title: format!("检测到 {} 条硬件错误记录", recent_count),
                description: format!(
                    "最近 7 天内 Windows 记录了 {} 条 WHEA 硬件错误（其中 {} 条严重），来源: {}",
                    recent_count, critical_count, source_summary
                ),
                recommendation: format!(
                    "建议检查 {} 相关硬件的连接、散热或更换硬件",
                    source_summary
                ),
            });
            whea_errors = whea;
        }

        // 3. 查询磁盘 SMART 详细属性
        let smart = query_smart_attributes();
        let critical_smart: Vec<_> = smart.iter().filter(|s| s.status == "critical").collect();
        let warning_smart: Vec<_> = smart.iter().filter(|s| s.status == "warning").collect();

        for cs in &critical_smart {
            findings.push(Finding {
                severity: "critical".to_string(),
                component: format!("Disk {}", cs.drive),
                title: format!("磁盘 {} - {} 严重", cs.drive, cs.attribute_name),
                description: cs.interpretation.clone(),
                recommendation: "立即备份重要数据并准备更换硬盘".to_string(),
            });
            memory_errors = true; // 磁盘有严重问题也算硬件故障
        }
        for ws in &warning_smart {
            findings.push(Finding {
                severity: "warning".to_string(),
                component: format!("Disk {}", ws.drive),
                title: format!("磁盘 {} - {} 警告", ws.drive, ws.attribute_name),
                description: ws.interpretation.clone(),
                recommendation: "密切关注，建议尽快备份数据".to_string(),
            });
        }
        smart_attrs = smart;

        // 4. 电池健康（笔记本）
        if let Some(b) = query_battery_health() {
            if b.health_percent < 50.0 {
                findings.push(Finding {
                    severity: "warning".to_string(),
                    component: "Battery".to_string(),
                    title: format!("电池健康度 {:.0}%", b.health_percent),
                    description: b.interpretation.clone(),
                    recommendation: "电池容量明显衰减，建议更换电池".to_string(),
                });
            } else if b.health_percent < 80.0 {
                findings.push(Finding {
                    severity: "info".to_string(),
                    component: "Battery".to_string(),
                    title: format!("电池健康度 {:.0}%", b.health_percent),
                    description: b.interpretation.clone(),
                    recommendation: "电池有一定损耗，续航会缩短".to_string(),
                });
            }
            battery = Some(b);
        }

        // 5. 内存 ECC 错误检测
        let mem_errors = query_memory_errors();
        if mem_errors > 0 {
            memory_errors = true;
            findings.push(Finding {
                severity: "critical".to_string(),
                component: "Memory".to_string(),
                title: format!("检测到 {} 条内存错误", mem_errors),
                description: format!(
                    "Windows 记录了 {} 条内存相关错误，可能存在故障内存条。",
                    mem_errors
                ),
                recommendation: "建议运行 Windows 内存诊断 (mdsched.exe)，并逐条内存测试定位故障条".to_string(),
            });
        }
    }

    // 计算总体状态
    let has_critical = findings.iter().any(|f| f.severity == "critical");
    let overall_status = if findings.is_empty() {
        "healthy".to_string()
    } else if has_critical {
        "critical".to_string()
    } else {
        "warnings".to_string()
    };

    let summary = if findings.is_empty() {
        "所有硬件诊断项检查通过，未发现明显硬件故障。".to_string()
    } else {
        let critical = findings.iter().filter(|f| f.severity == "critical").count();
        let warning = findings.iter().filter(|f| f.severity == "warning").count();
        let info = findings.iter().filter(|f| f.severity == "info").count();
        format!(
            "发现 {} 项问题：{} 项严重，{} 项警告，{} 项提示",
            findings.len(), critical, warning, info
        )
    };

    HardwareDiagnostics {
        overall_status,
        findings,
        problem_devices,
        whea_errors,
        smart_attributes: smart_attrs,
        battery,
        memory_errors_detected: memory_errors,
        summary,
    }
}

/// 启动 Windows 内存诊断工具（需要重启）
#[tauri::command]
pub fn launch_memory_diagnostic() -> Result<bool, String> {
    #[cfg(windows)]
    {
        match Command::new("mdsched.exe").spawn() {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("启动内存诊断失败: {}。需要管理员权限。", e)),
        }
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows 平台".to_string())
    }
}

// ========== 内部辅助函数 ==========

#[cfg(windows)]
fn query_problem_devices() -> Vec<ProblemDevice> {
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
                let name = parts[0].to_string();
                let device_id = parts[1].to_string();
                let code: u32 = parts[2].parse().unwrap_or(0);
                let (desc, cause, fix) = explain_problem_code(code);
                devices.push(ProblemDevice {
                    name,
                    device_id,
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

/// 根据 Windows ConfigManagerErrorCode 解释问题
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
            "资源冲突（IRQ/DMA）或资源不足".to_string(),
            "在设备管理器中查看资源冲突，禁用冲突设备".to_string(),
        ),
        18 => (
            "重新安装驱动程序".to_string(),
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
            "驱动程序不兼容".to_string(),
            "当前驱动不适用于此 Windows 版本".to_string(),
            "安装适用于当前 Windows 版本的驱动".to_string(),
        ),
        38 => (
            "驱动程序无法加载（上次启动失败）".to_string(),
            "驱动文件损坏或依赖缺失".to_string(),
            "重启电脑；若仍失败则重新安装驱动".to_string(),
        ),
        41 => (
            "驱动加载失败后重启".to_string(),
            "驱动程序加载过程中失败".to_string(),
            "更新或重新安装驱动程序".to_string(),
        ),
        42 => (
            "设备驱动重复加载".to_string(),
            "系统中存在重复的设备实例".to_string(),
            "在设备管理器中卸载重复设备".to_string(),
        ),
        43 => (
            "设备已停止（硬件故障）".to_string(),
            "Windows 收到设备故障报告，可能硬件损坏".to_string(),
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
            "安装已签名的驱动程序，或在高级启动中禁用签名强制".to_string(),
        ),
        _ => (
            format!("设备管理器错误码: {}", code),
            "未知错误类型".to_string(),
            "建议搜索该错误码或联系技术支持".to_string(),
        ),
    }
}

#[cfg(windows)]
fn query_whea_errors() -> Vec<WheaError> {
    // 查询最近 7 天的 WHEA 硬件错误日志
    let ps_command = r#"
try {
    $events = Get-WinEvent -FilterHashtable @{
        LogName='System'
        ProviderName='Microsoft-Windows-WHEA-Logger'
        StartTime=(Get-Date).AddDays(-7)
    } -MaxEvents 100 -ErrorAction SilentlyContinue
    if ($events) {
        foreach ($e in $events) {
            $type = switch ($e.Id) {
                17 { "Machine Check Exception (MCE)" }
                18 { "Corrected Machine Check" }
                19 { "Uncorrected Machine Check" }
                47 { "PCI Express Error" }
                48 { "PCI Express AER Error" }
                default { "WHEA Error (ID=$($e.Id))" }
            }
            $sev = if ($e.LevelDisplayName -eq 'Error' -or $e.LevelDisplayName -eq '错误') { "critical" } else { "warning" }
            $msg = ($e.Message -replace '[\r\n]+', ' ').Trim()
            if (!$msg) { $msg = $type }
            $time = $e.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss')
            Write-Output "$time|$type|$sev|$msg"
        }
    }
} catch {}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    let mut errors = Vec::new();
    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() >= 4 {
                let source = classify_whea_source(parts[1]);
                errors.push(WheaError {
                    time_created: parts[0].to_string(),
                    error_source: source,
                    error_type: parts[1].to_string(),
                    severity: parts[2].to_string(),
                    description: parts[3].to_string(),
                });
            }
        }
    }
    errors
}

#[cfg(windows)]
fn classify_whea_source(error_type: &str) -> String {
    let et = error_type.to_lowercase();
    if et.contains("memory") {
        "Memory".to_string()
    } else if et.contains("pcie") || et.contains("pci") {
        "PCIe".to_string()
    } else if et.contains("machine check") || et.contains("mce") {
        "CPU".to_string()
    } else if et.contains("usb") {
        "USB".to_string()
    } else if et.contains("disk") || et.contains("storage") {
        "Disk".to_string()
    } else {
        "Unknown".to_string()
    }
}

#[cfg(windows)]
fn summarize_whea_sources(errors: &[WheaError]) -> String {
    use std::collections::HashMap;
    let mut counts: HashMap<String, u32> = HashMap::new();
    for e in errors {
        *counts.entry(e.error_source.clone()).or_insert(0) += 1;
    }
    let mut entries: Vec<String> = counts
        .into_iter()
        .map(|(k, v)| format!("{}({})", k, v))
        .collect();
    entries.sort();
    entries.join(", ")
}

#[cfg(windows)]
fn detect_component_type(name: &str, device_id: &str) -> String {
    let combined = format!("{} {}", name, device_id).to_lowercase();
    if combined.contains("disk") || combined.contains("storage") || combined.contains("usb\\") {
        "Disk/USB".to_string()
    } else if combined.contains("display") || combined.contains("video") || combined.contains("gpu") {
        "GPU".to_string()
    } else if combined.contains("audio") || combined.contains("sound") {
        "Audio".to_string()
    } else if combined.contains("network") || combined.contains("net") {
        "Network".to_string()
    } else if combined.contains("processor") || combined.contains("cpu") {
        "CPU".to_string()
    } else if combined.contains("memory") || combined.contains("ram") {
        "Memory".to_string()
    } else if combined.contains("battery") {
        "Battery".to_string()
    } else if combined.contains("usb") {
        "USB".to_string()
    } else {
        "Other".to_string()
    }
}

#[cfg(windows)]
fn query_smart_attributes() -> Vec<SmartAttribute> {
    let mut attrs = Vec::new();

    // 通过 wmic 查询磁盘 SMART 基础状态
    let ps_command = r#"
Get-PhysicalDisk | ForEach-Object {
    $disk = $_
    $reliability = $disk | Get-StorageReliabilityCounter -ErrorAction SilentlyContinue
    if ($reliability) {
        $driveLetter = if ($disk.FriendlyName) { $disk.FriendlyName } else { "Disk$($disk.DeviceId)" }
        # Temperature
        if ($reliability.Temperature -gt 0) {
            Write-Output "$driveLetter|TEMP|$($reliability.Temperature)|C"
        }
        # Read errors
        if ($reliability.ReadErrorsTotal -gt 0) {
            Write-Output "$driveLetter|READERR|$($reliability.ReadErrorsTotal)|errors"
        }
        # Wear (SSD)
        if ($reliability.Wear -gt 0) {
            Write-Output "$driveLetter|WEAR|$($reliability.Wear)|percent"
        }
        # Power on hours
        if ($reliability.PowerOnHours -gt 0) {
            Write-Output "$driveLetter|HOURS|$($reliability.PowerOnHours)|hours"
        }
        # Start stop cycles
        if ($reliability.StartStopCycleCount -gt 0) {
            Write-Output "$driveLetter|CYCLES|$($reliability.StartStopCycleCount)|cycles"
        }
    }
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
                let drive = parts[0].to_string();
                let attr_id = match parts[1] {
                    "TEMP" => 1,
                    "READERR" => 2,
                    "WEAR" => 3,
                    "HOURS" => 4,
                    "CYCLES" => 5,
                    _ => 0,
                };
                let (name, threshold, interpretation, status) = interpret_smart_attr(
                    parts[1],
                    parts[2].parse().unwrap_or(0),
                    parts[3],
                );
                attrs.push(SmartAttribute {
                    drive: drive.clone(),
                    attribute_id: attr_id,
                    attribute_name: name,
                    raw_value: parts[2].parse().unwrap_or(0),
                    threshold,
                    status,
                    interpretation,
                });
            }
        }
    }

    // 检查磁盘健康状态
    let health_output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-PhysicalDisk | ForEach-Object { Write-Output \"$($_.FriendlyName)|$($_.HealthStatus)|$($_.OperationalStatus)\" }",
        ])
        .output();

    if let Ok(output) = health_output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                let drive = parts[0].to_string();
                let health = parts[1].to_lowercase();
                let oper = parts[2].to_lowercase();
                if health != "healthy" || oper != "ok" {
                    attrs.push(SmartAttribute {
                        drive: drive.clone(),
                        attribute_id: 99,
                        attribute_name: "Health Status".to_string(),
                        raw_value: 1,
                        threshold: 0,
                        status: "critical".to_string(),
                        interpretation: format!(
                            "磁盘健康状态: {}，运行状态: {}。磁盘可能已出现故障或即将损坏。",
                            parts[1], parts[2]
                        ),
                    });
                }
            }
        }
    }

    attrs
}

#[cfg(windows)]
fn interpret_smart_attr(attr: &str, value: u64, unit: &str) -> (String, u64, String, String) {
    match attr {
        "TEMP" => {
            let (status, interp) = if value >= 60 {
                ("critical", format!("温度 {}°C 严重过高，可能导致硬件损坏", value))
            } else if value >= 50 {
                ("warning", format!("温度 {}°C 偏高，检查散热", value))
            } else if value >= 40 {
                ("warning", format!("温度 {}°C 正常偏高", value))
            } else {
                ("ok", format!("温度 {}°C 正常", value))
            };
            ("Temperature".to_string(), 60, interp, status.to_string())
        }
        "READERR" => {
            let (status, interp) = if value > 1000 {
                ("critical", format!("读取错误 {} 次，磁盘可能有坏道", value))
            } else if value > 100 {
                ("warning", format!("读取错误 {} 次，建议监控", value))
            } else {
                ("ok", format!("读取错误 {} 次，正常范围", value))
            };
            ("Read Errors".to_string(), 100, interp, status.to_string())
        }
        "WEAR" => {
            let (status, interp) = if value >= 80 {
                ("critical", format!("SSD 磨损度 {}%，寿命即将耗尽", value))
            } else if value >= 50 {
                ("warning", format!("SSD 磨损度 {}%，寿命过半", value))
            } else {
                ("ok", format!("SSD 磨损度 {}%，状态良好", value))
            };
            ("SSD Wear".to_string(), 80, interp, status.to_string())
        }
        "HOURS" => {
            let years = value as f64 / 24.0 / 365.0;
            let interp = format!("通电 {} 小时（约 {:.1} 年）", value, years);
            ("Power On Hours".to_string(), 0, interp, "info".to_string())
        }
        "CYCLES" => {
            let interp = format!("启停 {} 次", value);
            ("Start/Stop Cycles".to_string(), 50000, interp, "ok".to_string())
        }
        _ => (format!("{} ({})", attr, unit), 0, "未知属性".to_string(), "info".to_string()),
    }
}

#[cfg(windows)]
fn query_battery_health() -> Option<BatteryHealth> {
    let ps_command = r#"
$bat = Get-WmiObject Win32_Battery -ErrorAction SilentlyContinue
if ($bat) {
    $designed = 0
    $fullCharge = 0
    $cycles = 0
    try {
        $batInfo = Get-WmiObject -Namespace "root\WMI" -Class BatteryStaticData -ErrorAction SilentlyContinue
        if ($batInfo) {
            $designed = $batInfo.DesignedCapacity
            $fullCharge = $batInfo.FullChargedCapacity
        }
    } catch {}
    try {
        $batFull = Get-WmiObject -Namespace "root\WMI" -Class BatteryFullChargedCapacity -ErrorAction SilentlyContinue
        if ($batFull -and $batFull.FullChargedCapacity -gt 0) {
            $fullCharge = $batFull.FullChargedCapacity
        }
    } catch {}
    if ($designed -gt 0 -and $fullCharge -gt 0) {
        $health = [math]::Round(($fullCharge / $designed) * 100, 1)
        Write-Output "$designed|$fullCharge|0|$health"
    }
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        let line = stdout.trim();
        if !line.is_empty() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let designed: u64 = parts[0].parse().unwrap_or(0);
                let full_charge: u64 = parts[1].parse().unwrap_or(0);
                let cycle: u32 = parts[2].parse().unwrap_or(0);
                let health: f32 = parts[3].parse().unwrap_or(0.0);
                let (status, interp) = if health < 50.0 {
                    ("poor", format!("电池容量仅剩 {:.0}%，需要更换电池", health))
                } else if health < 80.0 {
                    ("fair", format!("电池容量 {:.0}%，有损耗但可使用", health))
                } else {
                    ("good", format!("电池容量 {:.0}%，状态良好", health))
                };
                return Some(BatteryHealth {
                    designed_capacity: designed,
                    full_charge_capacity: full_charge,
                    cycle_count: cycle,
                    health_percent: health,
                    status: status.to_string(),
                    interpretation: interp,
                });
            }
        }
    }
    None
}

#[cfg(windows)]
fn query_memory_errors() -> u32 {
    // 查询内存相关的 WHEA 错误数量
    let ps_command = r#"
try {
    $events = Get-WinEvent -FilterHashtable @{
        LogName='System'
        ProviderName='Microsoft-Windows-WHEA-Logger'
        StartTime=(Get-Date).AddDays(-30)
    } -ErrorAction SilentlyContinue
    if ($events) {
        $memErrors = $events | Where-Object { $_.Message -match 'memory|RAM|DIMM' -or $_.Id -eq 18 -or $_.Id -eq 19 }
        Write-Output $memErrors.Count
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
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        stdout.trim().parse().unwrap_or(0)
    } else {
        0
    }
}
