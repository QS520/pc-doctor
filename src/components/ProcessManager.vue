<template>
  <div class="process-manager fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">进程管理</h1>
        <p class="page-subtitle">查看运行中的进程并管理资源占用</p>
      </div>
      <div class="header-actions">
        <label class="sort-wrap">
          <Icon name="sort" :size="12" :stroke-width="1.75" class="sort-icon" />
          <select v-model="sortBy" @change="loadProcesses" class="sort-select">
            <option value="cpu">按 CPU 排序</option>
            <option value="memory">按内存排序</option>
            <option value="name">按名称排序</option>
          </select>
          <Icon name="chevron-down" :size="12" :stroke-width="2" class="sort-caret" />
        </label>
        <button class="btn btn-ghost btn-sm" @click="loadProcesses" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          刷新
        </button>
      </div>
    </div>

    <!-- 进程列表 -->
    <div class="card process-card" v-if="!loading">
      <div class="process-table-wrapper">
        <table>
          <thead>
            <tr>
              <th>进程名称</th>
              <th class="num-col">PID</th>
              <th class="num-col">CPU</th>
              <th class="num-col">内存 (MB)</th>
              <th>状态</th>
              <th class="op-col">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="proc in processes" :key="proc.pid">
              <td>
                <div class="proc-name-cell">
                  <span class="proc-name">{{ proc.name }}</span>
                  <span v-if="proc.cpu_usage > 50 || proc.memory_mb > 500" class="tag tag-warning">高占用</span>
                </div>
              </td>
              <td class="num-col mono">{{ proc.pid }}</td>
              <td class="num-col">
                <div class="cpu-cell">
                  <span class="mono" :class="{ 'high-usage': proc.cpu_usage > 50, 'med-usage': proc.cpu_usage > 10 }">
                    {{ proc.cpu_usage.toFixed(1) }}%
                  </span>
                  <div class="mini-bar">
                    <div class="mini-fill" :class="getProgressClass(proc.cpu_usage)" :style="{ width: Math.min(proc.cpu_usage, 100) + '%' }"></div>
                  </div>
                </div>
              </td>
              <td class="num-col">
                <span class="mono" :class="{ 'high-usage': proc.memory_mb > 500 }">
                  {{ proc.memory_mb.toFixed(0) }}
                </span>
              </td>
              <td>
                <span class="status-inline">
                  <span class="dot" :class="proc.status === 'RunState' ? 'dot-success' : 'dot-info'"></span>
                  <span class="status-text">{{ translateStatus(proc.status) }}</span>
                </span>
              </td>
              <td class="op-col">
                <button class="btn btn-danger btn-sm" @click="killProc(proc)">
                  <Icon name="stop" :size="11" :stroke-width="2" />
                  结束
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在读取进程列表...</p>
    </div>

    <!-- 提示 -->
    <div class="tip-card" v-if="!loading">
      <span class="tip-icon"><Icon name="alert" :size="14" :stroke-width="2" /></span>
      <div>
        <p><strong>注意:</strong> 结束系统关键进程可能导致系统不稳定或蓝屏。</p>
        <p>建议只结束你认识的应用程序进程。如果不确定，请勿操作。</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const loading = ref(true);
const processes = ref([]);
const sortBy = ref("cpu");

async function loadProcesses() {
  loading.value = true;
  try {
    processes.value = await invoke("get_processes", { sortBy: sortBy.value });
  } catch (e) {
    console.error("Failed to load processes:", e);
  }
  loading.value = false;
}

async function killProc(proc) {
  if (!confirm(`确定要结束进程 "${proc.name}" (PID: ${proc.pid}) 吗？`)) return;

  try {
    await invoke("kill_process", { pid: proc.pid });
    // 从列表中移除
    processes.value = processes.value.filter(p => p.pid !== proc.pid);
  } catch (e) {
    alert("结束进程失败: " + e + "\n可能需要管理员权限。");
  }
}

function getProgressClass(percent) {
  if (percent > 80) return "high";
  if (percent > 40) return "medium";
  if (percent > 10) return "normal";
  return "low";
}

function translateStatus(status) {
  const map = {
    RunState: "运行中",
    SleepState: "睡眠",
    WaitState: "等待",
    Unknown: "未知",
  };
  return map[status] || status;
}

onMounted(loadProcesses);
</script>

<style scoped>
.process-manager {
  max-width: 950px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 22px;
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 80px 20px;
  color: var(--text-muted);
  font-size: 12px;
}

/* 排序选择 */
.sort-wrap {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px 4px 10px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  transition: border-color 0.15s ease;
}

.sort-wrap:hover {
  border-color: var(--border-light);
}

.sort-icon {
  flex-shrink: 0;
}

.sort-caret {
  flex-shrink: 0;
  color: var(--text-muted);
}

.sort-select {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  outline: none;
  padding: 0;
  appearance: none;
}

.sort-select option {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

/* 表格 */
.process-table-wrapper {
  max-height: 600px;
  overflow-y: auto;
}

.num-col {
  text-align: right;
}

.op-col {
  text-align: center;
  width: 80px;
}

.proc-name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
}

.proc-name {
  font-weight: 500;
  font-size: 12px;
}

.cpu-cell {
  display: flex;
  align-items: center;
  gap: 8px;
  justify-content: flex-end;
}

.mini-bar {
  width: 50px;
  height: 4px;
  background: var(--bg-input);
  border-radius: 2px;
  overflow: hidden;
}

.mini-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s;
}

.mini-fill.high {
  background: var(--danger);
}

.mini-fill.medium {
  background: var(--warning);
}

.mini-fill.normal {
  background: var(--accent);
}

.mini-fill.low {
  background: var(--success);
}

.high-usage {
  color: var(--danger);
  font-weight: 600;
}

.med-usage {
  color: var(--warning);
}

.status-inline {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.status-text {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 提示 */
.tip-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  margin-top: 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-left: 3px solid var(--warning);
  border-radius: var(--radius);
}

.tip-icon {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 1px;
  color: var(--warning);
}

.tip-card p {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 2px;
}

.tip-card p:last-child {
  margin-bottom: 0;
}

.tip-card strong {
  color: var(--text-primary);
}
</style>
