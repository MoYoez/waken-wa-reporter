"ui";

var DEFAULT_BASE_URL = "http://localhost:3000";
var BASE_URL = DEFAULT_BASE_URL;
var API_TOKEN = "";
var GENERATED_HASH_KEY = "";
var DEVICE_NAME = "";

var RESOLVED_DEVICE_TYPE = "mobile";
var RESOLVED_PUSH_MODE = "realtime";
var DEFAULT_POLL_MS = 2000;
var DEFAULT_HEARTBEAT_MS = 60000;
var POLL_MS = DEFAULT_POLL_MS;
var HEARTBEAT_MS = DEFAULT_HEARTBEAT_MS;
var METADATA = { source: "waken-wa" };

var ACTIVITY_PATH = "/api/activity";
var STORAGE_KEY = "waken_wa_activity_simple_v1";

var reportingRunning = false;
var reportingThread = null;

ui.layout(
  <vertical padding="12">
    <text text="Waken WA" textSize="20sp" textStyle="bold" />
    <text
      text="与电脑端一致：可粘贴后台 Base64 后点「导入」，或直接填地址和 Token。首次保存或开始时会自动生成设备密钥。"
      textSize="12sp"
      textColor="#666666"
      marginBottom="8"
    />

    <vertical layout_height="0" layout_weight="1">
      <text text="Base64 配置（可选）" textSize="12sp" textColor="#888888" />
      <input id="inBase64" hint="后台复制的一整段 Base64" lines="2" gravity="top" />

      <horizontal marginTop="4">
        <button id="btnImportBase64" text="导入 Base64" layout_weight="1" />
      </horizontal>

      <text text="API 地址" textSize="12sp" textColor="#888888" marginTop="8" />
      <input id="inBaseUrl" hint="例如 http://主机:3000" lines="1" />

      <text text="API Token" textSize="12sp" textColor="#888888" marginTop="6" />
      <input id="inApiToken" hint="后台 / 与电脑端相同" lines="1" inputType="textPassword" />

      <text text="设备名称（可选）" textSize="12sp" textColor="#888888" marginTop="6" />
      <input id="inDeviceName" hint="在后台展示的名称" lines="1" />

      <text text="轮询间隔（毫秒）" textSize="12sp" textColor="#888888" marginTop="6" />
      <input id="inPollMs" hint="检测前台频率，默认 2000" inputType="number" lines="1" />

      <text text="心跳间隔（毫秒）" textSize="12sp" textColor="#888888" marginTop="6" />
      <input id="inHeartbeatMs" hint="前台不变时最长多久再上报，0=关闭，默认 60000" inputType="number" lines="1" />
    </vertical>

    <horizontal marginTop="8">
      <button id="btnSave" text="保存" layout_weight="1" />
      <button id="btnStart" text="开始" layout_weight="1" />
      <button id="btnStop" text="停止" layout_weight="1" />
    </horizontal>
    <text id="txtStatus" text="空闲" textSize="12sp" marginTop="6" textColor="#333333" />
  </vertical>
);

function trim(s) {
  return String(s == null ? "" : s).trim();
}

function inputGet(id) {
  var w = ui[id];
  if (!w) {
    return "";
  }
  if (w.getText) {
    return String(w.getText()).trim();
  }
  return String(w.text()).trim();
}

function inputSet(id, s) {
  var w = ui[id];
  if (!w) {
    return;
  }
  if (w.setText) {
    w.setText(s == null ? "" : String(s));
  } else {
    w.text(s == null ? "" : String(s));
  }
}

function setStatus(msg) {
  ui.run(function () {
    ui.txtStatus.setText(String(msg));
  });
}

function loadCfgObj() {
  try {
    var s = storages.create("waken_wa").get(STORAGE_KEY, "");
    if (!s) {
      return null;
    }
    return JSON.parse(String(s));
  } catch (e) {
    return null;
  }
}

function saveCfgObj(o) {
  storages.create("waken_wa").put(STORAGE_KEY, JSON.stringify(o));
}

/** Decode Base64 to UTF-8 string (standard backend export). */
function decodeBase64ToUtf8(encoded) {
  var B64 = android.util.Base64;
  var bytes = B64.decode(trim(encoded), B64.DEFAULT);
  return String(new java.lang.String(bytes, "UTF-8"));
}

/** Match config.baseUrlFromEndpoint / TrimSuffix(endpoint, "/api/activity"). */
function baseUrlFromEndpoint(endpoint) {
  var e = trim(endpoint);
  if (!e) {
    return DEFAULT_BASE_URL;
  }
  e = e.replace(/\/+$/, "");
  var suff = "/api/activity";
  if (e.length >= suff.length && e.substring(e.length - suff.length) === suff) {
    e = e.substring(0, e.length - suff.length);
  }
  e = e.replace(/\/+$/, "");
  return e || DEFAULT_BASE_URL;
}

