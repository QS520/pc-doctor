<template>
  <div class="page fade-in">
    <div class="header">
      <div>
        <h1 class="page-title">磁盘空间分析</h1>
        <p class="page-subtitle">按层级浏览磁盘空间占用，逐级深入定位大文件</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-ghost btn-sm" @click="refresh" :disabled="loading">
          <span v-if="loading" class="spinner" style="width:12px;height:12px"></span>
          <Icon v-else name="refresh" :size="13" :stroke-width="2" />
          刷新
        </button>
      </div>
    </div>

    <!-- 面包屑导航 -->
    <div class="breadcrumb" v-if="currentPath">
      <button class="crumb-item" @click="goToDrives">
        <Icon name="disc" :size="12" :stroke-width="1.75" />
        <span>磁盘</span>
      </button>
      <template v-for="(crumb, idx) in breadcrumbs" :key="idx">
        <Icon name="chevron-right" :size="11" class="crumb-sep" />
        <button
          :class="['crumb-item', { active: idx === breadcrumbs.length - 1 }]"
          @click="navigateToPath(crumb.path)"
        >
          <span>{{ crumb.name }}</span>
        </button>
      </template>
      <button
        class="crumb-copy-btn"
        :class="{ copied: copiedPaths[currentPath] }"
        :title="copiedPaths[currentPath] ? '已复制' : '复制当前路径'"
        @click="copyPath(currentPath)"
      >
        <Icon
          :name="copiedPaths[currentPath] ? 'check-check' : 'copy'"
          :size="11"
          :stroke-width="2"
        />
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-state">
      <div class="spinner" style="width:20px;height:20px"></div>
      <p>正在扫描...</p>
    </div>

    <template v-else>
      <!-- 第一级：磁盘列表 -->
      <div v-if="view === 'drives'" class="drives-grid">
        <div
          v-for="drive in drives"
          :key="drive.drive_letter"
          class="drive-card"
          @click="enterDrive(drive)"
        >
          <div class="drive-head">
            <Icon name="hard-drive" :size="16" :stroke-width="1.75" />
            <span class="drive-letter mono">{{ drive.drive_letter }}</span>
            <span class="drive-label">{{ drive.label }}</span>
            <button
              class="drive-copy-btn"
              :class="{ copied: copiedPaths[drive.drive_letter] }"
              :title="copiedPaths[drive.drive_letter] ? '已复制' : '复制磁盘路径'"
              @click.stop="copyPath(drive.drive_letter)"
            >
              <Icon
                :name="copiedPaths[drive.drive_letter] ? 'check-check' : 'copy'"
                :size="11"
                :stroke-width="2"
              />
            </button>
          </div>
          <div class="drive-bar">
            <div
              class="bar-fill"
              :class="getUsageClass(drive.usage_percent)"
              :style="{ width: drive.usage_percent + '%' }"
            ></div>
          </div>
          <div class="drive-meta">
            <span class="mono">{{ drive.used_gb }} / {{ drive.total_gb }} GB</span>
          </div>
          <div class="drive-sub">
            <span>{{ drive.drive_type }}</span>
            <span>·</span>
            <span class="mono">{{ drive.free_gb }} GB 可用</span>
            <span>·</span>
            <span class="mono">{{ drive.usage_percent }}%</span>
          </div>
          <div class="drive-fs">
            <Icon name="chevron-right" :size="12" class="enter-icon" />
          </div>
        </div>
      </div>

      <!-- 第二级及以下：目录/文件列表 -->
      <div v-else-if="view === 'list'" class="list-container">
        <!-- 返回按钮 -->
        <button
          v-if="currentResult.parent_path !== null || currentResult.is_root"
          class="back-btn"
          @click="goBack"
        >
          <Icon name="chevron-right" :size="12" :stroke-width="2" class="back-icon" />
          <span>返回上级</span>
        </button>

        <!-- 当前目录信息 -->
        <div class="dir-info">
          <div class="dir-stats">
            <span class="stat">
              <span class="stat-label">总大小</span>
              <span class="stat-value mono">{{ currentResult.total_size_display }}</span>
            </span>
            <span class="stat-divider"></span>
            <span class="stat">
              <span class="stat-label">条目数</span>
              <span class="stat-value mono">{{ currentResult.entries.length }}</span>
            </span>
            <span class="stat-divider"></span>
            <span class="stat">
              <span class="stat-label">扫描耗时</span>
              <span class="stat-value mono">{{ (currentResult.scan_time_ms / 1000).toFixed(2) }}s</span>
            </span>
            <span v-if="currentResult.has_partial" class="partial-tag" title="部分大目录因超时未完整扫描，显示为估算大小">
              <Icon name="alert" :size="11" :stroke-width="2" />
              <span>部分估算</span>
            </span>
          </div>
        </div>

        <!-- 条目列表 -->
        <div class="entries-table">
          <div class="entries-head">
            <span class="col-name">名称</span>
            <span class="col-size">大小</span>
            <span class="col-count">文件数</span>
            <span class="col-modified">修改日期</span>
            <span class="col-type">类型</span>
            <span class="col-action">操作</span>
          </div>
          <div
            v-for="entry in currentResult.entries"
            :key="entry.path"
            :class="['entry-row', { 'entry-dir': entry.is_dir, 'entry-file': !entry.is_dir }]"
            @click="entry.is_dir && enterDirectory(entry)"
          >
            <span class="col-name">
              <Icon
                :name="entry.is_dir ? 'folder' : getFileIcon(entry.extension)"
                :size="13"
                :stroke-width="1.75"
                class="entry-icon"
              />
              <span class="entry-name">{{ entry.name }}</span>
            </span>
            <span class="col-size mono" :class="getSizeClass(entry.size_bytes)">
              <span v-if="entry.is_dir && entry.size_bytes === 0 && entry.is_estimated" class="calculating">
                <span class="calc-dot"></span>计算中
              </span>
              <span v-else>{{ entry.size_display }}</span>
            </span>
            <span class="col-count mono">
              {{ entry.is_dir ? entry.file_count : '-' }}
            </span>
            <span class="col-modified mono">{{ entry.modified }}</span>
            <span class="col-type">
              <span v-if="entry.is_dir" class="tag tag-info">文件夹</span>
              <span v-else-if="entry.extension" class="tag tag-neutral mono">.{{ entry.extension }}</span>
              <span v-else class="tag tag-neutral">文件</span>
            </span>
            <span class="col-action">
              <button
                class="copy-btn"
                :class="{ copied: copiedPaths[entry.path] }"
                :title="copiedPaths[entry.path] ? '已复制' : '复制路径'"
                @click.stop="copyPath(entry.path)"
              >
                <Icon
                  :name="copiedPaths[entry.path] ? 'check-check' : 'copy'"
                  :size="11"
                  :stroke-width="2"
                />
                <span class="copy-label">{{ copiedPaths[entry.path] ? '已复制' : '复制路径' }}</span>
              </button>
            </span>
          </div>

          <div v-if="currentResult.entries.length === 0" class="empty-state">
            此目录为空或无法访问
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Icon from "./Icon.vue";

