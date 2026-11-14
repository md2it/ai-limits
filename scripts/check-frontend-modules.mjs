import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, extname, join, normalize, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import assert from "node:assert/strict";
import vm from "node:vm";

const projectRoot = resolve(import.meta.dirname, "..");
const modulesRoot = join(projectRoot, "frontend", "modules");

function collectJavaScriptFiles(directory) {
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry);
    return statSync(path).isDirectory() ? collectJavaScriptFiles(path) : extname(path) === ".js" ? [path] : [];
  });
}

const files = collectJavaScriptFiles(modulesRoot);
const importPattern = /(?:import|export)\s+(?:[\s\S]*?\s+from\s+)?["']([^"']+)["']/g;

for (const file of files) {
  const syntax = spawnSync(process.execPath, ["--check", file], { encoding: "utf8" });
  if (syntax.status !== 0) {
    process.stderr.write(syntax.stderr || syntax.stdout);
    process.exit(syntax.status ?? 1);
  }

  const source = readFileSync(file, "utf8");
  for (const match of source.matchAll(importPattern)) {
    const specifier = match[1];
    if (!specifier.startsWith(".")) continue;
    const resolved = normalize(resolve(dirname(file), specifier));
    const pathFromModules = relative(modulesRoot, resolved);
    if (pathFromModules.startsWith("..") || !existsSync(resolved)) {
      console.error(`Missing frontend module import: ${relative(projectRoot, file)} -> ${specifier}`);
      process.exit(1);
    }
  }
}

console.log(`Checked ${files.length} frontend ES modules.`);

// Exercise the real orchestration functions without a WebView or provider credentials.
// Main joins a suspended Cursor collection started by another surface, then receives its obsolete epoch.
const providersSource = readFileSync(join(modulesRoot, "providers.js"), "utf8");
const functionSource = (start, end) => providersSource.slice(providersSource.indexOf(start), providersSource.indexOf(end, providersSource.indexOf(start))).replace(/^export /, "");
const cursorFunctions = [
  functionSource("function acceptCursorEpoch(", "export function initProviders("),
  functionSource("export function initProviders(", "// The frontend remains"),
  functionSource("async function fetchSingleProviderLimits(", "// Reads the shared"),
  functionSource("async function refreshSingleProvider(", "export async function refreshEnabledProviders("),
].join("\n");
for (const secondEpoch of [2, 1]) {
  let click;
  let releaseOld;
  let joined;
  let requests = 0;
  let oldInFlight = true;
  let freshCollections = 0;
  const oldCollection = new Promise((resolve) => { releaseOld = resolve; });
  const joinObserved = new Promise((resolve) => { joined = resolve; });
  const cache = new Map();
  const rendered = [];
  const button = { dataset: { cursorSource: "ide" }, addEventListener: (_, handler) => { click = handler; } };
  const noop = () => {};
  const context = vm.createContext({
    cursorEpoch: 1, cursorSourceRefreshPending: false,
    providerRefreshInFlight: new Set(), providerRemoteRefreshInFlight: new Set(["cursor"]),
    providerDataCache: cache, document: { querySelectorAll: () => [button] },
    window: { __TAURI__: { core: { invoke: async (command) => {
      if (command === "select_cursor_credential_source") return 2;
      assert.equal(command, "get_single_provider_limits");
      requests += 1;
      assert.ok(requests <= 2, "Stale replies must not cause an unbounded refresh loop");
      if (oldInFlight) { joined(); return oldCollection; }
      freshCollections += 1;
      return { id: "cursor", cursorEpoch: secondEpoch, limits: ["new limits"], collectedAt: null };
    } }, event: { listen: noop } } },
    PROVIDER_UPDATED_EVENT: "updated", PROVIDER_REFRESH_STARTED_EVENT: "started", PROVIDER_REFRESH_FAILED_EVENT: "failed",
    initSectionSlotAlignment: noop, isScreenshotShowcase: false, settingsToQuery: () => ({}),
    getProviderBlock: () => ({}), getProviderNextRefreshAt: () => null, createEmptyProvider: () => ({ limits: [] }),
    updateProviderBlockData: (_, provider) => { rendered.push([...provider.limits]); },
    isProviderEnabled: () => true, updateRefreshVisual: noop, recordProviderUpdateNow: noop,
    cacheProviderData: (provider) => { cache.set(provider.id, provider); },
    recalculateProviderNextRefreshAt: noop, attachSectionHandlers: noop, scheduleSectionSlotAlignment: noop,
    setErrorState: (error) => { assert.fail(error); },
  });
  vm.runInContext(cursorFunctions, context);
  context.initProviders({ providerList: {}, statusLine: {} });
  assert.equal(context.providerRefreshInFlight.has("cursor"), false, "Old request belongs to another surface");
  const selecting = click();
  await joinObserved;
  assert.equal(requests, 1);
  oldInFlight = false; // The backend removes its coordinator entry before notifying joined callers.
  releaseOld({ id: "cursor", cursorEpoch: 1, limits: ["old account values"] });
  await selecting;
  assert.equal(requests, 2, "Settings selection must automatically request the new generation");
  assert.equal(freshCollections, 1);
  assert.ok(rendered.every((values) => !values.includes("old account values")));
  if (secondEpoch === 2) assert.deepEqual(cache.get("cursor").limits, ["new limits"]);
  else assert.equal(cache.has("cursor"), false, "A second stale reply must be discarded without another retry");
}
console.log("Checked Cursor cross-window joined-refresh recovery and bounded stale retry.");
