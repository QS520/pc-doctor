use serde::Serialize;
use sysinfo::System;
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

#[derive(Serialize, Clone)]
pub struct CpuInfo {
    pub brand: String,
    pub core_count: usize,
    pub usage: f32,
    pub frequency: u64,
}

#[derive(Serialize, Clone)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub used_gb: f64,
    pub free_gb: f64,
    pub usage_percent: f64,
}

#[derive(Serialize, Clone)]
pub struct DiskInfo {
    pub drive: String,
    pub total_gb: f64,
    pub free_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f64,
    pub drive_type: String,
    pub label: String,
    pub file_system: String,
    pub serial_number: String,
    pub model: String,
    pub interface_type: String,
    pub partition_style: String,
    pub is_ssd: bool,
    pub health_status: String,
}

#[derive(Serialize, Clone)]
pub struct SystemInfoResult {
    pub os_name: String,
    pub os_version: String,
    pub os_build: String,
    pub hostname: String,
    pub uptime_hours: f64,
    pub boot_time: String,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub network: NetworkInfo,
    pub battery: Option<BatteryInfo>,
}

#[derive(Serialize, Clone, Default)]
pub struct NetworkInfo {
    pub ip_address: String,
    pub mac_address: String,
    pub adapter_name: String,
}

#[derive(Serialize, Clone)]
pub struct BatteryInfo {
    pub has_battery: bool,
    pub is_charging: bool,
    pub percent: u8,
    pub time_remaining_min: Option<u32>,
}

/// 获取完整系统信息
#[tauri::command]
pub fn get_system_info() -> SystemInfoResult {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU 信息
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let core_count = sys.cpus().len();
    let cpu_usage = sys.global_cpu_usage();
    let cpu_freq = sys
        .cpus()
        .first()
        .map(|c| c.frequency())
        .unwrap_or(0);

    // 内存信息
    let total_mem = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_mem = sys.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let free_mem = total_mem - used_mem;
    let mem_percent = if total_mem > 0.0 {
        (used_mem / total_mem) * 100.0
    } else {
        0.0
    };

    // 磁盘信息 - 使用 Windows API 获取更准确的数据
    let disks = get_windows_disks();

    // 系统信息
    let os_name = System::name().unwrap_or_else(|| "Windows".to_string());
    let os_version = System::os_version().unwrap_or_default();
    let os_build = System::long_os_version().unwrap_or_default();
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let uptime_secs = System::uptime();
    let uptime_hours = uptime_secs as f64 / 3600.0;

    // 启动时间
    let boot_timestamp = System::uptime();
    let now = chrono::Local::now();
    let boot_time = now - chrono::Duration::seconds(boot_timestamp as i64);
    let boot_time_str = boot_time.format("%Y-%m-%d %H:%M:%S").to_string();

    SystemInfoResult {
        os_name,
        os_version,
        os_build,
        hostname,
        uptime_hours,
        boot_time: boot_time_str,
        cpu: CpuInfo {
            brand: cpu_brand,
            core_count,
            usage: cpu_usage,
            frequency: cpu_freq,
        },
        memory: MemoryInfo {
            total_gb: (total_mem * 100.0).round() / 100.0,
            used_gb: (used_mem * 100.0).round() / 100.0,
            free_gb: (free_mem * 100.0).round() / 100.0,
            usage_percent: (mem_percent * 10.0).round() / 10.0,
        },
        disks,
        network: get_network_info(),
        battery: get_battery_info(),
    }
}

/// 获取网络信息（IP 和 MAC 地址）
#[cfg(windows)]
fn get_network_info() -> NetworkInfo {
    use std::process::Command;

    // 获取活跃网卡的 IP 和 MAC
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-NetIPConfiguration | Where-Object { $_.IPv4DefaultGateway -ne $null } | Select-Object -First 1 | ForEach-Object { $ip=$_.IPv4Address.IPAddress; $if=$_.InterfaceAlias; $mac=(Get-NetAdapter | Where-Object { $_.Name -eq $if } | Select-Object -ExpandProperty MacAddress); Write-Output \"$ip|$mac|$if\" }",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();
        if !line.is_empty() {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                return NetworkInfo {
                    ip_address: parts[0].to_string(),
                    mac_address: parts[1].to_string(),
                    adapter_name: parts[2].to_string(),
                };
            }
        }
    }

    NetworkInfo::default()
}

#[cfg(not(windows))]
fn get_network_info() -> NetworkInfo {
    NetworkInfo::default()
}

