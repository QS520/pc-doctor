<template>
  <div class="startup-manager fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">开机加速</h1>
        <p class="page-subtitle">管理开机启动项，提升系统启动速度</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="refresh" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          刷新
        </button>
      </div>
    </div>

    <!-- 开机信息 -->
    <div class="boot-info" v-if="hasLoaded && bootDuration">
      <div class="boot-stat">
        <span class="boot-icon">
          <Icon name="clock" :size="15" :stroke-width="1.75" />
        </span>
        <div>
          <p class="stat-label">上次开机时间</p>
          <p class="stat-value mono">{{ bootDuration.last_boot_time }}</p>
        </div>
      </div>
      <div class="boot-divider"></div>
      <div class="boot-stat">
        <span class="boot-icon" :class="{ 'icon-slow': bootDuration.boot_duration_seconds > 60 }">
          <Icon name="zap" :size="15" :stroke-width="1.75" />
        </span>
        <div>
          <p class="stat-label">开机耗时</p>
          <p class="stat-value mono" :class="{ slow: bootDuration.boot_duration_seconds > 60 }">
            {{ bootDuration.boot_duration_display }}
          </p>
        </div>
      </div>
      <div class="boot-divider"></div>
      <div class="boot-stat">
        <span class="boot-icon">
          <Icon name="list" :size="15" :stroke-width="1.75" />
        </span>
        <div>
          <p class="stat-label">启动项数量</p>
          <p class="stat-value mono">{{ startupItems.length }}</p>
        </div>
      </div>
    </div>

    <div v-if="hasLoaded && bootDuration && bootDuration.boot_duration_seconds > 60" class="tip-card tip-warning">
      <span class="tip-icon"><Icon name="info" :size="14" :stroke-width="2" /></span>
      <p>开机耗时较长，建议禁用不必要的启动项来加速开机。</p>
    </div>

    <!-- 启动项列表 -->
    <div class="card" v-if="hasLoaded && !loading">
      <div class="card-header">
        <span class="card-title">启动项管理</span>
        <span class="tip-text">点击开关启用 / 禁用启动项</span>
      </div>
      <div class="card-body">
        <div class="startup-list">
          <div v-for="item in startupItems" :key="item.name + item.source" class="startup-row">
            <div class="startup-info">
              <div class="startup-name-row">
                <span class="startup-name">{{ item.name }}</span>
                <span class="status-inline">
                  <span class="dot" :class="item.enabled ? 'dot-success' : 'dot-warning'"></span>
                  <span class="status-text" :class="item.enabled ? 'text-success' : 'text-warning'">
                    {{ item.enabled ? '已启用' : '已禁用' }}
                  </span>
                </span>
                <span class="tag tag-neutral">{{ item.source }}</span>
              </div>
              <p class="startup-command mono" :title="item.command">{{ item.command }}</p>
            </div>
            <label class="toggle">
              <input
                type="checkbox"
                :checked="item.enabled"
                @change="toggleStartup(item)"
              />
              <span class="toggle-slider"></span>
            </label>
          </div>
        </div>

        <div v-if="startupItems.length === 0" class="empty-state">
          <Icon name="check" :size="20" :stroke-width="1.75" />
          <p>没有发现启动项</p>
        </div>
      </div>
    </div>

    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在读取启动项...</p>
    </div>

    <!-- 初始加载提示 -->
    <div class="scan-prompt" v-if="!hasLoaded && !loading">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始加载</p>
      <button class="btn btn-primary" @click="refresh">加载开机项</button>
    </div>

    <!-- 提示 -->
    <div class="tip-card tip-info" v-if="hasLoaded && !loading">
      <span class="tip-icon"><Icon name="alert" :size="14" :stroke-width="2" /></span>
      <div>
        <p>禁用启动项不会卸载程序，只是阻止它在开机时自动启动。</p>
        <p>系统关键服务不会显示在此列表中。</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const loading = ref(false);
const hasLoaded = ref(false);
const startupItems = ref([]);
const bootDuration = ref(null);

async function refresh() {
  loading.value = true;
  try {
    const [items, boot] = await Promise.all([
      invoke("get_startup_items"),
      invoke("get_boot_duration"),
    ]);
    startupItems.value = items;
    bootDuration.value = boot;
  } catch (e) {
    console.error("Failed to load startup items:", e);
  }
  loading.value = false;
  hasLoaded.value = true;
}

async function toggleStartup(item) {
  try {
    if (item.enabled) {
      await invoke("disable_startup_item", { name: item.name, source: item.source });
      item.enabled = false;
    } else {
      await invoke("enable_startup_item", { name: item.name, source: item.source });
      item.enabled = true;
    }
  } catch (e) {
    alert("操作失败: " + e);
  }
}
</script>

<style scoped>
.startup-manager {
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
  align-items: center;
  gap: 12px;
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

/* 开机信息 */
.boot-info {
  display: flex;
  align-items: stretch;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  margin-bottom: 12px;
}

.boot-stat {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px;
}

.boot-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  background: var(--accent-dim);
  color: var(--accent);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.boot-icon.icon-slow {
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
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-top: 2px;
}

.stat-value.slow {
  color: var(--danger);
}

.boot-divider {
  width: 1px;
  align-self: stretch;
  margin: 10px 0;
  background: var(--border);
}

/* 提示卡 */
.tip-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-bottom: 12px;
  padding: 12px 14px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.tip-warning {
  border-left: 3px solid var(--warning);
}

.tip-info {
  border-left: 3px solid var(--info);
}

.tip-icon {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 1px;
}

.tip-warning .tip-icon {
  color: var(--warning);
}

.tip-info .tip-icon {
  color: var(--info);
}

.tip-card p {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 2px;
}

.tip-card p:last-child {
  margin-bottom: 0;
}

/* 列表 */
.tip-text {
  font-size: 11px;
  color: var(--text-muted);
}

.startup-list {
  display: flex;
  flex-direction: column;
}

.startup-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 11px 0;
  border-bottom: 1px solid var(--border);
  gap: 16px;
}

.startup-row:last-child {
  border-bottom: none;
}

.startup-info {
  flex: 1;
  min-width: 0;
}

.startup-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 3px;
  flex-wrap: wrap;
}

.startup-name {
  font-weight: 600;
  font-size: 12.5px;
  color: var(--text-primary);
}

.status-inline {
  display: inline-flex;
  align-items: center;
  gap: 5px;
}

.status-text {
  font-size: 11px;
  font-weight: 500;
}

.startup-command {
  font-size: 11px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 开关 */
.toggle {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
  flex-shrink: 0;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--bg-input);
  border: 1px solid var(--border-light);
  border-radius: 20px;
  transition: 0.2s;
}

.toggle-slider::before {
  position: absolute;
  content: "";
  height: 14px;
  width: 14px;
  left: 2px;
  bottom: 2px;
  background: var(--text-muted);
  border-radius: 50%;
  transition: 0.2s;
}

.toggle input:checked + .toggle-slider {
  background: var(--accent);
  border-color: var(--accent);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(16px);
  background: #04201d;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-muted);
  font-size: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

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
</style>