const loading = ref(true);
const view = ref("drives"); // "drives" / "list"
const drives = ref([]);
const currentResult = ref(null);
const currentPath = ref("");
const copiedPaths = ref({}); // 记录已复制的路径，用于显示反馈
let sizeUpdateUnlisten = null; // 事件监听器清理函数

// 监听子目录大小更新事件
onMounted(async () => {
  try {
    sizeUpdateUnlisten = await listen("dir-size-update", (event) => {
      const { path, size_bytes, size_display, file_count, is_estimated } = event.payload;
      // 更新当前列表中对应条目的大小
      if (currentResult.value && currentResult.value.entries) {
        const entry = currentResult.value.entries.find((e) => e.path === path);
        if (entry) {
          entry.size_bytes = size_bytes;
          entry.size_display = size_display;
          entry.file_count = file_count;
          entry.is_estimated = is_estimated;
          // 重新排序（按大小降序，文件夹优先）
          currentResult.value.entries.sort((a, b) => {
            if (a.is_dir && !b.is_dir) return -1;
            if (!a.is_dir && b.is_dir) return 1;
            return b.size_bytes - a.size_bytes;
          });
          // 更新总大小
          currentResult.value.total_size_bytes = currentResult.value.entries.reduce(
            (sum, e) => sum + e.size_bytes,
            0
          );
          currentResult.value.total_size_display = formatSizeLocal(
            currentResult.value.total_size_bytes
          );
        }
      }
    });
  } catch (e) {
    console.log("Event listener not available (browser preview mode)");
  }

  await loadDrives();
});

