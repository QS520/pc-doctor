<template>
  <div class="service-manager fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">系统服务管理</h1>
        <p class="page-subtitle">查看与管理 Windows 系统服务，优化开机启动与后台资源占用</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="loadServices" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          刷新
        </button>
      </div>
    </div>

    <!-- 过滤栏 -->
    <div class="card filter-bar">
      <div class="filter-group">
        <span class="filter-label">分类</span>
        <select v-model="categoryFilter" class="filter-select">
          <option value="全部">全部</option>
          <option value="系统关键">系统关键</option>
          <option value="可优化">可优化</option>
          <option value="第三方">第三方</option>
          <option value="其他">其他</option>
        </select>
      </div>
      <div class="filter-group search-group">
        <Icon name="search" :size="13" :stroke-width="1.75" class="search-icon" />
        <input
          v-model="searchKeyword"
          type="text"
          class="filter-input"
          placeholder="输入服务名称..."
        />
      </div>
      <span class="filter-count">共 <span class="mono">{{ filteredServices.length }}</span> 项</span>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在读取系统服务列表...</p>
    </div>

    <!-- 服务表格 -->
    <div v-else class="card service-card">
      <div class="service-table-wrapper">
        <table>
          <thead>
            <tr>
              <th>服务名称</th>
              <th>状态</th>
              <th>启动类型</th>
              <th>分类</th>
              <th>可禁用</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="svc in filteredServices" :key="svc.name">
              <td>
                <div class="svc-name-cell">
                  <span class="svc-display" :title="svc.description">{{ svc.display_name }}</span>
                  <span class="svc-key mono">{{ svc.name }}</span>
                </div>
              </td>
              <td>
                <span class="tag" :class="statusClass(svc.status)">{{ svc.status }}</span>
              </td>
              <td>
                <span class="start-type" :class="startTypeClass(svc.start_type)">{{ svc.start_type }}</span>
              </td>
              <td>
                <span class="tag" :class="categoryClass(svc.category)">{{ svc.category }}</span>
              </td>
              <td>
                <span v-if="svc.is_safe_to_disable" class="tag tag-success">安全</span>
                <span v-else class="tag tag-danger">不可</span>
              </td>
              <td>
                <div class="action-btns">
                  <button
                    v-if="svc.is_safe_to_disable && svc.start_type !== '已禁用'"
                    class="btn btn-danger btn-sm"
                    @click="disableService(svc)"
                    :disabled="busy === svc.name"
                  >
                    禁用
                  </button>
                  <button
                    v-if="svc.start_type === '已禁用'"
                    class="btn btn-primary btn-sm"
                    @click="enableService(svc)"
                    :disabled="busy === svc.name"
                  >
                    启用
                  </button>
                  <button
                    v-if="svc.status === '已停止' && svc.start_type !== '已禁用'"
                    class="btn btn-primary btn-sm"
                    @click="startService(svc)"
                    :disabled="busy === svc.name"
                  >
                    <Icon name="play" :size="11" :stroke-width="2" />
                    启动
                  </button>
                  <button
                    v-if="svc.status === '运行中'"
                    class="btn btn-ghost btn-sm"
                    @click="stopService(svc)"
                    :disabled="busy === svc.name"
                  >
                    <Icon name="stop" :size="11" :stroke-width="2" />
                    停止
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="filteredServices.length === 0" class="empty-state">
        <Icon name="search" :size="22" :stroke-width="1.5" />
        <p>没有匹配的服务</p>
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

    <!-- 警告提示 -->
    <div class="card tip-card tip-warning">
      <Icon name="alert" :size="16" :stroke-width="2" class="tip-icon icon-warning" />
      <div class="tip-content">
        <p><strong>警告:</strong> 请勿禁用「系统关键」分类的服务，否则可能导致系统无法启动、蓝屏或功能异常。</p>
        <p>只有标记为「安全」的服务才建议禁用。如不确定，请保持默认设置。</p>
        <p>所有服务操作均需要以管理员身份运行此程序。</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const loading = ref(true);
const services = ref([]);
const categoryFilter = ref("全部");
const searchKeyword = ref("");
const busy = ref("");
const feedback = ref(null);

