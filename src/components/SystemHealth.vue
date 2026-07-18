<template>
  <div class="system-health fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">系统损坏深度分析</h1>
        <p class="page-subtitle">检测系统文件、注册表、启动配置、更新与激活状态</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-primary btn-sm" @click="analyze" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="activity" :size="13" :stroke-width="2" />
          {{ hasResult ? '重新分析' : '开始分析' }}
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
          <span class="stat-mini-label">严重</span>
          <span class="stat-mini-value mono text-danger">{{ criticalCount }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">警告</span>
          <span class="stat-mini-value mono text-warning">{{ warningCount }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">提示</span>
          <span class="stat-mini-value mono text-info">{{ infoCount }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">损坏文件</span>
          <span class="stat-mini-value mono">{{ result.corrupted_files.length }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">注册表</span>
          <span class="stat-mini-value mono">{{ result.registry_issues.length }}</span>
        </div>
        <div class="stat-mini-divider"></div>
        <div class="stat-mini">
          <span class="stat-mini-label">失败更新</span>
          <span class="stat-mini-value mono">{{ result.failed_updates.length }}</span>
        </div>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在深度分析系统状态...</p>
    </div>

    <!-- 初始空状态 -->
    <div v-else-if="!hasLoaded" class="scan-prompt">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始扫描</p>
      <button class="btn btn-primary" @click="analyze">开始扫描</button>
    </div>

    <!-- 健康状态 -->
    <div v-else-if="isHealthy" class="card empty-state healthy-empty">
      <span class="empty-icon-success">
        <Icon name="check" :size="28" :stroke-width="1.75" />
      </span>
      <h3>系统状态良好</h3>
      <p>未检测到系统损坏、注册表异常、启动问题或失败的更新。</p>
    </div>

    <template v-else>
      <!-- 分类问题展示 -->
      <div
        v-for="cat in issueCategories"
        :key="cat.key"
        class="card category-card"
        :class="cat.cardClass"
      >
        <div class="card-header">
          <div class="title-group">
            <Icon :name="cat.icon" :size="13" :stroke-width="1.75" :class="cat.iconClass" />
            <span class="card-title">{{ cat.label }}</span>
            <span class="tag" :class="cat.tagClass">{{ cat.items.length }}</span>
          </div>
          <span class="cat-key tag tag-neutral">{{ cat.key }}</span>
        </div>
        <div class="card-body">
          <div class="issue-list">
            <div
              v-for="(issue, i) in cat.items"
              :key="i"
              class="issue-item"
              :class="issueBorderClass(issue.severity)"
            >
              <div class="issue-head">
                <span class="dot" :class="dotClass(issue.severity)"></span>
                <span class="issue-title">{{ issue.title }}</span>
                <span class="tag" :class="tagClass(issue.severity)">{{ issue.severity }}</span>
              </div>
              <p class="issue-desc">{{ issue.description }}</p>
              <div class="issue-meta">
                <div class="meta-row">
                  <span class="meta-label">受影响组件</span>
                  <span class="meta-value">{{ issue.affected_component || '-' }}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-label">修复建议</span>
                  <span class="meta-value text-accent">{{ issue.recommendation }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 损坏文件表格 -->
      <div v-if="result.corrupted_files.length > 0" class="card section-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="folder" :size="13" :stroke-width="1.75" />
            <span class="card-title">损坏文件</span>
            <span class="tag tag-danger">{{ result.corrupted_files.length }}</span>
          </div>
        </div>
        <div class="card-body table-body">
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>文件路径</th>
                  <th>文件名</th>
                  <th>问题描述</th>
                  <th>可修复</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(file, i) in result.corrupted_files" :key="i">
                  <td class="mono path-cell">{{ file.file_path }}</td>
                  <td>{{ file.file_name }}</td>
                  <td class="issue-cell">{{ file.issue }}</td>
                  <td>
                    <span class="repair-inline">
                      <span class="dot" :class="file.can_repair ? 'dot-success' : 'dot-danger'"></span>
                      <span :class="file.can_repair ? 'text-success' : 'text-danger'">
                        {{ file.can_repair ? '可修复' : '不可修复' }}
                      </span>
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 注册表问题表格 -->
      <div v-if="result.registry_issues.length > 0" class="card section-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="database" :size="13" :stroke-width="1.75" />
            <span class="card-title">注册表问题</span>
            <span class="tag tag-warning">{{ result.registry_issues.length }}</span>
          </div>
        </div>
        <div class="card-body table-body">
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>Hive</th>
                  <th>键路径</th>
                  <th>问题描述</th>
                  <th>严重性</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(reg, i) in result.registry_issues" :key="i">
                  <td><span class="tag tag-info">{{ reg.hive }}</span></td>
                  <td class="mono path-cell">{{ reg.key_path }}</td>
                  <td class="issue-cell">{{ reg.issue }}</td>
                  <td>
                    <span class="repair-inline">
                      <span class="dot" :class="dotClass(reg.severity)"></span>
                      <span>{{ reg.severity }}</span>
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 启动配置卡片 -->
      <div class="card section-card" :class="bootCardClass">
        <div class="card-header">
          <div class="title-group">
            <Icon name="power" :size="13" :stroke-width="1.75" />
            <span class="card-title">启动配置</span>
          </div>
          <span class="tag" :class="bootTagClass">{{ result.boot_config.last_boot_status }}</span>
        </div>
        <div class="card-body">
          <p class="boot-desc">{{ result.boot_config.description }}</p>
          <div class="boot-grid">
            <div class="boot-field">
              <span class="field-label">上次启动状态</span>
              <span class="field-value">
                <span class="repair-inline">
                  <span class="dot" :class="bootDotClass"></span>
                  {{ result.boot_config.last_boot_status }}
                </span>
              </span>
            </div>
            <div class="boot-field">
              <span class="field-label">启动耗时</span>
              <span class="field-value mono">{{ result.boot_config.boot_time_seconds }} s</span>
            </div>
            <div class="boot-field">
              <span class="field-label">安全模式</span>
              <span class="field-value">
                <span class="repair-inline">
                  <span class="dot" :class="result.boot_config.safe_mode ? 'dot-warning' : 'dot-success'"></span>
                  {{ result.boot_config.safe_mode ? '是' : '否' }}
                </span>
              </span>
            </div>
          </div>
          <div v-if="result.boot_config.boot_errors.length > 0" class="boot-errors">
            <div class="boot-errors-head">
              <Icon name="alert" :size="12" :stroke-width="1.75" class="err-icon" />
              <span class="boot-errors-label">启动错误 ({{ result.boot_config.boot_errors.length }})</span>
            </div>
            <ul class="boot-errors-list">
              <li v-for="(err, i) in result.boot_config.boot_errors" :key="i" class="boot-error-item">
                <span class="dot dot-danger"></span>
                <span class="boot-error-text">{{ err }}</span>
              </li>
            </ul>
          </div>
        </div>
      </div>

      <!-- 失败更新表格 -->
      <div v-if="result.failed_updates.length > 0" class="card section-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="download" :size="13" :stroke-width="1.75" />
            <span class="card-title">失败的 Windows 更新</span>
            <span class="tag tag-danger">{{ result.failed_updates.length }}</span>
          </div>
        </div>
        <div class="card-body table-body">
          <div class="table-wrapper">
            <table>
              <thead>
                <tr>
                  <th>KB 号</th>
                  <th>标题</th>
                  <th>错误码</th>
                  <th>错误说明</th>
                  <th>时间</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(upd, i) in result.failed_updates" :key="i">
                  <td><span class="tag tag-neutral">{{ upd.kb_number }}</span></td>
                  <td class="title-cell">{{ upd.title }}</td>
                  <td class="mono">{{ upd.error_code || '-' }}</td>
                  <td class="issue-cell">{{ upd.error_description }}</td>
                  <td class="mono nowrap">{{ upd.timestamp }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- 激活状态卡片 -->
      <div class="card section-card" :class="activationCardClass">
        <div class="card-header">
          <div class="title-group">
            <Icon name="check" :size="13" :stroke-width="1.75" />
            <span class="card-title">激活状态</span>
          </div>
          <span class="tag" :class="result.activation.is_activated ? 'tag-success' : 'tag-danger'">
            {{ result.activation.is_activated ? '已激活' : '未激活' }}
          </span>
        </div>
        <div class="card-body">
          <div class="activation-grid">
            <div class="boot-field">
              <span class="field-label">激活状态</span>
              <span class="field-value">
                <span class="repair-inline">
                  <span class="dot" :class="result.activation.is_activated ? 'dot-success' : 'dot-danger'"></span>
                  {{ result.activation.is_activated ? '已激活' : '未激活' }}
                </span>
              </span>
            </div>
            <div class="boot-field">
              <span class="field-label">许可证状态</span>
              <span class="field-value">{{ result.activation.license_status }}</span>
            </div>
          </div>
          <p class="activation-desc">{{ result.activation.description }}</p>
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
const hasLoaded = ref(false);
const result = ref(null);
const isAdmin = ref(true);
const exportLoading = ref(false);

const hasResult = computed(() => result.value !== null);

const isHealthy = computed(() => {
  if (!result.value) return false;
  return (
    result.value.issues.length === 0 &&
    result.value.corrupted_files.length === 0 &&
    result.value.registry_issues.length === 0 &&
    result.value.failed_updates.length === 0 &&
    result.value.boot_config.boot_errors.length === 0
  );
});

const criticalCount = computed(() => {
  if (!result.value) return 0;
  return result.value.issues.filter(i => i.severity === "critical").length;
});

const warningCount = computed(() => {
  if (!result.value) return 0;
  return result.value.issues.filter(i => i.severity === "warning").length;
});

const infoCount = computed(() => {
  if (!result.value) return 0;
  return result.value.issues.filter(i => i.severity === "info").length;
});

const overallDotClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "dot-success";
  if (s === "warning" || s === "warnings") return "dot-warning";
  if (s === "critical") return "dot-danger";
  return "dot-info";
});

const overallTagClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "tag-success";
  if (s === "warning" || s === "warnings") return "tag-warning";
  if (s === "critical") return "tag-danger";
  return "tag-info";
});

const overallStatusLabel = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "系统状态正常";
  if (s === "warning" || s === "warnings") return "存在警告";
  if (s === "critical") return "状态严重";
  return "未知状态";
});

const statusBarClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.overall_status;
  if (s === "healthy") return "border-success";
  if (s === "warning" || s === "warnings") return "border-warning";
  if (s === "critical") return "border-danger";
  return "";
});

// 分类配置
const categoryConfig = {
  system_files: {
    label: "系统文件",
    icon: "shield",
    iconClass: "icon-info",
    tagClass: "tag-warning",
  },
  registry: {
    label: "注册表",
    icon: "database",
    iconClass: "icon-info",
    tagClass: "tag-warning",
  },
  boot: {
    label: "启动配置",
    icon: "power",
    iconClass: "icon-info",
    tagClass: "tag-warning",
  },
  updates: {
    label: "Windows 更新",
    icon: "download",
    iconClass: "icon-info",
    tagClass: "tag-warning",
  },
  activation: {
    label: "激活状态",
    icon: "check",
    iconClass: "icon-info",
    tagClass: "tag-warning",
  },
  integrity: {
    label: "系统完整性",
    icon: "activity",
    iconClass: "icon-info",
    tagClass: "tag-warning",
  },
};

const categoryOrder = [
  "system_files",
  "registry",
  "boot",
  "updates",
  "activation",
  "integrity",
];

const severityRank = { critical: 3, warning: 2, info: 1 };

function maxSeverity(items) {
  let max = 0;
  for (const it of items) {
    const r = severityRank[it.severity] || 0;
    if (r > max) max = r;
  }
  return max;
}