/** Same as config.FromBase64 in resolve.go — apiKey or token.items[0].token; endpoint or token.reportEndpoint. */
function fromBase64Config(encoded) {
  var jsonStr = decodeBase64ToUtf8(encoded);
  var c;
  try {
    c = JSON.parse(jsonStr);
  } catch (e) {
    throw new Error("Base64 解码后不是合法 JSON：" + (e.message || e));
  }
  var token = trim(c.apiKey);
  if (!token && c.token && Object.prototype.toString.call(c.token.items) === "[object Array]" && c.token.items.length > 0) {
    var it = c.token.items[0];
    if (it && it.token != null) {
      token = trim(it.token);
    }
  }
  if (!token) {
    throw new Error("Base64 中缺少 apiKey 或 token.items[0].token");
  }
  var endpoint = trim(c.endpoint);
  if (!endpoint && c.token) {
    endpoint = trim(c.token.reportEndpoint);
  }
  var baseURL = baseUrlFromEndpoint(endpoint);
  return { base_url: baseURL, api_token: token };
}

/** If API token field is empty, parse Base64 box and fill URL + token (Save / Start). */
function tryApplyBase64WhenTokenEmpty() {
  var b64 = trim(inputGet("inBase64"));
  if (!b64) {
    return;
  }
  if (trim(inputGet("inApiToken"))) {
    return;
  }
  var p = fromBase64Config(b64);
  inputSet("inBaseUrl", p.base_url);
  inputSet("inApiToken", p.api_token);
}

/** 32-char hex, same length as Go generateRandomHashKey (16 random bytes). */
function generateHashKey() {
  var r = new java.security.SecureRandom();
  var buf = java.lang.reflect.Array.newInstance(java.lang.Byte.TYPE, 16);
  r.nextBytes(buf);
  var out = [];
  for (var i = 0; i < 16; i++) {
    var v = (buf[i] + 256) % 256;
    var h = java.lang.Integer.toHexString(v);
    if (h.length < 2) {
      h = "0" + h;
    }
    out.push(String(h));
  }
  return out.join("");
}

function cfgToUi(c) {
  if (!c || typeof c !== "object") {
    return;
  }
  if (c.base_url != null) {
    inputSet("inBaseUrl", c.base_url);
  }
  if (c.api_token != null) {
    inputSet("inApiToken", c.api_token);
  }
  if (c.device_name != null) {
    inputSet("inDeviceName", c.device_name);
  }
  if (c.poll_ms != null) {
    inputSet("inPollMs", String(c.poll_ms));
  }
  if (c.heartbeat_ms != null) {
    inputSet("inHeartbeatMs", String(c.heartbeat_ms));
  }
}

function parsePositiveIntMs(label, raw) {
  var n = Number(raw);
  if (!isFinite(n) || Math.floor(n) !== n || n < 1) {
    throw new Error(label + "须为 ≥1 的整数（毫秒）");
  }
  return n;
}

function parseHeartbeatMsField(label, raw) {
  var n = Number(raw);
  if (!isFinite(n) || Math.floor(n) !== n || n < 0) {
    throw new Error(label + "须为 ≥0 的整数，0 表示关闭心跳");
  }
  return n;
}

/** Read poll / heartbeat from UI, then disk, then defaults. Updates globals. */
function applyIntervalSettingsFromUiAndDisk(disk) {
  disk = disk || {};
  var pollStr = trim(inputGet("inPollMs"));
  if (pollStr === "") {
    pollStr =
      disk.poll_ms != null && String(disk.poll_ms).trim() !== "" ? String(disk.poll_ms) : String(DEFAULT_POLL_MS);
  }
  POLL_MS = parsePositiveIntMs("轮询间隔", pollStr);

  var hbStr = trim(inputGet("inHeartbeatMs"));
  if (hbStr === "") {
    hbStr =
      disk.heartbeat_ms != null && String(disk.heartbeat_ms).trim() !== ""
        ? String(disk.heartbeat_ms)
        : String(DEFAULT_HEARTBEAT_MS);
  }
  HEARTBEAT_MS = parseHeartbeatMsField("心跳间隔", hbStr);
}

function readUiFields() {
  return {
    base_url: trim(inputGet("inBaseUrl")) || DEFAULT_BASE_URL,
    api_token: trim(inputGet("inApiToken")),
    device_name: trim(inputGet("inDeviceName")),
  };
}

