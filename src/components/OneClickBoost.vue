<template>
  <div class="one-click-boost fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">一键加速</h1>
        <p class="page-subtitle">一键释放内存与磁盘空间，结束高占用后台进程</p>
      </div>
    </div>

    <!-- 加速入口 -->
    <div class="card boost-entry">
      <div class="boost-entry-inner">
        <div class="boost-hero-icon">
          <Icon name="zap" :size="40" :stroke-width="1.5" />
        </div>
        <h2 class="boost-entry-title">一键释放内存与磁盘空间</h2>
        <p class="boost-entry-desc">结束高占用后台进程 · 清理临时文件 · 清理浏览器缓存 · 刷新 DNS 缓存</p>
        <button
          class="boost-btn"
          :class="{ running: boosting }"
          @click="runBoost"
          :disabled="boosting"
        >
          <span v-if="boosting" class="spinner" style="width:18px;height:18px;border-width:3px"></span>
          <Icon v-else name="rocket" :size="18" :stroke-width="2" />
          <span>{{ boosting ? '正在加速' : '一键加速' }}</span>
        </button>
        <p v-if="boosting" class="boost-hint">请稍候，正在清理系统资源...</p>
      </div>
    </div>

    <!-- 汇总卡片 -->
    <div v-if="result && result.success" class="card summary-card">
      <div class="summary-stat">
        <Icon name="memory-stick" :size="18" class="summary-icon" />
        <div>
          <p class="stat-label">释放内存</p>
          <p class="stat-value mono">{{ result.freed_memory_gb.toFixed(2) }} <small>GB</small></p>
        </div>
      </div>
      <div class="summary-divider"></div>
      <div class="summary-stat">
        <Icon name="trash" :size="18" class="summary-icon" />
        <div>
          <p class="stat-label">清理临时文件</p>
          <p class="stat-value mono">{{ result.cleaned_temp_mb.toFixed(1) }} <small>MB</small></p>
        </div>
      </div>
      <div class="summary-divider"></div>
      <div class="summary-stat">
        <Icon name="globe" :size="18" class="summary-icon" />
        <div>
          <p class="stat-label">清理浏览器缓存</p>
          <p class="stat-value mono">{{ result.cleaned_cache_mb.toFixed(1) }} <small>MB</small></p>
        </div>
      </div>
      <div class="summary-divider"></div>
      <div class="summary-stat">
        <Icon name="stop" :size="18" class="summary-icon" />
        <div>
          <p class="stat-label">结束进程</p>
          <p class="stat-value mono">{{ result.killed_processes }} <small>个</small></p>
        </div>
      </div>
    </div>

    <!-- 详细结果 -->
    <div v-if="result" class="card">
      <div class="card-header">
        <div class="title-group">
          <Icon name="list" :size="13" :stroke-width="1.75" />
          <span class="card-title">加速详情</span>
        </div>
        <span class="status-pill">
          <span class="dot" :class="result.success ? 'dot-success' : 'dot-danger'"></span>
          {{ result.success ? '成功' : '部分失败' }}
        </span>
      </div>
      <div class="card-body">
        <ul class="details-list">
          <li v-for="(line, i) in result.details" :key="i" class="detail-line">
            <Icon name="chevron-right" :size="11" :stroke-width="2" class="detail-bullet" />
            <span class="mono">{{ line }}</span>
          </li>
        </ul>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="card tip-card error-tip">
      <Icon name="alert" :size="16" class="tip-icon tip-danger" />
      <div>
        <p class="error-title">加速失败</p>
        <p class="error-text mono">{{ error }}</p>
      </div>
    </div>

    <!-- 提示 -->
    <div class="card tip-card">
      <Icon name="info" :size="16" class="tip-icon tip-info" />
      <div>
        <p>一键加速会自动结束 CPU 占用超过 5% 或内存超过 500MB 的非系统关键进程。</p>
        <p>系统关键进程（如 explorer、svchost、dwm 等）不会被结束，请放心使用。</p>
        <p>未保存的工作请先保存，避免数据丢失。</p>
      </div>
    </div>

    <!-- 二次确认弹窗 -->
    <div v-if="showConfirm" class="modal-mask" @click.self="cancelConfirm">
      <div class="modal">
        <div class="modal-head">
          <div class="modal-icon-wrap">
            <Icon name="alert" :size="18" :stroke-width="2" />
          </div>
          <h3 class="modal-title">确认执行一键加速？</h3>
        </div>
        <div class="modal-body">
          <p class="modal-text">此操作将执行以下动作：</p>
          <ul class="modal-list">
            <li><span class="dot dot-warning"></span>结束 CPU > 5% 或内存 > 500MB 的非关键进程</li>
            <li><span class="dot dot-info"></span>清理临时文件和浏览器缓存</li>
            <li><span class="dot dot-info"></span>刷新 DNS 缓存</li>
          </ul>
          <p class="modal-warn"><Icon name="alert" :size="12" :stroke-width="2" /> 请先保存正在编辑的文档，避免数据丢失。系统关键进程已自动保护。</p>
        </div>
        <div class="modal-actions">
          <button class="btn btn-ghost btn-sm" @click="cancelConfirm">取消</button>
          <button class="btn btn-primary btn-sm" @click="confirmBoost">
            <Icon name="zap" :size="13" :stroke-width="2" />
            确认加速
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const boosting = ref(false);
const result = ref(null);
const error = ref("");
const showConfirm = ref(false);