function severityToCardClass(maxSev) {
  if (maxSev >= 3) return "border-danger";
  if (maxSev >= 2) return "border-warning";
  if (maxSev >= 1) return "border-info";
  return "border-neutral";
}

function severityToTagClass(maxSev) {
  if (maxSev >= 3) return "tag-danger";
  if (maxSev >= 2) return "tag-warning";
  if (maxSev >= 1) return "tag-info";
  return "tag-neutral";
}

const issueCategories = computed(() => {
  if (!result.value) return [];
  const cats = [];
  for (const key of categoryOrder) {
    const items = result.value.issues.filter(i => i.category === key);
    if (items.length > 0) {
      const maxSev = maxSeverity(items);
      cats.push({
        key,
        ...categoryConfig[key],
        items,
        cardClass: severityToCardClass(maxSev),
        tagClass: severityToTagClass(maxSev),
      });
    }
  }
  // 其他未识别分类
  const known = new Set(categoryOrder);
  const others = result.value.issues.filter(i => !known.has(i.category));
  if (others.length > 0) {
    const maxSev = maxSeverity(others);
    cats.push({
      key: "other",
      label: "其他问题",
      icon: "alert",
      iconClass: "icon-info",
      tagClass: severityToTagClass(maxSev),
      items: others,
      cardClass: severityToCardClass(maxSev),
    });
  }
  return cats;
});

function dotClass(severity) {
  if (severity === "critical") return "dot-danger";
  if (severity === "warning") return "dot-warning";
  if (severity === "info") return "dot-info";
  return "dot-success";
}

