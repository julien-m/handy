/**
 * Native proof orchestration of feature 001 (microphone priority). It runs the ignored native
 * harness `src-tauri/tests/microphone_native.rs` on this machine's real audio devices and records
 * every command (argv, exit code, duration, stdout/stderr) and every observation line in a proof
 * directory, with the OS and, when given, the sha256 of the release binary under test. No device
 * name or audio is recorded: the harness prints timings and counters only.
 *
 * Usage:
 *   bun scripts/microphone-native-proof.ts --platform macos|windows|linux --out <dir>
 *     [--binary <absolute release binary>] [--interactive]
 *
 * `--interactive` adds the loss test, which asks the operator to unplug the default microphone.
 * The harness needs microphone permission; the enumeration probe alone does not.
 */
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

interface Options {
  platform: string;
  out: string;
  binary: string | null;
  interactive: boolean;
}
interface CommandRecord {
  test: string;
  argv: string[];
  code: number | null;
  ms: number;
  stdout: string;
  stderr: string;
}

const HOST_PLATFORMS: Record<string, string> = {
  darwin: "macos",
  win32: "windows",
  linux: "linux",
};
const TESTS = [
  "enumeration_is_bounded_and_groups_same_named_devices",
  "a_default_open_is_bounded_and_releases_the_device",
  "a_timed_out_open_drops_its_late_stream_and_frees_the_device",
];
const INTERACTIVE_TEST = "an_unplugged_microphone_ends_capture_with_its_prefix";

/** Parse `--name value` pairs and flags. */
const parseOptions = (argv: string[]): Options => {
  const value = (name: string): string | null => {
    const index = argv.indexOf(`--${name}`);
    return index >= 0 ? (argv[index + 1] ?? null) : null;
  };
  const binary = value("binary");
  if (binary !== null && !path.isAbsolute(binary)) {
    throw new Error("--binary must be an absolute path");
  }
  return {
    platform:
      value("platform") ?? HOST_PLATFORMS[process.platform] ?? "unknown",
    out: path.resolve(value("out") ?? "microphone-native-proof"),
    binary,
    interactive: argv.includes("--interactive"),
  };
};

/** sha256 of a file, or "absent". */
const sha256 = (file: string | null): string => {
  if (file === null || !fs.existsSync(file)) return "absent";
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
};

/** Run one ignored harness test through cargo; the harness owns the native calls. */
const runTest = (root: string, test: string): CommandRecord => {
  const argv = [
    "test",
    "--manifest-path",
    path.join(root, "src-tauri", "Cargo.toml"),
    "--test",
    "microphone_native",
    test,
    "--",
    "--ignored",
    "--exact",
    "--nocapture",
    "--test-threads=1",
  ];
  const started = performance.now();
  // stdin stays attached so the interactive test can reach the operator's terminal.
  const result = spawnSync("cargo", argv, {
    encoding: "utf8",
    stdio: ["inherit", "pipe", "pipe"],
  });
  return {
    test,
    argv: ["cargo", ...argv],
    code: result.status,
    ms: Math.round(performance.now() - started),
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
};

/** Observation lines printed by the harness (one JSON object per test). */
const observations = (records: CommandRecord[]): unknown[] =>
  records.flatMap((record) =>
    record.stdout
      .split("\n")
      .filter((line) => line.startsWith('{"'))
      .map((line) => JSON.parse(line) as unknown),
  );

const main = (): number => {
  const options = parseOptions(process.argv.slice(2));
  const host = HOST_PLATFORMS[process.platform] ?? "unknown";
  if (options.platform !== host) {
    console.error(
      `--platform ${options.platform} must run on that platform (this host is ${host})`,
    );
    return 2;
  }
  const root = path.resolve(import.meta.dirname, "..");
  const tests = options.interactive ? [...TESTS, INTERACTIVE_TEST] : TESTS;
  const records = tests.map((test) => runTest(root, test));
  fs.mkdirSync(options.out, { recursive: true });
  const metadata = {
    platform: options.platform,
    os_release: os.release(),
    arch: os.arch(),
    binary: options.binary,
    binary_sha256: sha256(options.binary),
    recorded_at: new Date().toISOString(),
  };
  const write = (name: string, data: unknown): void =>
    fs.writeFileSync(
      path.join(options.out, name),
      JSON.stringify(data, null, 2),
    );
  write("metadata.json", metadata);
  write("commands.json", records);
  write("observations.json", observations(records));
  const failed = records.filter((record) => record.code !== 0);
  for (const record of records) {
    console.error(
      `${record.code === 0 ? "PASS" : "FAIL"} ${record.test} (${record.ms} ms)`,
    );
  }
  return failed.length === 0 ? 0 : 1;
};

process.exit(main());
