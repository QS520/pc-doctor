<template>
  <div class="system-repair fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">系统修复</h1>
        <p class="page-subtitle">运行系统内置工具修复损坏文件、组件存储与磁盘错误</p>
      </div>
    </div>

    <!-- 修复工具 -->
    <div class="repair-grid">
      <!-- SFC -->
      <div class="card repair-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="shield" :size="13" :stroke-width="1.75" />
            <span class="card-title">SFC 系统文件检查</span>
          </div>
          <span class="dot" :class="running === 'sfc' ? 'dot-info' : (sfcResult ? (sfcResult.success ? 'dot-success' : 'dot-danger') : 'dot-muted')"></span>
        </div>
        <div class="card-body">
          <p class="repair-desc">扫描并修复损坏的 Windows 系统文件</p>
          <p class="repair-cmd mono">sfc /scannow</p>
          <div class="repair-footer">
            <button class="btn btn-primary btn-sm" @click="runSfc" :disabled="running === 'sfc'">
              <span v-if="running === 'sfc'" class="spinner" style="width:12px;height:12px"></span>
              <Icon v-else name="play" :size="11" :stroke-width="2" />
              {{ running === 'sfc' ? '扫描中' : '开始修复' }}
            </button>
            <span v-if="sfcResult" class="repair-time mono">耗时 {{ sfcResult.duration_secs }}s</span>
          </div>
        </div>
      </div>

      <!-- DISM -->
      <div class="card repair-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="database" :size="13" :stroke-width="1.75" />
            <span class="card-title">DISM 组件存储修复</span>
          </div>
          <span class="dot" :class="running === 'dism' ? 'dot-info' : (dismResult ? (dismResult.success ? 'dot-success' : 'dot-danger') : 'dot-muted')"></span>
        </div>
        <div class="card-body">
          <p class="repair-desc">修复 Windows 组件存储，解决 SFC 无法修复的问题</p>
          <p class="repair-cmd mono">DISM /Online /Cleanup-Image /RestoreHealth</p>
          <div class="repair-footer">
            <button class="btn btn-primary btn-sm" @click="runDism" :disabled="running === 'dism'">
              <span v-if="running === 'dism'" class="spinner" style="width:12px;height:12px"></span>
              <Icon v-else name="play" :size="11" :stroke-width="2" />
              {{ running === 'dism' ? '修复中' : '开始修复' }}
            </button>
            <span v-if="dismResult" class="repair-time mono">耗时 {{ dismResult.duration_secs }}s</span>
          </div>
        </div>
      </div>

      <!-- CHKDSK -->
      <div class="card repair-card">
        <div class="card-header">
          <div class="title-group">
            <Icon name="disc" :size="13" :stroke-width="1.75" />
            <span class="card-title">CHKDSK 磁盘检查</span>
          </div>
          <span class="dot" :class="running === 'chkdsk' ? 'dot-info' : (chkdskResult ? (chkdskResult.success ? 'dot-success' : 'dot-danger') : 'dot-muted')"></span>
        </div>
        <div class="card-body">
          <p class="repair-desc">扫描磁盘错误，修复文件系统问题</p>
          <p class="repair-cmd mono">chkdsk C: /scan</p>
          <div class="repair-footer">
            <button class="btn btn-primary btn-sm" @click="runChkdsk" :disabled="running === 'chkdsk'">
              <span v-if="running === 'chkdsk'" class="spinner" style="width:12px;height:12px"></span>
              <Icon v-else name="play" :size="11" :stroke-width="2" />
              {{ running === 'chkdsk' ? '检查中' : '开始检查' }}
            </button>
            <span v-if="chkdskResult" class="repair-time mono">耗时 {{ chkdskResult.duration_secs }}s</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 修复结果输出 -->
    <div v-if="activeOutput" class="card output-card">
      <div class="card-header">
        <div class="title-group">
          <Icon name="terminal" :size="13" :stroke-width="1.75" />
          <span class="card-title">{{ activeOutput.title }}</span>
        </div>
        <span class="status-pill">
          <span class="dot" :class="activeOutput.success ? 'dot-success' : 'dot-danger'"></span>
          {{ activeOutput.success ? '成功' : '失败' }}
        </span>
      </div>
      <div class="card-body">
        <pre class="output-text mono">{{ activeOutput.output || activeOutput.error }}</pre>
      </div>
    </div>

    <!-- 磁盘健康状态 -->
    <div class="card" v-if="diskHealth.length > 0">
      <div class="card-header">
        <div class="title-group">
          <Icon name="activity" :size="13" :stroke-width="1.75" />
          <span class="card-title">磁盘健康状态</span>
        </div>
      </div>
      <div class="card-body">
        <div class="disk-health-list">
          <div v-for="disk in diskHealth" :key="disk.drive" class="disk-health-item">
            <div class="disk-header">
              <Icon name="hard-drive" :size="16" class="disk-icon" />
              <div class="disk-info-block">
                <p class="disk-model">{{ disk.model }}</p>
                <p class="disk-info mono">{{ disk.total_size_gb }} GB · {{ disk.drive }}</p>
              </div>
              <span class="status-pill">
                <span class="dot" :class="disk.smart_ok ? 'dot-success' : 'dot-danger'"></span>
                {{ disk.smart_ok ? 'S.M.A.R.T 正常' : 'S.M.A.R.T 异常' }}
              </span>
            </div>
            <div class="disk-details" v-if="disk.details.length > 0">
              <div v-for="(detail, i) in disk.details" :key="i" class="detail-item">
                <span class="detail-key">{{ detail[0] }}</span>
                <span class="detail-val mono">{{ detail[1] }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 提示 -->
    <div class="card tip-card">
      <Icon name="alert" :size="16" class="tip-icon tip-warn" />
      <div>
        <p class="tip-title">重要提示</p>
        <p>1. SFC 和 DISM 需要以管理员身份运行此程序才能正常工作</p>
        <p>2. 建议按顺序执行: 先 SFC → 如果无法修复 → DISM → 再 SFC</p>
        <p>3. DISM 修复可能需要 10-30 分钟，请耐心等待</p>
        <p>4. CHKDSK 如需深度修复(含 /F 参数)，可能需要重启电脑</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const running = ref("");
const sfcResult = ref(null);
const dismResult = ref(null);
const chkdskResult = ref(null);
const diskHealth = ref([]);

const activeOutput = computed(() => {
  if (sfcResult.value) return { title: "SFC 输出", ...sfcResult.value };
  if (dismResult.value) return { title: "DISM 输出", ...dismResult.value };
  if (chkdskResult.value) return { title: "CHKDSK 输出", ...chkdskResult.value };
  return null;
});

async function runSfc() {
  running.value = "sfc";
  sfcResult.value = null;
  try {
    sfcResult.value = await invoke("run_sfc");
  } catch (e) {
    sfcResult.value = { success: false, output: "", error: String(e), duration_secs: 0 };
  }
  running.value = "";
}

async function runDism() {
  running.value = "dism";
  dismResult.value = null;
  try {
    dismResult.value = await invoke("run_dism");
  } catch (e) {
    dismResult.value = { success: false, output: "", error: String(e), duration_secs: 0 };
  }
  running.value = "";
}

async function runChkdsk() {
  running.value = "chkdsk";
  chkdskResult.value = null;
  try {
    chkdskResult.value = await invoke("run_chkdsk", { drive: "C:" });
  } catch (e) {
    chkdskResult.value = { success: false, output: "", error: String(e), duration_secs: 0 };
  }
  running.value = "";
}

async function loadDiskHealth() {
  try {
    diskHealth.value = await invoke("check_disk_health");
  } catch (e) {
    console.error("Failed to check disk health:", e);
  }
}

onMounted(loadDiskHealth);
</script>

<style scoped>
.system-repair {
  max-width: 1000px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 22px;
  flex-wrap: wrap;
  gap: 12px;
}

.title-group {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-secondary);
}