function tagClass(severity) {
  if (severity === "critical") return "tag-danger";
  if (severity === "warning") return "tag-warning";
  if (severity === "info") return "tag-info";
  return "tag-neutral";
}

function issueBorderClass(severity) {
  if (severity === "critical") return "issue-border-danger";
  if (severity === "warning") return "issue-border-warning";
  if (severity === "info") return "issue-border-info";
  return "";
}

// 启动配置相关计算
const bootDotClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.boot_config.last_boot_status;
  if (s === "normal") return "dot-success";
  if (s === "abnormal") return "dot-danger";
  return "dot-info";
});

const bootTagClass = computed(() => {
  if (!result.value) return "";
  const s = result.value.boot_config.last_boot_status;
  if (s === "normal") return "tag-success";
  if (s === "abnormal") return "tag-danger";
  return "tag-neutral";
});

const bootCardClass = computed(() => {
  if (!result.value) return "";
  const bc = result.value.boot_config;
  if (bc.last_boot_status === "abnormal" || bc.boot_errors.length > 0) {
    return "border-danger";
  }
  if (bc.last_boot_status === "unknown") return "border-warning";
  return "border-success";
});

const activationCardClass = computed(() => {
  if (!result.value) return "";
  return result.value.activation.is_activated ? "border-success" : "border-danger";
});

async function analyze() {
  loading.value = true;
  result.value = null;
  try {
    result.value = await invoke("diagnose_system_health");
  } catch (e) {
    console.error("System health diagnosis failed:", e);
    result.value = null;
  }
  loading.value = false;
  hasLoaded.value = true;
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
      reportType: "system",
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
.system-health {
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
  flex-wrap: wrap;
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

.text-danger { color: var(--danger); }
.text-warning { color: var(--warning); }
.text-info { color: var(--info); }
.text-success { color: var(--success); }

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

/* 初始扫描提示 */
.scan-prompt {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 80px 20px;
  color: var(--text-muted);
  text-align: center;
}
.scan-prompt p {
  font-size: 13px;
  margin: 0;
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

.icon-info { color: var(--info); }

.cat-key {
  text-transform: lowercase;
}

/* 分类卡片 */
.category-card {
  margin-bottom: 12px;
}

.category-card.border-success { border-left: 3px solid var(--success); }
.category-card.border-warning { border-left: 3px solid var(--warning); }
.category-card.border-danger  { border-left: 3px solid var(--danger); }
.category-card.border-info    { border-left: 3px solid var(--info); }
.category-card.border-neutral { border-left: 3px solid var(--text-faint); }

.issue-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.issue-item {
  padding: 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.issue-item.issue-border-danger  { border-left: 3px solid var(--danger); }
.issue-item.issue-border-warning { border-left: 3px solid var(--warning); }
.issue-item.issue-border-info    { border-left: 3px solid var(--info); }

.issue-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.issue-title {
  flex: 1;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  min-width: 0;
  word-break: break-all;
}

.issue-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.55;
  margin-bottom: 10px;
}

.issue-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.meta-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.meta-label {
  flex-shrink: 0;
  width: 72px;
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding-top: 1px;
}

.meta-value {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
  line-height: 1.5;
  word-break: break-all;
}

.text-accent {
  color: var(--accent);
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

.path-cell {
  color: var(--text-secondary);
  word-break: break-all;
  font-size: 11px;
}

.issue-cell {
  color: var(--text-secondary);
}

.title-cell {
  color: var(--text-primary);
}

.repair-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11.5px;
  color: var(--text-secondary);
}

/* 启动配置卡片 */
.boot-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.55;
  margin-bottom: 12px;
}

.boot-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 10px 16px;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  margin-bottom: 12px;
}

.boot-field {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.field-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.field-value {
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
}

.boot-errors {
  border-top: 1px dashed var(--border);
  padding-top: 10px;
}

.boot-errors-head {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 8px;
}

.err-icon {
  color: var(--danger);
  flex-shrink: 0;
}

.boot-errors-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.boot-errors-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.boot-error-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 10px;
  background: var(--danger-dim);
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
}

.boot-error-text {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  word-break: break-all;
}

/* 激活状态卡片 */
.activation-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 10px 16px;
  padding: 10px 12px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  margin-bottom: 12px;
}

.activation-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.55;
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
    gap: 10px 14px;
  }
  .stat-mini-divider {
    display: none;
  }
  .meta-row {
    flex-direction: column;
    gap: 2px;
  }
  .meta-label {
    width: auto;
  }
}
</style>
