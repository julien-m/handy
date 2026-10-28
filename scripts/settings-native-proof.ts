/**
 * macOS native proof harness for `handy settings` (feature 008, Step 7). It drives real
 * binaries against an isolated HOME (never the user's store) and records every command, exit
 * code, stdout/stderr, latency sample, store hash and native observation as JSON:
 *
 * 1. older: a running pre-008 Handy (its singleton socket exists) refuses CLI reads and writes
 *    with `unavailable` (5) and leaves the store untouched, without activating that app;
 * 2. offline: no Handy owns the store (the passive legacy probe is pointed at an absent path
 *    through the debug-only test seam): timed reads, writes, no app registration, no model files;
 * 3. app: the proof app (a debug build with its own Tauri identifier, so its single-instance
 *    socket does not reach the user's Handy) is launched start-hidden with a tray; checks the
 *    activation policy (no Dock icon) and the absence of a settings read before the service;
 * 4. live: timed reads and writes answered by the running app over the private channel, a routed
 *    live effect (tray icon visibility), the app staying hidden (no show-window callback);
 * 5. restart: the app is stopped without its exit handler (SIGTERM) and started again: the stale
 *    endpoint name is reclaimed and the channel answers again.
 *
 * Usage:
 *   bun scripts/settings-native-proof.ts --binary <abs> --app <abs> --home <abs> --out <dir>
 *     [--samples 30] [--window-list <abs compiled scripts/macos-window-list.swift>]
 *     [--older-pid <pid of a running pre-008 Handy>]
 */
import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

interface Options {
  binary: string;
  app: string;
  home: string;
  out: string;
  samples: number;
  windowList: string | null;
  olderPid: string | null;
}
interface CliRun {
  args: string[];
  env: string;
  code: number | null;
  ms: number;
  pid: number | undefined;
  stdout: string;
  stderr: string;
}
interface Check {
  name: string;
  passed: boolean;
  detail: string;
}

const LEGACY_SEAM = "HANDY_SETTINGS_TEST_LEGACY_SOCKET";
// Read budget of AC-001 (on an M1 release build; this harness records what it measures).
const READ_BUDGET_MS = 2000;
// A starting app publishes its endpoint after startup; 30 s covers a cold debug launch.
const APP_READY_TIMEOUT_MS = 30000;
const checks: Check[] = [];
const runs: CliRun[] = [];

/** Parse `--name value` pairs; required paths must be absolute. */
const parseOptions = (argv: string[]): Options => {
  const value = (name: string): string | null => {
    const index = argv.indexOf(`--${name}`);
    return index >= 0 ? (argv[index + 1] ?? null) : null;
  };
  const required = (name: string): string => {
    const found = value(name);
    if (!found || !path.isAbsolute(found)) {
      throw new Error(`--${name} <absolute path> is required`);
    }
    return found;
  };
  return {
    binary: required("binary"),
    app: required("app"),
    home: required("home"),
    out: path.resolve(value("out") ?? "native-proof"),
    samples: Number(value("samples") ?? "30"),
    windowList: value("window-list"),
    olderPid: value("older-pid"),
  };
};

/** Run the CLI once with an isolated HOME; `seam` points the legacy probe at an absent path. */
const cli = (options: Options, args: string[], seam: boolean): CliRun => {
  const env: Record<string, string> = {
    ...process.env,
    HOME: options.home,
  } as Record<string, string>;
  delete env[LEGACY_SEAM];
  if (seam) env[LEGACY_SEAM] = path.join(options.home, "no-legacy.sock");
  const started = performance.now();
  const result = spawnSync(options.binary, args, { env, encoding: "utf8" });
  const run: CliRun = {
    args,
    env: seam ? `HOME, ${LEGACY_SEAM}` : "HOME",
    code: result.status,
    ms: Math.round((performance.now() - started) * 10) / 10,
    pid: result.pid,
    stdout: result.stdout,
    stderr: result.stderr,
  };
  runs.push(run);
  return run;
};

