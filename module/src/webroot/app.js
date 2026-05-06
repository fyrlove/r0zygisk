let moduleDir = "/data/adb/modules/r0z";

const locateModuleShell = "MODDIR=\"\"; for base in /data/adb/modules /data/adb/modules_update /data/adb/ksu/modules /data/adb/ap/modules; do [ -d \"$base\" ] || continue; for prop in \"$base\"/*/module.prop; do [ -f \"$prop\" ] || continue; if grep -q '^id=r0z$' \"$prop\" 2>/dev/null || grep -q '^name=r0z$' \"$prop\" 2>/dev/null; then MODDIR=${prop%/module.prop}; break 2; fi; done; done; [ -n \"$MODDIR\" ] || MODDIR=/data/adb/modules/r0z";
let callbackCounter = 0;

const els = {
  statusText: document.getElementById("statusText"),
  message: document.getElementById("message"),
  refresh: document.getElementById("refresh"),
  copyLog: document.getElementById("copyLog"),
  healthScore: document.getElementById("healthScore"),
  monitorTitle: document.getElementById("monitorTitle"),
  monitorDesc: document.getElementById("monitorDesc"),
  daemonDot: document.getElementById("daemonDot"),
  daemonState: document.getElementById("daemonState"),
  daemonDetail: document.getElementById("daemonDetail"),
  zygoteDot: document.getElementById("zygoteDot"),
  zygoteState: document.getElementById("zygoteState"),
  zygoteDetail: document.getElementById("zygoteDetail"),
  rootState: document.getElementById("rootState"),
  rootDetail: document.getElementById("rootDetail"),
  moduleCount: document.getElementById("moduleCount"),
  moduleList: document.getElementById("moduleList"),
  hideCount: document.getElementById("hideCount"),
  hideSearch: document.getElementById("hideSearch"),
  hideList: document.getElementById("hideList"),
};

const hideState = {
  apps: [],
  hidden: new Set(),
  busyPackage: "",
};

function getBridge() {
  const names = ["ksu", "KernelSU", "kernelsu", "apatch", "APatch", "Android"];
  for (const name of names) {
    const bridge = window[name];
    if (bridge && typeof bridge.exec === "function") {
      return bridge;
    }
  }
  return null;
}

function normalizeResult(result) {
  if (result == null) {
    return "";
  }
  if (typeof result === "string") {
    const text = result.trim();
    if (text.startsWith("{") && text.endsWith("}")) {
      try {
        return normalizeResult(JSON.parse(text));
      } catch (_) {
        return result;
      }
    }
    return result;
  }
  if (typeof result === "object") {
    const parts = [];
    if ("errno" in result) {
      parts.push(`errno=${result.errno}`);
    }
    if (result.stdout) {
      parts.push(String(result.stdout).trim());
    }
    if (result.stderr) {
      parts.push(String(result.stderr).trim());
    }
    return parts.filter(Boolean).join("\n");
  }
  return String(result);
}

function execCompat(bridge, command) {
  return new Promise((resolve, reject) => {
    let settled = false;
    let timer;
    const callbackName = `r0z_exec_${Date.now()}_${callbackCounter++}`;
    const finish = (value) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      delete window[callbackName];
      resolve(value);
    };
    const fail = (error) => {
      if (settled) {
        return;
      }
      settled = true;
      clearTimeout(timer);
      delete window[callbackName];
      reject(error);
    };
    window[callbackName] = (errno, stdout, stderr) => {
      finish({ errno, stdout, stderr });
    };
    timer = setTimeout(() => {
      fail(new Error("exec 超时，管理器没有返回命令结果"));
    }, 15000);

    const handleReturn = (result) => {
      if (result && typeof result.then === "function") {
        result.then(finish, fail);
      } else if (result !== undefined) {
        finish(result);
      }
    };

    try {
      handleReturn(bridge.exec(command, "{}", callbackName));
      return;
    } catch (error3) {
      try {
        handleReturn(bridge.exec(command, callbackName));
        return;
      } catch (error2) {
        try {
          handleReturn(bridge.exec(command));
          return;
        } catch (error1) {
          fail(error1 || error2 || error3);
        }
      }
    }
  });
}

async function run(command) {
  const bridge = getBridge();
  if (!bridge) {
    throw new Error("当前管理器没有暴露 WebUI exec 接口");
  }

  return normalizeResult(await execCompat(bridge, command));
}

