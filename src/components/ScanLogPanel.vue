<template>
  <Transition name="slide-panel">
    <div v-if="store.visible" ref="panelRef" class="scan-log-panel" @keydown.enter="onEnter">
      <!-- Panel Header -->
      <div class="panel-header">
        <div class="header-left">
          <span class="status-dot" :class="dotClass"></span>
          <span class="panel-title">{{ store.currentTaskTitle }}</span>
        </div>
        <div class="header-right">
          <span class="elapsed mono">{{ store.elapsed.toFixed(1) }}s</span>
          <span class="log-count mono">{{ store.logCount }} lines</span>
        </div>
      </div>

      <!-- Log Body -->
      <div ref="logBodyRef" class="log-body">
        <!-- Scanline overlay effect -->
        <div class="scanlines"></div>

        <div
          v-for="(log, idx) in store.logs"
          :key="idx"
          class="log-line"
          :class="'log-' + log.level"
        >
          <span class="log-ts mono">{{ log.ts }}</span>
          <span class="log-msg">{{ log.message }}</span>
        </div>

        <!-- Blinking cursor when running -->
        <div v-if="store.status === 'running'" class="log-line log-dim cursor-line">
          <span class="log-ts mono">&nbsp;</span>
          <span class="log-msg"><span class="cursor">&#9608;</span></span>
        </div>
      </div>

      <!-- Footer -->
      <div class="panel-footer" v-if="store.status === 'done'">
        <span class="enter-hint">
          <kbd>Enter</kbd> 关闭面板
        </span>
        <button class="close-btn" @click="onClose">点击此处关闭 ✕</button>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { computed, ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import { useScanLogStore } from "../stores/scanLog";

const store = useScanLogStore();
const logBodyRef = ref(null);
const panelRef = ref(null);

const dotClass = computed(() => ({
  "dot-run": store.status === "running",
  "dot-done": store.status === "done",
}));

// Auto-scroll to bottom on new logs
watch(
  () => store.logCount,
  async () => {
    await nextTick();
    if (logBodyRef.value) {
      logBodyRef.value.scrollTop = logBodyRef.value.scrollHeight;
    }
  }
);

// Also scroll when status changes to done
watch(
  () => store.status,
  async () => {
    await nextTick();
    if (logBodyRef.value) {
      logBodyRef.value.scrollTop = logBodyRef.value.scrollHeight;
    }
  }
);

function onEnter() {
  if (store.status === "done") {
    store.dismiss();
  }
}

// 鼠标左键点击「关闭」按钮
function onClose() {
  if (store.status === "done") {
    store.dismiss();
  }
}

// 鼠标左键点击面板外部区域也可关闭（仅加载完成时）
function onGlobalClick(e) {
  if (!store.visible || store.status !== "done") return;
  if (panelRef.value && !panelRef.value.contains(e.target)) {
    store.dismiss();
  }
}

onMounted(() => {
  document.addEventListener("mousedown", onGlobalClick);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", onGlobalClick);
});
</script>

<style scoped>
/* ===== Panel Container ===== */
.scan-log-panel {
  position: fixed;
  top: 36px; /* below titlebar */
  right: 0;
  width: 380px;
  height: calc(100vh - 36px);
  background: #05080a;
  border-left: 1px solid #0f1a12;
  display: flex;
  flex-direction: column;
  z-index: 999;
  box-shadow: -8px 0 32px rgba(0, 0, 0, 0.6), -2px 0 8px rgba(0, 255, 65, 0.04);
  font-family: "JetBrains Mono", "SF Mono", "Cascadia Code", "Consolas", monospace;
  font-size: 12px;
  line-height: 1.6;
  color: #00ff41;
  outline: none;
}

/* ===== Header ===== */
.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: #080c0e;
  border-bottom: 1px solid #0f1a12;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.dot-run {
  background: #00ff41;
  box-shadow: 0 0 6px #00ff41, 0 0 12px rgba(0, 255, 65, 0.3);
  animation: pulse-dot 1.2s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; box-shadow: 0 0 6px #00ff41, 0 0 12px rgba(0, 255, 65, 0.3); }
  50% { opacity: 0.5; box-shadow: 0 0 3px #00ff41, 0 0 6px rgba(0, 255, 65, 0.15); }
}

.dot-done {
  background: #00cc33;
  box-shadow: 0 0 4px rgba(0, 204, 51, 0.4);
}

.panel-title {
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #00ff41;
}

.elapsed {
  font-size: 11px;
  color: #00aa33;
}

.log-count {
  font-size: 10px;
  color: #006622;
}

/* ===== Log Body ===== */
.log-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
  position: relative;
  /* Custom scrollbar for terminal feel */
}

.log-body::-webkit-scrollbar {
  width: 4px;
}

.log-body::-webkit-scrollbar-track {
  background: transparent;
}

.log-body::-webkit-scrollbar-thumb {
  background: #0f2a18;
  border-radius: 2px;
}

/* CRT scanline effect */
.scanlines {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 255, 65, 0.01) 2px,
    rgba(0, 255, 65, 0.01) 4px
  );
  z-index: 1;
}

/* ===== Log Lines ===== */
.log-line {
  padding: 1px 14px;
  display: flex;
  gap: 8px;
  position: relative;
  z-index: 2;
  white-space: pre-wrap;
  word-break: break-all;
  animation: log-fadein 0.08s ease-out;
}

@keyframes log-fadein {
  from { opacity: 0; transform: translateX(-4px); }
  to { opacity: 1; transform: translateX(0); }
}

.log-ts {
  flex-shrink: 0;
  color: #005522;
  font-size: 11px;
  user-select: none;
}

.log-msg {
  color: #00dd38;
}

/* Level colors */
.log-info .log-msg { color: #00ee44; }
.log-success .log-msg { color: #00ff66; font-weight: 600; }
.log-warning .log-msg { color: #ccaa00; }
.log-error .log-msg { color: #ff3333; font-weight: 500; }
.log-system .log-msg { color: #00bbcc; font-weight: 600; }
.log-dim .log-msg { color: #116628; }

/* Cursor blink */
.cursor-line {
  animation: none !important;
}

.cursor {
  animation: cursor-blink 0.8s step-end infinite;
  color: #00ff41;
}

@keyframes cursor-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

/* ===== Footer ===== */
.panel-footer {
  padding: 8px 14px;
  border-top: 1px solid #0f1a12;
  background: #080c0e;
  flex-shrink: 0;
  text-align: center;
}

.enter-hint {
  font-size: 11px;
  color: #007730;
  letter-spacing: 0.06em;
}

.enter-hint kbd {
  display: inline-block;
  padding: 1px 6px;
  background: #0a1510;
  border: 1px solid #0f2a18;
  border-radius: 3px;
  font-family: inherit;
  font-size: 10px;
  color: #00aa33;
  margin-right: 4px;
}

.close-btn {
  display: block;
  width: 100%;
  margin-top: 7px;
  padding: 6px 0;
  background: #0a1510;
  border: 1px solid #0f2a18;
  border-radius: 3px;
  font-family: inherit;
  font-size: 11px;
  color: #00dd38;
  letter-spacing: 0.04em;
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}

.close-btn:hover {
  background: #0f2a18;
  color: #00ff66;
}

.close-btn:active {
  transform: scale(0.98);
}

/* ===== Slide Transition ===== */
.slide-panel-enter-active {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1),
              opacity 0.25s ease;
}

.slide-panel-leave-active {
  transition: transform 0.25s cubic-bezier(0.4, 0, 1, 1),
              opacity 0.2s ease;
}

.slide-panel-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.slide-panel-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