/** Parsed single-line JSON envelope of a run, or null. */
const envelope = (run: CliRun): Record<string, unknown> | null => {
  const lines = run.stdout.trim().split("\n");
  if (lines.length !== 1) return null;
  try {
    return JSON.parse(lines[0]) as Record<string, unknown>;
  } catch {
    // Not an envelope (human output or nothing): the caller's check fails on null.
    return null;
  }
};

const check = (name: string, passed: boolean, detail: string): void => {
  checks.push({ name, passed, detail });
  console.error(`${passed ? "PASS" : "FAIL"} ${name}: ${detail}`);
};

const storePath = (options: Options): string =>
  path.join(
    options.home,
    "Library/Application Support/com.pais.handy/settings_store.json",
  );

/** sha256 of the store, or "absent". */
const storeHash = (options: Options): string => {
  const store = storePath(options);
  if (!fs.existsSync(store)) return "absent";
  return createHash("sha256").update(fs.readFileSync(store)).digest("hex");
};

/** `ApplicationType` of a process as LaunchServices sees it ("Foreground", "UIElement"...). */
const applicationType = (pid: string | number): string => {
  const asn = spawnSync("lsappinfo", ["find", `pid=${pid}`], {
    encoding: "utf8",
  }).stdout.trim();
  if (!asn) return "not registered";
  const info = spawnSync(
    "lsappinfo",
    ["info", "-only", "ApplicationType", asn],
    {
      encoding: "utf8",
    },
  ).stdout;
  return info.split("=")[1]?.trim().replaceAll('"', "") ?? "unknown";
};

/** On-screen windows of `pid` (compiled window-list helper), or null without the helper. */
const windows = (options: Options, pid: number): string[] | null => {
  if (!options.windowList) return null;
  const listed = spawnSync(options.windowList, [String(pid)], {
    encoding: "utf8",
  });
  return listed.stdout.split("\n").filter((line) => line.trim().length > 0);
};

/** Timed reads; every sample is kept. */
const timedReads = (
  options: Options,
  seam: boolean,
  label: string,
): number[] => {
  const samples: number[] = [];
  for (let index = 0; index < options.samples; index += 1) {
    const run = cli(options, ["settings", "get", "--json"], seam);
    if (envelope(run)?.ok === true) samples.push(run.ms);
  }
  const max = Math.max(...samples);
  check(
    `${label}: ${options.samples} reads succeed`,
    samples.length === options.samples,
    `${samples.length}/${options.samples} ok`,
  );
  check(
    `${label}: every successful read <= ${READ_BUDGET_MS} ms (this machine, this build)`,
    samples.length > 0 && max <= READ_BUDGET_MS,
    `max ${max} ms, median ${median(samples)} ms`,
  );
  return samples;
};

const median = (values: number[]): number => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)] ?? NaN;
};

/** Phase 1: a running pre-008 Handy refuses CLI access; nothing written, the app not activated. */
const olderPhase = (options: Options): void => {
  if (!options.olderPid) {
    check("older: running pre-008 Handy", false, "no --older-pid given");
    return;
  }
  const typeBefore = applicationType(options.olderPid);
  const before = storeHash(options);
  const read = cli(options, ["settings", "get", "--json"], false);
  const write = cli(
    options,
    ["settings", "set", "paste_method", "direct", "--json"],
    false,
  );
  const codes = [read.code, write.code];
  check(
    "older: read and write refused with exit 5",
    codes.every((c) => c === 5),
    `exits ${codes}`,
  );
  check(
    "older: store untouched",
    storeHash(options) === before,
    `store ${before}`,
  );
  const typeAfter = applicationType(options.olderPid);
  check(
    "older: older app not activated",
    typeBefore === typeAfter,
    `ApplicationType ${typeBefore} -> ${typeAfter}`,
  );
};

