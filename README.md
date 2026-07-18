# PC 急诊箱 - 便携式电脑急救工具

基于 **Tauri 2 + Rust + Vue 3** 构建的 Windows 系统维护工具，编译后单个 .exe 文件约 10-15MB，可放入 U 盘随身携带。

## 功能列表

### 概览
- **系统总览**：CPU/内存/磁盘实时监控、硬件信息（主板、内存条品牌、CPU 型号等）

### 清理与加速
- **C 盘清理**：临时文件、Windows 更新缓存、Prefetch、错误报告、浏览器缓存、回收站、DNS 刷新
- **开机加速**：注册表启动项、启动文件夹、计划任务管理
- **一键加速**：一键清理内存、关闭非必要进程（带确认对话框）
- **进程管理**：查看/结束进程，按 CPU/内存排序

### 磁盘
- **大文件扫描**：层级式磁盘浏览器
  - 第一级：列出所有磁盘（C:/D:/E:）及使用率
  - 逐级深入：文件夹和文件按大小降序排列
  - 面包屑导航、返回上级
  - 文件/文件夹路径一键复制
  - 只读浏览，不进行删除/修改操作
- **碎片整理**：分析和整理磁盘碎片、SSD TRIM

### 诊断与修复
- **硬件故障诊断**：WHEA 错误、设备管理器问题代码、SMART 磁盘健康、电池状态、内存诊断启动
- **驱动冲突诊断**：问题设备检测、版本冲突、驱动加载失败、未签名驱动、GPU 驱动年龄检查
- **系统损坏分析**：SFC CBS.log 检查、关键注册表验证、启动配置、失败更新、激活状态、系统还原点
- **蓝屏诊断**：Minidump 解析、事件日志、20+ 蓝屏代码中文解释
- **驱动检查**：已安装驱动列表、数字签名验证
- **网络诊断**：Ping、DNS、Traceroute、网络适配器信息
- **服务管理**：Windows 服务启停/启用/禁用
- **电源管理**：电源计划切换、CPU 节流信息
- **系统修复**：SFC、DISM、CHKDSK、磁盘 SMART 健康

### 全局功能
- **管理员权限检测**：自动检测并以 UAC 提权重启
- **诊断报告导出**：HTML/TXT 格式导出硬件/驱动/系统诊断报告

## 构建方法

### 前提条件 (Windows 上构建)
1. 安装 [Rust](https://rustup.rs/)
2. 安装 [Node.js](https://nodejs.org/) 18+
3. 安装 [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
4. 安装 [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)

### 构建步骤

```bash
# 1. 克隆仓库
git clone https://github.com/你的用户名/pc-doctor.git
cd pc-doctor

# 2. 安装前端依赖
npm install

# 3. 开发模式运行
npm run tauri dev

# 4. 构建生产版本 (生成 .exe)
npm run tauri build
```

构建完成后，在 `src-tauri/target/release/` 目录下会生成 `PC急诊箱.exe`，
这个文件就是**绿色便携版**，可以直接复制到 U 盘使用。

### 便携使用
- 将 `PC急诊箱.exe` 复制到 U 盘
- 在任意 Windows 电脑上双击运行，无需安装
- 建议右键"以管理员身份运行"以使用 SFC/DISM 等修复功能

## 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 前端 | Vue 3 + Vite | SPA 界面 |
| 桌面框架 | Tauri 2 | 系统原生 WebView，不内嵌 Chromium |
| 后端 | Rust | 高性能、内存安全 |
| 系统信息 | sysinfo crate | CPU/内存/进程 |
| Windows API | windows crate | 磁盘/注册表/Shell |
| 注册表 | winreg crate | 启动项读写 |
| 编码解码 | encoding_rs | GBK 输出解码 |
| 目录遍历 | walkdir crate | 磁盘空间分析 |
| 时间处理 | chrono crate | 时间戳格式化 |

## 项目结构

```
pc-doctor/
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── lib.rs           # Tauri 应用入口 + 命令注册
│   │   ├── main.rs          # 主程序入口
│   │   └── commands/        # 功能模块 (21 个)
│   │       ├── system_info.rs          # 系统信息
│   │       ├── hardware_info.rs        # 硬件信息
│   │       ├── disk_cleanup.rs         # C盘清理
│   │       ├── startup_manager.rs      # 开机启动项
│   │       ├── one_click_boost.rs      # 一键加速
│   │       ├── bsod_analyzer.rs        # 蓝屏诊断
│   │       ├── process_manager.rs      # 进程管理
│   │       ├── disk_analysis.rs        # 大文件扫描(旧)
│   │       ├── disk_explorer.rs        # 层级式磁盘浏览器
│   │       ├── disk_defrag.rs          # 碎片整理
│   │       ├── system_repair.rs        # 系统修复
│   │       ├── service_manager.rs      # 服务管理
│   │       ├── network_diagnostics.rs  # 网络诊断
│   │       ├── driver_check.rs         # 驱动检查
│   │       ├── power_plan.rs           # 电源管理
│   │       ├── hardware_diagnostics.rs # 硬件故障诊断
│   │       ├── driver_conflict.rs      # 驱动冲突诊断
│   │       ├── system_health.rs        # 系统损坏分析
│   │       ├── permission.rs           # 管理员权限检测
│   │       └── report_export.rs       # 诊断报告导出
│   ├── Cargo.toml           # Rust 依赖
│   └── tauri.conf.json      # Tauri 配置
├── src/                     # Vue 3 前端
│   ├── App.vue              # 主组件(侧边栏导航)
│   ├── components/          # 功能页面 (16 个)
│   │   ├── Dashboard.vue
│   │   ├── DiskCleanup.vue
│   │   ├── StartupManager.vue
│   │   ├── OneClickBoost.vue
│   │   ├── BsodAnalyzer.vue
│   │   ├── ProcessManager.vue
│   │   ├── DiskAnalysis.vue
│   │   ├── DiskDefrag.vue
│   │   ├── ServiceManager.vue
│   │   ├── NetworkDiagnostics.vue
│   │   ├── DriverCheck.vue
│   │   ├── PowerPlan.vue
│   │   ├── SystemRepair.vue
│   │   ├── HardwareDiagnostics.vue
│   │   ├── DriverConflict.vue
│   │   ├── SystemHealth.vue
│   │   └── Icon.vue          # SVG 图标组件
│   └── styles/
│       └── main.css         # 全局样式(深色主题)
├── package.json
└── vite.config.js
```

## 安全说明

- 所有危险操作均有二次确认
- 一键加速前显示影响范围确认对话框
- 磁盘浏览器为只读模式，不提供删除/修改/复制/粘贴操作
- 注册表操作仅移动值位置，不删除数据
- SFC/DISM 需管理员权限，应用会自动检测并提示提权

## 许可证

MIT License
