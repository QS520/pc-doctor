use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone, Debug)]
pub struct PowerPlan {
    pub guid: String,
    pub name: String,
    pub is_active: bool,
    pub description: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct PowerPlanResult {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CpuThrottleInfo {
    pub current_power_plan: String,
    pub cpu_min_state_percent: u32,
    pub cpu_max_state_percent: u32,
    pub is_throttled: bool,
    pub display_timeout_minutes: u32,
    pub sleep_timeout_minutes: u32,
}

/// 获取所有电源计划
#[tauri::command]
pub fn get_power_plans() -> Vec<PowerPlan> {
    let mut plans = Vec::new();

    #[cfg(windows)]
    {
        let output = Command::new("powercfg")
            .args(["/list"])
            .output();

        if let Ok(output) = output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);

            for line in stdout.lines() {
                let line = line.trim();
                // 格式: 电源方案 GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (平衡)
                if line.contains("GUID:") {
                    let guid_start = line.find("GUID:").map(|p| p + 5);
                    if let Some(start) = guid_start {
                        let rest = &line[start..].trim();
                        let guid_end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                        let guid = rest[..guid_end].to_string();

                        let name_part = &rest[guid_end..].trim();
                        let name = name_part.trim_matches('(').trim_matches(')').to_string();

                        let is_active = line.contains("*") || line.contains("活动") || line.contains("Active");

                        plans.push(PowerPlan {
                            guid,
                            name,
                            is_active,
                            description: if is_active { "当前活动方案".to_string() } else { String::new() },
                        });
                    }
                }
            }
        }
    }

    plans
}

/// 切换电源计划
#[tauri::command]
pub fn set_power_plan(guid: String) -> PowerPlanResult {
    #[cfg(windows)]
    {
        let output = Command::new("powercfg")
            .args(["/setactive", &guid])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    PowerPlanResult {
                        success: true,
                        message: format!("电源方案已切换 ({})", guid),
                    }
                } else {
                    let (stderr, _, _) = encoding_rs::UTF_8.decode(&output.stderr);
                    PowerPlanResult {
                        success: false,
                        message: format!("切换失败: {}。可能需要管理员权限。", stderr.trim()),
                    }
                }
            }
            Err(e) => PowerPlanResult {
                success: false,
                message: format!("执行失败: {}", e),
            },
        }
    }
    #[cfg(not(windows))]
    {
        let _ = guid;
        PowerPlanResult { success: false, message: "仅支持 Windows".to_string() }
    }
}

/// 获取 CPU 降频状态
#[tauri::command]
pub fn get_cpu_throttle_info() -> CpuThrottleInfo {
    #[cfg(windows)]
    {
        // 获取当前活动电源方案
        let plan_output = Command::new("powercfg")
            .args(["/getactivescheme"])
            .output();

        let mut current_plan = "未知".to_string();
        if let Ok(output) = plan_output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            let line = stdout.trim();
            // 格式: 电源方案 GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (平衡)
            if let Some(paren_start) = line.rfind('(') {
                if let Some(paren_end) = line.rfind(')') {
                    if paren_end > paren_start {
                        current_plan = line[paren_start + 1..paren_end].to_string();
                    }
                }
            }
        }

        // 获取 CPU 最小/最大频率状态
        let min_output = Command::new("powercfg")
            .args(["/query", "SCHEME_CURRENT", "SUB_PROCESSOR", "PROCTHROTTLEMIN"])
            .output();

        let mut min_percent = 5u32;
        if let Ok(output) = min_output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            // 查找 "当前交流电源设置索引" 或 "Current AC Power Setting Index"
            for line in stdout.lines() {
                let line = line.trim();
                if line.contains("当前") && line.contains("索引") || line.contains("Current AC") {
                    // 提取十六进制值
                    let hex_str: String = line.chars()
                        .rev()
                        .take_while(|c| c.is_ascii_hexdigit())
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                        min_percent = val;
                        break;
                    }
                }
            }
        }

        let max_output = Command::new("powercfg")
            .args(["/query", "SCHEME_CURRENT", "SUB_PROCESSOR", "PROCTHROTTLEMAX"])
            .output();

        let mut max_percent = 100u32;
        if let Ok(output) = max_output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.contains("当前") && line.contains("索引") || line.contains("Current AC") {
                    let hex_str: String = line.chars()
                        .rev()
                        .take_while(|c| c.is_ascii_hexdigit())
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                        max_percent = val;
                        break;
                    }
                }
            }
        }

        // 获取显示器超时
        let display_output = Command::new("powercfg")
            .args(["/query", "SCHEME_CURRENT", "SUB_VIDEO", "VIDEOIDLE"])
            .output();

        let mut display_timeout = 10u32;
        if let Ok(output) = display_output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.contains("当前") && line.contains("索引") || line.contains("Current AC") {
                    let hex_str: String = line.chars()
                        .rev()
                        .take_while(|c| c.is_ascii_hexdigit())
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                        display_timeout = val / 60; // 秒转分钟
                        break;
                    }
                }
            }
        }

        // 获取睡眠超时
        let sleep_output = Command::new("powercfg")
            .args(["/query", "SCHEME_CURRENT", "SUB_SLEEP", "STANDBYIDLE"])
            .output();

        let mut sleep_timeout = 30u32;
        if let Ok(output) = sleep_output {
            let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                if line.contains("当前") && line.contains("索引") || line.contains("Current AC") {
                    let hex_str: String = line.chars()
                        .rev()
                        .take_while(|c| c.is_ascii_hexdigit())
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    if let Ok(val) = u32::from_str_radix(&hex_str, 16) {
                        sleep_timeout = val / 60;
                        break;
                    }
                }
            }
        }

        let is_throttled = max_percent < 100;

        CpuThrottleInfo {
            current_power_plan: current_plan,
            cpu_min_state_percent: min_percent,
            cpu_max_state_percent: max_percent,
            is_throttled,
            display_timeout_minutes: display_timeout,
            sleep_timeout_minutes: sleep_timeout,
        }
    }
    #[cfg(not(windows))]
    {
        CpuThrottleInfo {
            current_power_plan: "仅支持 Windows".to_string(),
            cpu_min_state_percent: 0,
            cpu_max_state_percent: 0,
            is_throttled: false,
            display_timeout_minutes: 0,
            sleep_timeout_minutes: 0,
        }
    }
}

/// 设置 CPU 最大频率状态 (100 = 不降频)
#[tauri::command]
pub fn set_cpu_max_state(percent: u32) -> PowerPlanResult {
    #[cfg(windows)]
    {
        let output = Command::new("powercfg")
            .args(["/setacvalueindex", "SCHEME_CURRENT", "SUB_PROCESSOR", "PROCTHROTTLEMAX", &percent.to_string()])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    // 应用设置
                    let _ = Command::new("powercfg").args(["/setactive", "SCHEME_CURRENT"]).output();
                    PowerPlanResult {
                        success: true,
                        message: if percent >= 100 {
                            "CPU 最大性能已设为 100%（解除降频）".to_string()
                        } else {
                            format!("CPU 最大性能已设为 {}%", percent)
                        },
                    }
                } else {
                    PowerPlanResult {
                        success: false,
                        message: "设置失败，可能需要管理员权限。".to_string(),
                    }
                }
            }
            Err(e) => PowerPlanResult {
                success: false,
                message: format!("执行失败: {}", e),
            },
        }
    }
    #[cfg(not(windows))]
    {
        let _ = percent;
        PowerPlanResult { success: false, message: "仅支持 Windows".to_string() }
    }
}
