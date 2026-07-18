<template>
  <div class="power-plan fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">电源管理</h1>
        <p class="page-subtitle">切换电源方案、调节 CPU 性能状态与显示睡眠超时</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="loadAll" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          刷新
        </button>
      </div>
    </div>

    <!-- 初始扫描提示 -->
    <div class="scan-prompt" v-if="!hasLoaded && !loading">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始扫描</p>
      <button class="btn btn-primary" @click="loadAll">加载电源计划</button>
    </div>

    <!-- 1. 电源计划 -->
    <div class="card section-card" v-if="hasLoaded || loading">
      <div class="card-header">
        <div class="section-title">
          <Icon name="power" :size="13" :stroke-width="1.75" />
          <span>电源计划</span>
        </div>
      </div>
      <div class="card-body">
        <div v-if="loading" class="loading small">
          <div class="spinner" style="width:18px;height:18px"></div>
          <p>正在读取电源计划...</p>
        </div>
        <div v-else-if="plans.length > 0" class="plan-list">
          <label
            v-for="plan in plans"
            :key="plan.guid"
            class="plan-item"
            :class="{ active: plan.is_active, busy: busyGuid === plan.guid }"
          >
            <input
              type="radio"
              name="power-plan"
              :checked="plan.is_active"
              :disabled="plan.is_active || busyGuid === plan.guid"
              @change="setPlan(plan)"
            />
            <span class="plan-radio"></span>
            <div class="plan-info">
              <div class="plan-name-row">
                <span class="plan-name">{{ plan.name }}</span>
                <span v-if="plan.is_active" class="tag tag-success">当前方案</span>
              </div>
              <p class="plan-guid mono">{{ plan.guid }}</p>
              <p v-if="plan.description" class="plan-desc">{{ plan.description }}</p>
            </div>
          </label>
        </div>
        <div v-else class="empty-state">
          <Icon name="battery" :size="22" :stroke-width="1.5" />
          <p>未检测到电源计划</p>
        </div>
      </div>
    </div>

    <!-- 2. CPU 频率状态 -->
    <div v-if="throttle" class="card section-card">
      <div class="card-header">
        <div class="section-title">
          <Icon name="thermometer" :size="13" :stroke-width="1.75" />
          <span>CPU 频率状态</span>
        </div>
      </div>
      <div class="card-body">
        <div class="throttle-summary">
          <div class="throttle-stat">
            <span class="stat-label">当前电源方案</span>
            <span class="stat-value">{{ throttle.current_power_plan }}</span>
          </div>
          <div class="throttle-stat">
            <span class="stat-label">CPU 最小状态</span>
            <span class="stat-value mono">{{ throttle.cpu_min_state_percent }}%</span>
          </div>
          <div class="throttle-stat">
            <span class="stat-label">CPU 最大状态</span>
            <span class="stat-value mono" :class="throttle.is_throttled ? 'text-warning' : 'text-success'">
              {{ throttle.cpu_max_state_percent }}%
            </span>
          </div>
        </div>

        <!-- CPU 状态进度条 -->
        <div class="cpu-bar-wrap">
          <div class="cpu-bar-label">
            <span>最小 <span class="mono">{{ throttle.cpu_min_state_percent }}%</span></span>
            <span>最大 <span class="mono">{{ throttle.cpu_max_state_percent }}%</span></span>
          </div>
          <div class="cpu-bar">
            <div
              class="cpu-fill"
              :class="throttle.is_throttled ? 'throttled' : 'normal'"
              :style="{ width: throttle.cpu_max_state_percent + '%' }"
            ></div>
            <div
              class="cpu-marker"
              :style="{ left: throttle.cpu_min_state_percent + '%' }"
              title="最小状态"
            ></div>
          </div>
        </div>

        <!-- 降频警告 -->
        <div v-if="throttle.is_throttled" class="warn-card">
          <Icon name="alert" :size="16" :stroke-width="2" class="warn-icon icon-warning" />
          <div>
            <p><strong>CPU 当前处于降频状态 ({{ throttle.cpu_max_state_percent }}%)</strong></p>
            <p>CPU 最大频率被限制在 {{ throttle.cpu_max_state_percent }}%，可能影响高性能场景表现。</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 3. 设置 CPU 最大状态 -->
    <div v-if="throttle" class="card section-card">
      <div class="card-header">
        <div class="section-title">
          <Icon name="cpu" :size="13" :stroke-width="1.75" />
          <span>CPU 性能调节</span>
        </div>
      </div>
      <div class="card-body">
        <p class="slider-desc">将 CPU 最大状态设为 100% 可解除降频限制，恢复满血性能。</p>
        <div class="slider-row">
          <span class="slider-min mono">5%</span>
          <input
            v-model.number="cpuMaxState"
            type="range"
            min="5"
            max="100"
            step="5"
            class="cpu-slider"
          />
          <span class="slider-max mono">100%</span>
          <span class="slider-current mono">{{ cpuMaxState }}%</span>
        </div>
        <div class="slider-actions">
          <button
            class="btn btn-primary btn-sm"
            @click="setCpuMaxState(cpuMaxState)"
            :disabled="settingCpu"
          >
            <span v-if="settingCpu" class="spinner" style="width:12px;height:12px"></span>
            <Icon v-else name="check" :size="13" :stroke-width="2" />
            应用 ({{ cpuMaxState }}%)
          </button>
          <button
            class="btn btn-ghost btn-sm"
            @click="setCpuMaxState(100)"
            :disabled="settingCpu || throttle.cpu_max_state_percent === 100"
          >
            <Icon name="zap" :size="13" :stroke-width="2" />
            解除降频 (100%)
          </button>
        </div>
      </div>
    </div>

    <!-- 显示与睡眠超时 -->
    <div v-if="throttle" class="card section-card">
      <div class="card-header">
        <div class="section-title">
          <Icon name="clock" :size="13" :stroke-width="1.75" />
          <span>显示与睡眠超时</span>
        </div>
      </div>
      <div class="card-body">
        <div class="timeout-grid">
          <div class="timeout-item">
            <Icon name="monitor" :size="18" :stroke-width="1.75" class="timeout-icon icon-neutral" />
            <div>
              <p class="stat-label">关闭显示器</p>
              <p class="timeout-value">{{ formatTimeout(throttle.display_timeout_minutes) }}</p>
            </div>
          </div>
          <div class="timeout-item">
            <Icon name="power" :size="18" :stroke-width="1.75" class="timeout-icon icon-neutral" />
            <div>
              <p class="stat-label">进入睡眠</p>
              <p class="timeout-value">{{ formatTimeout(throttle.sleep_timeout_minutes) }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 操作反馈 -->
    <div v-if="feedback" class="card tip-card" :class="feedback.success ? 'tip-success' : 'tip-error'">
      <Icon
        :name="feedback.success ? 'check' : 'alert'"
        :size="16"
        :stroke-width="2"
        class="tip-icon"
        :class="feedback.success ? 'icon-success' : 'icon-danger'"
      />
      <p>{{ feedback.message }}</p>
    </div>

    <!-- 提示 -->
    <div class="card tip-card tip-info">
      <Icon name="info" :size="16" :stroke-width="2" class="tip-icon icon-info" />
      <div class="tip-content">
        <p>切换电源计划与调节 CPU 状态需要以管理员身份运行此程序。</p>
        <p>「高性能」方案与解除 CPU 降频会增加功耗与发热，笔记本用户请酌情使用。</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const loading = ref(true);
const hasLoaded = ref(false);
const plans = ref([]);
const throttle = ref(null);
const busyGuid = ref("");
const settingCpu = ref(false);
const cpuMaxState = ref(100);
const feedback = ref(null);

async function loadAll() {
  loading.value = true;
  feedback.value = null;
  try {
    const [planList, throttleInfo] = await Promise.all([
      invoke("get_power_plans"),
      invoke("get_cpu_throttle_info"),
    ]);
    plans.value = planList || [];
    throttle.value = throttleInfo;
    if (throttleInfo) {
      cpuMaxState.value = throttleInfo.cpu_max_state_percent;
    }
  } catch (e) {
    console.error("Failed to load power info:", e);
    feedback.value = { success: false, message: "加载电源信息失败: " + e };
  }
  loading.value = false;
  hasLoaded.value = true;
}

async function setPlan(plan) {
  if (plan.is_active) return;
  busyGuid.value = plan.guid;
  try {
    const res = await invoke("set_power_plan", { guid: plan.guid });
    feedback.value = res;
    if (res.success) {
      plans.value.forEach(p => {
        p.is_active = p.guid === plan.guid;
        p.description = p.is_active ? "当前活动方案" : "";
      });
      // 切换方案后刷新 CPU 状态
      await refreshThrottle();
    }
  } catch (e) {
    feedback.value = { success: false, message: "切换电源计划失败: " + e };
  }
  busyGuid.value = "";
}

async function setCpuMaxState(percent) {
  settingCpu.value = true;
  try {
    const res = await invoke("set_cpu_max_state", { percent: Number(percent) });
    feedback.value = res;
    if (res.success) {
      await refreshThrottle();
    }
  } catch (e) {
    feedback.value = { success: false, message: "设置 CPU 状态失败: " + e };
  }
  settingCpu.value = false;
}

async function refreshThrottle() {
  try {
    throttle.value = await invoke("get_cpu_throttle_info");
    if (throttle.value) {
      cpuMaxState.value = throttle.value.cpu_max_state_percent;
    }
  } catch (e) {
    console.error("Failed to refresh throttle info:", e);
  }
}

function formatTimeout(minutes) {
  if (!minutes || minutes === 0) return "从不";
  if (minutes < 60) return `${minutes} 分钟`;
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m === 0 ? `${h} 小时` : `${h} 小时 ${m} 分钟`;
}
</script>

<style scoped>
.power-plan {
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

.section-card {
  margin-bottom: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.section-title svg {
  color: var(--text-secondary);
}

/* 电源计划列表 */
.plan-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.plan-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
}

.plan-item:hover {
  border-color: var(--border-light);
}

.plan-item.active {
  border-color: var(--accent);
  background: var(--accent-dim);
}

.plan-item.busy {
  opacity: 0.7;
  cursor: progress;
}

.plan-item input[type="radio"] {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.plan-radio {
  position: relative;
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-light);
  border-radius: 50%;
  flex-shrink: 0;
  margin-top: 2px;
  transition: 0.15s;
}

.plan-item:hover .plan-radio {
  border-color: var(--accent);
}

.plan-item.active .plan-radio {
  border-color: var(--accent);
}

.plan-item.active .plan-radio::after {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
}

.plan-info {
  flex: 1;
  min-width: 0;
}

.plan-name-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 3px;
}

.plan-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.plan-guid {
  font-size: 10.5px;
  color: var(--text-muted);
  word-break: break-all;
}

.plan-desc {
  font-size: 11.5px;
  color: var(--accent);
  margin-top: 2px;
}

/* CPU 频率状态 */
.throttle-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 14px 32px;
  padding: 12px 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  margin-bottom: 14px;
}