onUnmounted(() => {
  if (sizeUpdateUnlisten) sizeUpdateUnlisten();
});

// 面包屑
const breadcrumbs = computed(() => {
  if (!currentPath.value) return [];
  // 将路径拆分为面包屑
  const parts = [];
  let path = currentPath.value;

  // 处理 Windows 路径
  const isWindows = path.length >= 2 && path[1] === ':';
  if (isWindows) {
    const driveLetter = path.substring(0, 2);
    const rest = path.substring(3); // 跳过 "C:\"
    parts.push({ name: driveLetter, path: driveLetter + '\\' });
    if (rest) {
      const segments = rest.split('\\').filter(s => s.length > 0);
      let currentPathStr = driveLetter + '\\';
      for (const seg of segments) {
        currentPathStr = currentPathStr + (currentPathStr.endsWith('\\') ? '' : '\\') + seg;
        parts.push({ name: seg, path: currentPathStr });
      }
    }
  } else {
    // Linux 路径
    const segments = path.split('/').filter(s => s.length > 0);
    let currentPathStr = '';
    for (const seg of segments) {
      currentPathStr = currentPathStr + '/' + seg;
      parts.push({ name: seg, path: currentPathStr });
    }
  }
  return parts;
});

async function refresh() {
  if (view.value === "drives") {
    await loadDrives();
  } else {
    await loadDirectory(currentPath.value);
  }
}

async function loadDrives() {
  loading.value = true;
  try {
    drives.value = await invoke("list_drives");
    view.value = "drives";
    currentPath.value = "";
  } catch (e) {
    console.error("Failed to list drives:", e);
    // 浏览器预览模式：显示演示数据
    drives.value = getDemoDrives();
    view.value = "drives";
    currentPath.value = "";
  }
  loading.value = false;
}

async function loadDirectory(path) {
  loading.value = true;
  try {
    // 快速扫描：立即返回列表（文件大小即时，子目录大小为 0 待计算）
    currentResult.value = await invoke("scan_directory", { path });
    currentPath.value = path;
    view.value = "list";
    loading.value = false;

    // 异步触发子目录大小计算（后台并行，通过事件推送结果）
    const subdirs = currentResult.value.entries
      .filter((e) => e.is_dir)
      .map((e) => e.path);
    if (subdirs.length > 0) {
      invoke("calculate_dir_sizes", { paths: subdirs }).catch((e) =>
        console.log("Size calc skipped (browser mode):", e)
      );
    }
  } catch (e) {
    console.error("Failed to scan directory:", e);
    // 浏览器预览模式：显示演示数据
    currentResult.value = getDemoDirectory(path);
    currentPath.value = path;
    view.value = "list";
    loading.value = false;
  }
}

// 格式化文件大小（前端版，用于事件更新）
function formatSizeLocal(bytes) {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIdx = 0;
  while (size >= 1024 && unitIdx < units.length - 1) {
    size /= 1024;
    unitIdx++;
  }
  return unitIdx === 0 ? `${bytes} B` : `${size.toFixed(2)} ${units[unitIdx]}`;
}

// === 演示数据（仅用于浏览器预览） ===
function getDemoDrives() {
  return [
    {
      drive_letter: "C:\\",
      label: "系统",
      drive_type: "本地磁盘",
      total_gb: 475.0,
      used_gb: 312.5,
      free_gb: 162.5,
      usage_percent: 66,
      file_system: "NTFS",
    },
    {
      drive_letter: "D:\\",
      label: "数据",
      drive_type: "本地磁盘",
      total_gb: 931.0,
      used_gb: 487.2,
      free_gb: 443.8,
      usage_percent: 52,
      file_system: "NTFS",
    },
    {
      drive_letter: "E:\\",
      label: "备份",
      drive_type: "本地磁盘",
      total_gb: 1863.0,
      used_gb: 1456.8,
      free_gb: 406.2,
      usage_percent: 78,
      file_system: "NTFS",
    },
  ];
}

