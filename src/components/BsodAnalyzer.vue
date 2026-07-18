<template>
  <div class="bsod-analyzer fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">蓝屏诊断</h1>
        <p class="page-subtitle">分析蓝屏崩溃记录与系统错误事件</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="loadErrors" :disabled="loadingErrors">
          <span v-if="loadingErrors" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="list" :size="13" :stroke-width="2" />
          系统错误日志
        </button>
        <button class="btn btn-ghost btn-sm" @click="analyze" :disabled="analyzing">
          <span v-if="analyzing" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          重新分析
        </button>
      </div>
    </div>

    <!-- 初始状态：开始扫描 -->
    <div v-if="!hasLoaded && !analyzing" class="scan-prompt">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始扫描</p>
      <button class="btn btn-primary" @click="analyze">开始扫描</button>
    </div>

    <!-- 概览卡片 -->
    <div class="overview-grid" v-if="result">
      <div class="stat-card">
        <span class="stat-icon" :class="{ 'icon-danger': result.crash_count > 0 }">
          <Icon name="alert" :size="16" :stroke-width="1.75" />
        </span>
        <div>
          <p class="stat-label">蓝屏次数</p>
          <p class="stat-value mono" :class="{ danger: result.crash_count > 0 }">{{ result.crash_count }}</p>
        </div>
      </div>
      <div class="stat-card">
        <span class="stat-icon">
          <Icon name="clock" :size="16" :stroke-width="1.75" />
        </span>
        <div>
          <p class="stat-label">最近一次蓝屏</p>
          <p class="stat-value mono">{{ result.last_crash_date }}</p>
        </div>
      </div>
    </div>

    <!-- 常见原因分析 -->
    <div class="card" v-if="result && result.common_causes.length > 0">
      <div class="card-header">
        <span class="title-group">
          <Icon name="wrench" :size="13" :stroke-width="1.75" />
          <span class="card-title">诊断建议</span>
        </span>
      </div>
      <div class="card-body">
        <div class="causes-list">
          <div v-for="(cause, i) in result.common_causes" :key="i" class="cause-item">
            <span class="cause-num mono">{{ i + 1 }}</span>
            <p>{{ cause }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 崩溃记录列表 -->
    <div class="card" v-if="result && result.crashes.length > 0">
      <div class="card-header">
        <span class="title-group">
          <Icon name="bug" :size="13" :stroke-width="1.75" />
          <span class="card-title">崩溃记录</span>
        </span>
        <span class="tag tag-danger">{{ result.crashes.length }} 条</span>
      </div>
      <div class="card-body">
        <div class="crash-list">
          <div v-for="(crash, i) in result.crashes" :key="i" class="crash-item">
            <div class="crash-header">
              <span class="dot dot-danger"></span>
              <span class="crash-date mono">{{ crash.date }}</span>
              <span class="tag tag-danger">{{ crash.bug_check_code }}</span>
              <span class="crash-name">{{ crash.bug_check_name }}</span>
            </div>
            <p class="crash-desc">{{ crash.description }}</p>
            <div v-if="crash.parameters.length > 0" class="crash-params">
              <span class="param-label">参数:</span>
              <code v-for="p in crash.parameters" :key="p" class="param-value mono">{{ p }}</code>
            </div>
            <div v-if="crash.dump_file" class="crash-dump">
              <Icon name="hard-drive" :size="12" :stroke-width="1.75" class="dump-icon" />
              <span class="dump-label">转储文件:</span>
              <code class="dump-path mono">{{ crash.dump_file }}</code>
              <span class="dump-size mono" v-if="crash.dump_size_mb > 0">{{ crash.dump_size_mb }} MB</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 系统错误日志 -->
    <div class="card" v-if="systemErrors.length > 0">
      <div class="card-header">
        <span class="title-group">
          <Icon name="alert" :size="13" :stroke-width="1.75" />
          <span class="card-title">系统错误事件 · 最近 50 条</span>
        </span>
      </div>
      <div class="card-body error-table-body">
        <div class="error-table-wrapper">
          <table>
            <thead>
              <tr>
                <th>时间</th>
                <th>来源</th>
                <th>事件ID</th>
                <th>级别</th>
                <th>消息</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(err, i) in systemErrors" :key="i">
                <td class="nowrap mono">{{ err.time }}</td>
                <td>{{ err.source }}</td>
                <td class="mono">{{ err.event_id }}</td>
                <td>
                  <span class="status-inline">
                    <span class="dot" :class="err.level.includes('严重') || err.level.includes('Critical') ? 'dot-danger' : 'dot-warning'"></span>
                    <span class="level-text">{{ err.level }}</span>
                  </span>
                </td>
                <td class="msg-cell">{{ err.message.substring(0, 200) }}{{ err.message.length > 200 ? '...' : '' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- 无崩溃 -->
    <div class="card empty-result" v-if="result && result.crash_count === 0">
      <span class="empty-icon">
        <Icon name="check" :size="28" :stroke-width="1.75" />
      </span>
      <h3>未检测到蓝屏记录</h3>
      <p>系统运行稳定，没有发现蓝屏崩溃记录。</p>
    </div>

    <div v-if="analyzing" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在分析蓝屏记录和系统事件...</p>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { useScanLogStore } from "../stores/scanLog";

const analyzing = ref(false);
const hasLoaded = ref(false);
const loadingErrors = ref(false);
const result = ref(null);
const systemErrors = ref([]);
const scanLog = useScanLogStore();

async function analyze() {
  analyzing.value = true;
  scanLog.startTask("蓝屏 dump 分析", "bsod");
  let ok = true;
  try {
    scanLog.pushLog("分析蓝屏 dump 与崩溃记录...", "info");
    result.value = await invoke("analyze_bsod");
    scanLog.pushLog(`分析完成：共 ${result.value.crash_count} 次蓝屏`, "success");
  } catch (e) {
    console.error("Analysis failed:", e);
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  analyzing.value = false;
  hasLoaded.value = true;
  if (ok) scanLog.complete(`蓝屏 dump 分析完成，共 ${result.value.crash_count} 次蓝屏记录`);
}

async function loadErrors() {
  loadingErrors.value = true;
  scanLog.startTask("系统错误日志", "bsod");
  let ok = true;
  try {
    scanLog.pushLog("读取最近 50 条系统错误事件...", "info");
    systemErrors.value = await invoke("get_system_errors", { limit: 50 });
    scanLog.pushLog(`读取到 ${systemErrors.value.length} 条错误事件`, "success");
  } catch (e) {
    console.error("Failed to load errors:", e);
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  loadingErrors.value = false;
  if (ok) scanLog.complete(`系统错误日志加载完成，共 ${systemErrors.value.length} 条`);
}
</script>

<style scoped>
.bsod-analyzer {
  max-width: 1600px;
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

.title-group {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  color: var(--text-muted);
}

/* 概览 */
.overview-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
  margin-bottom: 12px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.stat-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--info-dim);
  color: var(--info);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.stat-icon.icon-danger {
  background: var(--danger-dim);
  color: var(--danger);
}

.stat-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin-top: 2px;
}

.stat-value.danger {
  color: var(--danger);
}

.card {
  margin-bottom: 12px;
}

/* 诊断建议 */
.causes-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cause-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.cause-num {
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

.cause-item p {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

/* 崩溃记录 */
.crash-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.crash-item {
  padding: 12px;
  background: var(--bg-elevated);
  border-radius: var(--radius-sm);
  border-left: 3px solid var(--danger);
}

.crash-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.crash-date {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.crash-name {
  font-size: 12px;
  color: var(--accent);
  font-weight: 500;
}

.crash-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  line-height: 1.5;
}

.crash-params {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 6px;
}

.param-label {
  font-size: 11px;
  color: var(--text-muted);
}

.param-value {
  font-size: 11px;
  color: var(--accent);
  background: var(--bg-input);
  padding: 1px 6px;
  border-radius: 3px;
  border: 1px solid var(--border);
}

.crash-dump {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  flex-wrap: wrap;
}

.dump-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.dump-label {
  font-size: 11px;
  color: var(--text-muted);
}

.dump-path {
  font-size: 10.5px;
  color: var(--text-muted);
  background: var(--bg-input);
  padding: 1px 6px;
  border-radius: 3px;
  border: 1px solid var(--border);
  word-break: break-all;
}

.dump-size {
  font-size: 10.5px;
  color: var(--text-muted);
}

/* 错误表 */
.error-table-body {
  padding: 0;
}

.error-table-wrapper {
  max-height: 500px;
  overflow-y: auto;
}

.nowrap {
  white-space: nowrap;
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
  max-width: 400px;
  color: var(--text-secondary);
}

/* 空结果 */
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
</style>