/* 修复工具网格 */
.repair-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
  margin-bottom: 12px;
}

.repair-card {
  display: flex;
  flex-direction: column;
}

.repair-card .card-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex: 1;
}

.repair-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

.repair-cmd {
  font-size: 11.5px;
  color: var(--accent);
  background: var(--bg-input);
  border: 1px solid var(--border);
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  word-break: break-all;
}

.repair-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-top: auto;
}

.repair-time {
  font-size: 11px;
  color: var(--text-muted);
}

.dot-muted {
  background: var(--text-faint);
}

/* 状态药丸：圆点 + 文字，非色块 */
.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-secondary);
}

/* 输出 */
.output-card {
  margin-bottom: 12px;
}

.output-text {
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-size: 11.5px;
  max-height: 300px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
  line-height: 1.5;
}

/* 磁盘健康 */
.disk-health-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.disk-health-item {
  padding: 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.disk-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.disk-icon {
  color: var(--text-secondary);
  flex-shrink: 0;
}

.disk-info-block {
  flex: 1;
  min-width: 0;
}

.disk-model {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.disk-info {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 1px;
}

.disk-details {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 0 16px;
  border-top: 1px solid var(--border);
  padding-top: 8px;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  gap: 12px;
}

.detail-key {
  font-size: 11.5px;
  color: var(--text-muted);
}

.detail-val {
  font-size: 11.5px;
  color: var(--text-primary);
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
