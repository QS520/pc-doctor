<template>
  <div class="disk-cleanup fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">C 盘清理</h1>
        <p class="page-subtitle">扫描并清理系统垃圾文件，释放磁盘空间</p>
      </div>
      <div class="header-actions">
        <span v-if="scanResult" class="total-info">
          <Icon name="disc" :size="13" :stroke-width="1.75" />
          共发现 <strong class="mono">{{ scanResult.total_size_mb.toFixed(1) }}</strong> MB 垃圾
        </span>
        <button class="btn btn-ghost btn-sm" @click="scan" :disabled="scanning">
          <span v-if="scanning" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          重新扫描
        </button>
      </div>
    </div>

    <!-- 操作栏 -->
    <div class="card action-bar" v-if="scanResult && scanResult.categories.length > 0">
      <div class="action-left">
        <label class="checkbox-all">
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          <span>全选安全项</span>
        </label>
        <span class="selected-info">
          已选 <strong class="mono">{{ selectedIds.length }}</strong> 项 · 共
          <span class="mono">{{ selectedSize.toFixed(1) }}</span> MB
        </span>
      </div>
      <button class="btn btn-danger btn-sm" @click="clean" :disabled="cleaning || selectedIds.length === 0">
        <span v-if="cleaning" class="spinner" style="width:12px;height:12px"></span>
        <Icon v-else name="trash" :size="13" :stroke-width="2" />
        清理选中项
      </button>
    </div>

    <!-- 清理结果 -->
    <div v-if="cleanResult" class="card result-card" :class="{ success: cleanResult.success, error: !cleanResult.success }">
      <div class="result-header">
        <span class="result-icon" :class="cleanResult.success ? 'icon-success' : 'icon-error'">
          <Icon :name="cleanResult.success ? 'check' : 'alert'" :size="16" :stroke-width="2" />
        </span>
        <div>
          <p class="result-title">清理完成</p>
          <p class="result-detail">
            删除 <span class="mono">{{ cleanResult.deleted_files }}</span> 个文件 · 释放
            <span class="mono">{{ cleanResult.freed_mb.toFixed(1) }}</span> MB 空间
            <span v-if="cleanResult.skipped > 0">
              · 跳过 <span class="mono">{{ cleanResult.skipped }}</span> 个被占用文件
            </span>
          </p>
        </div>
      </div>
      <div v-if="cleanResult.errors.length > 0" class="result-errors">
        <p v-for="(err, i) in cleanResult.errors.slice(0, 5)" :key="i" class="error-line">{{ err }}</p>
      </div>
    </div>

    <!-- 初始扫描提示 -->
    <div class="scan-prompt" v-if="!hasLoaded && !scanning">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始扫描</p>
      <button class="btn btn-primary" @click="scan">开始扫描</button>
    </div>

    <!-- 分类列表 -->
    <div v-if="scanning" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在扫描垃圾文件...</p>
    </div>

    <div v-else-if="scanResult && scanResult.categories.length > 0" class="category-list">
      <div
        v-for="cat in scanResult.categories"
        :key="cat.id"
        class="category-item"
        :class="{ disabled: !cat.safe_to_delete }"
      >
        <label class="category-check">
          <input
            type="checkbox"
            v-model="selectedIds"
            :value="cat.id"
            :disabled="!cat.safe_to_delete && cat.id !== 'windows_old'"
          />
        </label>
        <div class="category-info">
          <div class="category-header">
            <Icon name="folder" :size="13" :stroke-width="1.75" class="cat-icon" />
            <h4>{{ cat.name }}</h4>
            <span class="tag" :class="cat.size_mb > 0 ? 'tag-info' : 'tag-neutral'">
              {{ cat.size_mb > 0 ? cat.size_mb.toFixed(1) + ' MB' : '0 MB' }}
            </span>
            <span v-if="cat.file_count > 0" class="file-count">
              <span class="mono">{{ cat.file_count }}</span> 个文件
            </span>
            <span v-if="!cat.safe_to_delete" class="tag tag-warning">需确认</span>
          </div>
          <p class="category-desc">{{ cat.description }}</p>
          <p class="category-path mono">{{ cat.path }}</p>
        </div>
      </div>
    </div>

    <!-- 快捷工具 -->
    <div class="card quick-tools" v-if="hasLoaded && !scanning">
      <div class="card-header">
        <span class="card-title">快捷工具</span>
      </div>
      <div class="card-body">
        <div class="tools-grid">
          <button class="tool-btn" @click="flushDns">
            <span class="tool-icon"><Icon name="globe" :size="16" :stroke-width="1.75" /></span>
            <span class="tool-text">
              <strong>刷新 DNS 缓存</strong>
              <small>解决网页打不开 / 解析错误</small>
            </span>
          </button>
          <button class="tool-btn" @click="emptyRecycle">
            <span class="tool-icon"><Icon name="trash" :size="16" :stroke-width="1.75" /></span>
            <span class="tool-text">
              <strong>清空回收站</strong>
              <small>立即释放回收站空间</small>
            </span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { useScanLogStore } from "../stores/scanLog";