function parseKeyValue(text) {
  return text.split("\n").reduce((acc, line) => {
    const index = line.indexOf("=");
    if (index > 0) {
      acc[line.slice(0, index)] = line.slice(index + 1);
    }
    return acc;
  }, {});
}

function parseStatus(description) {
  const match = description.match(/\[monitor:([^\]]+)\]/);
  const raw = match ? match[1] : description.trim();
  const parts = raw.split(",").map((item) => item.trim()).filter(Boolean);
  const status = {
    raw,
    monitor: parts[0] || "unknown",
    zygotes: [],
    daemons: [],
    root: "unknown",
  };

  parts.slice(1).forEach((part) => {
    if (part.startsWith("zygote")) {
      status.zygotes.push(part);
    } else if (part.startsWith("daemon")) {
      status.daemons.push(part);
      const rootMatch = part.match(/Root: ([A-Za-z]+)/);
      if (rootMatch) {
        status.root = rootMatch[1];
      }
    }
  });
  return status;
}

function readStatus(statusText, prop) {
  const text = statusText.trim();
  let json = null;
  let raw = prop.description || "";

  if (text) {
    try {
      json = JSON.parse(text);
      if (json && json.raw) {
        raw = String(json.raw);
      }
    } catch (_) {
      raw = text;
    }
  }

  const parsed = parseStatus(raw);
  if (json) {
    const daemonInfo = [json.daemon64_info, json.daemon32_info].filter(Boolean).join(" / ");
    const rootMatch = daemonInfo.match(/Root: ([A-Za-z]+)/);
    if (rootMatch) {
      parsed.root = rootMatch[1];
    }
  }
  parsed.json = json;
  return parsed;
}

function parseModules(listText) {
  return listText.split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const parts = line.split("|");
      return {
        id: parts[0] || "unknown",
        name: parts[1] || parts[0] || "unknown",
        version: parts[2] || "",
        disabled: parts[3] === "disabled",
      };
    });
}

function parseSimpleLines(text) {
  return text.split("\n").map((line) => line.trim()).filter(Boolean);
}

function escapeHtml(text) {
  return String(text)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;")
    .replaceAll("'", "&#39;");
}