.throttle-stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.throttle-stat .stat-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.throttle-stat .stat-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.cpu-bar-wrap {
  margin-bottom: 14px;
}

.cpu-bar-label {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.cpu-bar {
  position: relative;
  height: 10px;
  background: var(--bg-input);
  border-radius: 5px;
  overflow: visible;
}

.cpu-fill {
  height: 100%;
  border-radius: 5px;
  transition: width 0.4s ease;
}

.cpu-fill.normal {
  background: var(--success);
}

.cpu-fill.throttled {
  background: var(--warning);
}

.cpu-marker {
  position: absolute;
  top: -3px;
  width: 3px;
  height: 16px;
  background: var(--accent);
  border-radius: 2px;
  transform: translateX(-50%);
}

/* 降频警告 */
.warn-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  background: var(--warning-dim);
  border: 1px solid var(--warning);
  border-radius: var(--radius-sm);
}

.warn-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.icon-neutral { color: var(--text-secondary); }
.icon-success { color: var(--success); }
.icon-warning { color: var(--warning); }
.icon-danger { color: var(--danger); }
.icon-info { color: var(--info); }

.warn-card p {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  line-height: 1.6;
}

.warn-card p:last-child {
  margin-bottom: 0;
}

/* CPU 性能调节 */
.slider-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 14px;
  line-height: 1.6;
}

.slider-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 14px;
}

.slider-min,
.slider-max {
  font-size: 11px;
  color: var(--text-muted);
}

.slider-current {
  font-size: 14px;
  font-weight: 700;
  color: var(--accent);
  min-width: 44px;
  text-align: right;
}

.cpu-slider {
  flex: 1;
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  background: var(--bg-input);
  border-radius: 2px;
  outline: none;
}

.cpu-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  border: 2px solid var(--bg-surface);
  box-shadow: 0 0 0 1px var(--accent);
}

.cpu-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--accent);
  cursor: pointer;
  border: 2px solid var(--bg-surface);
  box-shadow: 0 0 0 1px var(--accent);
}

.slider-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

/* 显示与睡眠超时 */
.timeout-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
}

.timeout-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.timeout-icon {
  flex-shrink: 0;
}

.timeout-item .stat-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.timeout-value {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-top: 2px;
}

/* 加载 */
.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 12px;
}

.loading.small {
  padding: 36px 20px;
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

.tip-card.tip-success { border-color: var(--success); }
.tip-card.tip-error { border-color: var(--danger); }
.tip-card.tip-info { border-color: var(--border-light); }
</style>
