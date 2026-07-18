<template>
  <div class="disk-defrag fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">磁盘碎片整理</h1>
        <p class="page-subtitle">分析磁盘碎片情况，对 HDD 整理、对 SSD 执行 TRIM 优化</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="analyze" :disabled="loading || running">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          重新分析
        </button>
      </div>
    </div>

    <!-- 初始扫描提示 -->
    <div class="scan-prompt" v-if="!hasLoaded && !loading">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始扫描</p>
      <button class="btn btn-primary" @click="analyze">分析磁盘</button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在分析磁盘碎片情况...</p>
    </div>

    <!-- 磁盘列表 -->
    <div v-else-if="drives.length > 0" class="drive-list">
      <div v-for="drive in drives" :key="drive.drive" class="card drive-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="hard-drive" :size="13" :stroke-width="1.75" />
            <span class="drive-letter mono">{{ drive.drive }}:</span>
            <span class="tag" :class="drive.is_ssd ? 'tag-info' : 'tag-warning'">{{ drive.drive_type }}</span>
          </div>
          <span v-if="!drive.is_ssd" class="dot" :class="drive.fragmentation_percent > 30 ? 'dot-danger' : drive.fragmentation_percent > 10 ? 'dot-warning' : 'dot-success'"></span>
          <span v-else class="dot dot-info"></span>
        </div>
        <div class="card-body">
          <!-- 碎片信息 (仅 HDD) -->
          <div class="drive-meta" v-if="!drive.is_ssd">
            <div class="meta-item">
              <span class="meta-label">碎片率</span>
              <span class="mono meta-val" :class="fragClass(drive.fragmentation_percent)">{{ drive.fragmentation_percent.toFixed(1) }}%</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">碎片文件</span>
              <span class="mono meta-val">{{ drive.total_fragmented_files }}</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">分析时间</span>
              <span class="mono meta-val">{{ drive.last_analysis }}</span>
            </div>
          </div>

          <!-- 碎片率进度条 (仅 HDD) -->
          <div v-if="!drive.is_ssd" class="frag-bar">
            <div class="bar">
              <div class="bar-fill" :class="fragProgressClass(drive.fragmentation_percent)" :style="{ width: Math.min(drive.fragmentation_percent, 100) + '%' }"></div>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="drive-actions">
            <p class="drive-note">
              <Icon name="info" :size="12" class="note-icon" />
              {{ drive.is_ssd ? '检测到固态硬盘，建议使用 TRIM 优化而非碎片整理。' : '机械硬盘碎片整理可提升读写性能，过程可能较长。' }}
            </p>
            <template v-if="drive.is_ssd">
              <button class="btn btn-primary btn-sm" @click="runTrim(drive)" :disabled="running === drive.drive">
                <span v-if="running === drive.drive" class="spinner" style="width:12px;height:12px"></span>
                <Icon v-else name="zap" :size="12" :stroke-width="2" />
                TRIM 优化
              </button>
            </template>
            <template v-else>
              <button class="btn btn-primary btn-sm" @click="runDefrag(drive)" :disabled="running === drive.drive">
                <span v-if="running === drive.drive" class="spinner" style="width:12px;height:12px"></span>
                <Icon v-else name="wrench" :size="12" :stroke-width="2" />
                碎片整理
              </button>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="hasLoaded" class="card empty-state">
      <Icon name="hard-drive" :size="28" :stroke-width="1.5" class="empty-icon" />
      <p>未检测到磁盘，请以管理员身份运行程序后重新分析。</p>
    </div>

    <!-- 控制台输出 -->
    <div v-if="output" class="card output-card">
      <div class="card-header">
        <div class="title-group">
          <Icon name="terminal" :size="13" :stroke-width="1.75" />
          <span class="card-title">{{ output.title }}</span>
        </div>
        <div class="output-header-right">
          <span v-if="output.duration_secs" class="output-time mono">耗时 {{ output.duration_secs }}s</span>
          <span class="status-pill">
            <span class="dot" :class="output.success ? 'dot-success' : 'dot-danger'"></span>
            {{ output.success ? '成功' : '失败' }}
          </span>
        </div>
      </div>
      <div class="card-body">
        <pre class="output-text mono">{{ output.output || output.error || '（无输出）' }}</pre>
      </div>
    </div>

    <!-- 提示 -->
    <div class="card tip-card">
      <Icon name="alert" :size="16" class="tip-icon tip-warn" />
      <div>
        <p class="tip-title">重要提示</p>
        <p>1. 碎片整理与 TRIM 优化均需要以管理员身份运行此程序。</p>
        <p>2. SSD 固态硬盘请勿执行传统碎片整理，仅使用 TRIM 优化。</p>
        <p>3. 整理过程中请勿断电或强制关闭程序，以免损坏数据。</p>
        <p>4. 大容量 HDD 整理可能需要较长时间，请耐心等待。</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { useScanLogStore } from "../stores/scanLog";