/** Merge UI + disk; ensure generated_hash_key like config.ResolveGeneratedHashKey. */
function persistFromUi(requireToken) {
  tryApplyBase64WhenTokenEmpty();
  var disk = loadCfgObj() || {};
  var u = readUiFields();
  var token = u.api_token || trim(disk.api_token);
  if (requireToken && !token) {
    throw new Error("请填写 API Token");
  }
  var h = trim(disk.generated_hash_key);
  if (!h) {
    h = generateHashKey();
  }
  applyIntervalSettingsFromUiAndDisk(disk);
  var row = {
    base_url: u.base_url || trim(disk.base_url) || DEFAULT_BASE_URL,
    api_token: token,
    device_name: u.device_name,
    generated_hash_key: h,
    poll_ms: POLL_MS,
    heartbeat_ms: HEARTBEAT_MS,
  };
  saveCfgObj(row);
  return row;
}

function applyConfigFromSources() {
  tryApplyBase64WhenTokenEmpty();
  var disk = loadCfgObj() || {};
  var u = readUiFields();
  BASE_URL = u.base_url || trim(disk.base_url) || DEFAULT_BASE_URL;
  API_TOKEN = u.api_token || trim(disk.api_token);
  DEVICE_NAME = u.device_name || trim(disk.device_name);
  applyIntervalSettingsFromUiAndDisk(disk);
  var h = trim(disk.generated_hash_key);
  if (!h) {
    h = generateHashKey();
    saveCfgObj({
      base_url: BASE_URL,
      api_token: API_TOKEN,
      device_name: DEVICE_NAME,
      generated_hash_key: h,
      poll_ms: POLL_MS,
      heartbeat_ms: HEARTBEAT_MS,
    });
  }
  GENERATED_HASH_KEY = h;
}

function validateForRun() {
  if (!API_TOKEN) {
    throw new Error("请填写 API Token，或先导入 Base64");
  }
  if (!GENERATED_HASH_KEY) {
    throw new Error("缺少设备密钥，请先保存配置");
  }
}

function isArrayLike(input) {
  return Object.prototype.toString.call(input) === "[object Array]";
}

function isPlainObject(input) {
  return input !== null && typeof input === "object" && Object.prototype.toString.call(input) === "[object Object]";
}

function stripUndefinedDeep(input) {
  if (input === undefined) {
    return undefined;
  }
  if (input === null || typeof input !== "object") {
    return input;
  }
  if (isArrayLike(input)) {
    var arr = [];
    for (var i = 0; i < input.length; i++) {
      if (input[i] === undefined) {
        continue;
      }
      var ev = stripUndefinedDeep(input[i]);
      if (ev === undefined) {
        continue;
      }
      arr.push(ev);
    }
    return arr;
  }
  var out = {};
  for (var k in input) {
    if (!Object.prototype.hasOwnProperty.call(input, k)) {
      continue;
    }
    var v = input[k];
    if (v === undefined) {
      continue;
    }
    var nv = stripUndefinedDeep(v);
    if (nv === undefined) {
      continue;
    }
    out[k] = nv;
  }
  return out;
}

function tryDeviceBatteryPercent() {
  try {
    if (typeof device.getBattery === "function") {
      var n = Number(device.getBattery());
      if (isFinite(n) && Math.floor(n) === n && n >= 0 && n <= 100) {
        return n;
      }
    }
  } catch (e) {}
  return null;
}

function buildUrl(base) {
  var b = String(base || "").replace(/\/+$/, "");
  return b + ACTIVITY_PATH;
}

/** Stable id when user did not set a display name (like desktop hostname). */
function getDefaultDeviceId() {
  try {
    return device.getAndroidId();
  } catch (e) {
    return "unknown-device";
  }
}

/**
 * Match main.go device / deviceName rules:
 * - If friendly name set: device and device_name both use it (same as config deviceName driving device id).
 * - If empty: device = default id, device_name = same (if deviceName == "" { deviceName = device }).
 */
function resolveDevicePayload() {
  var friendly = String(DEVICE_NAME || "").trim();
  if (friendly) {
    return { device: friendly, device_name: friendly };
  }
  var id = getDefaultDeviceId();
  return { device: id, device_name: id };
}

function getForegroundLabel() {
  var pkg = "";
  try {
    pkg = currentPackage() || "";
  } catch (e) {}
  var act = "";
  try {
    act = currentActivity() || "";
  } catch (e) {}
  return {
    process_name: pkg || "unknown",
    process_title: act || "",
  };
}

function postActivity(body) {
  var clean = stripUndefinedDeep(body);
  if (!isPlainObject(clean)) {
    throw new Error("上报数据无效");
  }
  var url = buildUrl(BASE_URL);
  var res = http.postJson(url, clean, {
    headers: {
      Authorization: "Bearer " + API_TOKEN,
      "Content-Type": "application/json",
    },
  });
  if (res.statusCode !== 200 && res.statusCode !== 201) {
    throw new Error("接口错误 " + res.statusCode + "：" + res.body.string());
  }
  var j = {};
  try {
    j = res.body.json();
  } catch (e) {
    throw new Error("响应不是合法 JSON");
  }
  if (!j.success) {
    throw new Error("服务端返回失败（success=false）");
  }
}

