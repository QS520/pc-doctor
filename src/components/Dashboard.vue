<template>
  <div class="dashboard fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">系统总览</h1>
        <p class="page-subtitle">实时监测系统运行状态与硬件信息</p>
      </div>
      <button class="btn btn-ghost btn-sm" @click="refresh" :disabled="loading">
        <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
        <Icon v-else name="refresh" :size="13" :stroke-width="2" />
        刷新
      </button>
    </div>

    <div v-if="loading" class="loading">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在采集系统数据...</p>
    </div>

    <template v-else>
      <!-- 实时负载 -->
      <div class="metric-grid">
        <div class="metric">
          <div class="metric-head">
            <Icon name="cpu" :size="13" :stroke-width="1.75" />
            <span class="metric-label">CPU 负载</span>
            <span class="metric-value">{{ systemInfo.cpu.usage.toFixed(1) }}<span class="unit">%</span></span>
          </div>
          <div class="bar"><div class="bar-fill" :class="getBarClass(systemInfo.cpu.usage)" :style="{ width: systemInfo.cpu.usage + '%' }"></div></div>
          <div class="metric-meta">
            <span>{{ systemInfo.cpu.brand }}</span>
          </div>
          <div class="metric-sub">{{ systemInfo.cpu.core_count }} 核 · {{ (systemInfo.cpu.frequency / 1000).toFixed(2) }} GHz</div>
        </div>

        <div class="metric">
          <div class="metric-head">
            <Icon name="memory-stick" :size="13" :stroke-width="1.75" />
            <span class="metric-label">内存占用</span>
            <span class="metric-value">{{ systemInfo.memory.usage_percent.toFixed(1) }}<span class="unit">%</span></span>
          </div>
          <div class="bar"><div class="bar-fill" :class="getBarClass(systemInfo.memory.usage_percent)" :style="{ width: systemInfo.memory.usage_percent + '%' }"></div></div>
          <div class="metric-meta">
            <span class="mono">{{ systemInfo.memory.used_gb }} / {{ systemInfo.memory.total_gb }} GB</span>
          </div>
          <div class="metric-sub">可用 {{ systemInfo.memory.free_gb }} GB</div>
        </div>

        <div class="metric" v-for="disk in systemInfo.disks" :key="disk.drive">
          <div class="metric-head">
            <Icon name="disc" :size="13" :stroke-width="1.75" />
            <span class="metric-label">{{ disk.drive }}: 盘</span>
            <span class="metric-value">{{ disk.usage_percent.toFixed(1) }}<span class="unit">%</span></span>
          </div>
          <div class="bar"><div class="bar-fill" :class="getBarClass(disk.usage_percent)" :style="{ width: disk.usage_percent + '%' }"></div></div>
          <div class="metric-meta">
            <span class="mono">{{ disk.used_gb }} / {{ disk.total_gb }} GB</span>
          </div>
          <div class="metric-sub">{{ disk.drive_type }} · 剩余 {{ disk.free_gb }} GB</div>
        </div>
      </div>

      <!-- 两列布局：系统信息 + 硬件信息 -->
      <div class="two-col">
        <!-- 左列：系统信息 + 快捷操作 -->
        <div class="col">
          <div class="card">
            <div class="card-header">
              <span class="card-title">系统信息</span>
              <span class="dot dot-success"></span>
            </div>
            <div class="card-body">
              <div class="kv-row">
                <span class="kv-label">操作系统</span>
                <span class="kv-value">{{ systemInfo.os_build }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">主机名</span>
                <span class="kv-value mono">{{ systemInfo.hostname }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">开机时间</span>
                <span class="kv-value mono">{{ systemInfo.boot_time }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">已运行</span>
                <span class="kv-value">{{ formatUptime(systemInfo.uptime_hours) }}</span>
              </div>
            </div>
          </div>

          <div class="card">
            <div class="card-header">
              <span class="card-title">快捷操作</span>
            </div>
            <div class="card-body action-grid">
              <button class="action-btn" @click="$emit('navigate', 'cleanup')">
                <Icon name="broom" :size="15" />
                <span>清理垃圾</span>
              </button>
              <button class="action-btn" @click="$emit('navigate', 'boost')">
                <Icon name="zap" :size="15" />
                <span>一键加速</span>
              </button>
              <button class="action-btn" @click="$emit('navigate', 'startup')">
                <Icon name="rocket" :size="15" />
                <span>开机加速</span>
              </button>
              <button class="action-btn" @click="$emit('navigate', 'repair')">
                <Icon name="wrench" :size="15" />
                <span>系统修复</span>
              </button>
            </div>
          </div>
        </div>

        <!-- 右列：硬件信息（内存/CPU/主板/显卡） -->
        <div class="col">
          <div class="card">
            <div class="card-header" @click="showHardware = !showHardware" style="cursor:pointer;user-select:none">
              <span class="card-title">硬件信息</span>
              <Icon :name="showHardware ? 'chevron-down' : 'chevron-right'" :size="14" :stroke-width="2" />
            </div>

            <div class="card-body" v-if="showHardware && hardwareInfo">
              <!-- 内存条（用户重点需求） -->
              <div class="hw-block" v-if="hardwareInfo.memory_sticks && hardwareInfo.memory_sticks.length">
                <div class="hw-block-head">
                  <Icon name="memory-stick" :size="13" />
                  <span class="hw-block-title">内存条</span>
                  <span class="tag tag-info">{{ hardwareInfo.memory_sticks.length }} 条</span>
                </div>
                <table class="hw-table">
                  <thead>
                    <tr>
                      <th>插槽</th>
                      <th>厂商</th>
                      <th>容量</th>
                      <th>频率</th>
                      <th>类型</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="(stick, idx) in hardwareInfo.memory_sticks" :key="idx">
                      <td class="mono">{{ stick.bank_label || stick.device_locator || '-' }}</td>
                      <td>{{ stick.manufacturer || '-' }}</td>
                      <td class="mono">{{ stick.capacity_gb }} GB</td>
                      <td class="mono">{{ stick.speed_mhz }} MHz</td>
                      <td><span class="tag tag-neutral" v-if="stick.memory_type">{{ stick.memory_type }}</span><span v-else class="text-muted">-</span></td>
                    </tr>
                  </tbody>
                </table>
                <div class="hw-meta" v-if="hardwareInfo.memory_sticks.length">
                  总容量 <span class="mono text-secondary">{{ totalMemoryGB }} GB</span>
                  · 规格
                  <span class="mono text-secondary">{{ hardwareInfo.memory_sticks[0]?.form_factor || '-' }}</span>
                </div>
              </div>

              <!-- CPU -->
              <div class="hw-block" v-if="hardwareInfo.cpu">
                <div class="hw-block-head">
                  <Icon name="cpu" :size="13" />
                  <span class="hw-block-title">处理器</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">型号</span>
                  <span class="kv-value">{{ hardwareInfo.cpu.name }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">厂商</span>
                  <span class="kv-value">{{ hardwareInfo.cpu.manufacturer }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">核心 / 线程</span>
                  <span class="kv-value mono">{{ hardwareInfo.cpu.cores }} / {{ hardwareInfo.cpu.logical_cores }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">最大频率</span>
                  <span class="kv-value mono">{{ hardwareInfo.cpu.max_clock_mhz }} MHz</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">L2 / L3 缓存</span>
                  <span class="kv-value mono">{{ hardwareInfo.cpu.l2_cache || '-' }} / {{ hardwareInfo.cpu.l3_cache || '-' }}</span>
                </div>
              </div>

              <!-- 主板 -->
              <div class="hw-block" v-if="hardwareInfo.motherboard">
                <div class="hw-block-head">
                  <Icon name="motherboard" :size="13" />
                  <span class="hw-block-title">主板</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">厂商</span>
                  <span class="kv-value">{{ hardwareInfo.motherboard.manufacturer }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">产品</span>
                  <span class="kv-value">{{ hardwareInfo.motherboard.product }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">BIOS</span>
                  <span class="kv-value mono">{{ hardwareInfo.motherboard.bios_version }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">BIOS 日期</span>
                  <span class="kv-value mono">{{ hardwareInfo.motherboard.bios_date || '-' }}</span>
                </div>
              </div>

              <!-- GPU -->
              <div class="hw-block" v-if="hardwareInfo.gpu">
                <div class="hw-block-head">
                  <Icon name="monitor" :size="13" />
                  <span class="hw-block-title">显卡</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">型号</span>
                  <span class="kv-value">{{ hardwareInfo.gpu.name }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">显存</span>
                  <span class="kv-value mono">{{ hardwareInfo.gpu.adapter_ram_gb }} GB</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">驱动版本</span>
                  <span class="kv-value mono">{{ hardwareInfo.gpu.driver_version }}</span>
                </div>
              </div>

              <p class="hw-empty" v-if="!hardwareInfo.cpu && !hardwareInfo.motherboard && !hardwareInfo.gpu && !(hardwareInfo.memory_sticks && hardwareInfo.memory_sticks.length)">
                暂无硬件详细信息
              </p>
            </div>
            <div class="card-body" v-else-if="!hardwareInfo">
              <p class="hw-empty">硬件信息获取失败</p>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const emit = defineEmits(["navigate"]);
const loading = ref(true);
const systemInfo = ref({
  cpu: { brand: "", core_count: 0, usage: 0, frequency: 0 },
  memory: { total_gb: 0, used_gb: 0, free_gb: 0, usage_percent: 0 },
  disks: [],
  os_name: "",
  os_version: "",
  os_build: "",
  hostname: "",
  uptime_hours: 0,
  boot_time: "",
});

const hardwareInfo = ref(null);
const showHardware = ref(true);

const totalMemoryGB = computed(() => {
  if (!hardwareInfo.value?.memory_sticks) return 0;
  return hardwareInfo.value.memory_sticks.reduce((s, m) => s + (m.capacity_gb || 0), 0);
});

function getBarClass(percent) {
  if (percent > 85) return "high";
  if (percent > 60) return "medium";
  if (percent > 30) return "normal";
  return "low";
}

function formatUptime(hours) {
  const days = Math.floor(hours / 24);
  const h = Math.floor(hours % 24);
  if (days > 0) return `${days}天 ${h}小时`;
  return `${h}小时`;
}

async function refresh() {
  loading.value = true;
  try {
    systemInfo.value = await invoke("get_system_info");
  } catch (e) {
    console.error("Failed to get system info:", e);
  }
  try {
    hardwareInfo.value = await invoke("get_hardware_info");
  } catch (e) {
    console.error("Failed to get hardware info:", e);
    hardwareInfo.value = null;
  }
  loading.value = false;
}

onMounted(refresh);
</script>

<style scoped>
.dashboard {
  max-width: 1600px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 22px;
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

/* 实时负载 */
.metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
  margin-bottom: 16px;
}

.metric {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 13px;
}

.metric-head {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.metric-label {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--text-secondary);
  flex: 1;
}

.metric-value {
  font-family: var(--font-mono);
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.01em;
}

.metric-value .unit {
  font-size: 11px;
  color: var(--text-muted);
  margin-left: 1px;
  font-weight: 400;
}

.metric .bar {
  margin-bottom: 8px;
}

.metric-meta {
  font-size: 11.5px;
  color: var(--text-primary);
  font-weight: 500;
}

.metric-sub {
  font-size: 10.5px;
  color: var(--text-muted);
  margin-top: 1px;
}

/* 两列 */
.two-col {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: 12px;
}

.col {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 快捷操作 */
.action-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-light);
  color: var(--text-primary);
}

/* 硬件信息 */
.hw-block {
  padding: 14px 0;
  border-top: 1px solid var(--border);
}

.hw-block:first-child {
  padding-top: 0;
  border-top: none;
}

.hw-block-head {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 10px;
  color: var(--text-secondary);
}

.hw-block-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.hw-table {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  overflow: hidden;
  margin-bottom: 8px;
}

.hw-table th {
  padding: 6px 10px;
  font-size: 9.5px;
  background: var(--bg-elevated);
}

.hw-table td {
  padding: 6px 10px;
  font-size: 11.5px;
  border-bottom: 1px solid var(--border);
}

.hw-table tbody tr:last-child td {
  border-bottom: none;
}

.hw-meta {
  font-size: 11px;
  color: var(--text-muted);
}

.hw-empty {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

@media (max-width: 900px) {
  .two-col {
    grid-template-columns: 1fr;
  }
}
</style>
