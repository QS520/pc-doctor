<template>
  <div class="page fade-in hardware-diagnostics">
    <!-- 1. 页头 -->
    <div class="header">
      <div>
        <h1 class="page-title">硬件故障诊断</h1>
        <p class="page-subtitle">深度检测 CPU、内存、磁盘、GPU、电池及设备管理器中的硬件错误</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-primary btn-sm" @click="runDiagnosis" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          开始诊断
        </button>
        <button class="btn btn-ghost btn-sm" @click="exportReport" :disabled="exportLoading">
          <Icon name="download" :size="13" :stroke-width="2" />
          导出报告
        </button>
      </div>
    </div>

    <!-- 管理员权限提示 -->
    <div v-if="!isAdmin" class="admin-warning">
      <Icon name="alert" :size="14" :stroke-width="2" class="admin-warning-icon" />
      <span>未以管理员身份运行，部分诊断可能不完整。建议关闭后右键"以管理员身份运行"。</span>
    </div>

    <!-- 2. 加载态 -->
    <div v-if="loading" class="loading">
      <div class="spinner" style="width:24px;height:24px"></div>
      <p>正在诊断硬件...</p>
    </div>

    <!-- 3. 诊断结果 -->
    <template v-else-if="report">
      <!-- 顶部状态条 -->
      <div class="card status-bar" :class="statusBarClass">
        <span class="dot big-dot" :class="statusDotClass"></span>
        <div class="status-text">
          <p class="status-label">总体状态</p>
          <p class="status-headline">{{ statusHeadline }}</p>
          <p class="status-summary">{{ report.summary }}</p>
        </div>
      </div>

      <!-- 内存错误提示（置顶高亮） -->
      <div v-if="report.memory_errors_detected" class="card memory-warning">
        <Icon name="memory-stick" :size="16" :stroke-width="1.75" class="mw-icon" />
        <div class="mw-body">
          <p class="mw-title">检测到内存硬件错误</p>
          <p class="mw-desc">系统事件日志中存在内存相关的 WHEA 错误记录，建议使用页面底部的「Windows 内存诊断」进行深度检测。</p>
        </div>
      </div>

      <!-- Findings 列表 -->
      <div v-if="report.findings && report.findings.length > 0" class="card">
        <div class="card-header">
          <span class="title-group">
            <Icon name="bug" :size="13" :stroke-width="1.75" />
            <span class="card-title">诊断发现 · {{ report.findings.length }} 项</span>
          </span>
        </div>
        <div class="card-body">
          <div class="findings-list">
            <div
              v-for="(f, i) in report.findings"
              :key="i"
              class="finding"
              :class="findingSeverityClass(f.severity)"
            >
              <div class="finding-bar"></div>
              <div class="finding-body">
                <div class="finding-head">
                  <span class="dot" :class="findingDotClass(f.severity)"></span>
                  <span class="tag tag-neutral">{{ f.component }}</span>
                  <span class="finding-title">{{ f.title }}</span>
                </div>
                <p class="finding-desc">{{ f.description }}</p>
                <div class="rec-btn">
                  <Icon name="wrench" :size="12" :stroke-width="1.75" class="rec-icon" />
                  <span class="rec-text">{{ f.recommendation }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- WHEA 错误表格 -->
      <div v-if="report.whea_errors && report.whea_errors.length > 0" class="card">
        <div class="card-header">
          <span class="title-group">
            <Icon name="alert" :size="13" :stroke-width="1.75" />
            <span class="card-title">WHEA 硬件错误 · {{ report.whea_errors.length }} 条</span>
          </span>
        </div>
        <div class="card-body table-body">
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>时间</th>
                  <th>来源</th>
                  <th>类型</th>
                  <th>严重性</th>
                  <th>描述</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(err, i) in report.whea_errors" :key="i">
                  <td class="nowrap mono">{{ err.time_created }}</td>
                  <td>{{ err.error_source }}</td>
                  <td class="mono">{{ err.error_type }}</td>
                  <td>
                    <span class="status-inline">
                      <span class="dot" :class="wheaSeverityDot(err.severity)"></span>
                      <span class="level-text">{{ err.severity }}</span>
                    </span>
                  </td>
                  <td class="msg-cell">{{ err.description }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 问题设备表格 -->
      <div v-if="report.problem_devices && report.problem_devices.length > 0" class="card">
        <div class="card-header">
          <span class="title-group">
            <Icon name="plug" :size="13" :stroke-width="1.75" />
            <span class="card-title">问题设备 · {{ report.problem_devices.length }} 个</span>
          </span>
        </div>
        <div class="card-body table-body">
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>设备名称</th>
                  <th>错误码</th>
                  <th>可能原因</th>
                  <th>修复建议</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(d, i) in report.problem_devices" :key="i">
                  <td>
                    <div class="device-cell">
                      <span class="device-name">{{ d.name }}</span>
                      <span class="device-id mono">{{ d.device_id }}</span>
                    </div>
                  </td>
                  <td>
                    <span class="tag tag-danger mono">Code {{ d.problem_code }}</span>
                  </td>
                  <td class="msg-cell">{{ d.problem_description }}<span v-if="d.probable_cause" class="probable-cause"> · {{ d.probable_cause }}</span></td>
                  <td class="msg-cell">{{ d.fix_suggestion }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- SMART 属性表格 -->
      <div v-if="report.smart_attributes && report.smart_attributes.length > 0" class="card">
        <div class="card-header">
          <span class="title-group">
            <Icon name="hard-drive" :size="13" :stroke-width="1.75" />
            <span class="card-title">SMART 属性 · {{ report.smart_attributes.length }} 项</span>
          </span>
        </div>
        <div class="card-body table-body">
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>盘符</th>
                  <th class="num-col">ID</th>
                  <th>属性名</th>
                  <th class="num-col">原始值</th>
                  <th class="num-col">阈值</th>
                  <th>状态</th>
                  <th>解读</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(s, i) in report.smart_attributes" :key="i">
                  <td class="mono">{{ s.drive }}</td>
                  <td class="num-col mono">{{ s.attribute_id }}</td>
                  <td>{{ s.attribute_name }}</td>
                  <td class="num-col mono">{{ s.raw_value }}</td>
                  <td class="num-col mono">{{ s.threshold }}</td>
                  <td>
                    <span class="status-inline">
                      <span class="dot" :class="smartStatusDot(s.status)"></span>
                      <span class="level-text">{{ s.status }}</span>
                    </span>
                  </td>
                  <td class="msg-cell">{{ s.interpretation }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 电池健康卡片 -->
      <div v-if="report.battery" class="card">
        <div class="card-header">
          <span class="title-group">
            <Icon name="battery" :size="13" :stroke-width="1.75" />
            <span class="card-title">电池健康</span>
          </span>
          <span class="status-inline">
            <span class="dot" :class="batteryStatusDot"></span>
            <span class="level-text">{{ report.battery.status }}</span>
          </span>
        </div>
        <div class="card-body">
          <div class="battery-grid">
            <div class="battery-health">
              <div class="health-head">
                <span class="health-label">健康度</span>
                <span class="health-value mono" :class="batteryHealthColor">{{ report.battery.health_percent.toFixed(1) }}%</span>
              </div>
              <div class="bar">
                <div class="bar-fill" :class="batteryBarClass" :style="{ width: report.battery.health_percent + '%' }"></div>
              </div>
              <p class="health-interp">{{ report.battery.interpretation }}</p>
            </div>
            <div class="battery-stats">
              <div class="kv-row">
                <span class="kv-label">设计容量</span>
                <span class="kv-value mono">{{ report.battery.designed_capacity }} mWh</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">当前满充容量</span>
                <span class="kv-value mono">{{ report.battery.full_charge_capacity }} mWh</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">充放电循环</span>
                <span class="kv-value mono">{{ report.battery.cycle_count }} 次</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 无问题空状态 -->
      <div
        v-if="noIssuesFound"
        class="card empty-result"
      >
        <span class="empty-icon">
          <Icon name="check" :size="28" :stroke-width="1.75" />
        </span>
        <h3>硬件状态良好</h3>
        <p>未发现 WHEA 错误、问题设备或异常 SMART 属性。</p>
      </div>

      <!-- 4. 底部：内存诊断按钮 -->
      <div class="card mem-diag-card">
        <div class="mem-diag-info">
          <Icon name="memory-stick" :size="18" :stroke-width="1.75" class="mem-icon" />
          <div class="mem-diag-text">
            <p class="mem-title">Windows 内存诊断</p>
            <p class="mem-desc">运行 Windows 内置内存诊断工具，需要立即重启电脑。重启后系统将进入蓝色内存检测界面（耗时约 10-30 分钟），完成后再次重启并在通知中显示结果。</p>
          </div>
        </div>
        <button class="btn btn-danger btn-sm" @click="showMemoryConfirm = true">
          <Icon name="play" :size="13" :stroke-width="2" />
          启动内存诊断
        </button>
      </div>
    </template>

    <!-- 错误空状态 -->
    <div v-else class="card empty-state">
      <Icon name="alert" :size="22" :stroke-width="1.5" />
      <p>诊断失败或未获取到结果，请重新诊断。</p>
    </div>

    <!-- 二次确认弹窗 -->
    <div v-if="showMemoryConfirm" class="modal-overlay" @click.self="showMemoryConfirm = false">
      <div class="modal">
        <div class="modal-head">
          <Icon name="alert" :size="16" :stroke-width="2" class="modal-icon" />
          <span class="modal-title">确认启动内存诊断</span>
        </div>
        <div class="modal-body">
          <p>Windows 内存诊断将要求立即重启电脑。重启后系统会进入内存检测界面，可能需要 10-30 分钟，期间无法使用电脑。</p>
          <p class="modal-warn">请确认已保存所有打开的文档和工作。</p>
        </div>
        <div class="modal-foot">
          <button class="btn btn-ghost btn-sm" @click="showMemoryConfirm = false">取消</button>
          <button class="btn btn-danger btn-sm" @click="launchMemoryDiag">
            <Icon name="power" :size="13" :stroke-width="2" />
            确认重启并诊断
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const loading = ref(true);
const report = ref(null);
const showMemoryConfirm = ref(false);
const isAdmin = ref(true);
const exportLoading = ref(false);

/* —— 顶部状态条 —— */
const statusBarClass = computed(() => {
  if (!report.value) return "status-healthy";
  const s = report.value.overall_status;
  if (s === "critical") return "status-critical";
  if (s === "warnings") return "status-warning";
  return "status-healthy";
});

const statusDotClass = computed(() => {
  if (!report.value) return "dot-info";
  const s = report.value.overall_status;
  if (s === "critical") return "dot-danger";
  if (s === "warnings") return "dot-warning";
  return "dot-success";
});

const statusHeadline = computed(() => {
  if (!report.value) return "";
  const s = report.value.overall_status;
  if (s === "critical") return "存在严重硬件问题";
  if (s === "warnings") return "存在硬件警告";
  return "硬件状态正常";
});

/* —— 无问题空状态 —— */
const noIssuesFound = computed(() => {
  if (!report.value) return false;
  const r = report.value;
  return (
    (!r.findings || r.findings.length === 0) &&
    (!r.whea_errors || r.whea_errors.length === 0) &&
    (!r.problem_devices || r.problem_devices.length === 0) &&
    (!r.smart_attributes || r.smart_attributes.length === 0) &&
    !r.memory_errors_detected
  );
});

/* —— Finding 严重性样式 —— */
function findingSeverityClass(sev) {
  if (sev === "critical") return "sev-critical";
  if (sev === "warning") return "sev-warning";
  return "sev-info";
}

function findingDotClass(sev) {
  if (sev === "critical") return "dot-danger";
  if (sev === "warning") return "dot-warning";
  return "dot-info";
}

/* —— WHEA 严重性 —— */
function wheaSeverityDot(sev) {
  const s = (sev || "").toLowerCase();
  if (s.includes("crit") || s.includes("fatal") || s.includes("error")) return "dot-danger";
  if (s.includes("warn")) return "dot-warning";
  if (s.includes("info")) return "dot-info";
  return "dot-danger";
}

/* —— SMART 状态 —— */
function smartStatusDot(status) {
  const s = (status || "").toLowerCase();
  if (s.includes("crit")) return "dot-danger";
  if (s.includes("warn")) return "dot-warning";
  return "dot-success";
}

/* —— 电池相关 —— */
const batteryStatusDot = computed(() => {
  if (!report.value || !report.value.battery) return "dot-info";
  const hp = report.value.battery.health_percent;
  if (hp < 50) return "dot-danger";
  if (hp < 80) return "dot-warning";
  return "dot-success";
});

const batteryHealthColor = computed(() => {
  if (!report.value || !report.value.battery) return "";
  const hp = report.value.battery.health_percent;
  if (hp < 50) return "text-danger";
  if (hp < 80) return "text-warning";
  return "text-success";
});

const batteryBarClass = computed(() => {
  if (!report.value || !report.value.battery) return "low";
  const hp = report.value.battery.health_percent;
  if (hp < 50) return "high";
  if (hp < 80) return "medium";
  return "low";
});

/* —— 行为 —— */
async function runDiagnosis() {
  loading.value = true;
  try {
    report.value = await invoke("diagnose_hardware");
  } catch (e) {
    console.error("Hardware diagnosis failed:", e);
    report.value = null;
  }
  loading.value = false;
}

async function launchMemoryDiag() {
  showMemoryConfirm.value = false;
  try {
    await invoke("launch_memory_diagnostic");
  } catch (e) {
    alert("启动失败: " + e);
  }
}

/* —— 权限检测 —— */
async function checkPermission() {
  try {
    const status = await invoke("check_admin_status");
    isAdmin.value = status.is_admin;
  } catch (e) {
    console.error("Permission check failed:", e);
  }
}

/* —— 导出报告 —— */
async function exportReport() {
  if (!report.value) return;
  exportLoading.value = true;
  try {
    const result = await invoke("export_report", {
      reportType: "hardware",
      data: report.value
    });
    if (result.success) {
      alert(result.message);
    } else {
      alert("导出失败: " + result.message);
    }
  } catch (e) {
    alert("导出失败: " + String(e));
  }
  exportLoading.value = false;
}

onMounted(() => {
  checkPermission();
  runDiagnosis();
});
</script>

<style scoped>
.hardware-diagnostics {
  max-width: 1600px;
}

/* ===== 页头 ===== */
.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 18px;
}

.header-actions {
  display: flex;
  gap: 8px;
}

/* ===== 管理员权限提示 ===== */
.admin-warning {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--warning-dim);
  border-left: 3px solid var(--warning);
  padding: 10px 14px;
  border-radius: var(--radius-sm);
  margin-bottom: 12px;
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.admin-warning-icon {
  color: var(--warning);
  flex-shrink: 0;
}

.title-group {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  color: var(--text-muted);
}

/* ===== 加载 ===== */
.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 80px 20px;
  color: var(--text-muted);
  font-size: 12px;
}

/* ===== 顶部状态条 ===== */
.status-bar {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 18px;
  margin-bottom: 12px;
  border-left: 3px solid var(--border-light);
}

.status-bar.status-healthy {
  border-left-color: var(--success);
  background: linear-gradient(to right, var(--success-dim), var(--bg-surface) 55%);
}

.status-bar.status-warning {
  border-left-color: var(--warning);
  background: linear-gradient(to right, var(--warning-dim), var(--bg-surface) 55%);
}

.status-bar.status-critical {
  border-left-color: var(--danger);
  background: linear-gradient(to right, var(--danger-dim), var(--bg-surface) 55%);
}

.big-dot {
  width: 10px;
  height: 10px;
}

.status-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.status-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.status-headline {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
  margin-top: 2px;
}

.status-summary {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

/* ===== 内存错误提示 ===== */
.memory-warning {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  margin-bottom: 12px;
  border-color: var(--danger);
  border-left: 3px solid var(--danger);
}

.mw-icon {
  color: var(--danger);
  flex-shrink: 0;
}

.mw-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.mw-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.55;
}

/* ===== Findings ===== */
.card {
  margin-bottom: 12px;
}

.findings-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.finding {
  display: flex;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
}

.finding-bar {
  width: 3px;
  flex-shrink: 0;
}

.finding.sev-critical .finding-bar { background: var(--danger); }
.finding.sev-warning  .finding-bar { background: var(--warning); }
.finding.sev-info     .finding-bar { background: var(--info); }

.finding-body {
  flex: 1;
  padding: 10px 12px;
  min-width: 0;
}

.finding-head {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 6px;
}

.finding-title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.finding-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.55;
  margin-bottom: 8px;
}

.rec-btn {
  display: inline-flex;
  align-items: flex-start;
  gap: 6px;
  padding: 5px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  max-width: 100%;
}

.rec-icon {
  color: var(--accent);
  flex-shrink: 0;
  margin-top: 1px;
}

.rec-text {
  font-size: 11.5px;
  color: var(--text-secondary);
  line-height: 1.5;
  word-break: break-word;
}

/* ===== 表格 ===== */
.table-body {
  padding: 0;
}

.table-wrapper {
  max-height: 480px;
  overflow-y: auto;
}

.nowrap {
  white-space: nowrap;
}

.num-col {
  text-align: right;
}

.status-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.level-text {
  font-size: 12px;
  color: var(--text-secondary);
}

.msg-cell {
  max-width: 320px;
  color: var(--text-secondary);
}

.device-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.device-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.device-id {
  font-size: 10.5px;
  color: var(--text-muted);
  word-break: break-all;
}

.probable-cause {
  color: var(--text-muted);
}

/* ===== 电池 ===== */
.battery-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
}