function getDemoDirectory(path) {
  const isCRoot = path.toUpperCase().startsWith("C:");
  const demoEntries = isCRoot
    ? [
        { name: "Users", path: "C:\\Users", is_dir: true, size_bytes: 68 * 1024 * 1024 * 1024, size_display: "68.0 GB", modified: "2026-07-17 18:45", extension: null, file_count: 154231 },
        { name: "Windows", path: "C:\\Windows", is_dir: true, size_bytes: 32 * 1024 * 1024 * 1024, size_display: "32.0 GB", modified: "2026-07-15 10:23", extension: null, file_count: 89432 },
        { name: "Program Files", path: "C:\\Program Files", is_dir: true, size_bytes: 24 * 1024 * 1024 * 1024, size_display: "24.0 GB", modified: "2026-06-28 14:12", extension: null, file_count: 45123 },
        { name: "Program Files (x86)", path: "C:\\Program Files (x86)", is_dir: true, size_bytes: 18 * 1024 * 1024 * 1024, size_display: "18.0 GB", modified: "2026-06-28 14:12", extension: null, file_count: 38247 },
        { name: "ProgramData", path: "C:\\ProgramData", is_dir: true, size_bytes: 12 * 1024 * 1024 * 1024, size_display: "12.0 GB", modified: "2026-07-16 09:30", extension: null, file_count: 22834 },
        { name: "pagefile.sys", path: "C:\\pagefile.sys", is_dir: false, size_bytes: 8 * 1024 * 1024 * 1024, size_display: "8.0 GB", modified: "2026-07-18 08:00", extension: "sys", file_count: 0 },
        { name: "hiberfil.sys", path: "C:\\hiberfil.sys", is_dir: false, size_bytes: 6 * 1024 * 1024 * 1024, size_display: "6.0 GB", modified: "2026-07-17 22:00", extension: "sys", file_count: 0 },
        { name: "Temp", path: "C:\\Temp", is_dir: true, size_bytes: 1.2 * 1024 * 1024 * 1024, size_display: "1.2 GB", modified: "2026-07-18 07:45", extension: null, file_count: 342 },
      ]
    : [
        { name: "Videos", path: "D:\\Videos", is_dir: true, size_bytes: 156 * 1024 * 1024 * 1024, size_display: "156.0 GB", modified: "2026-07-10 20:15", extension: null, file_count: 1284 },
        { name: "Games", path: "D:\\Games", is_dir: true, size_bytes: 142 * 1024 * 1024 * 1024, size_display: "142.0 GB", modified: "2026-07-14 16:30", extension: null, file_count: 8234 },
        { name: "Projects", path: "D:\\Projects", is_dir: true, size_bytes: 48 * 1024 * 1024 * 1024, size_display: "48.0 GB", modified: "2026-07-17 11:22", extension: null, file_count: 23456 },
        { name: "Photos", path: "D:\\Photos", is_dir: true, size_bytes: 32 * 1024 * 1024 * 1024, size_display: "32.0 GB", modified: "2026-07-08 14:00", extension: null, file_count: 8421 },
        { name: "Downloads", path: "D:\\Downloads", is_dir: true, size_bytes: 18 * 1024 * 1024 * 1024, size_display: "18.0 GB", modified: "2026-07-18 09:15", extension: null, file_count: 542 },
        { name: "backup.zip", path: "D:\\backup.zip", is_dir: false, size_bytes: 4.5 * 1024 * 1024 * 1024, size_display: "4.5 GB", modified: "2026-07-05 23:45", extension: "zip", file_count: 0 },
        { name: "music_library.flac", path: "D:\\music_library.flac", is_dir: false, size_bytes: 856 * 1024 * 1024, size_display: "856 MB", modified: "2026-06-30 19:20", extension: "flac", file_count: 0 },
      ];

  const totalSize = demoEntries.reduce((sum, e) => sum + e.size_bytes, 0);
  return {
    current_path: path,
    parent_path: null,
    entries: demoEntries,
    total_size_bytes: totalSize,
    total_size_display: formatDemoSize(totalSize),
    entry_count: demoEntries.length,
    is_root: path.length <= 3,
    scan_time_ms: 1200,
    has_partial: false,
  };
}