/// 获取电池信息
#[cfg(windows)]
fn get_battery_info() -> Option<BatteryInfo> {
    use std::process::Command;

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "$b = Get-CimInstance -ClassName Win32_Battery -ErrorAction SilentlyContinue; if ($b) { $charging = if($b.BatteryStatus -eq 2){'True'}else{'False'}; $mins = $b.EstimatedChargeRemaining; Write-Output \"True|$charging|$($b.EstimatedChargeRemaining)|$($b.EstimatedRunTime)\" } else { Write-Output 'False' }",
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();
        if line.starts_with("True|") {
            let parts: Vec<&str> = line.split('|').collect();
            let is_charging = parts.get(1).map(|s| *s == "True").unwrap_or(false);
            let percent = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0u8);
            let time_min = parts.get(3).and_then(|s| s.parse::<u32>().ok()).filter(|&v| v > 0 && v < 100000);
            return Some(BatteryInfo {
                has_battery: true,
                is_charging,
                percent,
                time_remaining_min: time_min,
            });
        }
    }
    None
}

#[cfg(not(windows))]
fn get_battery_info() -> Option<BatteryInfo> {
    None
}

/// 获取 CPU 温度 (通过 WMI 查询)
#[tauri::command]
pub fn get_cpu_temperature() -> Result<f32, String> {
    #[cfg(windows)]
    {
        // 使用 PowerShell 通过 WMI 查询 CPU 温度
        // 注意: 不是所有主板都支持通过 WMI 读取温度
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty CurrentTemperature",
            ])
            .output()
            .map_err(|e| format!("执行失败: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let temp_str = stdout.trim();

        // WMI 返回的是开尔文温度 * 10
        if let Ok(temp_raw) = temp_str.parse::<f32>() {
            // 转换: (raw / 10) - 273.15 = 摄氏度
            let celsius = (temp_raw / 10.0) - 273.15;
            return Ok((celsius * 10.0).round() / 10.0);
        }

        // 备选方案: 通过 OpenHardwareMonitor 或 LibreHardwareMonitor 的 WMI 命名空间
        let output2 = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance -Namespace root/OpenHardwareMonitor -ClassName Sensor -Filter \"SensorType='Temperature'\" -ErrorAction SilentlyContinue | Where-Object { $_.Parent -like '*cpu*' } | Select-Object -First 1 -ExpandProperty Value",
            ])
            .output()
            .map_err(|e| format!("执行失败: {}", e))?;

        let stdout2 = String::from_utf8_lossy(&output2.stdout);
        if let Ok(temp) = stdout2.trim().parse::<f32>() {
            return Ok((temp * 10.0).round() / 10.0);
        }

        Err("无法读取 CPU 温度（主板不支持 WMI 温度查询，建议安装 LibreHardwareMonitor）".to_string())
    }
    #[cfg(not(windows))]
    {
        Err("仅支持 Windows 平台".to_string())
    }
}

/// 获取 Windows 磁盘列表（含可用空间）
#[cfg(windows)]
fn get_windows_disks() -> Vec<DiskInfo> {
    use windows::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetLogicalDriveStringsW, GetDriveTypeW,
    };

    // GetDriveTypeW 返回值常量（windows crate 中未导出，手动定义）
    const DRIVE_REMOVABLE: u32 = 2;
    const DRIVE_FIXED: u32 = 3;

    let mut result = Vec::new();

    unsafe {
        // 获取所有逻辑驱动器
        let mut buffer = [0u16; 256];
        let len = GetLogicalDriveStringsW(Some(&mut buffer));
        if len == 0 {
            return result;
        }

        // 解析双 null 结尾的字符串数组
        let mut start = 0;
        while start < len as usize {
            let end = buffer[start..]
                .iter()
                .position(|&c| c == 0)
                .map(|p| start + p)
                .unwrap_or(len as usize);

            if end > start {
                let drive_str = OsString::from_wide(&buffer[start..end])
                    .to_string_lossy()
                    .to_string();

                let drive_type = GetDriveTypeW(windows::core::PCWSTR(
                    buffer[start..end].as_ptr(),
                ));

                let type_str = match drive_type {
                    DRIVE_FIXED => "本地磁盘",
                    DRIVE_REMOVABLE => "可移动磁盘",
                    _ => "其他",
                };

                // 获取磁盘空间
                let mut free_bytes: u64 = 0;
                let mut total_bytes: u64 = 0;
                let mut available_bytes: u64 = 0;

                let success = GetDiskFreeSpaceExW(
                    windows::core::PCWSTR(buffer[start..end].as_ptr()),
                    Some(&mut available_bytes),
                    Some(&mut total_bytes),
                    Some(&mut free_bytes),
                );

                if success.is_ok() && total_bytes > 0 {
                    let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                    let free_gb = free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                    let used_gb = total_gb - free_gb;
                    let percent = (used_gb / total_gb) * 100.0;
                    let drive_letter = drive_str.chars().next().unwrap_or('C').to_string();

                    // 查询该磁盘的详细信息（卷标、文件系统、序列号）
                    let (label, file_system, serial_number) = query_volume_info(&drive_letter);
                    // 查询物理磁盘信息（型号、接口、是否 SSD）
                    let (model, interface_type, is_ssd, health_status, partition_style) =
                        query_physical_disk_info(&drive_letter);

                    result.push(DiskInfo {
                        drive: drive_letter,
                        total_gb: (total_gb * 100.0).round() / 100.0,
                        free_gb: (free_gb * 100.0).round() / 100.0,
                        used_gb: (used_gb * 100.0).round() / 100.0,
                        usage_percent: (percent * 10.0).round() / 10.0,
                        drive_type: type_str.to_string(),
                        label,
                        file_system,
                        serial_number,
                        model,
                        interface_type,
                        partition_style,
                        is_ssd,
                        health_status,
                    });
                }
            }

            start = end + 1;
        }
    }

    if result.is_empty() {
        // 退回方案: 至少返回 C 盘信息
        result.push(DiskInfo {
            drive: "C".to_string(),
            total_gb: 0.0,
            free_gb: 0.0,
            used_gb: 0.0,
            usage_percent: 0.0,
            drive_type: "本地磁盘".to_string(),
            label: String::new(),
            file_system: String::new(),
            serial_number: String::new(),
            model: String::new(),
            interface_type: String::new(),
            partition_style: String::new(),
            is_ssd: false,
            health_status: String::new(),
        });
    }

    result
}

