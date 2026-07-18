use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone, Debug)]
pub struct PingResult {
    pub host: String,
    pub success: bool,
    pub packets_sent: u32,
    pub packets_received: u32,
    pub loss_percent: f64,
    pub min_ms: f64,
    pub avg_ms: f64,
    pub max_ms: f64,
    pub raw_output: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DnsTestResult {
    pub domain: String,
    pub dns_server: String,
    pub resolve_time_ms: f64,
    pub resolved_ips: Vec<String>,
    pub success: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct TraceRouteHop {
    pub hop: u32,
    pub address: String,
    pub hostname: String,
    pub times_ms: Vec<f64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct NetworkAdapterInfo {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub ip_address: String,
    pub gateway: String,
    pub dns_servers: Vec<String>,
    pub link_speed_mbps: u64,
    pub status: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct NetworkInfo {
    pub adapters: Vec<NetworkAdapterInfo>,
    pub active_dns: Vec<String>,
}

/// Ping 测试
#[tauri::command]
pub fn ping_test(host: Option<String>) -> PingResult {
    let target = host.unwrap_or_else(|| "8.8.8.8".to_string());

    #[cfg(windows)]
    {
        let output = Command::new("ping")
            .args(["-n", "10", "-w", "2000", &target])
            .output();

        match output {
            Ok(output) => {
                let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
                let raw = stdout.to_string();

                let packets_sent = 10u32;
                let mut packets_received = 0u32;
                let mut loss = 100.0;
                let mut min_ms = 0.0;
                let mut avg_ms = 0.0;
                let mut max_ms = 0.0;

                // 解析丢包率
                if let Some(loss_pos) = raw.find("(") {
                    let rest = &raw[loss_pos..];
                    if let Some(percent_pos) = rest.find("%") {
                        let num_str: String = rest[1..percent_pos]
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '.')
                            .collect();
                        if let Ok(p) = num_str.parse::<f64>() {
                            loss = p;
                            packets_received = ((100.0 - loss) / 100.0 * packets_sent as f64) as u32;
                        }
                    }
                }

                // 解析延迟
                if let Some(eq_pos) = raw.find("=") {
                    let after_eq = &raw[eq_pos..];
                    if let Some(ms_pos) = after_eq.find("ms") {
                        let segment = &after_eq[..ms_pos];
                        let numbers: Vec<f64> = segment
                            .split(|c: char| !c.is_ascii_digit() && c != '.')
                            .filter_map(|s| s.parse::<f64>().ok())
                            .collect();
                        if numbers.len() >= 3 {
                            min_ms = numbers[numbers.len() - 3];
                            avg_ms = numbers[numbers.len() - 2];
                            max_ms = numbers[numbers.len() - 1];
                        }
                    }
                }

                let success = loss < 100.0;

                PingResult {
                    host: target,
                    success,
                    packets_sent,
                    packets_received,
                    loss_percent: loss,
                    min_ms,
                    avg_ms,
                    max_ms,
                    raw_output: raw,
                }
            }
            Err(e) => PingResult {
                host: target,
                success: false,
                packets_sent: 0,
                packets_received: 0,
                loss_percent: 100.0,
                min_ms: 0.0,
                avg_ms: 0.0,
                max_ms: 0.0,
                raw_output: format!("Ping 执行失败: {}", e),
            },
        }
    }
    #[cfg(not(windows))]
    {
        PingResult {
            host: target, success: false, packets_sent: 0, packets_received: 0,
            loss_percent: 100.0, min_ms: 0.0, avg_ms: 0.0, max_ms: 0.0,
            raw_output: "仅支持 Windows".to_string(),
        }
    }
}

/// DNS 解析测速
#[tauri::command]
pub fn dns_test(domain: Option<String>, dns_server: Option<String>) -> DnsTestResult {
    let target_domain = domain.unwrap_or_else(|| "www.baidu.com".to_string());
    let dns = dns_server.unwrap_or_else(|| "默认".to_string());

    #[cfg(windows)]
    {
        let start = std::time::Instant::now();

        let output = Command::new("nslookup")
            .args([&target_domain, &dns])
            .output();

        let elapsed_ms = start.elapsed().as_millis() as f64;

        match output {
            Ok(output) => {
                let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);
                let raw = stdout.to_string();

                let mut ips = Vec::new();
                for line in raw.lines() {
                    let line = line.trim();
                    if line.starts_with("Address:") || line.contains("Address:") {
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let ip = parts[1].trim();
                            if !ip.is_empty()
                                && ip != dns
                                && !ip.starts_with("192.168")
                                && !ip.starts_with("127.")
                            {
                                ips.push(ip.to_string());
                            }
                        }
                    }
                }

                let success = !ips.is_empty();

                DnsTestResult {
                    domain: target_domain,
                    dns_server: dns,
                    resolve_time_ms: elapsed_ms,
                    resolved_ips: ips,
                    success,
                }
            }
            Err(_e) => DnsTestResult {
                domain: target_domain,
                dns_server: dns,
                resolve_time_ms: 0.0,
                resolved_ips: Vec::new(),
                success: false,
            },
        }
    }
    #[cfg(not(windows))]
    {
        DnsTestResult { domain: target_domain, dns_server: dns, resolve_time_ms: 0.0, resolved_ips: Vec::new(), success: false }
    }
}

/// 路由追踪 (tracert)
#[tauri::command]
pub fn traceroute(host: Option<String>, max_hops: Option<u32>) -> Vec<TraceRouteHop> {
    let target = host.unwrap_or_else(|| "8.8.8.8".to_string());
    let hops = max_hops.unwrap_or(15);
    let mut result = Vec::new();

    #[cfg(windows)]
    {
        let output = Command::new("tracert")
            .args(["-d", "-h", &hops.to_string(), "-w", "2000", &target])
            .output();

        if let Ok(output) = output {
            let (stdout, _, _) = encoding_rs::GBK.decode(&output.stdout);

            for line in stdout.lines() {
                let line = line.trim();
                if line.is_empty() || line.contains("追踪") || line.contains("over a maximum") {
                    continue;
                }

                // 解析 tracert 输出格式:  1  <1 ms  <1 ms  <1 ms  192.168.1.1
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    if let Ok(hop_num) = parts[0].parse::<u32>() {
                        let mut times = Vec::new();
                        for i in 1..4 {
                            if parts[i].contains("ms") || parts[i] == "*" || parts[i].contains("<") {
                                let time_str: String = parts[i]
                                    .chars()
                                    .filter(|c| c.is_ascii_digit() || *c == '.')
                                    .collect();
                                if let Ok(t) = time_str.parse::<f64>() {
                                    times.push(t);
                                } else {
                                    times.push(0.0);
                                }
                            }
                        }

                        let address = parts.last().unwrap_or(&"").to_string();

                        result.push(TraceRouteHop {
                            hop: hop_num,
                            address: address.clone(),
                            hostname: address,
                            times_ms: times,
                        });
                    }
                }
            }
        }
    }