const filteredServices = computed(() => {
  let list = services.value;
  if (categoryFilter.value !== "全部") {
    list = list.filter(s => s.category.startsWith(categoryFilter.value));
  }
  const kw = searchKeyword.value.trim().toLowerCase();
  if (kw) {
    list = list.filter(
      s =>
        s.display_name.toLowerCase().includes(kw) ||
        s.name.toLowerCase().includes(kw)
    );
  }
  return list;
});

async function loadServices() {
  loading.value = true;
  feedback.value = null;
  try {
    services.value = await invoke("get_services");
  } catch (e) {
    console.error("Failed to load services:", e);
    feedback.value = { success: false, message: "加载服务列表失败: " + e };
  }
  loading.value = false;
}

async function disableService(svc) {
  if (!confirm(`确定要禁用服务「${svc.display_name}」吗？\n该服务将不再随系统启动。`)) return;
  busy.value = svc.name;
  try {
    const res = await invoke("disable_service", { name: svc.name });
    feedback.value = res;
    if (res.success) {
      svc.start_type = "已禁用";
      svc.status = "已停止";
    }
  } catch (e) {
    feedback.value = { success: false, message: "禁用失败: " + e };
  }
  busy.value = "";
}

async function enableService(svc) {
  busy.value = svc.name;
  try {
    const res = await invoke("enable_service", { name: svc.name });
    feedback.value = res;
    if (res.success) {
      svc.start_type = "手动";
    }
  } catch (e) {
    feedback.value = { success: false, message: "启用失败: " + e };
  }
  busy.value = "";
}

async function startService(svc) {
  busy.value = svc.name;
  try {
    const res = await invoke("start_service", { name: svc.name });
    feedback.value = res;
    if (res.success) {
      svc.status = "运行中";
    }
  } catch (e) {
    feedback.value = { success: false, message: "启动失败: " + e };
  }
  busy.value = "";
}

async function stopService(svc) {
  if (!confirm(`确定要停止服务「${svc.display_name}」吗？\n依赖该服务的程序可能受影响。`)) return;
  busy.value = svc.name;
  try {
    const res = await invoke("stop_service", { name: svc.name });
    feedback.value = res;
    if (res.success) {
      svc.status = "已停止";
    }
  } catch (e) {
    feedback.value = { success: false, message: "停止失败: " + e };
  }
  busy.value = "";
}

function statusClass(status) {
  if (status === "运行中") return "tag-success";
  if (status === "已停止") return "tag-warning";
  if (status === "已暂停") return "tag-info";
  return "tag-info";
}

function startTypeClass(type) {
  if (type === "已禁用") return "start-disabled";
  if (type === "自动") return "start-auto";
  return "start-other";
}

function categoryClass(category) {
  if (category.startsWith("系统关键")) return "tag-danger";
  if (category.startsWith("可优化")) return "tag-warning";
  if (category.startsWith("第三方")) return "tag-info";
  return "tag-info";
}

onMounted(loadServices);
</script>

<style scoped>
.service-manager {
  max-width: 1000px;
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

/* 过滤栏 */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 12px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-label {
  font-size: 10.5px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  white-space: nowrap;
}

.filter-select,
.filter-input {
  padding: 6px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  height: 28px;
}

.filter-select:focus,
.filter-input:focus {
  outline: none;
  border-color: var(--accent);
}

.search-group {
  position: relative;
  flex: 1;
  min-width: 220px;
}

.search-icon {
  position: absolute;
  left: 9px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  pointer-events: none;
}

.search-group .filter-input {
  width: 100%;
  padding-left: 28px;
}

.filter-count {
  font-size: 11.5px;
  color: var(--text-muted);
  margin-left: auto;
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

/* 服务表格 */
.service-card {
  overflow: hidden;
}

.service-table-wrapper {
  max-height: 560px;
  overflow-y: auto;
}

.svc-name-cell {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.svc-display {
  font-weight: 500;
  font-size: 12px;
  color: var(--text-primary);
}

.svc-key {
  font-size: 10.5px;
  color: var(--text-muted);
}

.start-type {
  font-size: 11.5px;
  font-weight: 500;
  font-family: var(--font-mono);
}

.start-auto { color: var(--accent); }
.start-disabled { color: var(--danger); }
.start-other { color: var(--text-secondary); }

.action-btns {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
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

.icon-success { color: var(--success); }
.icon-danger { color: var(--danger); }
.icon-warning { color: var(--warning); }

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
.tip-card.tip-warning { border-color: var(--warning); }
</style>
