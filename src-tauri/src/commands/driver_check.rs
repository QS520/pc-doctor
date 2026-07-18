use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone, Debug, Default)]
pub struct DriverInfo {
    pub device_name: String,
    pub device_class: String,
    pub manufacturer: String,
    pub driver_version: String,
    pub driver_date: String,
    pub driver_provider: String,
    pub status: String,
    pub is_up_to_date: bool,
    pub has_problem: bool,
    pub problem_description: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct DriverCheckResult {
    pub total_drivers: u32,
    pub problem_drivers: u32,
    pub drivers: Vec<DriverInfo>,
}

/// 检查所有驱动程序状态
#[tauri::command]
pub async fn check_drivers() -> DriverCheckResult {
    tokio::task::spawn_blocking(move || {
    let mut drivers = Vec::new();
    let mut problem_count = 0u32;

    #[cfg(windows)]
    {
        let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Get-CimInstance -ClassName Win32_PnPSignedDriver | ForEach-Object {
    $devName = $_.DeviceName
    if (!$devName) { $devName = $_.DeviceID }
    $devClass = $_.DeviceClass
    if (!$devClass) { $devClass = 'Unknown' }
    $mfr = $_.Manufacturer
    if (!$mfr) { $mfr = 'Unknown' }
    $ver = $_.DriverVersion
    if (!$ver) { $ver = 'Unknown' }
    $drvDate = ''
    if ($_.DriverDate) { $drvDate = $_.DriverDate.ToString('yyyy-MM-dd') }
    $provider = $_.DriverProviderName
    if (!$provider) { $provider = 'Unknown' }
    Write-Output "$devName|$devClass|$mfr|$ver|$drvDate|$provider"
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
                let parts: Vec<&str> = line.splitn(6, '|').collect();
                if parts.len() >= 6 {
                    let device_name = parts[0].to_string();
                    let device_class = parts[1].to_string();
                    let manufacturer = parts[2].to_string();
                    let driver_version = parts[3].to_string();
                    let driver_date = parts[4].to_string();
                    let driver_provider = parts[5].to_string();

                    // 检查驱动是否有问题 (通过 pnputil)
                    let has_problem = false;
                    let problem_desc = String::new();

                    // 判断是否可能过期 (超过 2 年的驱动标记为可能过期)
                    let is_up_to_date = check_driver_date(&driver_date);

                    if !is_up_to_date {
                        problem_count += 1;
                    }

                    drivers.push(DriverInfo {
                        device_name,
                        device_class,
                        manufacturer,
                        driver_version,
                        driver_date,
                        driver_provider,
                        status: if has_problem { "异常".to_string() }
                                else if !is_up_to_date { "可能过期".to_string() }
                                else { "正常".to_string() },
                        is_up_to_date,
                        has_problem,
                        problem_description: problem_desc,
                    });
                }
            }
        }

        // 同时检查有问题的设备
        let problem_ps = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Get-CimInstance -ClassName Win32_PnPEntity | Where-Object { $_.ConfigManagerErrorCode -ne 0 -and $_.ConfigManagerErrorCode -ne 22 } | ForEach-Object {
    $code = $_.ConfigManagerErrorCode
    $desc = switch ($code) {
        1 {"设备未正确配置"}
        3 {"驱动程序可能已损坏"}
        10 {"设备无法启动"}
        18 {"重新枚举失败"}
        19 {"注册表可能已损坏"}
        28 {"驱动程序未安装"}
        31 {"设备正常但需要重启"}
        32 {"设备被禁用"}
        37 {"设备无法初始化"}
        39 {"驱动程序已损坏"}
        41 {"Windows 成功加载驱动程序但找不到设备"}
        default {"错误代码: $code"}
    }
    Write-Output "$($_.Name)|$desc"
}
"#;

        let problem_output = Command::new("powershell")
            .args(["-NoProfile", "-Command", problem_ps.trim()])
            .output();

        if let Ok(problem_output) = problem_output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&problem_output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                if parts.len() == 2 {
                    // 标记有问题的驱动
                    for d in &mut drivers {
                        if d.device_name.contains(parts[0]) || parts[0].contains(&d.device_name) {
                            d.has_problem = true;
                            d.problem_description = parts[1].to_string();
                            d.status = "异常".to_string();
                            problem_count += 1;
                            break;
                        }
                    }
                }
            }
        }
    }

    // 去重 (同一设备可能有多条记录)
    drivers.dedup_by(|a: &mut DriverInfo, b: &mut DriverInfo| a.device_name == b.device_name && a.driver_version == b.driver_version);

    let total = drivers.len() as u32;

    DriverCheckResult {
        total_drivers: total,
        problem_drivers: problem_count,
        drivers,
    }
    })
    .await
    .unwrap_or_default()
}

/// 检查驱动日期是否过期 (超过 2 年视为可能过期)
#[allow(dead_code)]
fn check_driver_date(date_str: &str) -> bool {
    if date_str.is_empty() || date_str == "Unknown" {
        return true; // 未知日期不标记为过期
    }

    // 解析 yyyy-MM-dd 格式
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return true;
    }

    let year: i32 = parts[0].parse().unwrap_or(0);
    let month: u32 = parts[1].parse().unwrap_or(0);
    let _day: u32 = parts[2].parse().unwrap_or(0);

    if year == 0 {
        return true;
    }

    // 获取当前日期
    let now = chrono::Local::now();
    let current_year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2026);

    // 超过 2 年视为过期
    if current_year - year > 2 {
        return false;
    }

    if current_year - year == 2 {
        let current_month: u32 = now.format("%m").to_string().parse().unwrap_or(0);
        if month < current_month {
            return false;
        }
    }

    true
}