/** Phase 2: offline reads and writes under the shared lock, preset for the app phase. */
const offlinePhase = (options: Options): number[] => {
  const samples = timedReads(options, true, "offline");
  const presets = [
    ["start_hidden", "true"],
    ["show_tray_icon", "true"],
  ].map(([key, value]) =>
    cli(options, ["settings", "set", key, value, "--json"], true),
  );
  check(
    "offline: writes commit (start_hidden, show_tray_icon)",
    presets.every((run) => envelope(run)?.ok === true),
    presets.map((run) => run.stdout.trim()).join(" "),
  );
  const entries = fs.readdirSync(path.dirname(storePath(options))).sort();
  check(
    "offline: no model or app data created by the CLI",
    entries.every((entry) =>
      /^(settings_store\.json|settings\.lock|settings\.instance\.lock)$/.test(
        entry,
      ),
    ),
    entries.join(", "),
  );
  return samples;
};

/** Start the proof app hidden; resolve once the channel answers. */
const startApp = (options: Options): ChildProcess => {
  const log = fs.openSync(path.join(options.out, "app-stdio.log"), "a");
  const app = spawn(options.app, [], {
    env: { ...process.env, HOME: options.home },
    stdio: ["ignore", log, log],
    detached: true,
  });
  const deadline = performance.now() + APP_READY_TIMEOUT_MS;
  while (performance.now() < deadline) {
    const probe = cli(
      options,
      ["settings", "get", "start_hidden", "--json"],
      false,
    );
    if (envelope(probe)?.ok === true) return app;
  }
  throw new Error("the app never answered over the settings channel");
};

const appLog = (options: Options): string => {
  const log = path.join(
    options.home,
    "Library/Logs/com.pais.handy.proof/handy.log",
  );
  return fs.existsSync(log) ? fs.readFileSync(log, "utf8") : "";
};

/** Phase 3: activation policy and startup order of the running app. */
const appPhase = (options: Options, app: ChildProcess): void => {
  const pid = app.pid ?? 0;
  const type = applicationType(pid);
  check(
    "app: started hidden without Dock icon (Accessory)",
    type === "UIElement",
    `ApplicationType ${type}`,
  );
  const visible =
    windows(options, pid)?.filter((w) => w.includes('"layer":0')) ?? null;
  check(
    "app: no visible main window",
    visible !== null && visible.length === 0,
    JSON.stringify(visible),
  );
  const early = appLog(options).includes(
    "Settings read before the settings service started",
  );
  check(
    "app: no settings read before the service is installed",
    !early,
    `log mentions it: ${early}`,
  );
};

/** Phase 4: reads, writes and a live effect answered by the running app. */
const livePhase = (options: Options, app: ChildProcess): number[] => {
  const pid = app.pid ?? 0;
  const samples = timedReads(options, false, "live");
  const set = cli(
    options,
    ["settings", "set", "paste_delay_ms", "120", "--json"],
    false,
  );
  const output = cli(
    options,
    ["settings", "output", "set", "auto_submit", "true", "--json"],
    false,
  );
  const back = cli(options, ["settings", "output", "get", "--json"], false);
  const data = (envelope(back)?.data ?? {}) as Record<string, unknown>;
  check(
    "live: writes answered by the instance and read back",
    envelope(set)?.ok === true &&
      envelope(output)?.ok === true &&
      data.paste_delay_ms === 120 &&
      data.auto_submit === true,
    `${set.stdout.trim()} ${output.stdout.trim()}`,
  );
  trayEffect(options);
  const type = applicationType(pid);
  check(
    "live: CLI calls never showed the app (still Accessory)",
    type === "UIElement",
    `ApplicationType ${type}`,
  );
  return samples;
};

/**
 * Routed live effect: the tray visibility change runs in the app (its log records it). The menu
 * bar status window cannot show it here: macOS lists it with empty bounds whether visible or not.
 */