function shellQuote(text) {
  return `'${String(text).replaceAll("'", `'\\''`)}'`;
}

function setDot(dot, ok) {
  dot.classList.toggle("ok", ok === true);
  dot.classList.toggle("bad", ok === false);
}

function setBusy(isBusy) {
  els.refresh.disabled = isBusy;
  if (els.hideSearch) {
    els.hideSearch.disabled = isBusy;
  }
}

function renderModules(modules) {
  els.moduleCount.textContent = `${modules.length} 个`;
  if (!modules.length) {
    els.moduleList.className = "list empty";
    els.moduleList.textContent = "没有发现已安装的 r0z 模块，或当前管理器未授予读取权限。";
    return;
  }

  els.moduleList.className = "list";
  els.moduleList.innerHTML = modules.map((mod) => `
    <article class="module-item">
      <div>
        <strong>${mod.name}</strong>
        <span>${mod.id}${mod.version ? ` / ${mod.version}` : ""}</span>
      </div>
      <span class="pill">${mod.disabled ? "已禁用" : "启用中"}</span>
    </article>
  `).join("");
}

function filteredHideApps() {
  const keyword = (els.hideSearch?.value || "").trim().toLowerCase();
  if (!keyword) {
    return hideState.apps;
  }
  return hideState.apps.filter((pkg) => pkg.toLowerCase().includes(keyword));
}

function renderHideList() {
  const apps = filteredHideApps();
  els.hideCount.textContent = `${hideState.hidden.size} 条`;
  if (!apps.length) {
    els.hideList.className = "list empty";
    els.hideList.textContent = hideState.apps.length
      ? "没有匹配当前搜索条件的应用。"
      : "没有读取到第三方应用列表，或当前管理器未授予执行权限。";
    return;
  }

  els.hideList.className = "list";
  els.hideList.innerHTML = apps.map((pkg) => {
    const hidden = hideState.hidden.has(pkg);
    const busy = hideState.busyPackage === pkg;
    return `
      <article class="app-item">
        <div>
          <strong>${escapeHtml(pkg)}</strong>
          <span>${hidden ? "已加入隐藏列表，下次启动该应用时跳过模块注入。" : "未隐藏，应用会按当前策略正常进入 r0z 注入链路。"}</span>
        </div>
        <button class="button small ${hidden ? "toggle-on" : "toggle-off"}" type="button" data-hide-package="${escapeHtml(pkg)}" ${busy ? "disabled" : ""}>
          ${busy ? "处理中..." : hidden ? "取消隐藏" : "加入隐藏"}
        </button>
      </article>
    `;
  }).join("");
}

async function toggleHidePackage(pkg, hide) {
  hideState.busyPackage = pkg;
  renderHideList();
  const command = [
    locateModuleShell,
    `if [ -x "$MODDIR/bin/r0z-ctl" ]; then "$MODDIR/bin/r0z-ctl" hide-list ${hide ? "add" : "remove"} ${shellQuote(pkg)}; else echo "找不到 $MODDIR/bin/r0z-ctl"; exit 127; fi`,
  ].join("; ");
  try {
    await run(command);
    if (hide) {
      hideState.hidden.add(pkg);
      els.message.textContent = `${pkg} 已加入隐藏列表。重启目标应用后生效。`;
    } else {
      hideState.hidden.delete(pkg);
      els.message.textContent = `${pkg} 已从隐藏列表移除。重启目标应用后生效。`;
    }
  } catch (error) {
    els.message.textContent = `更新隐藏列表失败：${error.message || error}`;
  } finally {
    hideState.busyPackage = "";
    renderHideList();
  }
}

function renderDashboard(prop, statusText, processes, modules) {
  const status = readStatus(statusText, prop);
  const json = status.json || {};
  const daemonOk = json.daemon64 === "running" || json.daemon32 === "running" || status.daemons.some((item) => item.includes("running")) || /r0zd/.test(processes);
  const zygoteOk = json.zygote64 === "injected" || json.zygote32 === "injected" || status.zygotes.some((item) => /:injected\b/.test(item));
  const bridgeOk = Boolean(prop["native.bridge"] || /libzn_loader\.so/.test(statusText));
  const score = [bridgeOk, daemonOk, zygoteOk].filter(Boolean).length;

  els.healthScore.textContent = `${score}/3`;
  els.monitorTitle.textContent = zygoteOk ? "已注入" : daemonOk ? "等待 zygote 回写" : "等待 daemon 启动";
  els.monitorDesc.textContent = bridgeOk
    ? "native bridge 已配置，当前等待 daemon 与 zygote 完成状态回写。"
    : "还没有确认 native bridge 已生效，可能需要完整重启后再刷新。";

  setDot(els.daemonDot, daemonOk);
  els.daemonState.textContent = daemonOk ? "运行中" : "未运行";
  els.daemonDetail.textContent = status.daemons.join(" / ") || [json.daemon64 && `daemon64:${json.daemon64}`, json.daemon32 && `daemon32:${json.daemon32}`].filter(Boolean).join(" / ") || (daemonOk ? "进程表发现 r0zd" : "进程表未发现 r0zd");

  setDot(els.zygoteDot, zygoteOk);
  els.zygoteState.textContent = zygoteOk ? "已注入" : "未确认";
  els.zygoteDetail.textContent = status.zygotes.join(" / ") || [json.zygote64 && `zygote64:${json.zygote64}`, json.zygote32 && `zygote32:${json.zygote32}`].filter(Boolean).join(" / ") || "等待 zygote 状态回写";

  els.rootState.textContent = status.root === "unknown" ? "未识别" : status.root;
  els.rootDetail.textContent = prop.name ? `${prop.name} ${prop.version || ""}`.trim() : `无法读取 ${moduleDir}/module.prop`;

  renderModules(modules);
}

async function refreshStatus() {
  setBusy(true);
  els.statusText.textContent = "正在读取状态...";
  els.message.textContent = "正在刷新...";

  const command = [
    locateModuleShell,
    `echo '--- module.dir ---'`,
    `echo "$MODDIR"`,
    `echo '--- module.prop ---'`,
    `cat "$MODDIR/module.prop" 2>/dev/null || true`,
    `echo '--- bridge.prop ---'`,
    `getprop ro.dalvik.vm.native.bridge 2>/dev/null || true`,
    `echo '--- status.json ---'`,
    `cat "$MODDIR/status.json" 2>/dev/null || true`,
    `echo '--- processes ---'`,
    `ps -A 2>/dev/null | grep -E 'r0zd|app_process|zygote' | grep -v grep || true`,
    `echo '--- hide.apps ---'`,
    `if [ -x "$MODDIR/bin/r0z-ctl" ]; then "$MODDIR/bin/r0z-ctl" hide-list apps; fi`,
    `echo '--- hide.hidden ---'`,
    `if [ -x "$MODDIR/bin/r0z-ctl" ]; then "$MODDIR/bin/r0z-ctl" hide-list list; fi`,
    `echo '--- modules ---'`,
    "for d in /data/adb/modules/*; do if [ -d \"$d/zygisk\" ]; then id=$(basename \"$d\"); name=$(sed -n 's/^name=//p' \"$d/module.prop\" 2>/dev/null | head -n 1); ver=$(sed -n 's/^version=//p' \"$d/module.prop\" 2>/dev/null | head -n 1); state=enabled; [ -f \"$d/disable\" ] && state=disabled; [ -n \"$name\" ] || name=\"$id\"; echo \"$id|$name|$ver|$state\"; fi; done",
  ].join("; ");

  try {
    const output = await run(command);
    const detectedDir = (output.match(/--- module\.dir ---\n([\s\S]*?)\n--- module\.prop ---/) || [])[1];
    if (detectedDir && detectedDir.trim()) {
      moduleDir = detectedDir.trim();
    }
    const propText = (output.match(/--- module\.prop ---\n([\s\S]*?)\n--- bridge\.prop ---/) || [])[1] || "";
    const bridgeProp = (output.match(/--- bridge\.prop ---\n([\s\S]*?)\n--- status\.json ---/) || [])[1] || "";
    const statusText = (output.match(/--- status\.json ---\n([\s\S]*?)\n--- processes ---/) || [])[1] || "";
    const processes = (output.match(/--- processes ---\n([\s\S]*?)\n--- hide\.apps ---/) || [])[1] || "";
    const hideAppsText = (output.match(/--- hide\.apps ---\n([\s\S]*?)\n--- hide\.hidden ---/) || [])[1] || "";
    const hideHiddenText = (output.match(/--- hide\.hidden ---\n([\s\S]*?)\n--- modules ---/) || [])[1] || "";
    const modulesText = (output.match(/--- modules ---\n([\s\S]*)/) || [])[1] || "";
    const prop = parseKeyValue(propText);
    prop["native.bridge"] = bridgeProp.trim();
    const modules = parseModules(modulesText);
    hideState.apps = parseSimpleLines(hideAppsText);
    hideState.hidden = new Set(parseSimpleLines(hideHiddenText));

    els.statusText.textContent = output.trim() || "没有读取到状态输出";
    renderDashboard(prop, statusText, processes, modules);
    renderHideList();
    els.message.textContent = "状态已刷新。";
  } catch (error) {
    els.statusText.textContent = [
      "无法直接执行 WebUI 命令。",
      "",
      String(error.message || error),
    ].join("\n");
    els.healthScore.textContent = "--";
    els.monitorTitle.textContent = "无法连接执行接口";
    els.monitorDesc.textContent = "当前管理器没有提供可用 exec 桥，页面只能显示静态说明。";
    setDot(els.daemonDot, null);
    setDot(els.zygoteDot, null);
    els.daemonState.textContent = "--";
    els.zygoteState.textContent = "--";
    els.rootState.textContent = "--";
    hideState.apps = [];
    hideState.hidden = new Set();
    renderModules([]);
    renderHideList();
    els.message.textContent = "页面已加载，但管理器没有提供可用的执行接口。";
  } finally {
    setBusy(false);
  }
}

async function copyLog() {
  try {
    await navigator.clipboard.writeText(els.statusText.textContent || "");
    els.message.textContent = "诊断信息已复制。";
  } catch (_) {
    els.message.textContent = "当前 WebView 不允许复制，请手动选择诊断信息。";
  }
}

els.refresh.addEventListener("click", refreshStatus);
els.copyLog.addEventListener("click", copyLog);
els.hideSearch.addEventListener("input", renderHideList);
els.hideList.addEventListener("click", (event) => {
  const button = event.target.closest("[data-hide-package]");
  if (!button) {
    return;
  }
  const pkg = button.dataset.hidePackage;
  if (!pkg || hideState.busyPackage) {
    return;
  }
  toggleHidePackage(pkg, !hideState.hidden.has(pkg));
});

refreshStatus();
