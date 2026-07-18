use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::process::Command;
use sysinfo::System;

/// 取“上次开机 / 通电使用”时间，兼容 Windows 快速启动（Fast Startup）。
///
/// 快速启动开启时，关机只是把内核休眠，下次通电是“恢复”而非“重启”，
/// 因此 `Win32_OperatingSystem.LastBootUpTime` 不会更新，会停留在上一次“完整重启”——
/// 常被误认为“装机 / 激活时间”。
///
/// 取值优先级（取所有成功结果里最新的一条）：
/// 1. 安全日志 4624 最近一次交互式登录（LogonType 2/10/11）——每次真正通电登录都会记录；
/// 2. 系统日志 1074 最近一次用户发起的关机 / 重启；
/// 3. 系统日志内核启动事件 12（Kernel-General）；
/// 3.5 系统日志 6005 / 6009：事件日志服务启动 / 操作系统启动信息——
///     这两类事件在“每次通电开机（含快速启动的休眠恢复）”都会记录，且无需管理员权限，
///     是捕捉快速启动下真实开机时间最可靠的信号，应优先于 uptime / LastBootUpTime；
/// 4. 当前时间减去系统运行时长（`GetTickCount64` / `System::uptime()`，在快速启动下是准的）；
/// 5. 回退 `Win32_OperatingSystem.LastBootUpTime`（仅当前述均失败时使用）。
pub fn last_boot_time_string() -> String {
    last_boot_datetime()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "未知".to_string())
}

#[cfg(windows)]
pub fn last_boot_datetime() -> Option<DateTime<Local>> {
    let mut candidates: Vec<DateTime<Local>> = Vec::new();

    // 1) 最近一次交互式登录（安全日志 4624）
    if let Some(s) = ps_query(
        "Get-WinEvent -FilterHashtable @{LogName='Security'; Id=4624} -MaxEvents 400 -ErrorAction SilentlyContinue | Where-Object { $_.Properties[7].Value -in @(2,10,11) } | Select-Object -First 1 | ForEach-Object { $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss') }",
    ) {
        if let Some(dt) = parse_local(&s) {
            candidates.push(dt);
        }
    }

    // 2) 最近一次用户发起的关机 / 重启（系统日志 1074）
    if let Some(s) = ps_query(
        "Get-WinEvent -FilterHashtable @{LogName='System'; Id=1074} -MaxEvents 20 -ErrorAction SilentlyContinue | Select-Object -First 1 | ForEach-Object { $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss') }",
    ) {
        if let Some(dt) = parse_local(&s) {
            candidates.push(dt);
        }
    }

    // 3) 内核启动事件 12（系统日志）
    if let Some(s) = ps_query(
        "Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Microsoft-Windows-Kernel-General'; Id=12} -MaxEvents 1 -ErrorAction SilentlyContinue | ForEach-Object { $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss') }",
    ) {
        if let Some(dt) = parse_local(&s) {
            candidates.push(dt);
        }
    }

    // 3.5) 系统日志 6005 / 6009：事件日志服务启动 / 操作系统启动信息
    //      这两类事件在“每次通电开机（含快速启动的休眠恢复）”都会记录，且无需管理员权限，
    //      是捕捉快速启动下真实开机时间最可靠的信号，应当优先于 uptime / LastBootUpTime。
    if let Some(s) = ps_query(
        "Get-WinEvent -FilterHashtable @{LogName='System'; Id=6005,6009} -MaxEvents 1 -ErrorAction SilentlyContinue | ForEach-Object { $_.TimeCreated.ToString('yyyy-MM-dd HH:mm:ss') }",
    ) {
        if let Some(dt) = parse_local(&s) {
            candidates.push(dt);
        }
    }

    // 4) 基于系统运行时长反推（快速启动下 GetTickCount64 会被重置，比 LastBootUpTime 更准）
    let uptime_secs = System::uptime();
    if uptime_secs > 0 {
        candidates.push(Local::now() - chrono::Duration::seconds(uptime_secs as i64));
    }

    // 5) 回退 LastBootUpTime（转本地时区）
    if let Some(s) = ps_query(
        "(Get-CimInstance -ClassName Win32_OperatingSystem).LastBootUpTime | Get-Date -Format 'yyyy-MM-dd HH:mm:ss'",
    ) {
        if let Some(dt) = parse_local(&s) {
            candidates.push(dt);
        }
    }

    candidates.into_iter().max()
}

#[cfg(not(windows))]
pub fn last_boot_datetime() -> Option<DateTime<Local>> {
    None
}

/// 运行一段 PowerShell 脚本，截断空白后返回 stdout；空结果返回 None。
fn ps_query(script: &str) -> Option<String> {
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() || s == "未知" {
        None
    } else {
        Some(s)
    }
}

/// 把 `yyyy-MM-dd HH:mm:ss` 当作本地时间解析为 DateTime<Local>。
fn parse_local(s: &str) -> Option<DateTime<Local>> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()?;
    Local.from_local_datetime(&naive).earliest()
}
