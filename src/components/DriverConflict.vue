<template>
  <div class="driver-conflict fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">驱动冲突诊断</h1>
        <p class="page-subtitle">深度扫描设备驱动冲突、加载失败与签名问题</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-primary btn-sm" @click="scan" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="search" :size="13" :stroke-width="2" />
          {{ hasResult ? '重新扫描' : '开始扫描' }}
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

    <!-- 顶部状态条 -->
    <div v-if="result" class="card status-bar" :class="statusBarClass">
      <div class="status-bar-head">
        <span class="dot" :class="overallDotClass"></span>
        <span class="status-label">{{ overallStatusLabel }}</span>
        <span class="tag" :class="overallTagClass">{{ result.overall_status }}</span>
      </div>
      <p class="status-bar-summary">{{ result.summary }}</p>
      <div class="status-bar-stats">
        <div class="stat-mini">
          <span class="stat-mini-label">冲突</span>
          <span class="stat-mini-value mono">{{ result.conflicts.length }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">版本冲突</span>
          <span class="stat-mini-value mono">{{ result.version_conflicts.length }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">加载失败</span>
          <span class="stat-mini-value mono">{{ result.load_failures.length }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">未签名</span>
          <span class="stat-mini-value mono">{{ result.unsigned_drivers.length }}</span>
        </div>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在扫描驱动冲突...</p>
    </div>

    <!-- 初始空状态 -->
    <div v-else-if="!result" class="card empty-state initial-empty">
      <Icon name="monitor" :size="28" :stroke-width="1.5" />
      <p>点击「开始扫描」检测系统中的驱动冲突、加载失败与未签名驱动</p>
    </div>

    <!-- 健康状态 -->
    <div v-else-if="isHealthy" class="card empty-state healthy-empty">
      <span class="empty-icon-success">
        <Icon name="check" :size="28" :stroke-width="1.75" />
      </span>
      <h3>未检测到驱动冲突</h3>
      <p>所有设备驱动运行正常,无冲突或加载失败记录。</p>
    </div>

    <template v-else>
      <!-- 冲突类型分组 -->
      <div
        v-for="group in conflictGroups"
        :key="group.type"
        class="card conflict-group-card"
        :class="group.cardClass"
      >
        <div class="card-header">
          <div class="title-group">
            <Icon :name="group.icon" :size="13" :stroke-width="1.75" />
            <span class="card-title">{{ group.label }}</span>
            <span class="tag" :class="group.tagClass">{{ group.items.length }}</span>
          </div>
          <span class="type-tag tag" :class="group.tagClass">{{ group.tagText }}</span>
        </div>
        <div class="card-body">
          <div class="conflict-list">
            <div v-for="(item, i) in group.items" :key="i" class="conflict-item">
              <div class="conflict-head">
                <span class="conflict-device">{{ item.device_name }}</span>
                <span class="signed-inline">
                  <span class="dot" :class="item.is_signed ? 'dot-success' : 'dot-danger'"></span>
                  {{ item.is_signed ? '已签名' : '未签名' }}
                </span>
              </div>
              <div class="conflict-meta">
                <span class="meta-item">
                  <span class="meta-label">驱动</span>
                  <span class="meta-value">{{ item.driver_name || '-' }}</span>
                </span>
                <span class="meta-item">
                  <span class="meta-label">版本</span>
                  <span class="meta-value mono">{{ item.driver_version || '-' }}</span>
                </span>
                <span class="meta-item">
                  <span class="meta-label">日期</span>
                  <span class="meta-value mono">{{ item.driver_date || '-' }}</span>
                </span>
                <span class="meta-item" v-if="item.problem_code">
                  <span class="meta-label">代码</span>
                  <span class="meta-value mono">{{ item.problem_code }}</span>
                </span>
              </div>
              <div class="conflict-detail">
                <div class="detail-row">
                  <span class="detail-label">问题描述</span>
                  <span class="detail-text">{{ item.problem_description }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">可能原因</span>
                  <span class="detail-text">{{ item.probable_cause }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">修复建议</span>
                  <span class="detail-text text-accent">{{ item.fix_suggestion }}</span>
                </div>
              </div>
              <div class="conflict-device-id mono" v-if="item.device_id">
                <Icon name="cpu" :size="11" :stroke-width="1.75" />
                {{ item.device_id }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 未签名驱动 -->
      <div
        v-if="result.unsigned_drivers.length > 0"
        class="card conflict-group-card border-neutral"
      >
        <div class="card-header">
          <div class="title-group">
            <Icon name="shield" :size="13" :stroke-width="1.75" />
            <span class="card-title">未签名驱动</span>
            <span class="tag tag-neutral">{{ result.unsigned_drivers.length }}</span>
          </div>
          <span class="type-tag tag tag-neutral">unsigned</span>
        </div>
        <div class="card-body">
          <div class="conflict-list">
            <div
              v-for="(item, i) in result.unsigned_drivers"
              :key="'u' + i"
              class="conflict-item"
            >
              <div class="conflict-head">
                <span class="conflict-device">{{ item.device_name }}</span>
                <span class="tag tag-neutral">未签名</span>
              </div>
              <div class="conflict-meta">
                <span class="meta-item">
                  <span class="meta-label">驱动</span>
                  <span class="meta-value">{{ item.driver_name || '-' }}</span>
                </span>
                <span class="meta-item">
                  <span class="meta-label">版本</span>
                  <span class="meta-value mono">{{ item.driver_version || '-' }}</span>
                </span>
                <span class="meta-item">
                  <span class="meta-label">日期</span>
                  <span class="meta-value mono">{{ item.driver_date || '-' }}</span>
                </span>
                <span class="meta-item" v-if="item.problem_code">
                  <span class="meta-label">代码</span>
                  <span class="meta-value mono">{{ item.problem_code }}</span>
                </span>
              </div>
              <div class="conflict-detail">
                <div class="detail-row">
                  <span class="detail-label">问题描述</span>
                  <span class="detail-text">{{ item.problem_description }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">可能原因</span>
                  <span class="detail-text">{{ item.probable_cause }}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">修复建议</span>
                  <span class="detail-text text-accent">{{ item.fix_suggestion }}</span>
                </div>
              </div>
              <div class="conflict-device-id mono" v-if="item.device_id">
                <Icon name="cpu" :size="11" :stroke-width="1.75" />
                {{ item.device_id }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 版本冲突 -->
      <div v-if="result.version_conflicts.length > 0" class="card section-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="refresh" :size="13" :stroke-width="1.75" />
            <span class="card-title">版本冲突</span>
            <span class="tag tag-info">{{ result.version_conflicts.length }}</span>
          </div>
        </div>
        <div class="card-body">
          <div class="version-conflict-list">
            <div
              v-for="(vc, i) in result.version_conflicts"
              :key="i"
              class="version-conflict-item"
            >
              <div class="vc-head">
                <Icon name="cpu" :size="14" :stroke-width="1.75" class="vc-icon" />
                <span class="vc-driver-name">{{ vc.driver_name }}</span>
              </div>
              <p class="vc-desc">{{ vc.description }}</p>
              <div class="vc-section">
                <span class="vc-section-label">涉及设备</span>
                <div class="vc-chips">
                  <span v-for="(d, j) in vc.devices" :key="j" class="vc-chip">{{ d }}</span>
                </div>
              </div>
              <div class="vc-section">
                <span class="vc-section-label">版本列表</span>
                <div class="vc-chips">
                  <span v-for="(v, j) in vc.versions" :key="j" class="vc-chip mono">{{ v }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 加载失败事件 -->
      <div v-if="result.load_failures.length > 0" class="card section-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="alert" :size="13" :stroke-width="1.75" />
            <span class="card-title">驱动加载失败事件</span>
            <span class="tag tag-danger">{{ result.load_failures.length }}</span>
          </div>
        </div>
        <div class="card-body table-body">
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>时间</th>
                  <th>驱动名称</th>
                  <th>事件 ID</th>
                  <th>失败原因</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(fail, i) in result.load_failures" :key="i">
                  <td class="mono nowrap">{{ fail.timestamp }}</td>
                  <td>{{ fail.driver_name }}</td>
                  <td class="mono">{{ fail.event_id }}</td>
                  <td class="fail-reason">{{ fail.failure_reason }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 推荐操作 -->
      <div v-if="result.recommendations.length > 0" class="card section-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="check" :size="13" :stroke-width="1.75" />
            <span class="card-title">推荐操作</span>
          </div>
        </div>
        <div class="card-body">
          <div class="recommendation-list">
            <div
              v-for="(rec, i) in result.recommendations"
              :key="i"
              class="recommendation-item"
            >
              <span class="rec-num mono">{{ i + 1 }}</span>
              <span class="rec-text">{{ rec }}</span>
              <Icon
                name="chevron-right"
                :size="13"
                :stroke-width="1.75"
                class="rec-chevron"
              />
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const loading = ref(false);
const result = ref(null);
const isAdmin = ref(true);
const exportLoading = ref(false);

const hasResult = computed(() => result.value !== null);

const isHealthy = computed(() => {
  if (!result.value) return false;
  return (
    result.value.conflicts.length === 0 &&
    result.value.version_conflicts.length === 0 &&
    result.value.load_failures.length === 0 &&
    result.value.unsigned_drivers.length === 0
  );
});

const overallDotClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "dot-success";
  if (s === "warnings") return "dot-warning";
  if (s === "critical") return "dot-danger";
  return "dot-info";
});

const overallTagClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "tag-success";
  if (s === "warnings") return "tag-warning";
  if (s === "critical") return "tag-danger";
  return "tag-info";
});

const overallStatusLabel = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "状态正常";
  if (s === "warnings") return "存在警告";
  if (s === "critical") return "状态严重";
  return "未知状态";
});

const statusBarClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "border-success";
  if (s === "warnings") return "border-warning";
  if (s === "critical") return "border-danger";
  return "";
});

// 冲突类型分组配置
const groupConfig = {
  missing_driver: {
    label: "缺少驱动",
    icon: "x",
    tagClass: "tag-warning",
    cardClass: "border-warning",
    tagText: "missing_driver",
  },
  driver_corrupt: {
    label: "损坏驱动",
    icon: "alert",
    tagClass: "tag-danger",
    cardClass: "border-danger",
    tagText: "driver_corrupt",
  },
  resource_conflict: {
    label: "资源冲突",
    icon: "cpu",
    tagClass: "tag-warning",
    cardClass: "border-warning",
    tagText: "resource_conflict",
  },
  hardware_failure: {
    label: "硬件故障",
    icon: "wrench",
    tagClass: "tag-danger",
    cardClass: "border-danger",
    tagText: "hardware_failure",
  },
  version_mismatch: {
    label: "版本不匹配",
    icon: "refresh",
    tagClass: "tag-info",
    cardClass: "border-info",
    tagText: "version_mismatch",
  },
};

const groupOrder = [
  "missing_driver",
  "driver_corrupt",
  "resource_conflict",
  "hardware_failure",
  "version_mismatch",
];

const conflictGroups = computed(() => {
  if (!result.value) return [];
  const groups = [];
  for (const type of groupOrder) {
    const items = result.value.conflicts.filter(
      c => c.conflict_type === type
    );
    if (items.length > 0) {
      groups.push({
        type,
        ...groupConfig[type],
        items,
      });
    }
  }
  // 其他未识别类型
  const known = new Set(groupOrder);
  const others = result.value.conflicts.filter(
    c => !known.has(c.conflict_type)
  );
  if (others.length > 0) {
    groups.push({
      type: "other",
      label: "其他冲突",
      icon: "alert",
      tagClass: "tag-neutral",
      cardClass: "border-neutral",
      tagText: "other",
      items: others,
    });
  }
  return groups;
});

async function scan() {
  loading.value = true;
  result.value = null;
  try {
    result.value = await invoke("diagnose_driver_conflicts");
  } catch (e) {
    console.error("Driver conflict diagnosis failed:", e);
    result.value = null;
  }
  loading.value = false;
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
  if (!result.value) return;
  exportLoading.value = true;
  try {
    const res = await invoke("export_report", {
      reportType: "driver",
      data: result.value
    });
    if (res.success) {
      alert(res.message);
    } else {
      alert("导出失败: " + res.message);
    }
  } catch (e) {
    alert("导出失败: " + String(e));
  }
  exportLoading.value = false;
}

onMounted(() => {
  checkPermission();
});
</script>

<style scoped>
.driver-conflict {
  max-width: 1600px;
}

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

/* 管理员权限提示 */
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

/* 顶部状态条 */
.status-bar {
  padding: 14px 16px;
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.status-bar.border-success { border-left: 3px solid var(--success); }
.status-bar.border-warning { border-left: 3px solid var(--warning); }
.status-bar.border-danger  { border-left: 3px solid var(--danger); }
.status-bar.border-info    { border-left: 3px solid var(--info); }

.status-bar-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.status-bar-summary {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.status-bar-stats {
  display: flex;
  align-items: center;
  gap: 14px;
  padding-top: 8px;
  border-top: 1px solid var(--border);
}

.stat-mini {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.stat-mini-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.stat-mini-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.stat-mini-divider {
  width: 1px;
  height: 14px;
  background: var(--border);
}

/* 加载 */
.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 80px 20px;
  color: var(--text-muted);
  font-size: 12px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 50px 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

.initial-empty {
  color: var(--text-muted);
}

.healthy-empty {
  padding: 40px 20px;
}

.empty-icon-success {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  background: var(--success-dim);
  color: var(--success);
  border-radius: 50%;
  margin-bottom: 6px;
}

.healthy-empty h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.healthy-empty p {
  font-size: 12px;
  color: var(--text-muted);
}

/* 卡片通用 */
.section-card {
  margin-bottom: 12px;
}

.title-group {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  color: var(--text-secondary);
}

.type-tag {
  text-transform: lowercase;
}

/* 冲突分组卡片 */
.conflict-group-card {
  margin-bottom: 12px;
}

.conflict-group-card.border-success { border-left: 3px solid var(--success); }
.conflict-group-card.border-warning { border-left: 3px solid var(--warning); }
.conflict-group-card.border-danger  { border-left: 3px solid var(--danger); }
.conflict-group-card.border-info    { border-left: 3px solid var(--info); }
.conflict-group-card.border-neutral { border-left: 3px solid var(--text-faint); }

.conflict-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.conflict-item {
  padding: 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.conflict-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}

.conflict-device {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  word-break: break-all;
}

.signed-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-secondary);
}

.conflict-meta {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 8px 16px;
  margin-bottom: 10px;
  padding: 8px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.meta-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  min-width: 0;
}

.meta-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}

.meta-value {
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
}

.conflict-detail {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.detail-label {
  flex-shrink: 0;
  width: 64px;
  font-size: 11px;
  color: var(--text-muted);
  padding-top: 1px;
}

.detail-text {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.55;
  flex: 1;
  word-break: break-all;
}

.text-accent {
  color: var(--accent);
}

.conflict-device-id {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 10px;
  padding-top: 8px;
  border-top: 1px dashed var(--border);
  font-size: 10.5px;
  color: var(--text-muted);
  word-break: break-all;
}

/* 版本冲突 */
.version-conflict-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.version-conflict-item {
  padding: 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-left: 3px solid var(--info);
  border-radius: var(--radius-sm);
}

.vc-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.vc-icon {
  color: var(--info);
  flex-shrink: 0;
}

.vc-driver-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  word-break: break-all;
}

.vc-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 10px;
}

.vc-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 8px;
}

.vc-section:last-child {
  margin-bottom: 0;
}

.vc-section-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.vc-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.vc-chip {
  padding: 2px 8px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: 3px;
  font-size: 11px;
  color: var(--text-secondary);
}

.vc-chip.mono {
  color: var(--accent);
  background: var(--accent-dim);
  border-color: transparent;
}

/* 表格 */
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

.fail-reason {
  color: var(--text-secondary);
}

/* 推荐操作 */
.recommendation-list {
  display: flex;
  flex-direction: column;
}

.recommendation-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 4px;
  border-bottom: 1px solid var(--border);
}

.recommendation-item:last-child {
  border-bottom: none;
}

.rec-num {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  background: var(--accent-dim);
  color: var(--accent);
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
}

.rec-text {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.rec-chevron {
  color: var(--text-muted);
  flex-shrink: 0;
}

@media (max-width: 720px) {
  .status-bar-stats {
    flex-wrap: wrap;
    gap: 10px 14px;
  }
  .stat-mini-divider {
    display: none;
  }
  .detail-row {
    flex-direction: column;
    gap: 2px;
  }
  .detail-label {
    width: auto;
  }
}
</style>
