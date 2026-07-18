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

    <div class="scan-prompt" v-else-if="!hasLoaded">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始加载</p>
      <button class="btn btn-primary" @click="refresh">开始加载</button>
    </div>

    <template v-else>
      <!-- 实时负载仪表盘 -->
      <div class="metric-grid">
        <div class="metric metric-cpu">
          <div class="metric-head">
            <Icon name="cpu" :size="13" :stroke-width="1.75" />
            <span class="metric-label">CPU 负载</span>
            <span class="metric-value">{{ systemInfo.cpu.usage.toFixed(1) }}<span class="unit">%</span></span>
          </div>
          <div class="bar"><div class="bar-fill" :class="getBarClass(systemInfo.cpu.usage)" :style="{ width: systemInfo.cpu.usage + '%' }"></div></div>
          <div class="metric-meta">{{ systemInfo.cpu.brand }}</div>
          <div class="metric-sub">{{ systemInfo.cpu.core_count }} 核 · {{ (systemInfo.cpu.frequency / 1000).toFixed(2) }} GHz</div>
        </div>

        <div class="metric metric-mem">
          <div class="metric-head">
            <Icon name="memory-stick" :size="13" :stroke-width="1.75" />
            <span class="metric-label">内存占用</span>
            <span class="metric-value">{{ systemInfo.memory.usage_percent.toFixed(1) }}<span class="unit">%</span></span>
          </div>
          <div class="bar"><div class="bar-fill" :class="getBarClass(systemInfo.memory.usage_percent)" :style="{ width: systemInfo.memory.usage_percent + '%' }"></div></div>
          <div class="metric-meta mono">{{ systemInfo.memory.used_gb }} / {{ systemInfo.memory.total_gb }} GB</div>
          <div class="metric-sub">可用 {{ systemInfo.memory.free_gb }} GB</div>
        </div>

        <!-- 电池（如果有） -->
        <div class="metric metric-battery" v-if="systemInfo.battery">
          <div class="metric-head">
            <Icon name="battery" :size="13" :stroke-width="1.75" />
            <span class="metric-label">电池</span>
            <span class="metric-value">{{ systemInfo.battery.percent }}<span class="unit">%</span></span>
          </div>
          <div class="bar"><div class="bar-fill" :class="getBatteryBarClass(systemInfo.battery)" :style="{ width: systemInfo.battery.percent + '%' }"></div></div>
          <div class="metric-meta">{{ systemInfo.battery.is_charging ? '充电中' : '使用电池' }}</div>
          <div class="metric-sub" v-if="systemInfo.battery.time_remaining_min">
            剩余 {{ Math.floor(systemInfo.battery.time_remaining_min / 60) }}h {{ systemInfo.battery.time_remaining_min % 60 }}m
          </div>
          <div class="metric-sub" v-else>接通电源</div>
        </div>
      </div>

      <!-- 三列布局 -->
      <div class="three-col">
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

          <!-- 网络信息（新增） -->
          <div class="card" v-if="systemInfo.network && (systemInfo.network.ip_address || systemInfo.network.mac_address)">
            <div class="card-header">
              <span class="card-title">网络信息</span>
              <Icon name="wifi" :size="13" :stroke-width="1.75" />
            </div>
            <div class="card-body">
              <div class="kv-row" v-if="systemInfo.network.ip_address">
                <span class="kv-label">IP 地址</span>
                <span class="kv-value mono">{{ systemInfo.network.ip_address }}</span>
              </div>
              <div class="kv-row" v-if="systemInfo.network.mac_address">
                <span class="kv-label">MAC 地址</span>
                <span class="kv-value mono">{{ systemInfo.network.mac_address }}</span>
              </div>
              <div class="kv-row" v-if="systemInfo.network.adapter_name">
                <span class="kv-label">适配器</span>
                <span class="kv-value">{{ systemInfo.network.adapter_name }}</span>
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

        <!-- 中列：CPU + 主板 -->
        <div class="col">
          <!-- CPU 硬件 -->
          <div class="card" v-if="hardwareInfo && hardwareInfo.cpu">
            <div class="card-header" @click="hwCpu = !hwCpu" style="cursor:pointer;user-select:none">
              <span class="card-title">处理器 CPU</span>
              <Icon :name="hwCpu ? 'chevron-down' : 'chevron-right'" :size="14" :stroke-width="2" />
            </div>
            <div class="card-body" v-if="hwCpu">
              <div class="kv-row">
                <span class="kv-label">型号</span>
                <span class="kv-value">{{ hardwareInfo.cpu.name }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">厂商</span>
                <span class="kv-value">{{ hardwareInfo.cpu.manufacturer }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">插槽</span>
                <span class="kv-value mono">{{ hardwareInfo.cpu.socket || '-' }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">核心 / 线程</span>
                <span class="kv-value mono">{{ hardwareInfo.cpu.cores }} / {{ hardwareInfo.cpu.logical_cores }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">最大频率</span>
                <span class="kv-value mono">{{ hardwareInfo.cpu.max_clock_mhz }} MHz</span>
              </div>
              <div class="kv-row" v-if="hardwareInfo.cpu.current_clock_mhz">
                <span class="kv-label">当前频率</span>
                <span class="kv-value mono">{{ hardwareInfo.cpu.current_clock_mhz }} MHz</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">L2 / L3 缓存</span>
                <span class="kv-value mono">{{ formatCache(hardwareInfo.cpu.l2_cache_kb) }} / {{ formatCache(hardwareInfo.cpu.l3_cache_kb) }}</span>
              </div>
              <div class="kv-row" v-if="hardwareInfo.cpu.voltage">
                <span class="kv-label">电压</span>
                <span class="kv-value mono">{{ hardwareInfo.cpu.voltage }} V</span>
              </div>
            </div>
          </div>

          <!-- 主板 -->
          <div class="card" v-if="hardwareInfo && hardwareInfo.motherboard">
            <div class="card-header" @click="hwMb = !hwMb" style="cursor:pointer;user-select:none">
              <span class="card-title">主板</span>
              <Icon :name="hwMb ? 'chevron-down' : 'chevron-right'" :size="14" :stroke-width="2" />
            </div>
            <div class="card-body" v-if="hwMb">
              <div class="kv-row">
                <span class="kv-label">厂商</span>
                <span class="kv-value">{{ hardwareInfo.motherboard.manufacturer }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">产品</span>
                <span class="kv-value">{{ hardwareInfo.motherboard.product }}</span>
              </div>
              <div class="kv-row" v-if="hardwareInfo.motherboard.version">
                <span class="kv-label">版本</span>
                <span class="kv-value mono">{{ hardwareInfo.motherboard.version }}</span>
              </div>
              <div class="kv-row" v-if="hardwareInfo.motherboard.serial">
                <span class="kv-label">序列号</span>
                <span class="kv-value mono">{{ hardwareInfo.motherboard.serial }}</span>
              </div>
              <div class="kv-row">
                <span class="kv-label">BIOS 版本</span>
                <span class="kv-value mono">{{ hardwareInfo.motherboard.bios_version }}</span>
              </div>
              <div class="kv-row" v-if="hardwareInfo.motherboard.bios_date">
                <span class="kv-label">BIOS 日期</span>
                <span class="kv-value mono">{{ hardwareInfo.motherboard.bios_date }}</span>
              </div>
              <div class="kv-row" v-if="hardwareInfo.motherboard.bios_manufacturer">
                <span class="kv-label">BIOS 厂商</span>
                <span class="kv-value">{{ hardwareInfo.motherboard.bios_manufacturer }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 右列：硬盘 + 内存条 + 显卡 -->
        <div class="col">
          <!-- 硬盘（按物理磁盘显示：大小/厂商/类型） -->
          <div class="card" v-if="physicalDisks.length">
            <div class="card-header" @click="hwDisk = !hwDisk" style="cursor:pointer;user-select:none">
              <span class="card-title">硬盘</span>
              <span class="tag tag-info">{{ physicalDisks.length }}</span>
              <Icon :name="hwDisk ? 'chevron-down' : 'chevron-right'" :size="14" :stroke-width="2" />
            </div>
            <div class="card-body" v-if="hwDisk">
              <div class="hw-disk-item" v-for="(disk, idx) in physicalDisks" :key="idx">
                <div class="hw-disk-head">
                  <Icon name="hard-drive" :size="13" />
                  <span class="hw-disk-name">{{ disk.model || '未知硬盘' }}</span>
                </div>
                <div class="hw-disk-tags">
                  <span class="tag" :class="disk.is_ssd ? 'tag-success' : 'tag-neutral'">
                    {{ disk.is_ssd ? '固态 SSD' : '机械 HDD' }}
                  </span>
                  <span class="tag tag-neutral" v-if="disk.size_gb > 0">{{ formatDiskSize(disk.size_gb) }}</span>
                </div>
                <div class="kv-row" v-if="disk.model">
                  <span class="kv-label">厂商/型号</span>
                  <span class="kv-value">{{ disk.model }}</span>
                </div>
                <div class="kv-row" v-if="disk.size_gb > 0">
                  <span class="kv-label">容量</span>
                  <span class="kv-value mono">{{ formatDiskSize(disk.size_gb) }}</span>
                </div>
                <div class="kv-row" v-if="disk.interface_type && disk.interface_type !== '-'">
                  <span class="kv-label">接口</span>
                  <span class="kv-value mono">{{ disk.interface_type }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 内存条（相同规格合并显示） -->
          <div class="card" v-if="hardwareInfo && hardwareInfo.memory_sticks && hardwareInfo.memory_sticks.length">
            <div class="card-header" @click="hwMem = !hwMem" style="cursor:pointer;user-select:none">
              <span class="card-title">内存</span>
              <span class="tag tag-info">{{ hardwareInfo.memory_sticks.length }} 条 · {{ totalMemoryGB }} GB</span>
              <Icon :name="hwMem ? 'chevron-down' : 'chevron-right'" :size="14" :stroke-width="2" />
            </div>
            <div class="card-body" v-if="hwMem">
              <div class="mem-stick" v-for="(stick, idx) in mergedMemorySticks" :key="idx">
                <div class="mem-stick-head">
                  <Icon name="memory-stick" :size="12" />
                  <span class="mem-stick-label">{{ stick.memory_type || '内存' }}</span>
                  <span class="tag tag-neutral" v-if="stick.count > 1">×{{ stick.count }}</span>
                </div>
                <div class="kv-row" v-if="stick.manufacturer && stick.manufacturer !== 'Unknown'">
                  <span class="kv-label">厂商</span>
                  <span class="kv-value">{{ stick.manufacturer }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">单条容量</span>
                  <span class="kv-value mono">{{ stick.capacity_gb }} GB</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">总容量</span>
                  <span class="kv-value mono">{{ (stick.capacity_gb * stick.count).toFixed(2) }} GB</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">频率</span>
                  <span class="kv-value mono">{{ stick.speed_mhz }} MHz</span>
                </div>
                <div class="kv-row" v-if="stick.configured_speed_mhz && stick.configured_speed_mhz !== stick.speed_mhz">
                  <span class="kv-label">实际频率</span>
                  <span class="kv-value mono">{{ stick.configured_speed_mhz }} MHz</span>
                </div>
                <div class="kv-row" v-if="stick.part_number">
                  <span class="kv-label">型号</span>
                  <span class="kv-value mono">{{ stick.part_number }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 显卡（支持多张） -->
          <div class="card" v-if="hardwareInfo && hardwareInfo.gpus && hardwareInfo.gpus.length">
            <div class="card-header" @click="hwGpu = !hwGpu" style="cursor:pointer;user-select:none">
              <span class="card-title">显卡 {{ hardwareInfo.gpus.length > 1 ? `(${hardwareInfo.gpus.length})` : '' }}</span>
              <Icon :name="hwGpu ? 'chevron-down' : 'chevron-right'" :size="14" :stroke-width="2" />
            </div>
            <div class="card-body" v-if="hwGpu">
              <div class="gpu-block" v-for="(gpu, idx) in hardwareInfo.gpus" :key="idx">
                <div class="gpu-block-head">
                  <Icon name="monitor" :size="12" />
                  <span class="gpu-block-name">{{ gpu.name }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">厂商</span>
                  <span class="kv-value">{{ gpu.manufacturer || '-' }}</span>
                </div>
                <div class="kv-row">
                  <span class="kv-label">显存</span>
                  <span class="kv-value mono">{{ gpu.adapter_ram_gb }} GB</span>
                </div>
                <div class="kv-row" v-if="gpu.video_processor">
                  <span class="kv-label">处理器</span>
                  <span class="kv-value">{{ gpu.video_processor }}</span>
                </div>
                <div class="kv-row" v-if="gpu.driver_version">
                  <span class="kv-label">驱动版本</span>
                  <span class="kv-value mono">{{ gpu.driver_version }}</span>
                </div>
                <div class="kv-row" v-if="gpu.driver_date">
                  <span class="kv-label">驱动日期</span>
                  <span class="kv-value mono">{{ gpu.driver_date }}</span>
                </div>
              </div>
            </div>
          </div>

          <p class="hw-empty" v-if="hardwareInfo && !hardwareInfo.cpu && !hardwareInfo.motherboard && !(hardwareInfo.gpus && hardwareInfo.gpus.length) && !(hardwareInfo.memory_sticks && hardwareInfo.memory_sticks.length)">
            暂无硬件详细信息
          </p>
          <p class="hw-empty" v-if="!hardwareInfo">硬件信息获取失败</p>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";

const emit = defineEmits(["navigate"]);
const loading = ref(false);
const hasLoaded = ref(false);
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
  network: { ip_address: "", mac_address: "", adapter_name: "" },
  battery: null,
});

const hardwareInfo = ref(null);
const hwCpu = ref(true);
const hwMb = ref(true);
const hwMem = ref(true);
const hwGpu = ref(true);
const hwDisk = ref(true);

const totalMemoryGB = computed(() => {
  if (!hardwareInfo.value?.memory_sticks) return 0;
  return hardwareInfo.value.memory_sticks.reduce((s, m) => s + (m.capacity_gb || 0), 0);
});

// 合并相同的内存条（按厂商+容量+频率+型号分组）
const mergedMemorySticks = computed(() => {
  if (!hardwareInfo.value?.memory_sticks) return [];
  const groups = {};
  for (const stick of hardwareInfo.value.memory_sticks) {
    // 用关键字段作为分组键（忽略序列号和插槽位置）
    const key = [
      stick.manufacturer || "",
      stick.capacity_gb || 0,
      stick.speed_mhz || 0,
      stick.memory_type || "",
      stick.part_number || "",
    ].join("|");
    if (!groups[key]) {
      groups[key] = { ...stick, count: 1 };
    } else {
      groups[key].count += 1;
    }
  }
  return Object.values(groups);
});

// 物理硬盘列表（直接查询 Get-PhysicalDisk，和系统修复使用相同逻辑）
const physicalDisks = ref([]);

function formatDiskSize(gb) {
  if (!gb || gb <= 0) return '';
  if (gb >= 1000) return (gb / 1000).toFixed(2) + ' TB';
  return gb.toFixed(0) + ' GB';
}

function getBarClass(percent) {
  if (percent > 85) return "high";
  if (percent > 60) return "medium";
  if (percent > 30) return "normal";
  return "low";
}

function getBatteryBarClass(battery) {
  if (battery.is_charging) return "normal";
  if (battery.percent < 20) return "high";
  if (battery.percent < 50) return "medium";
  return "normal";
}

function formatUptime(hours) {
  const days = Math.floor(hours / 24);
  const h = Math.floor(hours % 24);
  if (days > 0) return `${days}天 ${h}小时`;
  return `${h}小时`;
}

function formatCache(kb) {
  if (!kb) return "-";
  if (kb >= 1024) return `${(kb / 1024).toFixed(1)} MB`;
  return `${kb} KB`;
}

async function refresh() {
  loading.value = true;
  try {
    // 快速获取系统信息（磁盘只含容量，不含详情，秒级返回）
    systemInfo.value = await invoke("get_system_info");
  } catch (e) {
    console.error("Failed to get system info:", e);
  }
  loading.value = false;

  // 异步获取硬件详情（不阻塞 UI）
  try {
    hardwareInfo.value = await invoke("get_hardware_info");
  } catch (e) {
    console.error("Failed to get hardware info:", e);
    hardwareInfo.value = null;
  }

  // 异步查询物理硬盘列表（直接用 Get-PhysicalDisk，和系统修复相同逻辑）
  try {
    physicalDisks.value = await invoke("query_physical_disks");
  } catch (e) {
    console.error("Failed to query physical disks:", e);
    physicalDisks.value = [];
  }
  hasLoaded.value = true;
}

onMounted(() => {
  // 不自动加载，等待用户点击"开始加载"
});
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

/* 实时负载仪表盘 */
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
  padding: 14px 15px;
  border-left: 3px solid var(--accent);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.metric:hover {
  border-color: var(--border-strong, var(--accent));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.metric-cpu { border-left-color: var(--info); }
.metric-mem { border-left-color: var(--accent); }
.metric-battery { border-left-color: var(--success); }

.metric-head {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 10px;
  color: var(--text-secondary);
}

.metric-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  flex: 1;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.metric-value {
  font-family: var(--font-mono);
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
  line-height: 1;
}

.metric-value .unit {
  font-size: 12px;
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

/* 三列布局 */
.three-col {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
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

/* 内存条 */
.mem-stick {
  padding: 10px 0;
  border-top: 1px solid var(--border);
}

.mem-stick:first-child {
  padding-top: 0;
  border-top: none;
}

.mem-stick-head {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.mem-stick-label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

/* GPU 块 */
.gpu-block {
  padding: 10px 0;
  border-top: 1px solid var(--border);
}

.gpu-block:first-child {
  padding-top: 0;
  border-top: none;
}

.gpu-block-head {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 8px;
  color: var(--text-secondary);
}

.gpu-block-name {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

/* 硬盘项 */
.hw-disk-item {
  padding: 12px 0;
  border-top: 1px solid var(--border);
}

.hw-disk-item:first-child {
  padding-top: 0;
  border-top: none;
}

.hw-disk-head {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 8px;
  color: var(--accent);
}

.hw-disk-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hw-disk-tags {
  display: flex;
  gap: 5px;
  margin-bottom: 8px;
}

.disk-capacity {
  margin-bottom: 10px;
}

.disk-capacity .bar {
  margin-bottom: 5px;
}

.disk-capacity-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  color: var(--text-muted);
}

.disk-usage-pct {
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--text-secondary);
}

.tag-success {
  background: var(--success-dim);
  color: var(--success);
  border-color: var(--success);
}

.tag-warning {
  background: var(--warning-dim);
  color: var(--warning);
  border-color: var(--warning);
}

.hw-empty {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

@media (max-width: 1200px) {
  .three-col {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 800px) {
  .three-col {
    grid-template-columns: 1fr;
  }
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