function buildBody(processName, processTitle) {
  if (!GENERATED_HASH_KEY || !String(GENERATED_HASH_KEY).trim()) {
    throw new Error("缺少设备哈希密钥");
  }
  if (!processName) {
    throw new Error("缺少前台应用标识");
  }
  var meta = stripUndefinedDeep(METADATA);
  if (!isPlainObject(meta)) {
    throw new Error("元数据格式错误");
  }
  var dev = resolveDevicePayload();
  var body = {
    generatedHashKey: String(GENERATED_HASH_KEY).trim(),
    device: dev.device,
    device_name: dev.device_name,
    device_type: RESOLVED_DEVICE_TYPE,
    process_name: String(processName).trim(),
    push_mode: RESOLVED_PUSH_MODE,
    metadata: meta,
  };
  var pt = processTitle == null ? "" : String(processTitle).trim();
  if (pt) {
    body.process_title = pt;
  }
  var batt = tryDeviceBatteryPercent();
  if (batt !== null) {
    body.battery_level = batt;
  }
  return body;
}

function stopReporting() {
  reportingRunning = false;
  setStatus("正在停止…");
  ui.run(function () {
    toast("已停止上报");
    setStatus("空闲");
  });
}

function startReporting() {
  if (reportingRunning) {
    toast("已在运行中");
    return;
  }
  try {
    applyConfigFromSources();
    validateForRun();
  } catch (e) {
    toast(String(e.message || e));
    setStatus("错误：" + String(e.message || e));
    return;
  }

  reportingRunning = true;
  var pollMsRun = POLL_MS;
  var heartbeatMsRun = HEARTBEAT_MS;
  setStatus("正在上报…（轮询 " + pollMsRun + "ms，心跳 " + heartbeatMsRun + "ms）");

  reportingThread = threads.start(function () {
    var lastName = "";
    var lastTitle = "";
    var lastReportAt = new Date().getTime();
    while (reportingRunning) {
      try {
        var fg = getForegroundLabel();
        var now = new Date().getTime();
        var changed = fg.process_name !== lastName || fg.process_title !== lastTitle;
        if (changed) {
          lastName = fg.process_name;
          lastTitle = fg.process_title;
          postActivity(buildBody(fg.process_name, fg.process_title));
          lastReportAt = now;
          log("已上报活动：" + fg.process_name);
        } else if (heartbeatMsRun > 0 && now - lastReportAt >= heartbeatMsRun) {
          postActivity(buildBody(fg.process_name, fg.process_title));
          lastReportAt = now;
          log("心跳上报（前台未变）：" + fg.process_name);
        }
      } catch (e) {
        log("上报失败：" + e);
      }
      sleep(pollMsRun);
    }
  });
}

ui.btnImportBase64.on("click", function () {
  try {
    var raw = trim(inputGet("inBase64"));
    if (!raw) {
      toast("请先粘贴 Base64");
      return;
    }
    var p = fromBase64Config(raw);
    inputSet("inBaseUrl", p.base_url);
    inputSet("inApiToken", p.api_token);
    toast("已填入 API 地址与 Token");
    setStatus("已从 Base64 导入");
  } catch (e) {
    toast("导入失败：" + (e.message || e));
    setStatus("Base64 导入失败");
  }
});

ui.btnSave.on("click", function () {
  try {
    persistFromUi(true);
    toast("已保存到本地");
    setStatus("已保存");
  } catch (e) {
    toast("保存失败：" + (e.message || e));
    setStatus("保存失败");
  }
});

ui.btnStart.on("click", function () {
  startReporting();
});

ui.btnStop.on("click", function () {
  stopReporting();
});

events.on("exit", function () {
  reportingRunning = false;
});

ui.post(function () {
  var loaded = loadCfgObj();
  if (loaded) {
    cfgToUi(loaded);
    if (loaded.poll_ms == null) {
      inputSet("inPollMs", String(DEFAULT_POLL_MS));
    }
    if (loaded.heartbeat_ms == null) {
      inputSet("inHeartbeatMs", String(DEFAULT_HEARTBEAT_MS));
    }
    setStatus("已加载本地配置");
  } else {
    inputSet("inBaseUrl", BASE_URL);
    inputSet("inPollMs", String(DEFAULT_POLL_MS));
    inputSet("inHeartbeatMs", String(DEFAULT_HEARTBEAT_MS));
    setStatus("空闲 — 填写地址与 Token，保存后开始");
  }
});