const trayEffect = (options: Options): void => {
  const count = (state: string) =>
    appLog(options).split(`Tray visibility set to: ${state}`).length - 1;
  const before = [count("false"), count("true")];
  cli(options, ["settings", "set", "show_tray_icon", "false", "--json"], false);
  cli(options, ["settings", "set", "show_tray_icon", "true", "--json"], false);
  spawnSync("sleep", ["1"]);
  const after = [count("false"), count("true")];
  check(
    "live: show_tray_icon routed writes ran the tray live effect in the app",
    after[0] === before[0] + 1 && after[1] === before[1] + 1,
    `app log "Tray visibility set to" false/true: ${before} -> ${after}`,
  );
};

/** CLI processes never registered with LaunchServices (no app, window, Dock or tray). */
const registrationCheck = (launches: string, appPid: number): void => {
  const created = (pid: number) =>
    new RegExp(`kLSNotifyApplicationCreation[^\\n]*"pid"=${pid}\\b`).test(
      launches,
    );
  const cliPids = runs
    .map((run) => run.pid)
    .filter((pid): pid is number => pid !== undefined);
  const registered = cliPids.filter(created);
  check(
    "cli: no CLI process registered as an application (the app itself is seen)",
    registered.length === 0 && created(appPid),
    `${cliPids.length} CLI runs, registered ${registered.length}; app registration seen: ${created(appPid)}`,
  );
};

const stop = (app: ChildProcess): void => {
  if (app.pid) process.kill(app.pid, "SIGTERM");
  spawnSync("sleep", ["2"]);
};

const write = (options: Options, name: string, value: unknown): void => {
  fs.writeFileSync(
    path.join(options.out, name),
    `${JSON.stringify(value, null, 2)}\n`,
  );
};

const main = async (): Promise<void> => {
  const options = parseOptions(process.argv.slice(2));
  fs.mkdirSync(options.out, { recursive: true });
  fs.mkdirSync(options.home, { recursive: true });
  // `script` gives lsappinfo a terminal, so each notification is flushed as it arrives; the
  // stream goes to a scratch file (it describes every app of the session) and only the proof
  // app's lines are kept.
  const stream = path.join(options.home, "lsappinfo-listen.txt");
  const listener = spawn(
    "script",
    ["-q", "/dev/null", "lsappinfo", "listen", "+all", "wait", "240"],
    { stdio: ["ignore", fs.openSync(stream, "w"), "ignore"] },
  );
  olderPhase(options);
  const offline = offlinePhase(options);
  const app = startApp(options);
  appPhase(options, app);
  const live = livePhase(options, app);
  stop(app);
  const again = startApp(options);
  check(
    "restart: stale endpoint reclaimed, channel answers again",
    true,
    `new pid ${again.pid}`,
  );
  stop(again);
  const after = cli(options, ["settings", "get", "--json"], true);
  check(
    "restart: offline access returns once the app is gone",
    envelope(after)?.ok === true,
    `exit ${after.code}`,
  );
  spawnSync("sleep", ["1"]);
  listener.kill();
  const launches = fs.readFileSync(stream, "utf8");
  registrationCheck(launches, app.pid ?? 0);
  // Keep only the proof app's notifications: the stream also describes the user's other apps.
  const ours = launches
    .split("\n")
    .filter((line) => line.includes(options.app));
  write(options, "launch-notifications.json", ours);
  write(options, "latency.json", {
    offline_ms: offline,
    live_ms: live,
    budget_ms: READ_BUDGET_MS,
  });
  // Full settings reads repeat the same document: keep the start of each output.
  const clip = (text: string) =>
    text.length > 240 ? `${text.slice(0, 240)}…` : text;
  write(
    options,
    "runs.json",
    runs.map((run) => ({
      ...run,
      stdout: clip(run.stdout),
      stderr: clip(run.stderr),
    })),
  );
  write(options, "checks.json", checks);
  write(options, "app-log.txt", appLog(options).split("\n"));
  process.exit(checks.every((entry) => entry.passed) ? 0 : 1);
};

await main();
