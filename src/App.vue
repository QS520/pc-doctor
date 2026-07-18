<template>
  <div class="app">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2v8" />
            <path d="m4.93 10.93 1.41 1.41" />
            <path d="M2 18h2" />
            <path d="M20 18h2" />
            <path d="m19.07 10.93-1.41 1.41" />
            <path d="M22 22H2" />
            <path d="m8 22 4-10 4 10" />
          </svg>
        </div>
        <div class="brand-text">
          <span class="brand-name">PC 急诊箱</span>
          <span class="brand-version">v1.0 · 便携版</span>
        </div>
      </div>

      <div class="nav-section" v-for="(group, gi) in navGroups" :key="gi">
        <div class="nav-section-label">{{ group.label }}</div>
        <button
          v-for="item in group.items"
          :key="item.id"
          :class="['nav-item', { active: activeView === item.id }]"
          @click="activeView = item.id"
        >
          <Icon :name="item.icon" :size="15" :stroke-width="1.75" />
          <span class="nav-label">{{ item.label }}</span>
        </button>
      </div>

      <div class="sidebar-foot">
        <!-- 主题切换 -->
        <div class="theme-switcher">
          <button
            :class="['theme-btn', { active: themeMode === 'dark' }]"
            title="深色主题"
            @click="setTheme('dark')"
          >
            <Icon name="moon" :size="13" :stroke-width="2" />
          </button>
          <button
            :class="['theme-btn', { active: themeMode === 'light' }]"
            title="浅色主题"
            @click="setTheme('light')"
          >
            <Icon name="sun" :size="13" :stroke-width="2" />
          </button>
          <div class="theme-divider"></div>
          <button
            v-for="color in accentColors"
            :key="color.id"
            :class="['accent-btn', { active: accentColor === color.id }]"
            :style="{ background: color.preview }"
            :title="color.label"
            @click="setAccent(color.id)"
          ></button>
        </div>

        <div class="perm-status" v-if="permissionChecked">
          <span :class="['dot', isAdmin ? 'dot-success' : 'dot-warning']"></span>
          <span class="perm-text">{{ isAdmin ? '管理员权限' : '需要提权' }}</span>
        </div>
        <button class="elevate-btn" v-if="permissionChecked && !isAdmin" @click="requestElevation">
          <Icon name="shield" :size="11" :stroke-width="2" />
          <span>以管理员重启</span>
        </button>
        <div class="port-tag">
          <Icon name="shield" :size="11" :stroke-width="2" />
          <span>USB 便携</span>
        </div>
      </div>
    </aside>

    <main class="content">
      <component :is="currentComponent" @navigate="handleNavigate" />
    </main>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./components/Icon.vue";
import Dashboard from "./components/Dashboard.vue";
import DiskCleanup from "./components/DiskCleanup.vue";
import StartupManager from "./components/StartupManager.vue";
import OneClickBoost from "./components/OneClickBoost.vue";
import BsodAnalyzer from "./components/BsodAnalyzer.vue";
import ProcessManager from "./components/ProcessManager.vue";
import DiskAnalysis from "./components/DiskAnalysis.vue";
import DiskDefrag from "./components/DiskDefrag.vue";
import ServiceManager from "./components/ServiceManager.vue";
import NetworkDiagnostics from "./components/NetworkDiagnostics.vue";
import DriverCheck from "./components/DriverCheck.vue";
import PowerPlan from "./components/PowerPlan.vue";
import SystemRepair from "./components/SystemRepair.vue";
import HardwareDiagnostics from "./components/HardwareDiagnostics.vue";
import DriverConflict from "./components/DriverConflict.vue";
import SystemHealth from "./components/SystemHealth.vue";

const navGroups = [
  {
    label: "概览",
    items: [{ id: "dashboard", label: "系统总览", icon: "dashboard" }],
  },
  {
    label: "清理与加速",
    items: [
      { id: "cleanup", label: "C盘清理", icon: "broom" },
      { id: "startup", label: "开机加速", icon: "rocket" },
      { id: "boost", label: "一键加速", icon: "zap" },
      { id: "process", label: "进程管理", icon: "cpu" },
    ],
  },
  {
    label: "磁盘",
    items: [
      { id: "disk", label: "大文件扫描", icon: "folder" },
      { id: "defrag", label: "碎片整理", icon: "disc" },
    ],
  },
  {
    label: "诊断与修复",
    items: [
      { id: "hwdiag", label: "硬件故障诊断", icon: "alert" },
      { id: "drvconflict", label: "驱动冲突", icon: "bug" },
      { id: "syshealth", label: "系统损坏分析", icon: "shield" },
      { id: "bsod", label: "蓝屏诊断", icon: "bug" },
      { id: "drivers", label: "驱动检查", icon: "monitor" },
      { id: "network", label: "网络诊断", icon: "globe" },
      { id: "services", label: "服务管理", icon: "settings" },
      { id: "power", label: "电源管理", icon: "battery" },
      { id: "repair", label: "系统修复", icon: "wrench" },
    ],
  },
];