/// 查询卷信息：卷标、文件系统、序列号
#[cfg(windows)]
fn query_volume_info(drive_letter: &str) -> (String, String, String) {
    use std::process::Command;

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$d = Get-CimInstance Win32_LogicalDisk -Filter \"DeviceID='{}:'\"; $vol = Get-Volume -DriveLetter {} -ErrorAction SilentlyContinue; $label = if($vol){{$vol.FileSystemLabel}}else{{$d.VolumeName}}; if(!$label){{$label='-'}}; $fs = if($vol){{$vol.FileSystem}}else{{$d.FileSystem}}; if(!$fs){{$fs='-'}}; $serial = $d.VolumeSerialNumber; if(!$serial){{$serial='-'}}; Write-Output \"$label|$fs|$serial\""
            , drive_letter, drive_letter),
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();
        if !line.is_empty() {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                return (
                    parts[0].to_string(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                );
            }
        }
    }
    (String::new(), String::new(), String::new())
}

/// 查询物理磁盘信息：型号、接口类型、是否 SSD、健康状态、分区样式
#[cfg(windows)]
fn query_physical_disk_info(drive_letter: &str) -> (String, String, bool, String, String) {
    use std::process::Command;

    // 通过分区 -> 磁盘映射获取物理磁盘
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$part = Get-Partition -DriveLetter {} -ErrorAction SilentlyContinue; if($part){{ $diskNum = $part.DiskNumber; $pd = Get-PhysicalDisk -ErrorAction SilentlyContinue | Where-Object {{$_.DeviceId -eq $diskNum}}; if($pd){{ $model = $pd.FriendlyName; $bus = $pd.BusType.ToString(); $media = $pd.MediaType.ToString(); $health = $pd.HealthStatus.ToString(); Write-Output \"$model|$bus|$media|$health|$diskNum\" }} }}", drive_letter),
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.trim();
        if !line.is_empty() && line.contains('|') {
            let parts: Vec<&str> = line.splitn(5, '|').collect();
            if parts.len() >= 4 {
                let model = parts[0].to_string();
                let bus = parts[1].to_string();
                let media = parts[2].to_string();
                let health = parts[3].to_string();
                let disk_num = parts.get(4).map(|s| *s).unwrap_or("");

                let is_ssd = media.to_lowercase().contains("ssd") || media.to_lowercase().contains("solid");
                let partition_style = query_partition_style(disk_num);

                return (model, bus, is_ssd, health, partition_style);
            }
        }
    }
    (
        String::new(),
        String::new(),
        false,
        String::new(),
        String::new(),
    )
}

/// 查询分区样式（MBR 或 GPT）
#[cfg(windows)]
fn query_partition_style(disk_number: &str) -> String {
    use std::process::Command;

    if disk_number.is_empty() {
        return String::new();
    }

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "$d = Get-Disk -Number {} -ErrorAction SilentlyContinue; if($d){{ Write-Output $d.PartitionStyle.ToString() }}", disk_number),
        ])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let style = stdout.trim();
        if !style.is_empty() {
            return style.to_string();
        }
    }
    String::new()
}

#[cfg(not(windows))]
fn query_volume_info(_drive_letter: &str) -> (String, String, String) {
    (String::new(), String::new(), String::new())
}

#[cfg(not(windows))]
fn query_physical_disk_info(_drive_letter: &str) -> (String, String, bool, String, String) {
    (
        String::new(),
        String::new(),
        false,
        String::new(),
        String::new(),
    )
}

#[cfg(not(windows))]
fn get_windows_disks() -> Vec<DiskInfo> {
    vec![]
}
