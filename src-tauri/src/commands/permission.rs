use serde::Serialize;
use std::process::Command;

/// 权限状态信息
#[derive(Serialize, Clone, Debug)]
pub struct PermissionStatus {
    pub is_admin: bool,
    pub elevation_level: String,       // "elevated" / "normal" / "unknown"
    pub user_name: String,
    pub integrity_level: String,      // "High" / "Medium" / "Low" / "Unknown"
    pub warnings: Vec<String>,        // 需要管理员权限才能完成的功能列表
    pub can_elevate: bool,            // 是否可以请求提权
    pub recommendation: String,
}

/// 检测当前是否以管理员身份运行
#[tauri::command]
pub fn check_admin_status() -> PermissionStatus {
    #[cfg(windows)]
    {
        let is_admin = check_windows_admin();

        // 获取用户名
        let user_name = std::env::var("USERNAME").unwrap_or_else(|_| "未知".to_string());

        // 检查完整性级别
        let integrity_level = check_integrity_level();

        let elevation_level = if is_admin {
            "elevated".to_string()
        } else if integrity_level == "High" {
            "elevated".to_string()
        } else {
            "normal".to_string()
        };

        // 枚举需要管理员权限的功能
        let mut warnings = Vec::new();
        if !is_admin {
            warnings.push("系统修复 (SFC/DISM/CHKDSK) 无法运行".to_string());
            warnings.push("磁盘碎片整理功能受限".to_string());
            warnings.push("WHEA 硬件错误日志读取可能不完整".to_string());
            warnings.push("Windows 更新失败记录可能无法访问".to_string());
            warnings.push("CBS.log 系统文件损坏日志读取受限".to_string());
            warnings.push("注册表关键项 (HKLM) 读取受限".to_string());
            warnings.push("电源计划切换无法生效".to_string());
            warnings.push("系统服务启用/禁用无法生效".to_string());
        }

        let recommendation = if is_admin {
            "已获得管理员权限，所有功能均可使用。".to_string()
        } else {
            "建议右键点击程序，选择\"以管理员身份运行\"以获得完整的诊断和修复能力。当前部分诊断可能返回不完整的结果。".to_string()
        };

        PermissionStatus {
            is_admin,
            elevation_level,
            user_name,
            integrity_level,
            warnings,
            can_elevate: true,
            recommendation,
        }
    }
    #[cfg(not(windows))]
    {
        PermissionStatus {
            is_admin: true,
            elevation_level: "unknown".to_string(),
            user_name: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            integrity_level: "Unknown".to_string(),
            warnings: vec![],
            can_elevate: false,
            recommendation: "仅支持 Windows 平台".to_string(),
        }
    }
}

/// 以管理员身份重启程序（UAC 提权）
#[tauri::command]
pub fn request_elevation() -> Result<bool, String> {
    #[cfg(windows)]
    {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("无法获取当前程序路径: {}", e))?;

        // 使用 ShellExecuteW 触发 UAC 提权弹窗
        use std::os::windows::ffi::OsStrExt;
        let exe_str: Vec<u16> = std::ffi::OsStr::new(&exe_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let verb: Vec<u16> = "runas\0".encode_utf16().collect();

        extern "system" {
            fn ShellExecuteW(
                hwnd: *mut std::ffi::c_void,
                operation: *const u16,
                file: *const u16,
                parameters: *const u16,
                directory: *const u16,
                show_cmd: i32,
            ) -> *mut std::ffi::c_void;
        }

        unsafe {
            let result = ShellExecuteW(
                std::ptr::null_mut(),
                verb.as_ptr(),
                exe_str.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1, // SW_SHOWNORMAL
            );

            // ShellExecuteW 返回值大于 32 表示成功
            let handle_val = result as isize;
            if handle_val > 32 {
                // 成功启动提权实例，当前实例可以退出
                Ok(true)
            } else {
                Err(format!("提权失败，错误码: {}。用户可能拒绝了 UAC 请求。", handle_val))
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("echo");
        Err("仅支持 Windows 平台".to_string())
    }
}

// ========== 内部辅助函数 ==========

#[cfg(windows)]
fn check_windows_admin() -> bool {
    // 方法1：尝试访问需要管理员权限的资源
    // 尝试打开一个需要管理员权限的注册表键
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    // 尝试打开 SAM 子键（只有管理员能读）
    if hklm.open_subkey_with_flags("SAM", KEY_READ).is_ok() {
        return true;
    }

    // 方法2：检查完整性级别（通过 whoami /groups）
    let output = Command::new("whoami")
        .args(["/groups"])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        // 检查是否包含 Mandatory Label\High
        if stdout.contains("S-1-16-12288") || stdout.to_lowercase().contains("high mandatory level") {
            return true;
        }
        // 检查是否在 Administrators 组中
        if stdout.contains("S-1-5-32-544") {
            // 在 Administrators 组中，但完整性级别可能是 Medium（UAC 未提权）
            // 进一步检查完整性级别
            if stdout.contains("S-1-16-12288") {
                return true;
            }
        }
    }

    // 方法3：尝试 net session（需要管理员权限）
    let output = Command::new("net")
        .args(["session"])
        .output();

    if let Ok(output) = output {
        return output.status.success();
    }

    false
}

#[cfg(windows)]
fn check_integrity_level() -> String {
    let output = Command::new("whoami")
        .args(["/groups"])
        .output();

    if let Ok(output) = output {
        let (stdout, _, _) = encoding_rs::UTF_8.decode(&output.stdout);
        let stdout_lower = stdout.to_lowercase();

        if stdout_lower.contains("s-1-16-12288") || stdout_lower.contains("high mandatory level") {
            return "High".to_string();
        }
        if stdout_lower.contains("s-1-16-8192") || stdout_lower.contains("medium mandatory level") {
            return "Medium".to_string();
        }
        if stdout_lower.contains("s-1-16-4096") || stdout_lower.contains("low mandatory level") {
            return "Low".to_string();
        }
    }
    "Unknown".to_string()
}
