use serde::{Serialize, Deserialize};
use std::fs;
use std::io::Write;
use chrono::Local;

/// 导出的报告类型
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExportRequest {
    pub report_type: String,    // "hardware" / "driver" / "system" / "full"
    pub data: serde_json::Value, // 原始诊断数据
}

/// 导出结果
#[derive(Serialize, Clone, Debug)]
pub struct ExportResult {
    pub success: bool,
    pub file_path: String,
    pub file_size_kb: f64,
    pub format: String,         // "html" / "txt"
    pub message: String,
}

/// 导出诊断报告为 HTML 文件
#[tauri::command]
pub fn export_report(
    report_type: String,
    data: serde_json::Value,
    output_path: Option<String>,
) -> Result<ExportResult, String> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let default_name = format!("pc-doctor-report_{}_{}.html", report_type, timestamp);

    let path = output_path.unwrap_or_else(|| {
        // 默认保存到桌面
        #[cfg(windows)]
        {
            let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Public".to_string());
            format!("{}\\Desktop\\{}", user_profile, default_name)
        }
        #[cfg(not(windows))]
        {
            format!("/tmp/{}", default_name)
        }
    });

    let (title, html) = match report_type.as_str() {
        "hardware" => generate_hardware_report_html(&data, &timestamp),
        "driver" => generate_driver_report_html(&data, &timestamp),
        "system" => generate_system_report_html(&data, &timestamp),
        "full" => generate_full_report_html(&data, &timestamp),
        _ => return Err(format!("未知报告类型: {}", report_type)),
    };

    let _ = title; // 标题已嵌入 HTML

    // 写入文件
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("创建文件失败: {} - {}", path, e))?;

    file.write_all(html.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let metadata = fs::metadata(&path)
        .map_err(|e| format!("获取文件信息失败: {}", e))?;

    let size_kb = (metadata.len() as f64) / 1024.0;

    Ok(ExportResult {
        success: true,
        file_path: path.clone(),
        file_size_kb: (size_kb * 100.0).round() / 100.0,
        format: "html".to_string(),
        message: format!("报告已导出到: {}（{} KB）", path, (size_kb * 100.0).round() / 100.0),
    })
}

