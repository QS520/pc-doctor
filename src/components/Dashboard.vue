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
      <!-- 顶部仪表盘（4张指标卡片） -->
      <div class="gauge-grid">
        <!-- CPU -->
        <div class="gauge-card">
          <div class="gauge-head">
            <div class="gauge-title">
              <div class="gauge-ic cpu">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect width="16" height="16" x="4" y="4" rx="2"/><rect width="6" height="6" x="9" y="9" rx="1"/><path d="M15 2v2"/><path d="M15 20v2"/><path d="M2 15h2"/><path d="M2 9h2"/><path d="M20 15h2"/><path d="M20 9h2"/><path d="M9 2v2"/><path d="M9 20v2"/></svg>
              </div>
              <span class="gauge-label">CPU 负载</span>
            </div>
            <span class="gauge-trend" :class="trendOf(systemInfo.cpu.usage).cls">{{ trendOf(systemInfo.cpu.usage).text }}</span>
          </div>
          <div class="gauge-body">
            <svg class="gauge-ring" viewBox="0 0 120 120">
              <circle class="gauge-ring-bg" cx="60" cy="60" r="50" />
              <circle
                class="gauge-ring-fill"
                :class="ringStatus(systemInfo.cpu.usage, 80, 90, 'is-cpu')"
                cx="60" cy="60" r="50"
                :stroke-dasharray="314.2"
                :stroke-dashoffset="314.2 * (1 - systemInfo.cpu.usage / 100)"
              />
              <text class="gauge-ring-text" x="60" y="62" text-anchor="middle">{{ systemInfo.cpu.usage.toFixed(0) }}<tspan class="gauge-ring-unit" x="74" y="62">%</tspan></text>
            </svg>
            <div class="gauge-stats">
              <span class="gauge-stat-value">{{ systemInfo.cpu.core_count }} 核</span>
              <span class="gauge-stat-label">{{ (systemInfo.cpu.frequency / 1000).toFixed(2) }} GHz</span>
            </div>
          </div>
          <div class="gauge-foot">
            <span class="gauge-meta">{{ systemInfo.cpu.brand || 'CPU' }}</span>
          </div>
        </div>

        <!-- 内存 -->
        <div class="gauge-card">
          <div class="gauge-head">
            <div class="gauge-title">
              <div class="gauge-ic mem">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><path d="M18 7V5a1 1 0 0 0-1-1H6.5a.5.5 0 0 0-.5.5v17a.5.5 0 0 0 .5.5H17a1 1 0 0 0 1-1v-2"/><path d="M14 10V2H7.5a.5.5 0 0 0-.5.5v17a.5.5 0 0 0 .5.5H17a1 1 0 0 0 1-1v-2"/><path d="M2 12h4"/><path d="M22 12h-4"/></svg>
              </div>
              <span class="gauge-label">内存占用</span>
            </div>
            <span class="gauge-trend" :class="trendOf(systemInfo.memory.usage_percent).cls">{{ trendOf(systemInfo.memory.usage_percent).text }}</span>
          </div>
          <div class="gauge-body">
            <svg class="gauge-ring" viewBox="0 0 120 120">
              <circle class="gauge-ring-bg" cx="60" cy="60" r="50" />
              <circle
                class="gauge-ring-fill"
                :class="ringStatus(systemInfo.memory.usage_percent, 80, 90, 'is-warn')"
                cx="60" cy="60" r="50"
                :stroke-dasharray="314.2"
                :stroke-dashoffset="314.2 * (1 - systemInfo.memory.usage_percent / 100)"
              />
              <text class="gauge-ring-text" x="60" y="62" text-anchor="middle">{{ systemInfo.memory.usage_percent.toFixed(1) }}<tspan class="gauge-ring-unit" x="80" y="62">%</tspan></text>
            </svg>
            <div class="gauge-stats">
              <span class="gauge-stat-value">{{ systemInfo.memory.used_gb }} GB</span>
              <span class="gauge-stat-label">已用 / {{ systemInfo.memory.total_gb }} GB</span>
            </div>
          </div>
          <div class="gauge-foot">
            <span class="gauge-meta">{{ memorySpeed ? memorySpeed + ' MHz' : '内存' }}</span>
          </div>
        </div>

        <!-- 磁盘（取第一个物理硬盘使用率） -->
        <div class="gauge-card" v-if="physicalDisks.length">
          <div class="gauge-head">
            <div class="gauge-title">
              <div class="gauge-ic disk">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="12" rx="10" ry="5"/><path d="M12 12v.01"/><path d="M2 12h20"/></svg>
              </div>
              <span class="gauge-label">磁盘使用</span>
            </div>
            <span class="gauge-trend" :class="trendOf(diskUsagePercent).cls">{{ trendOf(diskUsagePercent).text }}</span>
          </div>
          <div class="gauge-body">
            <svg class="gauge-ring" viewBox="0 0 120 120">
              <circle class="gauge-ring-bg" cx="60" cy="60" r="50" />
              <circle
                class="gauge-ring-fill"
                :class="ringStatus(diskUsagePercent, 80, 90, 'is-success')"
                cx="60" cy="60" r="50"
                :stroke-dasharray="314.2"
                :stroke-dashoffset="314.2 * (1 - diskUsagePercent / 100)"
              />
              <text class="gauge-ring-text" x="60" y="62" text-anchor="middle">{{ diskUsagePercent.toFixed(0) }}<tspan class="gauge-ring-unit" x="74" y="62">%</tspan></text>
            </svg>
            <div class="gauge-stats">
              <span class="gauge-stat-value">{{ totalDiskSize }}</span>
              <span class="gauge-stat-label">{{ physicalDisks.length }} 块硬盘</span>
            </div>
          </div>
          <div class="gauge-foot">
            <span class="gauge-meta">{{ physicalDisks[0]?.model || '硬盘' }}</span>
          </div>
        </div>
        <div class="gauge-card" v-else>
          <div class="gauge-head">
            <div class="gauge-title">
              <div class="gauge-ic disk">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="12" rx="10" ry="5"/><path d="M12 12v.01"/><path d="M2 12h20"/></svg>
              </div>
              <span class="gauge-label">磁盘使用</span>
            </div>
            <span class="gauge-trend trend-neutral">暂无</span>
          </div>
          <div class="gauge-body">
            <svg class="gauge-ring" viewBox="0 0 120 120">
              <circle class="gauge-ring-bg" cx="60" cy="60" r="50" />
              <text class="gauge-ring-text" x="60" y="62" text-anchor="middle">--<tspan class="gauge-ring-unit" x="74" y="62">%</tspan></text>
            </svg>
            <div class="gauge-stats">
              <span class="gauge-stat-value">--</span>
              <span class="gauge-stat-label">未检测到硬盘</span>
            </div>
          </div>
          <div class="gauge-foot">
            <span class="gauge-meta">暂无数据</span>
          </div>
        </div>

        <!-- 电池（如果有） -->
        <div class="gauge-card" v-if="systemInfo.battery">
          <div class="gauge-head">
            <div class="gauge-title">
              <div class="gauge-ic battery">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect width="16" height="10" x="2" y="7" rx="2" ry="2"/><line x1="22" x2="22" y1="11" y2="13"/><rect x="4" y="9" width="10" height="6" rx="1" fill="currentColor"/></svg>
              </div>
              <span class="gauge-label">电池</span>
            </div>
            <span class="gauge-trend trend-neutral">{{ systemInfo.battery.is_charging ? '充电中' : '放电中' }}</span>
          </div>
          <div class="gauge-body">
            <svg class="gauge-ring" viewBox="0 0 120 120">
              <circle class="gauge-ring-bg" cx="60" cy="60" r="50" />
              <circle
                class="gauge-ring-fill is-info"
                cx="60" cy="60" r="50"
                :stroke-dasharray="314.2"
                :stroke-dashoffset="314.2 * (1 - systemInfo.battery.percent / 100)"
              />
              <text class="gauge-ring-text" x="60" y="62" text-anchor="middle">{{ systemInfo.battery.percent }}<tspan class="gauge-ring-unit" x="74" y="62">%</tspan></text>
            </svg>
            <div class="gauge-stats">
              <span class="gauge-stat-value">{{ systemInfo.battery.is_charging ? '充电' : batteryTimeRemaining }}</span>
              <span class="gauge-stat-label">{{ systemInfo.battery.is_charging ? '进行中' : '剩余使用时间' }}</span>
            </div>
          </div>
          <div class="gauge-foot">
            <span class="gauge-meta">{{ systemInfo.battery.is_charging ? '正在充电' : '建议连接电源' }}</span>
          </div>
        </div>
        <div class="gauge-card" v-else>
          <div class="gauge-head">
            <div class="gauge-title">
              <div class="gauge-ic battery">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"><rect width="16" height="10" x="2" y="7" rx="2" ry="2"/><line x1="22" x2="22" y1="11" y2="13"/><rect x="4" y="9" width="10" height="6" rx="1" fill="currentColor"/></svg>
              </div>
              <span class="gauge-label">设备状态</span>
            </div>
            <span class="gauge-trend trend-ok">正常</span>
          </div>
          <div class="gauge-body">
            <div class="gauge-placeholder-body">
              <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            </div>
          </div>
          <div class="gauge-foot">
            <span class="gauge-meta">运行正常</span>
          </div>
        </div>
      </div>

      <!-- 系统健康评分横幅 -->
      <div class="health-banner">
        <div class="health-main">
          <div class="health-score" :style="healthRingStyle">
            <span>{{ healthScore }}</span>
          </div>
          <div class="health-text">
            <h3>{{ healthScore >= 80 ? '系统状态良好' : healthScore >= 60 ? '系统需要关注' : '系统状态不佳' }}</h3>
            <p>已运行 {{ formatUptime(systemInfo.uptime_hours) }} · 实时监测中</p>
          </div>
        </div>
        <div class="health-tags">
          <span class="health-tag success">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            硬件正常
          </span>
          <span class="health-tag" :class="systemInfo.memory.usage_percent > 80 ? 'warning' : 'success'">
            <svg v-if="systemInfo.memory.usage_percent > 80" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
            <svg v-else width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
            内存{{ systemInfo.memory.usage_percent > 80 ? '偏高' : '正常' }}
          </span>
          <span class="health-tag warning" v-if="systemInfo.battery && !systemInfo.battery.is_charging && systemInfo.battery.percent < 30">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
            电量偏低
          </span>
        </div>
      </div>

      <!-- 两列主内容区 -->
      <div class="main-content">
        <!-- 左列 -->
        <div class="col-left">
          <!-- 系统信息 -->
          <div class="card">
            <div class="card-header">
              <span class="card-title">系统信息</span>
              <span class="status-dot" title="运行正常"></span>
            </div>
            <div class="card-body">
              <div class="kv-grid">
                <div class="kv-item">
                  <span class="kv-label">操作系统</span>
                  <span class="kv-value">{{ systemInfo.os_build }}</span>
                </div>
                <div class="kv-item">
                  <span class="kv-label">主机名</span>
                  <span class="kv-value mono">{{ systemInfo.hostname }}</span>
                </div>
                <div class="kv-item">
                  <span class="kv-label">开机时间</span>
                  <span class="kv-value mono">{{ systemInfo.boot_time }}</span>
                </div>
                <div class="kv-item">
                  <span class="kv-label">已运行</span>
                  <span class="kv-value">{{ formatUptime(systemInfo.uptime_hours) }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 网络信息 -->
          <div class="card" v-if="systemInfo.network && (systemInfo.network.ip_address || systemInfo.network.mac_address)">
            <div class="card-header">
              <span class="card-title">网络信息</span>
              <Icon name="wifi" :size="14" :stroke-width="1.75" />
            </div>
            <div class="card-body">
              <div class="network-row" v-if="systemInfo.network.ip_address">
                <span class="network-label">IP 地址</span>
                <span class="network-value mono">{{ systemInfo.network.ip_address }}</span>
              </div>
              <div class="network-row" v-if="systemInfo.network.mac_address">
                <span class="network-label">MAC 地址</span>
                <span class="network-value mono">{{ systemInfo.network.mac_address }}</span>
              </div>
              <div class="network-row" v-if="systemInfo.network.adapter_name">
                <span class="network-label">适配器</span>
                <span class="network-value">{{ systemInfo.network.adapter_name }}</span>
              </div>
            </div>
          </div>

          <!-- 快捷操作 -->
          <div class="card">
            <div class="card-header">
              <span class="card-title">快捷操作</span>
            </div>
            <div class="card-body">
              <div class="action-grid">
                <button class="action-btn action-cleanup" @click="$emit('navigate', 'cleanup')">
                  <div class="action-icon">
                    <Icon name="broom" :size="16" />
                  </div>
                  <span>清理垃圾</span>
                </button>
                <button class="action-btn action-boost" @click="$emit('navigate', 'boost')">
                  <div class="action-icon">
                    <Icon name="zap" :size="16" />
                  </div>
                  <span>一键加速</span>
                </button>
                <button class="action-btn action-startup" @click="$emit('navigate', 'startup')">
                  <div class="action-icon">
                    <Icon name="rocket" :size="16" />
                  </div>
                  <span>开机加速</span>
                </button>
                <button class="action-btn action-repair" @click="$emit('navigate', 'repair')">
                  <div class="action-icon">
                    <Icon name="wrench" :size="16" />
                  </div>
                  <span>系统修复</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 右列：硬件概览 -->
        <div class="col-right">
          <div class="card">
            <div class="card-header">
              <span class="card-title">硬件概览</span>
              <span class="card-tag">{{ hardwareCount }} 项</span>
            </div>
            <div class="card-body">
              <div class="hw-list">

                <!-- CPU -->
                <div class="hw-item hw-cpu" v-if="hardwareInfo && hardwareInfo.cpu" @click="hwCpu = !hwCpu">
                  <div class="hw-item-icon">
                    <Icon name="cpu" :size="18" :stroke-width="1.75" />
                  </div>
                  <div class="hw-item-info">
                    <div class="hw-item-name">{{ hardwareInfo.cpu.name }}</div>
                    <div class="hw-item-tags">
                      <span class="hw-tag hw-tag-info">{{ hardwareInfo.cpu.cores }} 核 / {{ hardwareInfo.cpu.logical_cores }} 线程</span>
                      <span class="hw-tag">{{ hardwareInfo.cpu.max_clock_mhz }} MHz</span>
                    </div>
                  </div>
                  <Icon :name="hwCpu ? 'chevron-down' : 'chevron-right'" :size="16" class="hw-item-arrow" />
                </div>
                <div class="hw-detail" v-if="hwCpu && hardwareInfo && hardwareInfo.cpu">
                  <div class="hw-detail-grid">
                    <div class="hw-detail-item">
                      <span class="hwd-label">厂商</span>
                      <span class="hwd-value">{{ hardwareInfo.cpu.manufacturer }}</span>
                    </div>
                    <div class="hw-detail-item">
                      <span class="hwd-label">插槽</span>
                      <span class="hwd-value mono">{{ hardwareInfo.cpu.socket || '-' }}</span>
                    </div>
                    <div class="hw-detail-item">
                      <span class="hwd-label">L2 缓存</span>
                      <span class="hwd-value mono">{{ formatCache(hardwareInfo.cpu.l2_cache_kb) }}</span>
                    </div>
                    <div class="hw-detail-item">
                      <span class="hwd-label">L3 缓存</span>
                      <span class="hwd-value mono">{{ formatCache(hardwareInfo.cpu.l3_cache_kb) }}</span>
                    </div>
                  </div>
                </div>

                <!-- 主板 -->
                <div class="hw-item hw-mb" v-if="hardwareInfo && hardwareInfo.motherboard" @click="hwMb = !hwMb">
                  <div class="hw-item-icon">
                    <Icon name="server" :size="18" :stroke-width="1.75" />
                  </div>
                  <div class="hw-item-info">
                    <div class="hw-item-name">{{ hardwareInfo.motherboard.product }}</div>
                    <div class="hw-item-tags">
                      <span class="hw-tag">{{ hardwareInfo.motherboard.manufacturer }}</span>
                      <span class="hw-tag" v-if="hardwareInfo.motherboard.bios_version">BIOS {{ hardwareInfo.motherboard.bios_version }}</span>
                    </div>
                  </div>
                  <Icon :name="hwMb ? 'chevron-down' : 'chevron-right'" :size="16" class="hw-item-arrow" />
                </div>
                <div class="hw-detail" v-if="hwMb && hardwareInfo && hardwareInfo.motherboard">
                  <div class="hw-detail-grid">
                    <div class="hw-detail-item">
                      <span class="hwd-label">版本</span>
                      <span class="hwd-value mono">{{ hardwareInfo.motherboard.version || '-' }}</span>
                    </div>
                    <div class="hw-detail-item">
                      <span class="hwd-label">序列号</span>
                      <span class="hwd-value mono">{{ hardwareInfo.motherboard.serial || '-' }}</span>
                    </div>
                    <div class="hw-detail-item">
                      <span class="hwd-label">BIOS 日期</span>
                      <span class="hwd-value mono">{{ hardwareInfo.motherboard.bios_date || '-' }}</span>
                    </div>
                    <div class="hw-detail-item">
                      <span class="hwd-label">BIOS 厂商</span>
                      <span class="hwd-value">{{ hardwareInfo.motherboard.bios_manufacturer || '-' }}</span>
                    </div>
                  </div>
                </div>

                <!-- 硬盘 -->
                <div
                  class="hw-item hw-disk"
                  v-for="(disk, idx) in physicalDisks"
                  :key="'disk-' + idx"
                  @click="toggleDisk(idx)"
                >
                  <div class="hw-item-icon">
                    <Icon name="hard-drive" :size="18" :stroke-width="1.75" />
                  </div>
                  <div class="hw-item-info">
                    <div class="hw-item-name">{{ disk.model || '未知硬盘' }}</div>
                    <div class="hw-item-tags">
                      <span class="hw-tag" :class="disk.is_ssd ? 'hw-tag-success' : 'hw-tag-warning'">
                        {{ disk.is_ssd ? 'SSD 固态' : 'HDD 机械' }}
                      </span>
                      <span class="hw-tag" v-if="disk.size_gb > 0">{{ formatDiskSize(disk.size_gb) }}</span>
                      <span class="hw-tag" v-if="disk.interface_type && disk.interface_type !== '-'">{{ disk.interface_type }}</span>
                    </div>
                  </div>
                  <Icon :name="openDisks.includes(idx) ? 'chevron-down' : 'chevron-right'" :size="16" class="hw-item-arrow" />
                </div>
                <template v-for="(disk, idx) in physicalDisks" :key="'disk-detail-' + idx">
                  <div class="hw-detail" v-if="openDisks.includes(idx)">
                    <div class="hw-detail-grid">
                      <div class="hw-detail-item">
                        <span class="hwd-label">容量</span>
                        <span class="hwd-value mono">{{ formatDiskSize(disk.size_gb) }}</span>
                      </div>
                      <div class="hw-detail-item">
                        <span class="hwd-label">接口</span>
                        <span class="hwd-value mono">{{ disk.interface_type || '-' }}</span>
                      </div>
                      <div class="hw-detail-item">
                        <span class="hwd-label">健康状态</span>
                        <span class="hwd-value" :class="disk.health_status === 'Healthy' ? 'text-success' : ''">{{ disk.health_status || '-' }}</span>
                      </div>
                      <div class="hw-detail-item">
                        <span class="hwd-label">类型</span>
                        <span class="hwd-value">{{ disk.is_ssd ? '固态硬盘' : '机械硬盘' }}</span>
                      </div>
                    </div>
                  </div>
                </template>

                <!-- 内存 -->
                <div class="hw-item hw-mem" v-if="mergedMemorySticks.length" @click="hwMem = !hwMem">
                  <div class="hw-item-icon">
                    <Icon name="memory-stick" :size="18" :stroke-width="1.75" />
                  </div>
                  <div class="hw-item-info">
                    <div class="hw-item-name">{{ memorySummaryName }}</div>
                    <div class="hw-item-tags">
                      <span class="hw-tag hw-tag-warning">{{ totalMemorySticksCount }} 条 · {{ totalMemoryGB }} GB</span>
                      <span class="hw-tag">{{ memorySpeed }} MHz</span>
                    </div>
                  </div>
                  <Icon :name="hwMem ? 'chevron-down' : 'chevron-right'" :size="16" class="hw-item-arrow" />
                </div>
                <div class="hw-detail" v-if="hwMem && mergedMemorySticks.length">
                  <div class="mem-list">
                    <div class="mem-stick-item" v-for="(stick, idx) in mergedMemorySticks" :key="idx">
                      <div class="msi-head">
                        <span class="msi-type">{{ stick.memory_type || '内存' }}</span>
                        <span class="hw-tag" v-if="stick.count > 1">×{{ stick.count }}</span>
                      </div>
                      <div class="msi-grid">
                        <div class="msi-item">
                          <span class="msi-label">单条容量</span>
                          <span class="msi-value mono">{{ stick.capacity_gb }} GB</span>
                        </div>
                        <div class="msi-item">
                          <span class="msi-label">总容量</span>
                          <span class="msi-value mono">{{ (stick.capacity_gb * stick.count).toFixed(2) }} GB</span>
                        </div>
                        <div class="msi-item">
                          <span class="msi-label">频率</span>
                          <span class="msi-value mono">{{ stick.speed_mhz }} MHz</span>
                        </div>
                        <div class="msi-item">
                          <span class="msi-label">厂商</span>
                          <span class="msi-value">{{ stick.manufacturer && stick.manufacturer !== 'Unknown' ? stick.manufacturer : '-' }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- 显卡 -->
                <div class="hw-item hw-gpu" v-if="hardwareInfo && hardwareInfo.gpus && hardwareInfo.gpus.length" @click="hwGpu = !hwGpu">
                  <div class="hw-item-icon">
                    <Icon name="monitor" :size="18" :stroke-width="1.75" />
                  </div>
                  <div class="hw-item-info">
                    <div class="hw-item-name">{{ gpuSummaryName }}</div>
                    <div class="hw-item-tags">
                      <span class="hw-tag hw-tag-danger">{{ gpuCount }} 张</span>
                      <span class="hw-tag" v-for="(gpu, i) in hardwareInfo.gpus.slice(0, 2)" :key="i">{{ gpu.adapter_ram_gb }} GB</span>
                    </div>
                  </div>
                  <Icon :name="hwGpu ? 'chevron-down' : 'chevron-right'" :size="16" class="hw-item-arrow" />
                </div>
                <div class="hw-detail" v-if="hwGpu && hardwareInfo && hardwareInfo.gpus && hardwareInfo.gpus.length">
                  <div class="gpu-list">
                    <div class="gpu-stick-item" v-for="(gpu, idx) in hardwareInfo.gpus" :key="idx">
                      <div class="gsi-head">{{ gpu.name }}</div>
                      <div class="gsi-grid">
                        <div class="gsi-item">
                          <span class="gsi-label">厂商</span>
                          <span class="gsi-value">{{ gpu.manufacturer || '-' }}</span>
                        </div>
                        <div class="gsi-item">
                          <span class="gsi-label">显存</span>
                          <span class="gsi-value mono">{{ gpu.adapter_ram_gb }} GB</span>
                        </div>
                        <div class="gsi-item" v-if="gpu.driver_version">
                          <span class="gsi-label">驱动版本</span>
                          <span class="gsi-value mono">{{ gpu.driver_version }}</span>
                        </div>
                        <div class="gsi-item" v-if="gpu.driver_date">
                          <span class="gsi-label">驱动日期</span>
                          <span class="gsi-value mono">{{ gpu.driver_date }}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <p class="hw-empty" v-if="hardwareInfo && !hardwareInfo.cpu && !hardwareInfo.motherboard && !(hardwareInfo.gpus && hardwareInfo.gpus.length) && !(hardwareInfo.memory_sticks && hardwareInfo.memory_sticks.length) && !physicalDisks.length">
                  暂无硬件详细信息
                </p>
                <p class="hw-empty" v-if="!hardwareInfo && !physicalDisks.length">硬件信息获取失败</p>

              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { useScanLogStore } from "../stores/scanLog";

const emit = defineEmits(["navigate"]);
const scanLog = useScanLogStore();
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
const hwCpu = ref(false);
const hwMb = ref(false);
const hwMem = ref(false);
const hwGpu = ref(false);
const openDisks = ref([]);

// 物理硬盘列表
const physicalDisks = ref([]);

// 计算硬盘总使用率（取第一个盘或平均值
const diskUsagePercent = computed(() => {
  if (!systemInfo.value.disks || systemInfo.value.disks.length === 0) return 0;
  // 取第一个磁盘的使用率
  return systemInfo.value.disks[0].usage_percent || 0;
});

const totalDiskSize = computed(() => {
  if (physicalDisks.value.length === 0) return '-';
  const total = physicalDisks.value.reduce((s, d) => s + (d.size_gb || 0), 0);
  if (total >= 1000) return (total / 1000).toFixed(1) + ' TB';
  return total.toFixed(0) + ' GB';
});

const totalMemoryGB = computed(() => {
  if (!hardwareInfo.value?.memory_sticks) return 0;
  return Math.round(hardwareInfo.value.memory_sticks.reduce((s, m) => s + (m.capacity_gb || 0), 0));
});

const totalMemorySticksCount = computed(() => {
  if (!hardwareInfo.value?.memory_sticks) return 0;
  return hardwareInfo.value.memory_sticks.length;
});

const memorySpeed = computed(() => {
  if (!hardwareInfo.value?.memory_sticks || hardwareInfo.value.memory_sticks.length === 0) return '-';
  return hardwareInfo.value.memory_sticks[0].speed_mhz || '-';
});

const memorySummaryName = computed(() => {
  if (!hardwareInfo.value?.memory_sticks || hardwareInfo.value.memory_sticks.length === 0) return '内存';
  const first = hardwareInfo.value.memory_sticks[0];
  return `${first.memory_type || 'DDR'} 内存`;
});

const gpuCount = computed(() => {
  if (!hardwareInfo.value?.gpus) return 0;
  return hardwareInfo.value.gpus.length;
});

const gpuSummaryName = computed(() => {
  if (!hardwareInfo.value?.gpus || hardwareInfo.value.gpus.length === 0) return '显卡';
  return hardwareInfo.value.gpus[0].name || '显卡';
});

const hardwareCount = computed(() => {
  let count = 0;
  if (hardwareInfo.value?.cpu) count++;
  if (hardwareInfo.value?.motherboard) count++;
  count += physicalDisks.value.length;
  if (hardwareInfo.value?.memory_sticks?.length) count++;
  if (hardwareInfo.value?.gpus?.length) count++;
  return count;
});

// 合并相同的内存条
const mergedMemorySticks = computed(() => {
  if (!hardwareInfo.value?.memory_sticks) return [];
  const groups = {};
  for (const stick of hardwareInfo.value.memory_sticks) {
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

const batteryTimeRemaining = computed(() => {
  if (!systemInfo.value.battery) return '-';
  if (systemInfo.value.battery.time_remaining_min) {
    const h = Math.floor(systemInfo.value.battery.time_remaining_min / 60);
    const m = systemInfo.value.battery.time_remaining_min % 60;
    return `剩余 ${h}h ${m}m`;
  }
  return '接通电源';
});

function toggleDisk(idx) {
  const i = openDisks.value.indexOf(idx);
  if (i > -1) {
    openDisks.value.splice(i, 1);
  } else {
    openDisks.value.push(idx);
  }
}

function formatDiskSize(gb) {
  if (!gb || gb <= 0) return '';
  if (gb >= 1000) return (gb / 1000).toFixed(2) + ' TB';
  return gb.toFixed(0) + ' GB';
}

function formatUptime(hours) {
  const days = Math.floor(hours / 24);
  const h = Math.floor(hours % 24);
  if (days > 0) return `${days}天 ${h}小时`;
  return `${h}小时`;
}

function formatCache(kb) {
  if (!kb) return "-";
  if (kb >= 1024) return (kb / 1024).toFixed(1) + " MB";
  return kb + " KB";
}

// 进度环状态色：默认按语义色，偏高=黄，过高=红
function ringStatus(v, warn = 80, bad = 90, normalColor = "is-success") {
  if (v >= bad) return "is-bad";
  if (v >= warn) return "is-warn";
  return normalColor;
}

// 趋势标签：根据负载高低返回样式类与文案
function trendOf(v, warn = 80, bad = 90) {
  if (v >= bad) return { cls: "trend-bad", text: "过高" };
  if (v >= warn) return { cls: "trend-warn", text: "偏高" };
  return { cls: "trend-ok", text: "正常" };
}

// 系统健康分（基于实时数据粗略评估）
const healthScore = computed(() => {
  let score = 100;
  if (systemInfo.value.cpu.usage > 80) score -= 10;
  if (systemInfo.value.memory.usage_percent > 80) score -= 15;
  if (diskUsagePercent.value > 85) score -= 10;
  if (systemInfo.value.battery && !systemInfo.value.battery.is_charging && systemInfo.value.battery.percent < 30) score -= 10;
  return Math.max(0, Math.min(100, score));
});

const healthRingStyle = computed(() => {
  const s = healthScore.value;
  const color = s >= 80 ? "var(--success)" : s >= 60 ? "var(--warning)" : "var(--danger)";
  return { background: `conic-gradient(${color} 0% ${s}%, var(--border) ${s}% 100%)` };
});

async function refresh(silent = false) {
  if (loading.value) return; // 防止与前一次加载重叠
  if (!silent) loading.value = true;
  const doLog = !silent;
  if (doLog) {
    scanLog.startTask("系统数据采集", "dashboard");

    // ===== 预步骤日志（invoke 前展示采集流程） =====
    scanLog.pushPhases([
      "初始化系统采集器...",
      { msg: "连接系统信息接口 (WMI / 性能计数器)", level: "info" },
      "准备硬件探测模块 (CPU/主板/内存/显卡/硬盘)",
      "校准实时采样时间窗口...",
      { msg: "开始采集系统运行状态与硬件概览", level: "warning" },
    ]);
  }
  try {
    if (doLog) scanLog.pushLog("读取 CPU / 内存 / 磁盘 / 网络", "dim");
    systemInfo.value = await invoke("get_system_info");
    if (doLog) {
      scanLog.pushLog("系统信息获取成功", "success");
      const si = systemInfo.value;
      scanLog.pushSeparator("系统运行状态");
      scanLog.pushDetail("操作系统", si.os_build || si.os_name || "-", "info");
      scanLog.pushDetail("主机名", si.hostname || "-", "dim");
      scanLog.pushDetail("已运行时长", formatUptime(si.uptime_hours), "dim");
      scanLog.pushDetail("CPU 使用率", si.cpu.usage.toFixed(1) + "%", si.cpu.usage > 80 ? "warning" : "success");
      scanLog.pushDetail(
        "内存占用",
        `${si.memory.usage_percent.toFixed(1)}% (${si.memory.used_gb}/${si.memory.total_gb} GB)`,
        si.memory.usage_percent > 80 ? "warning" : "success"
      );
      if (si.disks && si.disks.length > 0) {
        const d0 = si.disks[0];
        scanLog.pushDetail(
          "系统盘使用率",
          (d0.usage_percent || 0).toFixed(1) + "%",
          (d0.usage_percent || 0) > 85 ? "warning" : "success"
        );
      }
      if (si.network && si.network.ip_address) {
        scanLog.pushDetail("IP 地址", si.network.ip_address, "dim");
      }
      if (si.battery) {
        scanLog.pushDetail(
          "电池电量",
          `${si.battery.percent}%${si.battery.is_charging ? " (充电中)" : ""}`,
          si.battery.percent < 30 && !si.battery.is_charging ? "warning" : "success"
        );
      }
    }
  } catch (e) {
    console.error("Failed to get system info:", e);
    if (doLog) scanLog.pushLog("系统信息获取失败: " + String(e), "error");
  }

  // 异步获取硬件详情
  try {
    if (doLog) scanLog.pushLog("读取硬件设备清单...", "dim");
    hardwareInfo.value = await invoke("get_hardware_info");
    if (doLog) {
      scanLog.pushLog("硬件信息获取成功", "success");
      const hi = hardwareInfo.value;
      scanLog.pushSeparator("硬件概览");
      if (hi.motherboard) {
        scanLog.pushDetail("主板", hi.motherboard.product || "-", "info");
        scanLog.pushDetail("  厂商", hi.motherboard.manufacturer || "-", "dim");
      }
      if (hi.cpu) {
        scanLog.pushDetail("CPU", `${hi.cpu.name} (${hi.cpu.cores}核/${hi.cpu.logical_cores}线程)`, "info");
        scanLog.pushDetail("  主频", `${(hi.cpu.max_clock_mhz / 1000).toFixed(2)} GHz`, "dim");
      }
      if (hi.memory_sticks && hi.memory_sticks.length > 0) {
        const total = hi.memory_sticks.reduce((s, m) => s + (m.capacity_gb || 0), 0);
        scanLog.pushDetail("内存", `${total} GB · ${hi.memory_sticks.length} 条`, "info");
        scanLog.pushDetail("  频率", `${memorySpeed} MHz`, "dim");
      }
      if (hi.gpus && hi.gpus.length > 0) {
        scanLog.pushDetail("显卡", `${hi.gpus.length} 张 (${gpuSummaryName})`, "info");
        hi.gpus.slice(0, 3).forEach((g, i) => {
          scanLog.pushDetail(`  显卡${i + 1}`, `${g.name} · ${g.adapter_ram_gb} GB 显存`, "dim");
        });
      }
    }
  } catch (e) {
    console.error("Failed to get hardware info:", e);
    hardwareInfo.value = null;
    if (doLog) scanLog.pushLog("硬件信息获取失败: " + String(e), "warning");
  }

  // 异步查询物理硬盘列表
  try {
    physicalDisks.value = await invoke("query_physical_disks");
    if (doLog) {
      scanLog.pushLog(`检测到 ${physicalDisks.value.length} 块物理硬盘`, "success");
      scanLog.pushSeparator("物理硬盘");
      if (physicalDisks.value.length === 0) {
        scanLog.pushDetail("结果", "未检测到物理硬盘", "warning");
      } else {
        physicalDisks.value.forEach((d, i) => {
          scanLog.pushDetail(
            `硬盘${i + 1} ${d.model || "未知型号"}`,
            `${d.is_ssd ? "SSD" : "HDD"} · ${formatDiskSize(d.size_gb)} · 健康: ${d.health_status || "-"}`,
            d.health_status === "Healthy" ? "success" : "warning"
          );
        });
      }
    }
  } catch (e) {
    console.error("Failed to query physical disks:", e);
    physicalDisks.value = [];
    if (doLog) scanLog.pushLog("硬盘查询失败: " + String(e), "warning");
  }
  hasLoaded.value = true;
  if (!silent) {
    loading.value = false;
    scanLog.pushSeparator();
    scanLog.pushLog(
      `综合健康评分: ${healthScore.value} / 100`,
      healthScore.value >= 80 ? "success" : healthScore.value >= 60 ? "warning" : "error"
    );
    scanLog.complete("系统总览已就绪");
  }
}

let refreshTimer = null;

onMounted(() => {
  // 首次进入立即加载
  refresh();
  // 每 30 秒静默刷新一次（不打断用户操作、不显示 loading）
  refreshTimer = setInterval(() => refresh(true), 30000);
});

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer);
});
</script>

<style scoped>
.dashboard {
  max-width: 1600px;
}

/* ========== 头部 ========== */
.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 20px;
}

.page-title {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.page-subtitle {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
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

/* ========== 仪表盘网格 ========== */
.gauge-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
  margin-bottom: 18px;
}

.gauge-card {
  border-radius: var(--radius-lg);
  padding: 18px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  color: var(--text-primary);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
  box-shadow: var(--shadow-sm);
}

.gauge-card:hover {
  border-color: var(--border-light);
  box-shadow: var(--shadow);
}

.gauge-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.gauge-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.gauge-ic {
  width: 26px;
  height: 26px;
  border-radius: 7px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.gauge-ic.cpu     { background: rgba(249, 115, 22, 0.12); color: #f97316; }
.gauge-ic.mem     { background: var(--warning-dim); color: var(--warning); }
.gauge-ic.disk    { background: var(--success-dim); color: var(--success); }
.gauge-ic.battery { background: var(--info-dim);    color: var(--info); }

.gauge-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: 0.01em;
}

.gauge-trend {
  font-size: 10.5px;
  padding: 2px 8px;
  border-radius: 999px;
  font-weight: 500;
  white-space: nowrap;
}

.trend-ok      { background: var(--success-dim); color: var(--success); }
.trend-warn    { background: var(--warning-dim); color: var(--warning); }
.trend-bad     { background: var(--danger-dim);  color: var(--danger); }
.trend-neutral { background: var(--bg-elevated); color: var(--text-muted); }

.gauge-body {
  display: flex;
  align-items: center;
  gap: 16px;
}

.gauge-ring {
  width: 84px;
  height: 84px;
  flex-shrink: 0;
}

.gauge-ring-bg {
  fill: none;
  stroke: var(--bg-elevated);
  stroke-width: 8;
}

.gauge-ring-fill {
  fill: none;
  stroke: var(--accent);
  stroke-width: 8;
  stroke-linecap: round;
  transform: rotate(-90deg);
  transform-origin: center;
  transition: stroke-dashoffset 0.8s ease, stroke 0.3s ease;
}

.gauge-ring-fill.is-accent { stroke: var(--accent); }
.gauge-ring-fill.is-success { stroke: var(--success); }
.gauge-ring-fill.is-cpu    { stroke: #f97316; }
.gauge-ring-fill.is-warn   { stroke: var(--warning); }
.gauge-ring-fill.is-bad    { stroke: var(--danger); }
.gauge-ring-fill.is-info   { stroke: var(--info); }

.gauge-ring-text {
  fill: var(--text-primary);
  font-size: 20px;
  font-weight: 700;
  font-family: var(--font-mono, monospace);
}

.gauge-ring-unit {
  font-size: 11px;
  font-weight: 500;
  fill: var(--text-muted);
}

.gauge-stats {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.gauge-stat-value {
  font-size: 18px;
  font-weight: 700;
  font-family: var(--font-mono, monospace);
  color: var(--text-primary);
}

.gauge-stat-label {
  font-size: 11px;
  color: var(--text-muted);
}

.gauge-placeholder-body {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 84px;
  height: 84px;
  flex-shrink: 0;
}

.gauge-foot {
  margin-top: 12px;
  padding-top: 10px;
  border-top: 1px solid var(--border);
}

.gauge-meta {
  font-size: 11.5px;
  color: var(--text-muted);
  font-weight: 500;
}

/* ========== 健康评分横幅 ========== */
.health-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 18px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  margin-bottom: 18px;
  box-shadow: var(--shadow-sm);
}

.health-main {
  display: flex;
  align-items: center;
  gap: 14px;
}

.health-score {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  flex-shrink: 0;
}

.health-score::before {
  content: '';
  position: absolute;
  width: 42px;
  height: 42px;
  border-radius: 50%;
  background: var(--bg-surface);
}

.health-score span {
  position: relative;
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
}

.health-text h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 2px;
}

.health-text p {
  font-size: 12px;
  color: var(--text-muted);
  margin: 0;
}

.health-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.health-tag {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 500;
}

.health-tag.success { background: var(--success-dim); color: var(--success); }
.health-tag.warning { background: var(--warning-dim); color: var(--warning); }

/* ========== 主内容两列布局 ========== */
.main-content {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: 16px;
}

.col-left,
.col-right {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ========== 卡片通用 ========== */
.card {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  overflow: hidden;
  transition: box-shadow 0.2s ease, border-color 0.2s ease;
}

.card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
  border-color: var(--border-strong, var(--border));
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
}

.card-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.card-tag {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--accent-dim, #eef2ff);
  color: var(--accent);
  font-weight: 500;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--success);
  box-shadow: 0 0 0 3px var(--success-dim);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.card-body {
  padding: 16px;
}

/* ========== 系统信息 kv-grid ========== */
.kv-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px 20px;
}

.kv-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.kv-label {
  font-size: 12px;
  color: var(--text-muted);
}

.kv-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.kv-value.mono {
  font-family: var(--font-mono, monospace);
}

/* ========== 网络信息 ========== */
.network-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px dashed var(--border);
}

.network-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.network-row:first-child {
  padding-top: 0;
}

.network-label {
  font-size: 12px;
  color: var(--text-muted);
}

.network-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.network-value.mono {
  font-family: var(--font-mono, monospace);
}

/* ========== 快捷操作 ========== */
.action-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: var(--bg-elevated);
  border: 1px solid transparent;
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-strong);
  color: var(--text-primary);
  transform: translateY(-1px);
}

