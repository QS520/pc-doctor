use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub start_type: String,
    pub process_id: u32,
    pub description: String,
    pub is_safe_to_disable: bool,
    pub category: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ServiceOperationResult {
    pub success: bool,
    pub message: String,
}

/// 获取所有 Windows 服务 —— 异步执行避免阻塞 UI
#[tauri::command]
pub async fn get_services() -> Vec<ServiceInfo> {
    tokio::task::spawn_blocking(|| {
    let mut services = Vec::new();

    #[cfg(windows)]
    {
        let ps_command = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
Get-CimInstance -ClassName Win32_Service | ForEach-Object {
    $name = $_.Name
    $display = $_.DisplayName
    $status = $_.State
    $startType = $_.StartMode
    $pid = $_.ProcessId
    $desc = ($_.Description -replace '[\r\n]+', ' ')
    if (!$desc) { $desc = '' }
    if (!$display) { $display = $name }
    Write-Output "$name|$display|$status|$startType|$pid|$desc"
}
"#;

        let output = std::process::Command::new("powershell")
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
                    let name = parts[0].to_string();
                    let display_name = parts[1].to_string();
                    let status_str = parts[2].to_string();
                    let start_type_raw = parts[3].to_string();
                    let pid: u32 = parts[4].parse().unwrap_or(0);
                    let description = parts[5].trim().to_string();

                    // 判断是否安全禁用
                    let (safe, category) = classify_service(&name, &display_name);

                    let start_type = match start_type_raw.as_str() {
                        "Auto" => "自动",
                        "Manual" => "手动",
                        "Disabled" => "已禁用",
                        "Boot" => "引导",
                        "System" => "系统",
                        _ => &start_type_raw,
                    }.to_string();

                    let status_cn = match status_str.as_str() {
                        "Running" => "运行中",
                        "Stopped" => "已停止",
                        "Paused" => "已暂停",
                        "Start Pending" => "正在启动",
                        "Stop Pending" => "正在停止",
                        _ => &status_str,
                    }.to_string();

                    services.push(ServiceInfo {
                        name,
                        display_name,
                        status: status_cn,
                        start_type,
                        process_id: pid,
                        description,
                        is_safe_to_disable: safe,
                        category,
                    });
                }
            }
        }

        // 按类别和名称排序
        services.sort_by(|a, b| {
            a.category.cmp(&b.category)
                .then_with(|| a.display_name.cmp(&b.display_name))
        });
    }

    services
    })
    .await
    .unwrap_or_default()
}

/// 禁用服务
#[tauri::command]
pub async fn disable_service(name: String) -> ServiceOperationResult {
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            let output = std::process::Command::new("sc")
                .args(["config", &name, "start=", "disabled"])
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        // 尝试停止服务
                        let _ = std::process::Command::new("sc")
                            .args(["stop", &name])
                            .output();

                        ServiceOperationResult {
                            success: true,
                            message: format!("服务 {} 已禁用", name),
                        }
                    } else {
                        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
                        ServiceOperationResult {
                            success: false,
                            message: format!("禁用失败: {}。可能需要管理员权限。", stderr.trim()),
                        }
                    }
                }
                Err(e) => ServiceOperationResult {
                    success: false,
                    message: format!("执行失败: {}", e),
                },
            }
        }
        #[cfg(not(windows))]
        {
            let _ = name;
            ServiceOperationResult { success: false, message: "仅支持 Windows".to_string() }
        }
    })
    .await
    .unwrap_or_default()
}

/// 启用服务（设置为手动）
#[tauri::command]
pub async fn enable_service(name: String) -> ServiceOperationResult {
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            let output = std::process::Command::new("sc")
                .args(["config", &name, "start=", "demand"])
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        ServiceOperationResult {
                            success: true,
                            message: format!("服务 {} 已设为手动启动", name),
                        }
                    } else {
                        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
                        ServiceOperationResult {
                            success: false,
                            message: format!("启用失败: {}。可能需要管理员权限。", stderr.trim()),
                        }
                    }
                }
                Err(e) => ServiceOperationResult {
                    success: false,
                    message: format!("执行失败: {}", e),
                },
            }
        }
        #[cfg(not(windows))]
        {
            let _ = name;
            ServiceOperationResult { success: false, message: "仅支持 Windows".to_string() }
        }
    })
    .await
    .unwrap_or_default()
}

/// 启动服务
#[tauri::command]
pub async fn start_service(name: String) -> ServiceOperationResult {
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            let output = std::process::Command::new("sc")
                .args(["start", &name])
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        ServiceOperationResult {
                            success: true,
                            message: format!("服务 {} 已启动", name),
                        }
                    } else {
                        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
                        ServiceOperationResult {
                            success: false,
                            message: format!("启动失败: {}", stderr.trim()),
                        }
                    }
                }
                Err(e) => ServiceOperationResult {
                    success: false,
                    message: format!("执行失败: {}", e),
                },
            }
        }
        #[cfg(not(windows))]
        {
            let _ = name;
            ServiceOperationResult { success: false, message: "仅支持 Windows".to_string() }
        }
    })
    .await
    .unwrap_or_default()
}