/// 导出纯文本报告
#[tauri::command]
pub fn export_report_text(
    report_type: String,
    data: serde_json::Value,
    output_path: Option<String>,
) -> Result<ExportResult, String> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let clean_ts = timestamp.replace(':', "_").replace(' ', "_");
    let default_name = format!("pc-doctor-report_{}_{}.txt", report_type, clean_ts);

    let path = output_path.unwrap_or_else(|| {
        #[cfg(windows)]
        {
            let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Public".to_string());
            format!("{}\\Desktop\\{}", user_profile, default_name)
        }
        #[cfg(not(windows))]
        {
            format!("/tmp/{}", default_name)
        }
    });

    let content = match report_type.as_str() {
        "hardware" => generate_hardware_report_text(&data, &timestamp),
        "driver" => generate_driver_report_text(&data, &timestamp),
        "system" => generate_system_report_text(&data, &timestamp),
        _ => return Err(format!("未知报告类型: {}", report_type)),
    };

    let mut file = fs::File::create(&path)
        .map_err(|e| format!("创建文件失败: {} - {}", path, e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    let metadata = fs::metadata(&path)
        .map_err(|e| format!("获取文件信息失败: {}", e))?;

    let size_kb = (metadata.len() as f64) / 1024.0;

    Ok(ExportResult {
        success: true,
        file_path: path.clone(),
        file_size_kb: (size_kb * 100.0).round() / 100.0,
        format: "txt".to_string(),
        message: format!("报告已导出到: {}（{} KB）", path, (size_kb * 100.0).round() / 100.0),
    })
}

// ========== HTML 报告生成 ==========

fn generate_hardware_report_html(data: &serde_json::Value, timestamp: &str) -> (String, String) {
    let title = "硬件故障诊断报告";

    let overall_status = data.get("overall_status").and_then(|v| v.as_str()).unwrap_or("unknown");
    let summary = data.get("summary").and_then(|v| v.as_str()).unwrap_or("");
    let findings = data.get("findings").and_then(|v| v.as_array());
    let problem_devices = data.get("problem_devices").and_then(|v| v.as_array());
    let whea_errors = data.get("whea_errors").and_then(|v| v.as_array());
    let smart_attrs = data.get("smart_attributes").and_then(|v| v.as_array());

    let status_color = match overall_status {
        "healthy" => "#3fb950",
        "warnings" => "#d29922",
        "critical" => "#f85149",
        _ => "#8b949e",
    };
    let status_text = match overall_status {
        "healthy" => "健康",
        "warnings" => "有警告",
        "critical" => "严重",
        _ => "未知",
    };

    let mut findings_html = String::new();
    if let Some(findings) = findings {
        for f in findings {
            let sev = f.get("severity").and_then(|v| v.as_str()).unwrap_or("info");
            let component = f.get("component").and_then(|v| v.as_str()).unwrap_or("");
            let ftitle = f.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let desc = f.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let rec = f.get("recommendation").and_then(|v| v.as_str()).unwrap_or("");
            let color = match sev {
                "critical" => "#f85149",
                "warning" => "#d29922",
                _ => "#58a6ff",
            };
            findings_html.push_str(&format!(
                r#"<div class="finding" style="border-left-color: {}">
                    <div class="finding-head"><span class="tag" style="background:{}22;color:{}">{}</span><strong>{}</strong></div>
                    <p class="desc">{}</p>
                    <p class="rec">建议: {}</p>
                </div>"#,
                color, color, color, sev.to_uppercase(), ftitle, desc.replace('\n', "<br>"), rec
            ));
        }
    }

    let mut whea_html = String::new();
    if let Some(errors) = whea_errors {
        if !errors.is_empty() {
            whea_html.push_str("<table><thead><tr><th>时间</th><th>来源</th><th>类型</th><th>严重性</th><th>描述</th></tr></thead><tbody>");
            for e in errors {
                whea_html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    e.get("time_created").and_then(|v| v.as_str()).unwrap_or(""),
                    e.get("error_source").and_then(|v| v.as_str()).unwrap_or(""),
                    e.get("error_type").and_then(|v| v.as_str()).unwrap_or(""),
                    e.get("severity").and_then(|v| v.as_str()).unwrap_or(""),
                    e.get("description").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
            whea_html.push_str("</tbody></table>");
        }
    }

    let mut smart_html = String::new();
    if let Some(attrs) = smart_attrs {
        if !attrs.is_empty() {
            smart_html.push_str("<table><thead><tr><th>盘符</th><th>属性</th><th>原始值</th><th>阈值</th><th>状态</th><th>解读</th></tr></thead><tbody>");
            for a in attrs {
                let status = a.get("status").and_then(|v| v.as_str()).unwrap_or("ok");
                let color = match status {
                    "critical" => "#f85149",
                    "warning" => "#d29922",
                    _ => "#3fb950"
                };
                smart_html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td style='color:{}'>{}</td><td>{}</td></tr>",
                    a.get("drive").and_then(|v| v.as_str()).unwrap_or(""),
                    a.get("attribute_name").and_then(|v| v.as_str()).unwrap_or(""),
                    a.get("raw_value").and_then(|v| v.as_u64()).unwrap_or(0),
                    a.get("threshold").and_then(|v| v.as_u64()).unwrap_or(0),
                    color, status,
                    a.get("interpretation").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
            smart_html.push_str("</tbody></table>");
        }
    }

    let battery_html = if let Some(bat) = data.get("battery") {
        if !bat.is_null() {
            let health = bat.get("health_percent").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let color = if health < 50.0 { "#f85149" } else if health < 80.0 { "#d29922" } else { "#3fb950" };
            format!(
                r#"<div class="card">
                    <h3>电池健康</h3>
                    <div class="battery-health">
                        <div class="health-bar"><div style="width: {}%; background: {}"></div></div>
                        <p>健康度: <span style="color: {}; font-weight: bold">{:.1}%</span></p>
                        <p>设计容量: {} mWh · 实际满充: {} mWh · 循环次数: {}</p>
                        <p>{}</p>
                    </div>
                </div>"#,
                health, color, color, health,
                bat.get("designed_capacity").and_then(|v| v.as_u64()).unwrap_or(0),
                bat.get("full_charge_capacity").and_then(|v| v.as_u64()).unwrap_or(0),
                bat.get("cycle_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                bat.get("interpretation").and_then(|v| v.as_str()).unwrap_or("")
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>{} - {}</title>
<style>
body {{ font-family: -apple-system, "Microsoft YaHei", sans-serif; background: #0d1117; color: #e6edf3; padding: 30px; max-width: 900px; margin: 0 auto; line-height: 1.6; }}
h1 {{ font-size: 22px; border-bottom: 2px solid #2dd4bf; padding-bottom: 10px; }}
h2 {{ font-size: 16px; color: #2dd4bf; margin-top: 30px; }}
h3 {{ font-size: 14px; color: #8b949e; }}
.meta {{ color: #5c6573; font-size: 12px; margin-bottom: 20px; }}
.status {{ display: inline-block; padding: 4px 12px; border-radius: 4px; font-weight: bold; background: {}22; color: {}; }}
.card {{ background: #131a23; border: 1px solid #1c232e; border-radius: 6px; padding: 16px; margin: 12px 0; }}
.finding {{ background: #131a23; border-left: 3px solid; padding: 12px 16px; margin: 8px 0; border-radius: 0 4px 4px 0; }}
.finding-head {{ display: flex; align-items: center; gap: 10px; margin-bottom: 6px; }}
.tag {{ display: inline-block; padding: 1px 6px; border-radius: 3px; font-size: 11px; font-weight: bold; font-family: monospace; }}
.desc {{ font-size: 12px; color: #8b949e; margin: 4px 0; }}
.rec {{ font-size: 12px; color: #2dd4bf; margin: 4px 0; }}
table {{ width: 100%; border-collapse: collapse; margin: 12px 0; font-size: 12px; }}
th {{ text-align: left; padding: 8px; background: #18202b; color: #5c6573; font-size: 11px; text-transform: uppercase; border-bottom: 1px solid #27313e; }}
td {{ padding: 8px; border-bottom: 1px solid #1c232e; }}
.battery-health .health-bar {{ height: 8px; background: #0a0e14; border-radius: 4px; overflow: hidden; margin: 8px 0; }}
.battery-health .health-bar div {{ height: 100%; border-radius: 4px; transition: width 0.3s; }}
.summary {{ font-size: 14px; padding: 12px 16px; background: {}22; border: 1px solid {}44; border-radius: 6px; margin: 12px 0; }}
</style>
</head>
<body>
<h1>硬件故障诊断报告</h1>
<div class="meta">PC 急诊箱 · 生成时间: {}</div>
<div class="summary">
    总体状态: <span class="status">{}</span><br>
    {}
</div>

<h2>诊断发现</h2>
{}

<h2>WHEA 硬件错误记录</h2>
{}

<h2>SMART 磁盘属性</h2>
{}

{}

</body>
</html>"#,
        title, timestamp,
        status_color, status_color,
        status_color, status_color,
        timestamp,
        status_text, summary,
        if findings_html.is_empty() { "<p>未发现硬件问题</p>" } else { &findings_html },
        if whea_html.is_empty() { "<p>未检测到 WHEA 错误记录</p>" } else { &whea_html },
        if smart_html.is_empty() { "<p>未检测到磁盘 SMART 问题</p>" } else { &smart_html },
        battery_html
    );

    (title.to_string(), html)
}

fn generate_driver_report_html(data: &serde_json::Value, timestamp: &str) -> (String, String) {
    let overall_status = data.get("overall_status").and_then(|v| v.as_str()).unwrap_or("unknown");
    let summary = data.get("summary").and_then(|v| v.as_str()).unwrap_or("");
    let conflicts = data.get("conflicts").and_then(|v| v.as_array());
    let version_conflicts = data.get("version_conflicts").and_then(|v| v.as_array());
    let load_failures = data.get("load_failures").and_then(|v| v.as_array());
    let recommendations = data.get("recommendations").and_then(|v| v.as_array());

    let status_color = match overall_status {
        "healthy" => "#3fb950",
        "warnings" => "#d29922",
        "critical" => "#f85149",
        _ => "#8b949e",
    };
    let status_text = match overall_status {
        "healthy" => "健康",
        "warnings" => "有警告",
        "critical" => "严重",
        _ => "未知",
    };

    let mut conflicts_html = String::new();
    if let Some(conflicts) = conflicts {
        for c in conflicts {
            let ct = c.get("conflict_type").and_then(|v| v.as_str()).unwrap_or("unknown");
            let color = match ct {
                "missing_driver" | "driver_corrupt" | "hardware_failure" => "#f85149",
                "resource_conflict" => "#d29922",
                _ => "#58a6ff",
            };
            conflicts_html.push_str(&format!(
                r#"<div class="finding" style="border-left-color: {}">
                    <div class="finding-head"><span class="tag" style="background:{}22;color:{}">{}</span><strong>{}</strong></div>
                    <p class="desc">问题: {}</p>
                    <p class="desc">原因: {}</p>
                    <p class="rec">建议: {}</p>
                </div>"#,
                color, color, color, ct,
                c.get("device_name").and_then(|v| v.as_str()).unwrap_or(""),
                c.get("problem_description").and_then(|v| v.as_str()).unwrap_or(""),
                c.get("probable_cause").and_then(|v| v.as_str()).unwrap_or(""),
                c.get("fix_suggestion").and_then(|v| v.as_str()).unwrap_or("")
            ));
        }
    }

    let mut version_html = String::new();
    if let Some(vc) = version_conflicts {
        for v in vc {
            let devices = v.get("devices").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|d| d.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            let versions = v.get("versions").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|d| d.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            version_html.push_str(&format!(
                "<div class='card'><strong>{}</strong><p class='desc'>设备: {}</p><p class='desc'>版本: {}</p></div>",
                v.get("driver_name").and_then(|v| v.as_str()).unwrap_or(""),
                devices, versions
            ));
        }
    }

    let mut failures_html = String::new();
    if let Some(failures) = load_failures {
        if !failures.is_empty() {
            failures_html.push_str("<table><thead><tr><th>时间</th><th>驱动</th><th>原因</th></tr></thead><tbody>");
            for f in failures {
                failures_html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    f.get("timestamp").and_then(|v| v.as_str()).unwrap_or(""),
                    f.get("driver_name").and_then(|v| v.as_str()).unwrap_or(""),
                    f.get("failure_reason").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
            failures_html.push_str("</tbody></table>");
        }
    }

    let mut rec_html = String::new();
    if let Some(recs) = recommendations {
        for r in recs {
            if let Some(text) = r.as_str() {
                rec_html.push_str(&format!("<div class='card'><span style='color: #2dd4bf'>→</span> {}</div>", text));
            }
        }
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>驱动冲突诊断报告 - {}</title>
<style>
body {{ font-family: -apple-system, "Microsoft YaHei", sans-serif; background: #0d1117; color: #e6edf3; padding: 30px; max-width: 900px; margin: 0 auto; line-height: 1.6; }}
h1 {{ font-size: 22px; border-bottom: 2px solid #2dd4bf; padding-bottom: 10px; }}
h2 {{ font-size: 16px; color: #2dd4bf; margin-top: 30px; }}
.meta {{ color: #5c6573; font-size: 12px; margin-bottom: 20px; }}
.status {{ display: inline-block; padding: 4px 12px; border-radius: 4px; font-weight: bold; background: {}22; color: {}; }}
.card {{ background: #131a23; border: 1px solid #1c232e; border-radius: 6px; padding: 12px 16px; margin: 8px 0; font-size: 12px; }}
.finding {{ background: #131a23; border-left: 3px solid; padding: 12px 16px; margin: 8px 0; border-radius: 0 4px 4px 0; }}
.finding-head {{ display: flex; align-items: center; gap: 10px; margin-bottom: 6px; }}
.tag {{ display: inline-block; padding: 1px 6px; border-radius: 3px; font-size: 11px; font-weight: bold; font-family: monospace; }}
.desc {{ font-size: 12px; color: #8b949e; margin: 4px 0; }}
.rec {{ font-size: 12px; color: #2dd4bf; margin: 4px 0; }}
table {{ width: 100%; border-collapse: collapse; margin: 12px 0; font-size: 12px; }}
th {{ text-align: left; padding: 8px; background: #18202b; color: #5c6573; font-size: 11px; text-transform: uppercase; border-bottom: 1px solid #27313e; }}
td {{ padding: 8px; border-bottom: 1px solid #1c232e; }}
.summary {{ font-size: 14px; padding: 12px 16px; background: {}22; border: 1px solid {}44; border-radius: 6px; margin: 12px 0; }}
</style>
</head>
<body>
<h1>驱动冲突诊断报告</h1>
<div class="meta">PC 急诊箱 · 生成时间: {}</div>
<div class="summary">总体状态: <span class="status">{}</span><br>{}</div>

<h2>驱动冲突</h2>
{}

<h2>版本冲突</h2>
{}

<h2>加载失败记录</h2>
{}

<h2>推荐操作</h2>
{}

</body>
</html>"#,
        timestamp,
        status_color, status_color,
        status_color, status_color,
        timestamp,
        status_text, summary,
        if conflicts_html.is_empty() { "<p>未检测到驱动冲突</p>" } else { &conflicts_html },
        if version_html.is_empty() { "<p>未检测到版本冲突</p>" } else { &version_html },
        if failures_html.is_empty() { "<p>未检测到加载失败</p>" } else { &failures_html },
        if rec_html.is_empty() { "<p>无推荐操作</p>" } else { &rec_html }
    );

    ("驱动冲突诊断报告".to_string(), html)
}

fn generate_system_report_html(data: &serde_json::Value, timestamp: &str) -> (String, String) {
    let overall_status = data.get("overall_status").and_then(|v| v.as_str()).unwrap_or("unknown");
    let summary = data.get("summary").and_then(|v| v.as_str()).unwrap_or("");
    let issues = data.get("issues").and_then(|v| v.as_array());
    let corrupted_files = data.get("corrupted_files").and_then(|v| v.as_array());
    let registry_issues = data.get("registry_issues").and_then(|v| v.as_array());
    let boot_config = data.get("boot_config");
    let failed_updates = data.get("failed_updates").and_then(|v| v.as_array());
    let activation = data.get("activation");
    let recommendations = data.get("recommendations").and_then(|v| v.as_array());

    let status_color = match overall_status {
        "healthy" => "#3fb950",
        "warnings" => "#d29922",
        "critical" => "#f85149",
        _ => "#8b949e",
    };
    let status_text = match overall_status {
        "healthy" => "健康",
        "warnings" => "有警告",
        "critical" => "严重",
        _ => "未知",
    };

    let mut issues_html = String::new();
    if let Some(issues) = issues {
        for i in issues {
            let sev = i.get("severity").and_then(|v| v.as_str()).unwrap_or("info");
            let cat = i.get("category").and_then(|v| v.as_str()).unwrap_or("");
            let color = match sev {
                "critical" => "#f85149",
                "warning" => "#d29922",
                _ => "#58a6ff"
            };
            issues_html.push_str(&format!(
                r#"<div class="finding" style="border-left-color: {}">
                    <div class="finding-head"><span class="tag" style="background:{}22;color:{}">{} · {}</span><strong>{}</strong></div>
                    <p class="desc">{}</p>
                    <p class="rec">建议: {}</p>
                </div>"#,
                color, color, color, cat, sev.to_uppercase(),
                i.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                i.get("description").and_then(|v| v.as_str()).unwrap_or("").replace('\n', "<br>"),
                i.get("recommendation").and_then(|v| v.as_str()).unwrap_or("")
            ));
        }
    }

    let mut files_html = String::new();
    if let Some(files) = corrupted_files {
        if !files.is_empty() {
            files_html.push_str("<table><thead><tr><th>路径</th><th>问题</th><th>可修复</th></tr></thead><tbody>");
            for f in files {
                files_html.push_str(&format!(
                    "<tr><td style='font-family:monospace;font-size:11px'>{}</td><td>{}</td><td>{}</td></tr>",
                    f.get("file_path").and_then(|v| v.as_str()).unwrap_or(""),
                    f.get("issue").and_then(|v| v.as_str()).unwrap_or(""),
                    if f.get("can_repair").and_then(|v| v.as_bool()).unwrap_or(false) { "是" } else { "否" }
                ));
            }
            files_html.push_str("</tbody></table>");
        }
    }

    let mut reg_html = String::new();
    if let Some(regs) = registry_issues {
        if !regs.is_empty() {
            reg_html.push_str("<table><thead><tr><th>Hive</th><th>路径</th><th>问题</th><th>严重性</th></tr></thead><tbody>");
            for r in regs {
                reg_html.push_str(&format!(
                    "<tr><td>{}</td><td style='font-family:monospace;font-size:11px'>{}</td><td>{}</td><td>{}</td></tr>",
                    r.get("hive").and_then(|v| v.as_str()).unwrap_or(""),
                    r.get("key_path").and_then(|v| v.as_str()).unwrap_or(""),
                    r.get("issue").and_then(|v| v.as_str()).unwrap_or(""),
                    r.get("severity").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
            reg_html.push_str("</tbody></table>");
        }
    }

    let mut updates_html = String::new();
    if let Some(updates) = failed_updates {
        if !updates.is_empty() {
            updates_html.push_str("<table><thead><tr><th>KB</th><th>标题</th><th>错误码</th><th>说明</th><th>时间</th></tr></thead><tbody>");
            for u in updates {
                updates_html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td style='font-family:monospace'>{}</td><td>{}</td><td>{}</td></tr>",
                    u.get("kb_number").and_then(|v| v.as_str()).unwrap_or(""),
                    u.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    u.get("error_code").and_then(|v| v.as_str()).unwrap_or(""),
                    u.get("error_description").and_then(|v| v.as_str()).unwrap_or(""),
                    u.get("timestamp").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
            updates_html.push_str("</tbody></table>");
        }
    }

    let boot_html = if let Some(boot) = boot_config {
        let status = boot.get("last_boot_status").and_then(|v| v.as_str()).unwrap_or("unknown");
        let errors = boot.get("boot_errors").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|e| e.as_str()).collect::<Vec<_>>().join("<br>"))
            .unwrap_or_default();
        let safe = boot.get("safe_mode").and_then(|v| v.as_bool()).unwrap_or(false);
        let desc = boot.get("description").and_then(|v| v.as_str()).unwrap_or("");
        format!(
            "<div class='card'><strong>启动状态:</strong> {}<br><strong>安全模式:</strong> {}<br><strong>描述:</strong> {}<br>{}</div>",
            status, if safe { "是" } else { "否" }, desc,
            if errors.is_empty() { String::new() } else { format!("<strong>启动错误:</strong><br>{}", errors) }
        )
    } else {
        String::new()
    };

    let act_html = if let Some(act) = activation {
        let activated = act.get("is_activated").and_then(|v| v.as_bool()).unwrap_or(false);
        let status = act.get("license_status").and_then(|v| v.as_str()).unwrap_or("");
        let desc = act.get("description").and_then(|v| v.as_str()).unwrap_or("");
        format!(
            "<div class='card'><strong>激活状态:</strong> {} ({})<br>{} </div>",
            if activated { "已激活" } else { "未激活" }, status, desc
        )
    } else {
        String::new()
    };

    let mut rec_html = String::new();
    if let Some(recs) = recommendations {
        for r in recs {
            if let Some(text) = r.as_str() {
                rec_html.push_str(&format!("<div class='card'><span style='color: #2dd4bf'>→</span> {}</div>", text));
            }
        }
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>系统损坏深度分析报告 - {}</title>
<style>
body {{ font-family: -apple-system, "Microsoft YaHei", sans-serif; background: #0d1117; color: #e6edf3; padding: 30px; max-width: 900px; margin: 0 auto; line-height: 1.6; }}
h1 {{ font-size: 22px; border-bottom: 2px solid #2dd4bf; padding-bottom: 10px; }}
h2 {{ font-size: 16px; color: #2dd4bf; margin-top: 30px; }}
.meta {{ color: #5c6573; font-size: 12px; margin-bottom: 20px; }}
.status {{ display: inline-block; padding: 4px 12px; border-radius: 4px; font-weight: bold; background: {}22; color: {}; }}
.card {{ background: #131a23; border: 1px solid #1c232e; border-radius: 6px; padding: 12px 16px; margin: 8px 0; font-size: 12px; }}
.finding {{ background: #131a23; border-left: 3px solid; padding: 12px 16px; margin: 8px 0; border-radius: 0 4px 4px 0; }}
.finding-head {{ display: flex; align-items: center; gap: 10px; margin-bottom: 6px; }}
.tag {{ display: inline-block; padding: 1px 6px; border-radius: 3px; font-size: 11px; font-weight: bold; font-family: monospace; }}
.desc {{ font-size: 12px; color: #8b949e; margin: 4px 0; }}
.rec {{ font-size: 12px; color: #2dd4bf; margin: 4px 0; }}
table {{ width: 100%; border-collapse: collapse; margin: 12px 0; font-size: 12px; }}
th {{ text-align: left; padding: 8px; background: #18202b; color: #5c6573; font-size: 11px; text-transform: uppercase; border-bottom: 1px solid #27313e; }}
td {{ padding: 8px; border-bottom: 1px solid #1c232e; }}
.summary {{ font-size: 14px; padding: 12px 16px; background: {}22; border: 1px solid {}44; border-radius: 6px; margin: 12px 0; }}
</style>
</head>
<body>
<h1>系统损坏深度分析报告</h1>
<div class="meta">PC 急诊箱 · 生成时间: {}</div>
<div class="summary">总体状态: <span class="status">{}</span><br>{}</div>

<h2>问题列表</h2>
{}

<h2>损坏的系统文件</h2>
{}

<h2>注册表问题</h2>
{}

<h2>启动配置</h2>
{}

<h2>Windows 更新失败</h2>
{}

<h2>激活状态</h2>
{}

<h2>推荐操作</h2>
{}

</body>
</html>"#,
        timestamp,
        status_color, status_color,
        status_color, status_color,
        timestamp,
        status_text, summary,
        if issues_html.is_empty() { "<p>未检测到系统问题</p>" } else { &issues_html },
        if files_html.is_empty() { "<p>未检测到损坏的系统文件</p>" } else { &files_html },
        if reg_html.is_empty() { "<p>未检测到注册表问题</p>" } else { &reg_html },
        if boot_html.is_empty() { "<p>无法获取启动配置</p>" } else { &boot_html },
        if updates_html.is_empty() { "<p>未检测到失败的更新</p>" } else { &updates_html },
        if act_html.is_empty() { "<p>无法获取激活状态</p>" } else { &act_html },
        if rec_html.is_empty() { "<p>无推荐操作</p>" } else { &rec_html }
    );

    ("系统损坏深度分析报告".to_string(), html)
}

fn generate_full_report_html(data: &serde_json::Value, timestamp: &str) -> (String, String) {
    // 聚合报告
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>PC 急诊箱完整诊断报告 - {}</title>
<style>
body {{ font-family: -apple-system, "Microsoft YaHei", sans-serif; background: #0d1117; color: #e6edf3; padding: 30px; max-width: 900px; margin: 0 auto; line-height: 1.6; }}
h1 {{ font-size: 22px; border-bottom: 2px solid #2dd4bf; padding-bottom: 10px; }}
h2 {{ font-size: 16px; color: #2dd4bf; margin-top: 30px; }}
.meta {{ color: #5c6573; font-size: 12px; margin-bottom: 20px; }}
</style>
</head>
<body>
<h1>PC 急诊箱完整诊断报告</h1>
<div class="meta">生成时间: {}</div>"#,
        timestamp, timestamp
    );

    if let Some(hw) = data.get("hardware") {
        let (_, hw_html) = generate_hardware_report_html(hw, timestamp);
        // 提取 body 内容
        let body_content = extract_body_content(&hw_html);
        html.push_str(&format!("<h2>硬件故障诊断</h2>{}", body_content));
    }

    if let Some(drv) = data.get("driver") {
        let (_, drv_html) = generate_driver_report_html(drv, timestamp);
        let body_content = extract_body_content(&drv_html);
        html.push_str(&format!("<h2>驱动冲突诊断</h2>{}", body_content));
    }

    if let Some(sys) = data.get("system") {
        let (_, sys_html) = generate_system_report_html(sys, timestamp);
        let body_content = extract_body_content(&sys_html);
        html.push_str(&format!("<h2>系统损坏分析</h2>{}", body_content));
    }

    html.push_str("</body></html>");
    ("PC 急诊箱完整诊断报告".to_string(), html)
}

fn extract_body_content(html: &str) -> String {
    if let Some(start) = html.find("<body>") {
        if let Some(end) = html.rfind("</body>") {
            return html[start + 6..end].to_string();
        }
    }
    html.to_string()
}

// ========== 文本报告生成 ==========

fn generate_hardware_report_text(data: &serde_json::Value, timestamp: &str) -> String {
    let mut out = String::new();
    out.push_str("========================================\n");
    out.push_str("        硬件故障诊断报告\n");
    out.push_str("========================================\n");
    out.push_str(&format!("生成时间: {}\n\n", timestamp));

    out.push_str(&format!("总体状态: {}\n", data.get("overall_status").and_then(|v| v.as_str()).unwrap_or("unknown")));
    out.push_str(&format!("{}\n\n", data.get("summary").and_then(|v| v.as_str()).unwrap_or("")));

    if let Some(findings) = data.get("findings").and_then(|v| v.as_array()) {
        out.push_str("【诊断发现】\n");
        for f in findings {
            out.push_str(&format!("  [{}] {} ({})\n",
                f.get("severity").and_then(|v| v.as_str()).unwrap_or(""),
                f.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                f.get("component").and_then(|v| v.as_str()).unwrap_or("")
            ));
            out.push_str(&format!("    描述: {}\n", f.get("description").and_then(|v| v.as_str()).unwrap_or("")));
            out.push_str(&format!("    建议: {}\n\n", f.get("recommendation").and_then(|v| v.as_str()).unwrap_or("")));
        }
    }

    if let Some(errors) = data.get("whea_errors").and_then(|v| v.as_array()) {
        if !errors.is_empty() {
            out.push_str("【WHEA 硬件错误记录】\n");
            for e in errors {
                out.push_str(&format!("  {} [{}] {} - {}\n",
                    e.get("time_created").and_then(|v| v.as_str()).unwrap_or(""),
                    e.get("severity").and_then(|v| v.as_str()).unwrap_or(""),
                    e.get("error_source").and_then(|v| v.as_str()).unwrap_or(""),
                    e.get("error_type").and_then(|v| v.as_str()).unwrap_or("")
                ));
            }
            out.push_str("\n");
        }
    }

    if let Some(bat) = data.get("battery") {
        if !bat.is_null() {
            out.push_str("【电池健康】\n");
            out.push_str(&format!("  健康度: {:.1}%\n", bat.get("health_percent").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32));
            out.push_str(&format!("  设计容量: {} mWh\n", bat.get("designed_capacity").and_then(|v| v.as_u64()).unwrap_or(0)));
            out.push_str(&format!("  实际满充: {} mWh\n", bat.get("full_charge_capacity").and_then(|v| v.as_u64()).unwrap_or(0)));
            out.push_str(&format!("  循环次数: {}\n\n", bat.get("cycle_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32));
        }
    }

    out
}

fn generate_driver_report_text(data: &serde_json::Value, timestamp: &str) -> String {
    let mut out = String::new();
    out.push_str("========================================\n");
    out.push_str("        驱动冲突诊断报告\n");
    out.push_str("========================================\n");
    out.push_str(&format!("生成时间: {}\n\n", timestamp));

    out.push_str(&format!("总体状态: {}\n", data.get("overall_status").and_then(|v| v.as_str()).unwrap_or("unknown")));
    out.push_str(&format!("{}\n\n", data.get("summary").and_then(|v| v.as_str()).unwrap_or("")));

    if let Some(conflicts) = data.get("conflicts").and_then(|v| v.as_array()) {
        out.push_str("【驱动冲突】\n");
        for c in conflicts {
            out.push_str(&format!("  [{}] {}\n",
                c.get("conflict_type").and_then(|v| v.as_str()).unwrap_or(""),
                c.get("device_name").and_then(|v| v.as_str()).unwrap_or("")
            ));
            out.push_str(&format!("    问题: {}\n", c.get("problem_description").and_then(|v| v.as_str()).unwrap_or("")));
            out.push_str(&format!("    原因: {}\n", c.get("probable_cause").and_then(|v| v.as_str()).unwrap_or("")));
            out.push_str(&format!("    建议: {}\n\n", c.get("fix_suggestion").and_then(|v| v.as_str()).unwrap_or("")));
        }
    }

    if let Some(recs) = data.get("recommendations").and_then(|v| v.as_array()) {
        out.push_str("【推荐操作】\n");
        for r in recs {
            if let Some(text) = r.as_str() {
                out.push_str(&format!("  → {}\n", text));
            }
        }
    }

    out
}

fn generate_system_report_text(data: &serde_json::Value, timestamp: &str) -> String {
    let mut out = String::new();
    out.push_str("========================================\n");
    out.push_str("        系统损坏深度分析报告\n");
    out.push_str("========================================\n");
    out.push_str(&format!("生成时间: {}\n\n", timestamp));

    out.push_str(&format!("总体状态: {}\n", data.get("overall_status").and_then(|v| v.as_str()).unwrap_or("unknown")));
    out.push_str(&format!("{}\n\n", data.get("summary").and_then(|v| v.as_str()).unwrap_or("")));

    if let Some(issues) = data.get("issues").and_then(|v| v.as_array()) {
        out.push_str("【问题列表】\n");
        for i in issues {
            out.push_str(&format!("  [{}] {} ({})\n",
                i.get("severity").and_then(|v| v.as_str()).unwrap_or(""),
                i.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                i.get("category").and_then(|v| v.as_str()).unwrap_or("")
            ));
            out.push_str(&format!("    建议: {}\n\n", i.get("recommendation").and_then(|v| v.as_str()).unwrap_or("")));
        }
    }

    if let Some(files) = data.get("corrupted_files").and_then(|v| v.as_array()) {
        if !files.is_empty() {
            out.push_str("【损坏的系统文件】\n");
            for f in files {
                out.push_str(&format!("  {}\n    {}\n", f.get("file_path").and_then(|v| v.as_str()).unwrap_or(""), f.get("issue").and_then(|v| v.as_str()).unwrap_or("")));
            }
            out.push_str("\n");
        }
    }

    if let Some(recs) = data.get("recommendations").and_then(|v| v.as_array()) {
        out.push_str("【推荐操作】\n");
        for r in recs {
            if let Some(text) = r.as_str() {
                out.push_str(&format!("  → {}\n", text));
            }
        }
    }

    out
}
