mod commands;

use commands::{
    disk_analysis, disk_cleanup, disk_defrag, disk_explorer, driver_check, hardware_info,
    network_diagnostics, one_click_boost, power_plan, process_manager,
    service_manager, startup_manager, system_info, system_repair, bsod_analyzer,
    hardware_diagnostics, driver_conflict, system_health, permission, report_export,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // === 系统信息 ===
            system_info::get_system_info,
            system_info::get_cpu_temperature,
            // === 硬件信息 ===
            hardware_info::get_hardware_info,
            // === 一键加速 ===
            one_click_boost::one_click_boost,
            // === C盘清理 ===
            disk_cleanup::scan_junk_files,
            disk_cleanup::clean_junk_files,
            disk_cleanup::empty_recycle_bin,
            disk_cleanup::flush_dns_cache,
            disk_cleanup::get_disk_space,
            // === 开机优化 ===
            startup_manager::get_startup_items,
            startup_manager::disable_startup_item,
            startup_manager::enable_startup_item,
            startup_manager::get_boot_duration,
            // === 蓝屏诊断 ===
            bsod_analyzer::analyze_bsod,
            bsod_analyzer::get_system_errors,
            // === 进程管理 ===
            process_manager::get_processes,
            process_manager::kill_process,
            // === 磁盘分析 ===
            disk_analysis::scan_large_files,
            disk_analysis::scan_directory_sizes,
            // === 系统修复 ===
            system_repair::run_sfc,
            system_repair::run_dism,
            system_repair::run_chkdsk,
            system_repair::check_disk_health,
            // === 磁盘碎片整理 ===
            disk_defrag::analyze_defrag,
            disk_defrag::run_defrag,
            disk_defrag::run_trim_all,
            // === 系统服务管理 ===
            service_manager::get_services,
            service_manager::disable_service,
            service_manager::enable_service,
            service_manager::start_service,
            service_manager::stop_service,
            // === 网络诊断 ===
            network_diagnostics::ping_test,
            network_diagnostics::dns_test,
            network_diagnostics::traceroute,
            network_diagnostics::get_network_info,
            // === 驱动检查 ===
            driver_check::check_drivers,
            // === 电源计划 ===
            power_plan::get_power_plans,
            power_plan::set_power_plan,
            power_plan::get_cpu_throttle_info,
            power_plan::set_cpu_max_state,
            // === 硬件故障深度诊断 ===
            hardware_diagnostics::diagnose_hardware,
            hardware_diagnostics::launch_memory_diagnostic,
            // === 驱动冲突诊断 ===
            driver_conflict::diagnose_driver_conflicts,
            // === 系统损坏深度分析 ===
            system_health::diagnose_system_health,
            // === 管理员权限检测 ===
            permission::check_admin_status,
            permission::request_elevation,
            // === 诊断报告导出 ===
            report_export::export_report,
            report_export::export_report_text,
            // === 磁盘空间分析（层级式） ===
            disk_explorer::list_drives,
            disk_explorer::scan_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
