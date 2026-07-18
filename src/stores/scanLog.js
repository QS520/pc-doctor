import { defineStore } from "pinia";
import { ref, computed } from "vue";

export const useScanLogStore = defineStore("scanLog", () => {
  const visible = ref(false);
  const logs = ref([]);
  const status = ref("idle"); // idle | running | done
  const currentTaskTitle = ref("");
  const currentSource = ref(""); // which page triggered the scan
  const startTime = ref(0);
  const endTime = ref(0);

  const elapsed = computed(() => {
    if (startTime.value === 0) return 0;
    const end = endTime.value > 0 ? endTime.value : Date.now();
    return Math.round((end - startTime.value) / 100) / 10;
  });

  const logCount = computed(() => logs.value.length);

  /** Open panel & start a new task session */
  function startTask(title, source = "") {
    logs.value = [];
    status.value = "running";
    currentTaskTitle.value = title;
    currentSource.value = source || title;
    startTime.value = Date.now();
    endTime.value = 0;
    visible.value = true;
    pushLog(`[INIT] ${title}`, "system");
  }

  /** Push a log line */
  function pushLog(message, level = "info") {
    // level: info | success | warning | error | system | dim
    const now = new Date();
    const ts =
      String(now.getHours()).padStart(2, "0") +
      ":" +
      String(now.getMinutes()).padStart(2, "0") +
      ":" +
      String(now.getSeconds()).padStart(2, "0");
    logs.value.push({ ts, message, level });
  }

  /** Mark task as completed */
  function complete(summary = "") {
    status.value = "done";
    endTime.value = Date.now();
    if (summary) {
      pushLog(`[DONE] ${summary}`, "success");
    } else {
      pushLog("[DONE] 操作完成", "success");
    }
    pushLog("--- 按 Enter 关闭面板 ---", "dim");
  }

  /** Mark task as failed */
  function fail(reason = "") {
    status.value = "done";
    endTime.value = Date.now();
    pushLog(`[FAIL] ${reason || "操作失败"}`, "error");
    pushLog("--- 按 Enter 关闭面板 ---", "dim");
  }

  /** Dismiss the panel (user pressed Enter or switched page) */
  function dismiss() {
    visible.value = false;
    // Don't clear logs immediately so we can animate out; clear on next startTask
  }

  /** Force close + reset (e.g. page switch) */
  function reset() {
    visible.value = false;
    logs.value = [];
    status.value = "idle";
    currentTaskTitle.value = "";
    currentSource.value = "";
    startTime.value = 0;
    endTime.value = 0;
  }

  return {
    visible,
    logs,
    status,
    currentTaskTitle,
    currentSource,
    startTime,
    endTime,
    elapsed,
    logCount,
    startTask,
    pushLog,
    complete,
    fail,
    dismiss,
    reset,
  };
});