/// 停止服务
#[tauri::command]
pub async fn stop_service(name: String) -> ServiceOperationResult {
    tokio::task::spawn_blocking(move || {
        #[cfg(windows)]
        {
            let output = std::process::Command::new("sc")
                .args(["stop", &name])
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        ServiceOperationResult {
                            success: true,
                            message: format!("服务 {} 已停止", name),
                        }
                    } else {
                        let (stderr, _, _) = encoding_rs::GBK.decode(&output.stderr);
                        ServiceOperationResult {
                            success: false,
                            message: format!("停止失败: {}", stderr.trim()),
                        }
                    }
                }
                Err(e) => ServiceOperationResult {
                    success: false,
                    message: format!("执行失败: {}", e),
                },
            }
        }
        #[cfg(not(windows))]
        {
            let _ = name;
            ServiceOperationResult { success: false, message: "仅支持 Windows".to_string() }
        }
    })
    .await
    .unwrap_or_default()
}

/// 对服务进行分类，判断是否安全禁用
fn classify_service(name: &str, display_name: &str) -> (bool, String) {
    let name_lower = name.to_lowercase();
    let display_lower = display_name.to_lowercase();

    // 系统关键服务 - 绝对不能禁用
    let critical_keywords = [
        "windows", "system", "security", "network", "audio", "plug",
        "rpc", "dcom", "eventlog", "schedule", "winmgmt", "cryptography",
        "base", "filter", "engine", "power", "time", "dhcp", "dnsclient",
        "lanman", "server", "workstation", "browser", "firewall",
        "defender", "securityhealth", "wdf", "pci", "acpi", "battery",
        "display", "mouse", "keyboard", "usb", "hid", "bluetooth",
        "print", "spooler", "upnp", "ssdpsrv",
    ];

    for kw in &critical_keywords {
        if name_lower.contains(kw) || display_lower.contains(kw) {
            return (false, "系统关键".to_string());
        }
    }

    // 可安全禁用的常见服务
    let safe_services = [
        ("diagnostictroubleshootingsvc", "诊断", "可优化"),
        ("diagsvc", "诊断", "可优化"),
        ("mapsbroker", "地图服务", "可优化"),
        ("retaildemo", "零售演示", "可优化"),
        ("wiselogin", "WiFi 感知", "可优化"),
        ("wiservice", "WiFi 感知", "可优化"),
        ("lfssvc", "地理位置", "可优化"),
        ("walletservice", "钱包服务", "可优化"),
        ("phonesvc", "电话服务", "可优化"),
        ("sensrsvc", "传感器", "可优化"),
        ("scardsvr", "智能卡", "可优化"),
        ("scardenu", "智能卡", "可优化"),
        ("fax", "传真", "可优化"),
        ("wercplsupport", "错误报告", "可优化"),
        ("wersvc", "错误报告", "可优化"),
        ("dmwappushsvc", "WAP 推送", "可优化"),
        ("dps", "诊断策略", "可优化"),
        ("trksvr", "分布式链接跟踪", "可优化"),
        ("trkwks", "分布式链接跟踪", "可优化"),
        ("cdpsvc", "连接设备平台", "可优化"),
        ("cdpusersvc", "连接设备平台", "可优化"),
        ("bthserv", "蓝牙(未使用时)", "可优化"),
        ("permission", "权限管理", "可优化"),
        ("authtokensvc", "认证令牌", "可优化"),
        ("tokenbroker", "令牌代理", "可优化"),
        ("cloudidsvc", "云身份", "可优化"),
        ("fhsvc", "文件历史", "可优化"),
        ("browser", "计算机浏览器", "可优化"),
        ("iisadmin", "IIS 管理", "可优化"),
        ("w3svc", "Web 发布", "可优化"),
        ("mysql", "MySQL", "可优化"),
        ("postgresql", "PostgreSQL", "可优化"),
        ("tomcat", "Tomcat", "可优化"),
        ("nginx", "Nginx", "可优化"),
        ("redis", "Redis", "可优化"),
        ("mongodb", "MongoDB", "可优化"),
    ];

    for (keyword, desc, cat) in &safe_services {
        if name_lower.contains(keyword) {
            return (true, format!("{} - {}", cat, desc));
        }
    }

    // 第三方软件服务
    let third_party_keywords = [
        ("adobe", "Adobe"), ("google", "Google 更新"), ("alibaba", "阿里"),
        ("tencent", "腾讯"), ("baidu", "百度"), ("360", "360"),
        ("kingsoft", "金山"), ("sogou", "搜狗"), ("nutstore", "坚果云"),
        ("everything", "Everything"), ("sunlogin", "向日葵"),
        ("teamviewer", "TeamViewer"), ("anydesk", "AnyDesk"),
        ("nvidia", "NVIDIA"), ("amdryzen", "AMD"), ("razer", "雷蛇"),
        ("corsair", "海盗船"), ("icue", "iCUE"), ("steam", "Steam"),
        ("epic", "Epic"), ("origin", "Origin"), ("uplay", "Uplay"),
        ("launcher", "启动器"), ("update", "更新服务"),
    ];

    for (keyword, desc) in &third_party_keywords {
        if name_lower.contains(keyword) || display_lower.contains(keyword) {
            return (true, format!("第三方 - {}", desc));
        }
    }

    // 默认分类
    (false, "其他".to_string())
}