.battery-health {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.health-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.health-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.health-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.1;
}

.health-interp {
  font-size: 11.5px;
  color: var(--text-muted);
  line-height: 1.55;
}

.battery-stats {
  display: flex;
  flex-direction: column;
  justify-content: center;
}

/* ===== 无问题空状态 ===== */
.empty-result {
  text-align: center;
  padding: 40px 20px;
}

.empty-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  background: var(--success-dim);
  color: var(--success);
  border-radius: 50%;
  margin-bottom: 12px;
}

.empty-result h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.empty-result p {
  font-size: 12px;
  color: var(--text-muted);
}

/* ===== 底部内存诊断卡片 ===== */
.mem-diag-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px;
}

.mem-diag-info {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.mem-icon {
  color: var(--accent);
  flex-shrink: 0;
  margin-top: 1px;
}

.mem-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 3px;
}

.mem-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  line-height: 1.6;
  max-width: 640px;
}

/* ===== 错误空状态 ===== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 12px;
}

/* ===== 二次确认弹窗 ===== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.15s ease;
}

.modal {
  width: 440px;
  max-width: calc(100vw - 32px);
  background: var(--bg-surface);
  border: 1px solid var(--border-light);
  border-radius: var(--radius);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.modal-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
}

.modal-icon {
  color: var(--warning);
}

.modal-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-body {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.modal-body p {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.modal-warn {
  color: var(--warning) !important;
}

.modal-foot {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 10px 14px;
  border-top: 1px solid var(--border);
  background: var(--bg-elevated);
}

/* ===== 响应式 ===== */
@media (max-width: 720px) {
  .battery-grid {
    grid-template-columns: 1fr;
  }
  .mem-diag-card {
    flex-direction: column;
    align-items: flex-start;
  }
  .mem-diag-card .btn {
    width: 100%;
    justify-content: center;
  }
  .msg-cell {
    max-width: none;
  }
}
</style>
