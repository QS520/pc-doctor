use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct MemoryStick {
    pub bank_label: String,
    pub device_locator: String,
    pub manufacturer: String,
    pub part_number: String,
    pub serial_number: String,
    pub capacity_gb: f64,
    pub speed_mhz: u32,
    pub memory_type: String,
    pub form_factor: String,
    pub configured_speed_mhz: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct CpuHardwareInfo {
    pub name: String,
    pub manufacturer: String,
    pub socket: String,
    pub cores: u32,
    pub logical_cores: u32,
    pub max_clock_mhz: u32,
    pub current_clock_mhz: u32,
    pub l2_cache_kb: u32,
    pub l3_cache_kb: u32,
    pub voltage: f32,
}

#[derive(Serialize, Clone, Debug)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub version: String,
    pub serial: String,
    pub bios_version: String,
    pub bios_date: String,
    pub bios_manufacturer: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct GpuInfo {
    pub name: String,
    pub manufacturer: String,
    pub driver_version: String,
    pub driver_date: String,
    pub adapter_ram_gb: f64,
    pub video_processor: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct HardwareInfo {
    pub memory_sticks: Vec<MemoryStick>,
    pub cpu: Option<CpuHardwareInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub gpus: Vec<GpuInfo>,
}

/// 获取硬件信息（内存条、CPU、主板、显卡）
#[tauri::command]
pub fn get_hardware_info() -> HardwareInfo {
    let mut memory_sticks = Vec::new();
    let mut cpu = None;
    let mut motherboard = None;
    let mut gpus = Vec::new();

    #[cfg(windows)]
    {
        // 查询内存条信息
        memory_sticks = query_wmi_memory();

        // 查询 CPU 硬件信息
        cpu = query_wmi_cpu();

        // 查询主板信息
        motherboard = query_wmi_motherboard();

        // 查询显卡信息
        gpus = query_wmi_gpu();
    }

    HardwareInfo {
        memory_sticks,
        cpu,
        motherboard,
        gpus,
    }
}

#[cfg(windows)]
fn query_wmi_memory() -> Vec<MemoryStick> {
    let mut sticks = Vec::new();

    let ps_command = r#"
Get-CimInstance -ClassName Win32_PhysicalMemory | ForEach-Object {
    $typeCode = $_.SMBIOSMemoryType
    $typeName = switch ($typeCode) {
        20 {"DDR"}
        21 {"DDR2"}
        24 {"DDR3"}
        26 {"DDR4"}
        34 {"DDR5"}
        default {"Unknown($typeCode)"}
    }
    $formCode = $_.FormFactor
    $formName = switch ($formCode) {
        8 {"DIMM"}
        12 {"SODIMM"}
        15 {"FB-DIMM"}
        default {"Unknown"}
    }
    $line = "$($_.BankLabel)|$($_.DeviceLocator)|$($_.Manufacturer)|$($_.PartNumber)|$($_.SerialNumber)|$([math]::Round($_.Capacity / 1GB, 2))|$($_.Speed)|$typeName|$formName|$($_.ConfiguredClockSpeed)"
    Write-Output $line
}
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 10 {
                sticks.push(MemoryStick {
                    bank_label: parts[0].to_string(),
                    device_locator: parts[1].to_string(),
                    manufacturer: parts[2].trim().to_string(),
                    part_number: parts[3].trim().to_string(),
                    serial_number: parts[4].trim().to_string(),
                    capacity_gb: parts[5].parse().unwrap_or(0.0),
                    speed_mhz: parts[6].parse().unwrap_or(0),
                    memory_type: parts[7].to_string(),
                    form_factor: parts[8].to_string(),
                    configured_speed_mhz: parts[9].parse().unwrap_or(0),
                });
            }
        }
    }

    sticks
}

#[cfg(windows)]
fn query_wmi_cpu() -> Option<CpuHardwareInfo> {
    let ps_command = r#"
$cpu = Get-CimInstance -ClassName Win32_Processor | Select-Object -First 1
$l2 = ($cpu.L2CacheSizeKB)
$l3 = ($cpu.L3CacheSizeKB)
$volt = if ($cpu.CurrentVoltage) { [float]$cpu.CurrentVoltage / 10 } else { 0 }
$line = "$($cpu.Name)|$($cpu.Manufacturer)|$($cpu.SocketDesignation)|$($cpu.NumberOfCores)|$($cpu.NumberOfLogicalProcessors)|$($cpu.MaxClockSpeed)|$($cpu.CurrentClockSpeed)|$l2|$l3|$volt"
Write-Output $line
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        let line = stdout.trim();
        if !line.is_empty() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 10 {
                return Some(CpuHardwareInfo {
                    name: parts[0].to_string(),
                    manufacturer: parts[1].to_string(),
                    socket: parts[2].to_string(),
                    cores: parts[3].parse().unwrap_or(0),
                    logical_cores: parts[4].parse().unwrap_or(0),
                    max_clock_mhz: parts[5].parse().unwrap_or(0),
                    current_clock_mhz: parts[6].parse().unwrap_or(0),
                    l2_cache_kb: parts[7].parse().unwrap_or(0),
                    l3_cache_kb: parts[8].parse().unwrap_or(0),
                    voltage: parts[9].parse().unwrap_or(0.0),
                });
            }
        }
    }

    None
}

#[cfg(windows)]
fn query_wmi_motherboard() -> Option<MotherboardInfo> {
    let ps_command = r#"
$board = Get-CimInstance -ClassName Win32_BaseBoard
$bios = Get-CimInstance -ClassName Win32_BIOS
$line = "$($board.Manufacturer)|$($board.Product)|$($board.Version)|$($board.SerialNumber)|$($bios.SMBIOSBIOSVersion)|$($bios.ReleaseDate.ToString('yyyy-MM-dd'))|$($bios.Manufacturer)"
Write-Output $line
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        let line = stdout.trim();
        if !line.is_empty() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 7 {
                return Some(MotherboardInfo {
                    manufacturer: parts[0].to_string(),
                    product: parts[1].to_string(),
                    version: parts[2].to_string(),
                    serial: parts[3].to_string(),
                    bios_version: parts[4].to_string(),
                    bios_date: parts[5].to_string(),
                    bios_manufacturer: parts[6].to_string(),
                });
            }
        }
    }

    None
}

#[cfg(windows)]
fn query_wmi_gpu() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    let ps_command = r#"
Get-CimInstance -ClassName Win32_VideoController | ForEach-Object {
    $ram = if ($_.AdapterRAM) { [math]::Round($_.AdapterRAM / 1GB, 2) } else { 0 }
    $drvDate = if ($_.DriverDate) { $_.DriverDate.ToString('yyyy-MM-dd') } else { 'Unknown' }
    $line = "$($_.Name)|$($_.VideoProcessor)|$($_.AdapterCompatibility)|$($_.DriverVersion)|$drvDate|$ram"
    Write-Output $line
}
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_command.trim()])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 6 {
                gpus.push(GpuInfo {
                    name: parts[0].to_string(),
                    video_processor: parts[1].to_string(),
                    manufacturer: parts[2].to_string(),
                    driver_version: parts[3].to_string(),
                    driver_date: parts[4].to_string(),
                    adapter_ram_gb: parts[5].parse().unwrap_or(0.0),
                });
            }
        }
    }

    gpus
}