const loading = ref(false);
const hasLoaded = ref(false);
const running = ref("");
const drives = ref([]);
const output = ref(null);
const scanLog = useScanLogStore();

async function analyze() {
  loading.value = true;
  drives.value = [];
  output.value = null;
  scanLog.startTask("磁盘碎片分析", "defrag");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示分析流程） =====
  scanLog.pushPhases([
    "初始化卷分析引擎...",
    { msg: "读取各磁盘卷信息与文件系统结构", level: "info" },
    "分析文件碎片分布与簇占用情况",
    { msg: "计算每块磁盘的碎片率...", level: "warning" },
  ]);

  try {
    drives.value = await invoke("analyze_defrag");

    // ===== 结果详情报告（碎片分析结果） =====
    scanLog.pushSeparator("碎片分析结果");
    scanLog.pushLog(`分析完成，共 ${drives.value.length} 块磁盘`, "success");
    drives.value.forEach((d) => {
      if (d.is_ssd) {
        scanLog.pushDetail(`${d.drive}: (${d.drive_type})`, "固态硬盘 · 建议执行 TRIM 优化", "info");
      } else {
        const lvl =
          d.fragmentation_percent > 30 ? "error" :
          d.fragmentation_percent > 10 ? "warning" : "success";
        scanLog.pushDetail(
          `${d.drive}: (${d.drive_type})`,
          `碎片率 ${d.fragmentation_percent.toFixed(1)}% · 碎片文件 ${d.total_fragmented_files} 个`,
          lvl
        );
      }
    });
    scanLog.complete(`碎片分析完成，共 ${drives.value.length} 块磁盘`);
  } catch (e) {
    console.error("Analyze defrag failed:", e);
    output.value = {
      title: "分析失败",
      success: false,
      output: "",
      error: String(e),
      duration_secs: 0,
    };
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  loading.value = false;
  hasLoaded.value = true;
}

async function runDefrag(drive) {
  if (!confirm(`确定要对 ${drive.drive}: 盘进行碎片整理吗？\n整理期间磁盘性能可能下降，请勿中断。`)) return;
  running.value = drive.drive;
  output.value = null;
  scanLog.startTask("碎片整理", "defrag");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示整理流程） =====
  scanLog.pushPhases([
    `创建 ${drive.drive}: 盘整理计划...`,
    { msg: `锁定卷 ${drive.drive}: 防止写入冲突`, level: "info" },
    { msg: "开始文件碎片整理 (按簇重排，过程可能较长)", level: "warning" },
  ]);

  try {
    const res = await invoke("run_defrag", { drive: drive.drive, optimizeSsd: false });
    output.value = { title: `${drive.drive}: 盘碎片整理`, ...res };

    // ===== 结果详情报告 =====
    scanLog.pushSeparator("整理结果");
    scanLog.pushDetail("目标磁盘", `${drive.drive}: 盘 (${drive.drive_type})`, "info");
    scanLog.pushDetail("执行结果", res.success ? "成功" : "失败", res.success ? "success" : "error");
    if (res.duration_secs != null) {
      scanLog.pushDetail("耗时", `${res.duration_secs} 秒`, "dim");
    }
    if (res.output) {
      scanLog.pushLog("控制台输出摘要:", "info");
      const snippet = res.output.length > 200 ? res.output.slice(0, 200) + "..." : res.output;
      scanLog.pushDetail("输出", snippet, "dim");
    }
    scanLog.complete(`${drive.drive}: 盘碎片整理完成`);
  } catch (e) {
    output.value = {
      title: `${drive.drive}: 盘碎片整理`,
      success: false,
      output: "",
      error: String(e),
      duration_secs: 0,
    };
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  running.value = "";
  // 完成后重新分析
  setTimeout(() => analyze(), 1500);
}

async function runTrim(drive) {
  if (!confirm(`确定要对所有 SSD 执行 TRIM 优化吗？`)) return;
  running.value = drive.drive;
  output.value = null;
  scanLog.startTask("TRIM 优化", "defrag");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示优化流程） =====
  scanLog.pushPhases([
    `定位固态硬盘 ${drive.drive}: ...`,
    { msg: "枚举待优化的 SSD 卷", level: "info" },
    { msg: "向 SSD 控制器发送 TRIM 指令回收空闲块", level: "warning" },
  ]);

  try {
    const res = await invoke("run_trim_all");
    output.value = { title: "TRIM 优化", ...res };

    // ===== 结果详情报告 =====
    scanLog.pushSeparator("TRIM 结果");
    scanLog.pushDetail("目标磁盘", `${drive.drive}: 盘 (${drive.drive_type})`, "info");
    scanLog.pushDetail("执行结果", res.success ? "成功" : "失败", res.success ? "success" : "error");
    if (res.duration_secs != null) {
      scanLog.pushDetail("耗时", `${res.duration_secs} 秒`, "dim");
    }
    if (res.output) {
      scanLog.pushLog("控制台输出摘要:", "info");
      const snippet = res.output.length > 200 ? res.output.slice(0, 200) + "..." : res.output;
      scanLog.pushDetail("输出", snippet, "dim");
    }
    scanLog.complete("TRIM 优化完成");
  } catch (e) {
    output.value = {
      title: "TRIM 优化",
      success: false,
      output: "",
      error: String(e),
      duration_secs: 0,
    };
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  running.value = "";
  setTimeout(() => analyze(), 1500);
}

function fragClass(percent) {
  if (percent > 30) return "frag-high";
  if (percent > 10) return "frag-medium";
  return "frag-low";
}

function fragProgressClass(percent) {
  if (percent > 30) return "high";
  if (percent > 10) return "medium";
  return "normal";
}
</script>

<style scoped>
.disk-defrag {
  max-width: 1600px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 22px;
  flex-wrap: wrap;
  gap: 12px;
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.title-group {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-secondary);
}

.drive-letter {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 0.02em;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 60px 20px;
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

.drive-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 12px;
}

.drive-card .card-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.drive-meta {
  display: flex;
  align-items: center;
  gap: 28px;
  flex-wrap: wrap;
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.meta-label {
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.meta-val {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.frag-high {
  color: var(--danger);
}

.frag-medium {
  color: var(--warning);
}

.frag-low {
  color: var(--success);
}

.frag-bar {
  margin-top: -4px;
}

/* 操作区：仅在其上方有内容时显示分隔线 */
.drive-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.drive-card .card-body > .drive-actions:not(:first-child) {
  padding-top: 12px;
  border-top: 1px solid var(--border);
}

.drive-note {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
  flex: 1;
  min-width: 200px;
}

.note-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 50px 20px;
  color: var(--text-muted);
  font-size: 12px;
}

.empty-icon {
  color: var(--text-faint);
}

/* 输出 */
.output-card {
  margin-bottom: 12px;
}

.output-header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.output-time {
  font-size: 11px;
  color: var(--text-muted);
}

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-secondary);
}

.output-text {
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 11.5px;
  max-height: 320px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.5;
}

/* 提示卡 */
.tip-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  background: var(--bg-elevated);
}

.tip-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.tip-warn {
  color: var(--warning);
}

.tip-card p {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 3px;
  line-height: 1.6;
}

.tip-card .tip-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 5px;
}
</style>