    result
}

/// 获取网络适配器信息
#[tauri::command]
pub fn get_network_info() -> NetworkInfo {
    let mut adapters = Vec::new();
    let mut dns_servers = Vec::new();

    #[cfg(windows)]
    {
        let ps_command = r#"
Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -or $_.Status -eq 'Disconnected' } | ForEach-Object {
    $adapter = $_
    $ipConfig = Get-NetIPAddress -InterfaceIndex $adapter.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue
    $dnsConfig = Get-DnsClientServerAddress -InterfaceIndex $adapter.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue
    $gateway = Get-NetRoute -InterfaceIndex $adapter.ifIndex -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Select-Object -First 1

    $ip = if ($ipConfig) { $ipConfig.IPAddress } else { '' }
    $gw = if ($gateway) { $gateway.NextHop } else { '' }
    $dns = if ($dnsConfig) { $dnsConfig.ServerAddresses -join ',' } else { '' }
    $speed = if ($adapter.LinkSpeed) { $adapter.LinkSpeed } else { '0' }

    $line = "$($adapter.Name)|$($adapter.InterfaceDescription)|$($adapter.MacAddress)|$ip|$gw|$dns|$speed|$($adapter.Status)"
    Write-Output $line
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
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 8 {
                    let dns_list: Vec<String> = parts[6]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.trim().to_string())
                        .collect();

                    let speed_str = parts[7].replace("Gbps", "000").replace("Mbps", "").replace("Mbps", "").trim().to_string();
                    let speed: u64 = speed_str.parse().unwrap_or(0);

                    adapters.push(NetworkAdapterInfo {
                        name: parts[0].to_string(),
                        description: parts[1].to_string(),
                        mac_address: parts[2].to_string(),
                        ip_address: parts[3].to_string(),
                        gateway: parts[4].to_string(),
                        dns_servers: dns_list.clone(),
                        link_speed_mbps: speed,
                        status: if parts[7].contains("Up") { "已连接".to_string() } else { "未连接".to_string() },
                    });

                    dns_servers.extend(dns_list);
                }
            }
        }

        // 去重 DNS
        dns_servers.sort();
        dns_servers.dedup();
    }

    NetworkInfo { adapters, active_dns: dns_servers }
}
