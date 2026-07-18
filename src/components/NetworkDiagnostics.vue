<template>
  <div class="network-diagnostics fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">网络诊断</h1>
        <p class="page-subtitle">排查网络连接、DNS 解析与路由链路问题</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="loadAdapters" :disabled="loadingAdapters">
          <span v-if="loadingAdapters" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          刷新适配器
        </button>
      </div>
    </div>

    <!-- 初始状态：开始扫描 -->
    <div v-if="!hasLoaded && !loadingAdapters" class="scan-prompt">
      <Icon name="search" :size="32" />
      <p>点击下方按钮开始扫描</p>
      <button class="btn btn-primary" @click="loadAdapters">开始扫描</button>
    </div>

    <!-- 1. 网络适配器 -->
    <div class="card section-card" v-if="hasLoaded || loadingAdapters">
      <div class="card-header">
        <div class="section-title">
          <Icon name="wifi" :size="13" :stroke-width="1.75" />
          <span>网络适配器</span>
        </div>
      </div>
      <div class="card-body">
        <div v-if="loadingAdapters" class="loading small">
          <div class="spinner" style="width:18px;height:18px"></div>
          <p>正在读取网络适配器信息...</p>
        </div>
        <div v-else-if="adapters.length > 0" class="adapter-list">
          <div v-for="(adapter, i) in adapters" :key="i" class="adapter-item">
            <div class="adapter-head">
              <span class="adapter-name">{{ adapter.name }}</span>
              <span class="adapter-status">
                <span class="dot" :class="adapter.status === '已连接' ? 'dot-success' : 'dot-warning'"></span>
                {{ adapter.status }}
              </span>
            </div>
            <p class="adapter-desc">{{ adapter.description }}</p>
            <div class="adapter-grid">
              <div class="adapter-field">
                <span class="field-label">IP 地址</span>
                <span class="field-value mono">{{ adapter.ip_address || '-' }}</span>
              </div>
              <div class="adapter-field">
                <span class="field-label">网关</span>
                <span class="field-value mono">{{ adapter.gateway || '-' }}</span>
              </div>
              <div class="adapter-field">
                <span class="field-label">DNS 服务器</span>
                <span class="field-value mono">{{ adapter.dns_servers.join(', ') || '-' }}</span>
              </div>
              <div class="adapter-field">
                <span class="field-label">链路速度</span>
                <span class="field-value mono">{{ adapter.link_speed_mbps }} Mbps</span>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="empty-state">
          <Icon name="wifi" :size="22" :stroke-width="1.5" />
          <p>未检测到网络适配器</p>
        </div>
      </div>
    </div>

    <!-- 2. Ping 测试 -->
    <div class="card section-card">
      <div class="card-header">
        <div class="section-title">
          <Icon name="activity" :size="13" :stroke-width="1.75" />
          <span>Ping 测试</span>
        </div>
      </div>
      <div class="card-body">
        <div class="test-input-row">
          <input
            v-model="pingHost"
            type="text"
            class="test-input"
            placeholder="输入主机或 IP"
            @keyup.enter="runPing"
          />
          <button class="btn btn-primary btn-sm" @click="runPing" :disabled="pinging">
            <span v-if="pinging" class="spinner" style="width:12px;height:12px"></span>
            <Icon v-else name="activity" :size="13" :stroke-width="2" />
            开始 Ping
          </button>
        </div>
        <div v-if="pingResult" class="test-result">
          <div class="result-summary">
            <div class="result-stat">
              <span class="stat-label">目标</span>
              <span class="stat-value mono">{{ pingResult.host }}</span>
            </div>
            <div class="result-stat">
              <span class="stat-label">发送 / 接收</span>
              <span class="stat-value mono">{{ pingResult.packets_sent }} / {{ pingResult.packets_received }}</span>
            </div>
            <div class="result-stat">
              <span class="stat-label">丢包率</span>
              <span class="stat-value mono" :class="lossClass(pingResult.loss_percent)">
                {{ pingResult.loss_percent.toFixed(1) }}%
              </span>
            </div>
            <div class="result-stat">
              <span class="stat-label">最小 / 平均 / 最大</span>
              <span class="stat-value mono">
                {{ pingResult.min_ms.toFixed(1) }} / {{ pingResult.avg_ms.toFixed(1) }} / {{ pingResult.max_ms.toFixed(1) }} ms
              </span>
            </div>
          </div>
          <details class="raw-output">
            <summary>查看原始输出</summary>
            <pre class="output-text">{{ pingResult.raw_output }}</pre>
          </details>
        </div>
      </div>
    </div>

    <!-- 3. DNS 测试 -->
    <div class="card section-card">
      <div class="card-header">
        <div class="section-title">
          <Icon name="search" :size="13" :stroke-width="1.75" />
          <span>DNS 解析测试</span>
        </div>
      </div>
      <div class="card-body">
        <div class="test-input-row">
          <input
            v-model="dnsDomain"
            type="text"
            class="test-input"
            placeholder="输入域名"
            @keyup.enter="runDns"
          />
          <button class="btn btn-primary btn-sm" @click="runDns" :disabled="dnsing">
            <span v-if="dnsing" class="spinner" style="width:12px;height:12px"></span>
            <Icon v-else name="search" :size="13" :stroke-width="2" />
            解析测试
          </button>
        </div>
        <div v-if="dnsResult" class="test-result">
          <div class="result-summary">
            <div class="result-stat">
              <span class="stat-label">域名</span>
              <span class="stat-value mono">{{ dnsResult.domain }}</span>
            </div>
            <div class="result-stat">
              <span class="stat-label">解析耗时</span>
              <span class="stat-value mono">{{ dnsResult.resolve_time_ms.toFixed(0) }} ms</span>
            </div>
            <div class="result-stat">
              <span class="stat-label">状态</span>
              <span class="tag" :class="dnsResult.success ? 'tag-success' : 'tag-danger'">
                {{ dnsResult.success ? '解析成功' : '解析失败' }}
              </span>
            </div>
          </div>
          <div v-if="dnsResult.resolved_ips.length > 0" class="dns-ips">
            <span class="field-label">解析到的 IP</span>
            <div class="ip-chips">
              <span v-for="(ip, i) in dnsResult.resolved_ips" :key="i" class="ip-chip mono">{{ ip }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 4. 路由追踪 -->
    <div class="card section-card">
      <div class="card-header">
        <div class="section-title">
          <Icon name="signal" :size="13" :stroke-width="1.75" />
          <span>路由追踪 Traceroute</span>
        </div>
      </div>
      <div class="card-body">
        <div class="test-input-row">
          <input
            v-model="traceHost"
            type="text"
            class="test-input"
            placeholder="输入目标主机"
            @keyup.enter="runTrace"
          />
          <button class="btn btn-primary btn-sm" @click="runTrace" :disabled="tracing">
            <span v-if="tracing" class="spinner" style="width:12px;height:12px"></span>
            <Icon v-else name="signal" :size="13" :stroke-width="2" />
            开始追踪
          </button>
        </div>
        <div v-if="tracing" class="loading small">
          <div class="spinner" style="width:18px;height:18px"></div>
          <p>正在追踪路由，可能需要数分钟...</p>
        </div>
        <div v-else-if="traceHops.length > 0" class="trace-table-wrapper">
          <table>
            <thead>
              <tr>
                <th class="num-col">跳数</th>
                <th>地址</th>
                <th class="num-col">时间 1</th>
                <th class="num-col">时间 2</th>
                <th class="num-col">时间 3</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="hop in traceHops" :key="hop.hop">
                <td class="num-col">{{ hop.hop }}</td>
                <td class="mono">{{ hop.address }}</td>
                <td class="num-col mono">{{ formatTime(hop.times_ms[0]) }}</td>
                <td class="num-col mono">{{ formatTime(hop.times_ms[1]) }}</td>
                <td class="num-col mono">{{ formatTime(hop.times_ms[2]) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div v-else-if="traceDone" class="empty-state">
          <Icon name="signal" :size="22" :stroke-width="1.5" />
          <p>未获取到路由信息</p>
        </div>
      </div>
    </div>

    <!-- 提示 -->
    <div class="card tip-card tip-info">
      <Icon name="info" :size="16" :stroke-width="2" class="tip-icon icon-info" />
      <div class="tip-content">
        <p>网络诊断工具可帮助排查无法上网、网速慢、DNS 解析异常等问题。</p>
        <p>路由追踪过程较长（每个跳点最多等待 2 秒），请耐心等待。</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { useScanLogStore } from "../stores/scanLog";

const loadingAdapters = ref(false);
const hasLoaded = ref(false);
const adapters = ref([]);

const pingHost = ref("8.8.8.8");
const pinging = ref(false);
const pingResult = ref(null);

const dnsDomain = ref("www.baidu.com");
const dnsing = ref(false);
const dnsResult = ref(null);

const traceHost = ref("8.8.8.8");
const tracing = ref(false);
const traceHops = ref([]);
const traceDone = ref(false);
const scanLog = useScanLogStore();

async function loadAdapters() {
  loadingAdapters.value = true;
  scanLog.startTask("网络适配器扫描", "network");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示扫描流程） =====
  scanLog.pushPhases([
    "初始化网络接口枚举器...",
    { msg: "读取网卡适配器信息 (IP / 网关 / DNS)", level: "info" },
    { msg: "检测链路连接状态与速率", level: "warning" },
  ]);

  try {
    const info = await invoke("get_network_info");
    adapters.value = info.adapters || [];

    // ===== 结果详情报告（网络适配器） =====
    scanLog.pushSeparator("网络适配器");
    scanLog.pushLog(`检测到 ${adapters.value.length} 个网络适配器`, "success");
    if (adapters.value.length === 0) {
      scanLog.pushDetail("结果", "未检测到网络适配器", "warning");
    } else {
      adapters.value.forEach((a, i) => {
        scanLog.pushDetail(
          `${i + 1}. ${a.name}`,
          `${a.status} · ${a.link_speed_mbps} Mbps`,
          a.status === "已连接" ? "success" : "warning"
        );
        scanLog.pushDetail("  IP", a.ip_address || "-", "dim");
        scanLog.pushDetail("  网关", a.gateway || "-", "dim");
        scanLog.pushDetail("  DNS", (a.dns_servers && a.dns_servers.join(", ")) || "-", "dim");
      });
    }
    scanLog.complete(`网络适配器扫描完成，共 ${adapters.value.length} 个`);
  } catch (e) {
    console.error("Failed to load network info:", e);
    adapters.value = [];
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  loadingAdapters.value = false;
  hasLoaded.value = true;
}

async function runPing() {
  if (!pingHost.value.trim()) return;
  const host = pingHost.value.trim();
  pinging.value = true;
  pingResult.value = null;
  scanLog.startTask("Ping 测试", "network");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示测试流程） =====
  scanLog.pushPhases([
    `解析目标 ${host} ...`,
    { msg: "构造 ICMP 回显请求包", level: "info" },
    { msg: "发送探测包并等待回显应答...", level: "warning" },
  ]);

  try {
    pingResult.value = await invoke("ping_test", { host: host });

    // ===== 结果详情报告（Ping 结果） =====
    const r = pingResult.value;
    scanLog.pushSeparator("Ping 结果");
    scanLog.pushDetail("目标主机", r.host, "info");
    scanLog.pushDetail(
      "发送 / 接收",
      `${r.packets_sent} / ${r.packets_received} 包`,
      r.success ? "success" : "error"
    );
    scanLog.pushDetail(
      "丢包率",
      `${r.loss_percent.toFixed(1)}%`,
      r.loss_percent >= 100 ? "error" : r.loss_percent > 0 ? "warning" : "success"
    );
    scanLog.pushDetail("最小延迟", `${r.min_ms.toFixed(1)} ms`, "dim");
    scanLog.pushDetail(
      "平均延迟",
      `${r.avg_ms.toFixed(1)} ms`,
      r.avg_ms > 100 ? "warning" : "success"
    );
    scanLog.pushDetail("最大延迟", `${r.max_ms.toFixed(1)} ms`, "dim");

    scanLog.complete(`Ping 测试完成：${host} 丢包率 ${r.loss_percent.toFixed(1)}%`);
  } catch (e) {
    pingResult.value = {
      host,
      success: false,
      packets_sent: 0,
      packets_received: 0,
      loss_percent: 100.0,
      min_ms: 0,
      avg_ms: 0,
      max_ms: 0,
      raw_output: "Ping 执行失败: " + e,
    };
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  pinging.value = false;
}

async function runDns() {
  if (!dnsDomain.value.trim()) return;
  const domain = dnsDomain.value.trim();
  dnsing.value = true;
  dnsResult.value = null;
  scanLog.startTask("DNS 解析测试", "network");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示解析流程） =====
  scanLog.pushPhases([
    `准备解析域名 ${domain} ...`,
    { msg: "向 DNS 服务器发送查询请求", level: "info" },
    { msg: "等待 DNS 应答并校验记录...", level: "warning" },
  ]);

  try {
    dnsResult.value = await invoke("dns_test", { domain });

    // ===== 结果详情报告（DNS 解析结果） =====
    const r = dnsResult.value;
    scanLog.pushSeparator("DNS 解析结果");
    scanLog.pushDetail("域名", r.domain, "info");
    scanLog.pushDetail("解析耗时", `${r.resolve_time_ms.toFixed(0)} ms`, "dim");
    scanLog.pushDetail("解析状态", r.success ? "解析成功" : "解析失败", r.success ? "success" : "error");
    if (r.dns_server) scanLog.pushDetail("DNS 服务器", r.dns_server, "dim");
    if (r.resolved_ips && r.resolved_ips.length > 0) {
      scanLog.pushLog(`解析到 ${r.resolved_ips.length} 个 IP 地址`, "success");
      r.resolved_ips.slice(0, 5).forEach((ip, i) => scanLog.pushDetail(`  IP ${i + 1}`, ip, "dim"));
      if (r.resolved_ips.length > 5) {
        scanLog.pushDetail("...", `还有 ${r.resolved_ips.length - 5} 个未显示`, "dim");
      }
    } else if (r.success) {
      scanLog.pushDetail("解析 IP", "无返回记录", "warning");
    }

    scanLog.complete(`DNS 解析测试完成：${domain}`);
  } catch (e) {
    dnsResult.value = {
      domain,
      dns_server: "",
      resolve_time_ms: 0,
      resolved_ips: [],
      success: false,
    };
    console.error("DNS test failed:", e);
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  dnsing.value = false;
}

async function runTrace() {
  if (!traceHost.value.trim()) return;
  const host = traceHost.value.trim();
  tracing.value = true;
  traceDone.value = false;
  traceHops.value = [];
  scanLog.startTask("路由追踪", "network");
  let ok = true;

  // ===== 预步骤日志（invoke 前展示追踪流程） =====
  scanLog.pushPhases([
    `开始追踪到 ${host} 的路由...`,
    { msg: "逐跳增加 TTL 并发送探测包", level: "info" },
    { msg: "记录每一跳的往返延迟（可能耗时较长）...", level: "warning" },
  ]);

  try {
    traceHops.value = await invoke("traceroute", { host });

    // ===== 结果详情报告（路由追踪结果） =====
    const hops = traceHops.value;
    scanLog.pushSeparator("路由追踪结果");
    scanLog.pushLog(`路由追踪完成，共 ${hops.length} 跳`, "success");
    if (hops.length === 0) {
      scanLog.pushDetail("结果", "未获取到路由信息", "warning");
    } else {
      hops.slice(0, 8).forEach((h) => {
        const t1 = h.times_ms && h.times_ms[0] != null ? h.times_ms[0].toFixed(1) + "ms" : "-";
        const t2 = h.times_ms && h.times_ms[1] != null ? h.times_ms[1].toFixed(1) + "ms" : "-";
        const t3 = h.times_ms && h.times_ms[2] != null ? h.times_ms[2].toFixed(1) + "ms" : "-";
        const timedOut = !h.times_ms || h.times_ms.every((t) => t === null || t === 0);
        scanLog.pushDetail(
          `第 ${h.hop} 跳`,
          `${h.address || "请求超时"} · [${t1} / ${t2} / ${t3}]`,
          timedOut ? "warning" : "dim"
        );
      });
      if (hops.length > 8) {
        scanLog.pushDetail("...", `还有 ${hops.length - 8} 跳未显示`, "dim");
      }
    }

    scanLog.complete(`路由追踪完成：${host}，共 ${hops.length} 跳`);
  } catch (e) {
    console.error("Traceroute failed:", e);
    traceHops.value = [];
    scanLog.pushLog("失败: " + String(e), "error");
    scanLog.fail(String(e));
    ok = false;
  }
  tracing.value = false;
  traceDone.value = true;
}

function lossClass(loss) {
  if (loss >= 100) return "text-danger";
  if (loss > 0) return "text-warning";
  return "text-success";
}

function formatTime(t) {
  if (t === undefined || t === null) return "-";
  return t.toFixed(1) + " ms";
}
</script>

<style scoped>
.network-diagnostics {
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

/* 网络适配器 */
.adapter-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.adapter-item {
  padding: 12px 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.adapter-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 2px;
}

.adapter-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  flex: 1;
}

.adapter-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11.5px;
  color: var(--text-secondary);
}

.adapter-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  margin-bottom: 10px;
}

.adapter-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 10px 16px;
}

.adapter-field {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.field-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.field-value {
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
}

/* 测试输入 */
.test-input-row {
  display: flex;
  gap: 8px;
  margin-bottom: 14px;
}

.test-input {
  flex: 1;
  padding: 6px 10px;
  background: var(--bg-input);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  font-family: var(--font-mono);
  height: 28px;
}

.test-input:focus {
  outline: none;
  border-color: var(--accent);
}

.test-input::placeholder {
  color: var(--text-muted);
  font-family: var(--font);
}

/* 测试结果 */
.test-result {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.result-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 18px 28px;
  padding: 12px 14px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.result-stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.result-stat .stat-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.result-stat .stat-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

/* 原始输出 */
.raw-output summary {
  cursor: pointer;
  font-size: 11.5px;
  color: var(--text-muted);
  margin-bottom: 8px;
  user-select: none;
}

.raw-output summary:hover {
  color: var(--text-secondary);
}

.output-text {
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 11.5px;
  line-height: 1.6;
  max-height: 240px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

/* DNS IP 列表 */
.dns-ips {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ip-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.ip-chip {
  padding: 3px 9px;
  background: var(--accent-dim);
  color: var(--accent);
  border-radius: var(--radius-sm);
  font-size: 11.5px;
  font-weight: 500;
}

/* 路由追踪表格 */
.trace-table-wrapper {
  max-height: 420px;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.num-col {
  text-align: right;
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

.loading.small {
  padding: 36px 20px;
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
  margin-top: 4px;
}

.tip-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.icon-info { color: var(--info); }

.tip-card p {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  line-height: 1.6;
}

.tip-card p:last-child {
  margin-bottom: 0;
}

.tip-card.tip-info { border-color: var(--border-light); }
</style>