.action-icon {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.action-cleanup .action-icon { background: var(--info-dim); color: var(--info); }
.action-boost .action-icon { background: var(--success-dim); color: var(--success); }
.action-startup .action-icon { background: var(--warning-dim); color: var(--warning); }
.action-repair .action-icon { background: var(--accent-dim); color: var(--accent); }

/* ========== 硬件列表 ========== */
.hw-list {
  display: flex;
  flex-direction: column;
}

.hw-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.hw-item + .hw-item {
  margin-top: 4px;
}

.hw-item:hover {
  background: var(--bg-elevated);
}

.hw-item-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.hw-cpu .hw-item-icon { background: var(--info-dim); color: var(--info); }
.hw-mb .hw-item-icon { background: var(--accent-dim); color: var(--accent); }
.hw-disk .hw-item-icon { background: var(--success-dim); color: var(--success); }
.hw-mem .hw-item-icon { background: var(--warning-dim); color: var(--warning); }
.hw-gpu .hw-item-icon { background: var(--danger-dim); color: var(--danger); }

.hw-item-info {
  flex: 1;
  min-width: 0;
}

.hw-item-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hw-item-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.hw-tag {
  font-size: 11px;
  padding: 1px 7px;
  border-radius: 4px;
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-family: var(--font-mono, monospace);
  font-weight: 500;
  white-space: nowrap;
}

.hw-tag-info {
  background: var(--info-dim);
  color: var(--info);
}

.hw-tag-success {
  background: var(--success-dim);
  color: var(--success);
}

.hw-tag-warning {
  background: var(--warning-dim);
  color: var(--warning);
}

.hw-tag-danger {
  background: var(--danger-dim);
  color: var(--danger);
}

.hw-item-arrow {
  color: var(--text-muted);
  flex-shrink: 0;
  transition: transform 0.2s ease;
}

/* ========== 硬件展开详情 ========== */
.hw-detail {
  padding: 10px 14px 14px 14px;
  animation: slideDown 0.2s ease;
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.hw-detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px 16px;
  padding: 12px;
  background: var(--bg-elevated);
  border-radius: 8px;
}

.hw-detail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.hwd-label {
  font-size: 11px;
  color: var(--text-muted);
}

.hwd-value {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.hwd-value.mono {
  font-family: var(--font-mono, monospace);
}

.text-success {
  color: var(--success);
}

/* ========== 内存详情 ========== */
.mem-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mem-stick-item {
  padding: 12px;
  background: var(--bg-elevated);
  border-radius: 8px;
}

.msi-head {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.msi-type {
  font-size: 12px;
  font-weight: 600;
  color: var(--warning);
}

.msi-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 12px;
}

.msi-item {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.msi-label {
  font-size: 11px;
  color: var(--text-muted);
}

.msi-value {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.msi-value.mono {
  font-family: var(--font-mono, monospace);
}

/* ========== 显卡详情 ========== */
.gpu-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.gpu-stick-item {
  padding: 12px;
  background: var(--bg-elevated);
  border-radius: 8px;
}

.gsi-head {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.gsi-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 12px;
}

.gsi-item {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.gsi-label {
  font-size: 11px;
  color: var(--text-muted);
}

.gsi-value {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.gsi-value.mono {
  font-family: var(--font-mono, monospace);
}

.hw-empty {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}

/* ========== 响应式 ========== */
@media (max-width: 1200px) {
  .gauge-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  .main-content {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 600px) {
  .gauge-grid {
    grid-template-columns: 1fr;
  }
  .kv-grid {
    grid-template-columns: 1fr;
  }
  .action-grid {
    grid-template-columns: 1fr;
  }
  .hw-detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>