function formatDemoSize(bytes) {
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return gb.toFixed(1) + " GB";
  const mb = bytes / (1024 * 1024);
  return mb.toFixed(0) + " MB";
}

function enterDrive(drive) {
  // Windows: "C:" -> "C:\"
  const path = drive.drive_letter.endsWith('\\') ? drive.drive_letter : drive.drive_letter + '\\';
  loadDirectory(path);
}

function enterDirectory(entry) {
  loadDirectory(entry.path);
}

function goBack() {
  if (currentResult.value && currentResult.value.parent_path) {
    loadDirectory(currentResult.value.parent_path);
  } else {
    goToDrives();
  }
}

function goToDrives() {
  loadDrives();
}

function navigateToPath(path) {
  loadDirectory(path);
}

// 复制路径到剪贴板
async function copyPath(path) {
  try {
    // 使用 Clipboard API（在 Tauri WebView2 和现代浏览器中均可用）
    if (navigator.clipboard && navigator.clipboard.writeText) {
      await navigator.clipboard.writeText(path);
    } else {
      // 旧版环境降级方案
      const textarea = document.createElement("textarea");
      textarea.value = path;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
    }
    // 标记为已复制，1.5 秒后恢复
    copiedPaths.value[path] = true;
    setTimeout(() => {
      delete copiedPaths.value[path];
    }, 1500);
  } catch (e) {
    console.error("复制失败:", e);
    alert("复制失败: " + String(e));
  }
}

function getUsageClass(percent) {
  if (percent > 85) return "high";
  if (percent > 60) return "medium";
  if (percent > 30) return "normal";
  return "low";
}

function getSizeClass(bytes) {
  if (bytes >= 1024 * 1024 * 1024) return "text-danger"; // > 1GB
  if (bytes >= 100 * 1024 * 1024) return "text-warning"; // > 100MB
  return "";
}

function getFileIcon(ext) {
  if (!ext) return "list";
  const e = ext.toLowerCase();
  if (['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv'].includes(e)) return "play";
  if (['mp3', 'wav', 'flac', 'aac', 'ogg'].includes(e)) return "play";
  if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg'].includes(e)) return "eye";
  if (['zip', 'rar', '7z', 'tar', 'gz'].includes(e)) return "folder";
  if (['exe', 'msi', 'bat', 'cmd', 'ps1'].includes(e)) return "terminal";
  if (['dll', 'sys', 'drv'].includes(e)) return "settings";
  if (['txt', 'log', 'md', 'json', 'xml', 'csv'].includes(e)) return "list";
  if (['doc', 'docx', 'pdf'].includes(e)) return "list";
  if (['xls', 'xlsx'].includes(e)) return "chart";
  if (['iso', 'img', 'vhd'].includes(e)) return "disc";
  if (['ttf', 'otf', 'woff'].includes(e)) return "list";
  return "list";
}
</script>

<style scoped>
.page {
  max-width: 1600px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  margin-bottom: 16px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 12px;
}

/* 面包屑 */
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 8px 12px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  margin-bottom: 14px;
  overflow-x: auto;
  white-space: nowrap;
}

.crumb-item {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 7px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 11.5px;
  font-weight: 500;
}

.crumb-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.crumb-item.active {
  color: var(--accent);
}

.crumb-sep {
  color: var(--text-faint);
  flex-shrink: 0;
}

/* 磁盘列表 */
.drives-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 12px;
}

.drive-card {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px 16px;
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
}

.drive-card:hover {
  border-color: var(--border-light);
  background: var(--bg-hover);
}

.drive-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  color: var(--text-secondary);
}

