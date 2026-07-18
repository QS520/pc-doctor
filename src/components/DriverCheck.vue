<template>
  <div class="driver-check fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">驱动检查</h1>
        <p class="page-subtitle">扫描已安装驱动程序，识别过期或异常设备</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="checkDrivers" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          重新检查
        </button>
      </div>
    </div>

    <!-- 汇总统计 -->
    <div v-if="result" class="card summary-card">
      <div class="summary-stat">
        <Icon name="hard-drive" :size="18" :stroke-width="1.75" class="summary-icon icon-neutral" />
        <div>
          <p class="stat-label">驱动总数</p>
          <p class="stat-value mono">{{ result.total_drivers }}</p>
        </div>
      </div>
      <div class="summary-divider"></div>
      <div class="summary-stat">
        <Icon name="check" :size="18" :stroke-width="1.75" class="summary-icon icon-success" />
        <div>
          <p class="stat-label">正常</p>
          <p class="stat-value mono text-success">{{ normalCount }}</p>
        </div>
      </div>
      <div class="summary-divider"></div>
      <div class="summary-stat">
        <Icon name="alert" :size="18" :stroke-width="1.75" class="summary-icon icon-warning" />
        <div>
          <p class="stat-label">可能过期</p>
          <p class="stat-value mono text-warning">{{ expiredCount }}</p>
        </div>
      </div>
      <div class="summary-divider"></div>
      <div class="summary-stat">
        <Icon name="x" :size="18" :stroke-width="2" class="summary-icon icon-danger" />
        <div>
          <p class="stat-label">异常</p>
          <p class="stat-value mono text-danger">{{ problemCount }}</p>
        </div>
      </div>
    </div>

    <!-- 过滤栏 -->
    <div v-if="result" class="card filter-bar">
      <div class="filter-tabs">
        <button
          v-for="opt in filterOptions"
          :key="opt.value"
          :class="['tab-btn', { active: filter === opt.value }]"
          @click="filter = opt.value"
        >
          {{ opt.label }}
          <span class="tab-count mono">{{ opt.count }}</span>
        </button>
      </div>
      <div class="filter-group search-group">
        <Icon name="search" :size="13" :stroke-width="1.75" class="search-icon" />
        <input
          v-model="searchKeyword"
          type="text"
          class="filter-input"
          placeholder="搜索设备名称 / 厂商..."
        />
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在检查驱动程序状态...</p>
    </div>

    <!-- 驱动表格 -->
    <div v-else-if="result" class="card driver-card">
      <div class="driver-table-wrapper">
        <table>
          <thead>
            <tr>
              <th>设备名称</th>
              <th>类别</th>
              <th>厂商</th>
              <th>驱动版本</th>
              <th>驱动日期</th>
              <th>提供商</th>
              <th>状态</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(drv, i) in filteredDrivers" :key="i">
              <td>
                <div class="drv-name-cell">
                  <span class="drv-name" :title="drv.problem_description">{{ drv.device_name }}</span>
                  <span v-if="drv.problem_description" class="drv-problem">{{ drv.problem_description }}</span>
                </div>
              </td>
              <td>{{ drv.device_class }}</td>
              <td>{{ drv.manufacturer }}</td>
              <td class="mono">{{ drv.driver_version }}</td>
              <td class="mono">{{ drv.driver_date || '-' }}</td>
              <td>{{ drv.driver_provider }}</td>
              <td>
                <span class="tag" :class="statusClass(drv.status)">{{ drv.status }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-if="filteredDrivers.length === 0" class="empty-state">
        <Icon name="search" :size="22" :stroke-width="1.5" />
        <p>没有匹配的驱动</p>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="card empty-state">
      <Icon name="monitor" :size="22" :stroke-width="1.5" />
      <p>未获取到驱动信息，请重新检查。</p>
    </div>

    <!-- 提示 -->
    <div class="card tip-card tip-warning">
      <Icon name="alert" :size="16" :stroke-width="2" class="tip-icon icon-warning" />
      <div class="tip-content">
        <p><strong>警告:</strong> 过期或异常的驱动程序可能导致蓝屏 (BSOD)、系统卡顿、设备无法识别等性能问题。</p>
        <p>驱动日期超过 2 年将被标记为「可能过期」，建议前往设备厂商官网下载最新驱动。</p>
        <p>状态为「异常」的设备请右键「此电脑」→「管理」→「设备管理器」进行排查。</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const loading = ref(true);
const result = ref(null);
const filter = ref("all");
const searchKeyword = ref("");

const normalCount = computed(() => {
  if (!result.value) return 0;
  return result.value.drivers.filter(d => d.status === "正常").length;
});

const expiredCount = computed(() => {
  if (!result.value) return 0;
  return result.value.drivers.filter(d => d.status === "可能过期").length;
});

const problemCount = computed(() => {
  if (!result.value) return 0;
  return result.value.drivers.filter(d => d.status === "异常").length;
});

const filterOptions = computed(() => [
  { value: "all", label: "全部", count: result.value ? result.value.total_drivers : 0 },
  { value: "problem", label: "仅异常", count: problemCount.value },
  { value: "expired", label: "仅过期", count: expiredCount.value },
]);

const filteredDrivers = computed(() => {
  if (!result.value) return [];
  let list = result.value.drivers;
  if (filter.value === "problem") {
    list = list.filter(d => d.status === "异常");
  } else if (filter.value === "expired") {
    list = list.filter(d => d.status === "可能过期");
  }
  const kw = searchKeyword.value.trim().toLowerCase();
  if (kw) {
    list = list.filter(
      d =>
        d.device_name.toLowerCase().includes(kw) ||
        d.manufacturer.toLowerCase().includes(kw) ||
        d.driver_provider.toLowerCase().includes(kw)
    );
  }
  return list;
});

async function checkDrivers() {
  loading.value = true;
  result.value = null;
  try {
    result.value = await invoke("check_drivers");
  } catch (e) {
    console.error("Driver check failed:", e);
    result.value = null;
  }
  loading.value = false;
}

function statusClass(status) {
  if (status === "正常") return "tag-success";
  if (status === "可能过期") return "tag-warning";
  if (status === "异常") return "tag-danger";
  return "tag-info";
}

onMounted(checkDrivers);
</script>

<style scoped>
.driver-check {
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

/* 汇总统计 */
.summary-card {
  display: flex;
  align-items: center;
  padding: 16px 18px;
  margin-bottom: 12px;
}

.summary-stat {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  justify-content: center;
}

.summary-icon {
  flex-shrink: 0;
}

.icon-neutral { color: var(--text-secondary); }
.icon-success { color: var(--success); }
.icon-warning { color: var(--warning); }
.icon-danger { color: var(--danger); }

.stat-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
  margin-top: 2px;
}

.summary-divider {
  width: 1px;
  height: 36px;
  background: var(--border);
}

/* 过滤栏 */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.filter-tabs {
  display: flex;
  gap: 6px;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 11.5px;
  font-weight: 500;
  height: 28px;
}

.tab-btn:hover {
  border-color: var(--border-light);
  color: var(--text-primary);
}

.tab-btn.active {
  background: var(--accent-dim);
  border-color: var(--accent);
  color: var(--accent);
}

.tab-count {
  padding: 0 5px;
  background: var(--bg-hover);
  border-radius: 3px;
  font-size: 10.5px;
  color: var(--text-muted);
}

.tab-btn.active .tab-count {
  background: var(--accent-glow);
  color: var(--accent);
}

.filter-group {
  display: flex;
  align-items: center;
}

.search-group {
  position: relative;
  flex: 1;
  min-width: 200px;
}

.search-icon {
  position: absolute;
  left: 9px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  pointer-events: none;
}

.filter-input {
  width: 100%;
  padding: 6px 10px 6px 28px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  height: 28px;
}

.filter-input:focus {
  outline: none;
  border-color: var(--accent);
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

/* 驱动表格 */
.driver-card {
  overflow: hidden;
}

.driver-table-wrapper {
  max-height: 560px;
  overflow-y: auto;
}

.drv-name-cell {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.drv-name {
  font-weight: 500;
  font-size: 12px;
  color: var(--text-primary);
}

.drv-problem {
  font-size: 10.5px;
  color: var(--danger);
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

/* 提示卡片 */
.tip-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  margin-top: 12px;
}

.tip-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.tip-card p {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  line-height: 1.6;
}

.tip-card p:last-child {
  margin-bottom: 0;
}

.tip-card.tip-warning { border-color: var(--warning); }

@media (max-width: 720px) {
  .summary-card {
    flex-wrap: wrap;
    gap: 14px 0;
  }
  .summary-divider {
    display: none;
  }
  .summary-stat {
    flex: 1 1 45%;
  }
}
</style>