const activeView = ref("dashboard");

// ===== 主题管理 =====
const themeMode = ref("dark");
const accentColor = ref("teal");

const accentColors = [
  { id: "teal", label: "青色", preview: "#2dd4bf" },
  { id: "blue", label: "蓝色", preview: "#3b82f6" },
  { id: "purple", label: "紫色", preview: "#a855f7" },
  { id: "green", label: "绿色", preview: "#22c55e" },
  { id: "orange", label: "橙色", preview: "#f97316" },
  { id: "pink", label: "粉色", preview: "#ec4899" },
];

function setTheme(mode) {
  themeMode.value = mode;
  if (mode === "light") {
    document.documentElement.setAttribute("data-theme", "light");
  } else {
    document.documentElement.removeAttribute("data-theme");
  }
  localStorage.setItem("pc-doctor-theme", mode);
}

function setAccent(color) {
  accentColor.value = color;
  if (color === "teal") {
    document.documentElement.removeAttribute("data-accent");
  } else {
    document.documentElement.setAttribute("data-accent", color);
  }
  localStorage.setItem("pc-doctor-accent", color);
}

function loadTheme() {
  const savedTheme = localStorage.getItem("pc-doctor-theme");
  const savedAccent = localStorage.getItem("pc-doctor-accent");
  if (savedTheme) setTheme(savedTheme);
  if (savedAccent) setAccent(savedAccent);
}

const componentMap = {
  dashboard: Dashboard,
  cleanup: DiskCleanup,
  startup: StartupManager,
  boost: OneClickBoost,
  hwdiag: HardwareDiagnostics,
  drvconflict: DriverConflict,
  syshealth: SystemHealth,
  bsod: BsodAnalyzer,
  process: ProcessManager,
  disk: DiskAnalysis,
  defrag: DiskDefrag,
  services: ServiceManager,
  network: NetworkDiagnostics,
  drivers: DriverCheck,
  power: PowerPlan,
  repair: SystemRepair,
};

const currentComponent = computed(() => componentMap[activeView.value]);

// 管理员权限检测
const isAdmin = ref(true);
const permissionChecked = ref(false);

async function checkPermission() {
  try {
    const status = await invoke("check_admin_status");
    isAdmin.value = status.is_admin;
  } catch (e) {
    console.error("Permission check failed:", e);
    isAdmin.value = false;
  }
  permissionChecked.value = true;
}

async function requestElevation() {
  try {
    const result = await invoke("request_elevation");
    if (result) {
      // 提权实例已启动，当前实例可退出
      window.close();
    }
  } catch (e) {
    alert("提权失败: " + String(e));
  }
}

onMounted(() => {
  loadTheme();
  checkPermission();
});

function handleNavigate(view) {
  activeView.value = view;
}
</script>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  width: 100vw;
}

.sidebar {
  width: 208px;
  flex-shrink: 0;
  background: var(--bg-surface);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 14px 0;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 14px 14px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 8px;
}

.brand-mark {
  width: 30px;
  height: 30px;
  border-radius: 6px;
  background: linear-gradient(135deg, var(--accent), #14b8a6);
  color: #04201d;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.brand-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.brand-version {
  font-size: 10px;
  color: var(--text-muted);
  margin-top: 2px;
  font-family: var(--font-mono);
}

.nav-section {
  padding: 4px 8px;
}

.nav-section-label {
  font-size: 9.5px;
  font-weight: 600;
  color: var(--text-faint);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  padding: 8px 8px 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 12.5px;
  font-weight: 500;
  text-align: left;
  width: 100%;
  margin-bottom: 1px;
}

.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-dim);
  color: var(--accent);
}

.nav-item.active .icon {
  color: var(--accent);
}

.sidebar-foot {
  margin-top: auto;
  padding: 12px 14px 4px;
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 主题切换器 */
.theme-switcher {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 6px;
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  margin-bottom: 4px;
}

.theme-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 5px;
  color: var(--text-muted);
  transition: all 0.15s ease;
}

.theme-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.theme-btn.active {
  background: var(--accent-dim);
  color: var(--accent);
}

.theme-divider {
  width: 1px;
  height: 16px;
  background: var(--border);
  margin: 0 3px;
}

.accent-btn {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.accent-btn:hover {
  transform: scale(1.15);
}

.accent-btn.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 2px var(--bg-surface);
}

.perm-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10.5px;
  color: var(--text-secondary);
}

.perm-text {
  font-weight: 500;
}

.elevate-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 5px 8px;
  border-radius: var(--radius-sm);
  background: var(--warning-dim);
  border: 1px solid var(--warning);
  color: var(--warning);
  font-size: 10.5px;
  font-weight: 500;
}

.elevate-btn:hover {
  background: rgba(210, 153, 34, 0.2);
}

.port-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  border-radius: 3px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 10px;
  font-family: var(--font-mono);
  letter-spacing: 0.03em;
}

.content {
  flex: 1;
  overflow-y: auto;
  padding: 22px 28px;
  background: var(--bg-base);
}
</style>