.drive-letter {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.drive-label {
  font-size: 11.5px;
  color: var(--text-muted);
}

.drive-bar {
  height: 6px;
  background: var(--bg-input);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 10px;
}

.drive-meta {
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
  margin-bottom: 2px;
}

.drive-sub {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10.5px;
  color: var(--text-muted);
}

.drive-fs {
  position: absolute;
  right: 14px;
  top: 14px;
  color: var(--text-faint);
}

.drive-card:hover .drive-fs {
  color: var(--accent);
}

/* 目录列表 */
.list-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 5px 10px;
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  font-size: 11.5px;
  font-weight: 500;
  align-self: flex-start;
}

.back-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--border-light);
}

.back-icon {
  transform: rotate(180deg);
}

.dir-info {
  padding: 10px 14px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.dir-stats {
  display: flex;
  align-items: center;
  gap: 14px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 6px;
}

.stat-label {
  font-size: 10.5px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.stat-value {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 600;
}

.stat-divider {
  width: 1px;
  height: 14px;
  background: var(--border);
}

/* 条目表格 */
.entries-table {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}

.entries-head {
  display: grid;
  grid-template-columns: 1fr 90px 70px 90px 80px 110px;
  gap: 10px;
  padding: 8px 14px;
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border);
  font-size: 10.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-muted);
}

.entry-row {
  display: grid;
  grid-template-columns: 1fr 90px 70px 90px 80px 110px;
  gap: 10px;
  padding: 8px 14px;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
  align-items: center;
}

.entry-row:last-child {
  border-bottom: none;
}

.entry-row.entry-dir {
  cursor: pointer;
}

.entry-row.entry-dir:hover {
  background: var(--bg-hover);
}

.entry-row.entry-dir:hover .entry-icon {
  color: var(--accent);
}

.entry-row.entry-file {
  cursor: default;
}

.col-name {
  display: flex;
  align-items: center;
  gap: 7px;
  overflow: hidden;
}

.entry-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.entry-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-primary);
}

.entry-dir .entry-name {
  font-weight: 500;
}

.col-size {
  text-align: right;
  color: var(--text-secondary);
  font-size: 11.5px;
}

.col-count {
  text-align: right;
  color: var(--text-muted);
  font-size: 11.5px;
}

.col-modified {
  color: var(--text-muted);
  font-size: 11px;
}

.col-type {
  text-align: right;
}

/* 操作列 + 复制按钮 */
.col-action {
  text-align: right;
}

.copy-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 7px;
  border-radius: var(--radius-sm);
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-muted);
  font-size: 10.5px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.copy-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--border-light);
}

.copy-btn.copied {
  background: var(--accent-dim);
  color: var(--accent);
  border-color: var(--accent);
}

.copy-label {
  line-height: 1;
}

/* 磁盘卡片复制按钮 */
.drive-copy-btn {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-sm);
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-faint);
  cursor: pointer;
  transition: all 0.15s ease;
}

.drive-copy-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-color: var(--border);
}

.drive-copy-btn.copied {
  background: var(--accent-dim);
  color: var(--accent);
  border-color: var(--accent);
}

/* 面包屑复制按钮 */
.crumb-copy-btn {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-sm);
  background: transparent;
  border: 1px solid transparent;
  color: var(--text-faint);
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.crumb-copy-btn:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
  border-color: var(--border);
}

.crumb-copy-btn.copied {
  background: var(--accent-dim);
  color: var(--accent);
  border-color: var(--accent);
}

/* 部分估算标签 */
.partial-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  background: var(--warning-dim);
  border: 1px solid var(--warning);
  color: var(--warning);
  font-size: 10px;
  font-weight: 500;
  cursor: help;
}

/* 计算中动画 */
.calculating {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  color: var(--text-muted);
  font-style: italic;
}

.calc-dot {
  display: inline-block;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--accent);
  animation: calc-pulse 1s ease-in-out infinite;
}

@keyframes calc-pulse {
  0%, 100% { opacity: 0.3; transform: scale(0.8); }
  50% { opacity: 1; transform: scale(1.2); }
}

.empty-state {
  padding: 40px 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 12px;
}
</style>