function runBoost() {
  // 先弹二次确认
  showConfirm.value = true;
}

function cancelConfirm() {
  showConfirm.value = false;
}

async function confirmBoost() {
  showConfirm.value = false;
  boosting.value = true;
  result.value = null;
  error.value = "";
  try {
    const res = await invoke("one_click_boost");
    result.value = res;
    if (!res.success) {
      error.value = "加速未能完全完成，请查看详情。";
    }
  } catch (e) {
    error.value = String(e);
    console.error("One click boost failed:", e);
  }
  boosting.value = false;
}

onMounted(() => {
  // 进入页面时不自动执行，等待用户点击
});
</script>

<style scoped>
.one-click-boost {
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

.title-group {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-secondary);
}

/* 加速入口 */
.boost-entry {
  margin-bottom: 12px;
  padding: 44px 24px;
  text-align: center;
}

.boost-entry-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.boost-hero-icon {
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--accent-dim);
  color: var(--accent);
  border: 1px solid var(--accent-glow);
  margin-bottom: 4px;
}

.boost-entry-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.boost-entry-desc {
  font-size: 12px;
  color: var(--text-secondary);
  max-width: 560px;
  line-height: 1.6;
}

.boost-btn {
  margin-top: 10px;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 13px 38px;
  font-size: 15px;
  font-weight: 600;
  color: #04201d;
  background: var(--accent);
  border-radius: var(--radius);
  box-shadow: 0 4px 20px var(--accent-glow);
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.boost-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  transform: translateY(-1px);
  box-shadow: 0 6px 24px var(--accent-glow);
}

.boost-btn:disabled {
  cursor: not-allowed;
  opacity: 0.85;
}

.boost-btn.running {
  background: var(--bg-elevated);
  color: var(--text-secondary);
  box-shadow: none;
  border: 1px solid var(--border);
}

.boost-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 2px;
}

/* 汇总卡片 */
.summary-card {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
  padding: 20px 16px;
}

.summary-stat {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 10px;
  justify-content: center;
}

.summary-icon {
  color: var(--accent);
  flex-shrink: 0;
}

.stat-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  font-weight: 600;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin-top: 2px;
  letter-spacing: -0.01em;
}

.stat-value small {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  margin-left: 2px;
}

.summary-divider {
  width: 1px;
  height: 40px;
  background: var(--border);
  flex-shrink: 0;
}

/* 详情列表 */
.details-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.detail-line {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  font-size: 11.5px;
  color: var(--text-secondary);
  word-break: break-all;
  line-height: 1.5;
}

.detail-bullet {
  color: var(--accent);
  flex-shrink: 0;
  margin-top: 3px;
}

/* 状态药丸：圆点 + 文字 */
.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-secondary);
}

/* 提示卡 */
.tip-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  background: var(--bg-elevated);
  margin-top: 12px;
}

.tip-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.tip-info {
  color: var(--info);
}

.tip-danger {
  color: var(--danger);
}

.tip-card p {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 3px;
  line-height: 1.6;
}

.error-tip {
  border-color: var(--danger);
  background: var(--danger-dim);
}

.error-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--danger);
  margin-bottom: 5px;
}

.error-text {
  color: var(--danger);
  font-size: 11.5px;
  word-break: break-all;
}

@media (max-width: 900px) {
  .summary-card {
    flex-direction: column;
    gap: 16px;
  }
  .summary-divider {
    width: 80%;
    height: 1px;
  }
  .summary-stat {
    padding: 0;
  }
}

/* 二次确认弹窗 */
.modal-mask {
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

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal {
  background: var(--bg-surface);
  border: 1px solid var(--border-light);
  border-radius: var(--radius);
  width: 420px;
  max-width: calc(100vw - 40px);
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

.modal-head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
}

.modal-icon-wrap {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--warning-dim);
  color: var(--warning);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.modal-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-body {
  padding: 14px 16px;
}

.modal-text {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 10px;
}

.modal-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.modal-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-primary);
}

.modal-warn {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 9px 11px;
  background: var(--warning-dim);
  border-radius: var(--radius-sm);
  font-size: 11px;
  color: var(--warning);
  line-height: 1.5;
}

.modal-warn .icon {
  margin-top: 2px;
  flex-shrink: 0;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 11px 16px;
  border-top: 1px solid var(--border);
  background: var(--bg-elevated);
}
</style>