const scanLog = useScanLogStore();

const scanning = ref(false);
const hasLoaded = ref(false);
const cleaning = ref(false);
const scanResult = ref(null);
const cleanResult = ref(null);
const selectedIds = ref([]);

const allSelected = computed(() => {
  if (!scanResult.value) return false;
  const safeIds = scanResult.value.categories
    .filter(c => c.safe_to_delete)
    .map(c => c.id);
  return safeIds.every(id => selectedIds.value.includes(id));
});

const selectedSize = computed(() => {
  if (!scanResult.value) return 0;
  return scanResult.value.categories
    .filter(c => selectedIds.value.includes(c.id))
    .reduce((sum, c) => sum + c.size_mb, 0);
});

async function scan() {
  scanning.value = true;
  cleanResult.value = null;
  scanLog.startTask("C盘垃圾扫描", "cleanup");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示扫描流程） =====
  scanLog.pushPhases([
    "初始化垃圾扫描引擎...",
    { msg: "定位系统目录: AppData/Temp、Windows/Temp、回收站、预读取文件、浏览器缓存", level: "info" },
    "枚举临时文件与缓存路径...",
    "分析 Windows 组件存储 (WinSxS) 冗余与更新备份",
    { msg: "正在执行全盘垃圾统计...", level: "warning" },
  ]);

  try {
    scanResult.value = await invoke("scan_junk_files");
    const r = scanResult.value;

    // 默认选中所有安全项
    selectedIds.value = r.categories
      .filter(c => c.safe_to_delete && c.size_mb > 0)
      .map(c => c.id);

    const totalMB = r.total_size_mb.toFixed(1);
    const catCount = r.categories.length;

    // ===== 结果详情报告（垃圾分类统计） =====
    scanLog.pushSeparator("垃圾分类统计");
    let safeCount = 0, unsafeCount = 0;
    r.categories.forEach((c) => {
      if (c.safe_to_delete) safeCount++; else unsafeCount++;
      const lvl = c.size_mb > 0 ? "info" : "dim";
      scanLog.pushDetail(c.name, `${c.size_mb.toFixed(1)} MB · ${c.file_count} 个文件`, lvl);
    });

    scanLog.pushLog(`扫描完成: 共 ${catCount} 类垃圾`, "success");
    scanLog.pushDetail("可安全清理", `${safeCount} 类`, "success");
    if (unsafeCount > 0) {
      scanLog.pushDetail("需确认项", `${unsafeCount} 类 (如 Windows.old)`, "warning");
    }

    scanLog.pushSeparator();
    scanLog.pushLog(`可释放空间总计: ${totalMB} MB`, "info");
    scanLog.complete(`扫描完成 — 发现 ${catCount} 类垃圾，可释放 ${totalMB} MB`);
  } catch (e) {
    console.error("Scan failed:", e);
    scanLog.pushLog("扫描异常: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  scanning.value = false;
  hasLoaded.value = true;
}

function toggleAll(e) {
  if (e.target.checked) {
    selectedIds.value = scanResult.value.categories
      .filter(c => c.safe_to_delete)
      .map(c => c.id);
  } else {
    selectedIds.value = [];
  }
}

async function clean() {
  cleaning.value = true;
  cleanResult.value = null;
  scanLog.startTask("C盘垃圾清理", "cleanup");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示清理流程） =====
  scanLog.pushPhases([
    "校验待清理项参数...",
    { msg: `已选中 ${selectedIds.value.length} 个分类等待清理`, level: "info" },
    "锁定目标文件句柄，防止误删系统文件",
    "准备安全删除队列 (占用中文件将自动跳过)...",
  ]);

  try {
    cleanResult.value = await invoke("clean_junk_files", { categoryIds: selectedIds.value });
    const cr = cleanResult.value;

    scanLog.pushSeparator("清理结果");
    if (cr.success) {
      scanLog.pushDetail("已删除文件", `${cr.deleted_files} 个`, "success");
      scanLog.pushDetail("释放空间", `${cr.freed_mb.toFixed(1)} MB`, "success");
      if (cr.skipped > 0) {
        scanLog.pushDetail("跳过文件", `${cr.skipped} 个 (被占用)`, "warning");
      } else {
        scanLog.pushDetail("跳过文件", "0 个", "success");
      }
      if (cr.errors && cr.errors.length > 0) {
        scanLog.pushLog(`清理过程中有 ${cr.errors.length} 个文件处理失败`, "warning");
        cr.errors.slice(0, 5).forEach((err, i) => scanLog.pushDetail(String(i + 1), err, "dim"));
        if (cr.errors.length > 5) {
          scanLog.pushDetail("...", `还有 ${cr.errors.length - 5} 个错误未列出`, "dim");
        }
      }
      scanLog.pushSeparator();
      scanLog.complete(`清理完成 — 释放 ${cr.freed_mb.toFixed(1)} MB，删除 ${cr.deleted_files} 个文件`);
    } else {
      scanLog.pushLog("清理未完成 (部分项处理失败)", "error");
      if (cr.errors && cr.errors.length > 0) {
        cr.errors.slice(0, 5).forEach((err, i) => scanLog.pushDetail(String(i + 1), err, "dim"));
      }
      scanLog.fail("清理未完成");
    }
    // 重新扫描
    setTimeout(() => scan(), 1500);
  } catch (e) {
    cleaning.value = false;
    cleanResult.value = {
      success: false,
      deleted_files: 0,
      freed_mb: 0,
      errors: [String(e)],
      skipped: 0,
    };
    scanLog.pushLog("清理异常: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  cleaning.value = false;
}

async function flushDns() {
  try {
    await invoke("flush_dns_cache");
    alert("DNS 缓存已刷新");
  } catch (e) {
    alert("刷新失败: " + e);
  }
}

async function emptyRecycle() {
  if (!confirm("确定要清空回收站吗？此操作不可撤销。")) return;
  try {
    await invoke("empty_recycle_bin");
    alert("回收站已清空");
    scan();
  } catch (e) {
    alert("清空失败: " + e);
  }
}
</script>

<style scoped>
.disk-cleanup {
  max-width: 1600px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 16px;
  margin-bottom: 22px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 14px;
}

.total-info {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.total-info strong {
  color: var(--accent);
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

/* 操作栏 */
.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 14px;
  margin-bottom: 12px;
}

.action-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.checkbox-all {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 12.5px;
  color: var(--text-secondary);
}

.checkbox-all input {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}

.selected-info {
  font-size: 11.5px;
  color: var(--text-muted);
}

.selected-info strong {
  color: var(--text-secondary);
  font-weight: 600;
}

/* 结果卡片 */
.result-card {
  padding: 14px;
  margin-bottom: 12px;
  border-left: 3px solid var(--border-light);
}

.result-card.success {
  border-left-color: var(--success);
}

.result-card.error {
  border-left-color: var(--danger);
}

.result-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.result-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.result-icon.icon-success {
  background: var(--success-dim);
  color: var(--success);
}

.result-icon.icon-error {
  background: var(--warning-dim);
  color: var(--warning);
}

.result-title {
  font-weight: 600;
  font-size: 13px;
  color: var(--text-primary);
}

.result-detail {
  font-size: 11.5px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.result-errors {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--border);
}

.error-line {
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 4px;
  font-family: var(--font-mono);
  word-break: break-all;
}

/* 分类列表 */
.category-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.category-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  transition: border-color 0.15s ease;
}

.category-item:hover {
  border-color: var(--border-light);
}

.category-item.disabled {
  opacity: 0.6;
}

.category-check {
  display: flex;
  align-items: center;
  padding-top: 2px;
}

.category-check input {
  width: 14px;
  height: 14px;
  accent-color: var(--accent);
  cursor: pointer;
}

.category-info {
  flex: 1;
  min-width: 0;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
  flex-wrap: wrap;
}

.cat-icon {
  color: var(--text-muted);
}

.category-header h4 {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.file-count {
  font-size: 11px;
  color: var(--text-muted);
}

.category-desc {
  font-size: 11.5px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.category-path {
  font-size: 10.5px;
  color: var(--text-muted);
  word-break: break-all;
}

/* 快捷工具 */
.quick-tools {
  margin-top: 16px;
}

.tools-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 10px;
}

.tool-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  text-align: left;
  transition: all 0.15s ease;
}

.tool-btn:hover {
  border-color: var(--accent);
  background: var(--bg-hover);
}

.tool-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  background: var(--accent-dim);
  color: var(--accent);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.tool-text {
  display: flex;
  flex-direction: column;
}

.tool-text strong {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
}

.tool-text small {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 1px;
}
</style>
