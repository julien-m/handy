/**
 * Native proof orchestration of feature 009 (PIN mode and Enter rule), to run with the
 * maintainer on a release binary built with `destination-return` and `smart-paste-dispatch`
 * open. It runs the ignored native harnesses (`destination_platform.rs`, `enter_platform.rs`),
 * which read the workspace and never type, and records every command (argv, exit code,
 * duration, stdout/stderr) in a proof directory with the OS and the binary's sha256. With
 * `--interactive`, it then walks the operator through the manual protocol (SC-003 five pinned
 * dictations from three other applications, Enter counts per option, timing, focus, closed and
 * reopened pinned application, CLI parity) and records each observation the operator types.
 * The script itself never activates an application, sends a key or touches the clipboard: the
 * operator dictates with Handy. No transcript or window title is recorded, only the safe test
 * phrase the operator chose and the counts.
 *
 * Usage:
 *   bun scripts/pin-enter-native-proof.ts --binary <absolute release binary>
 *     --platform macos|windows --out <dir> [--interactive]
 */
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import readline from "node:readline/promises";

interface Options {
  binary: string;
  platform: string;
  out: string;
  interactive: boolean;
}
interface CommandRecord {
  target: string;
  argv: string[];
  code: number | null;
  ms: number;
  stdout: string;
  stderr: string;
}
interface Observation {
  check: string;
  question: string;
  answer: string;
}

/** Ignored native harnesses (read-only: no key, text or activation). */
const HARNESSES = ["destination_platform", "enter_platform"];
const FEATURES = "destination-return,smart-paste-dispatch";

/** Manual protocol of the plan (Step 7); each line is one recorded observation. */
const PROTOCOL: [string, string][] = [
  [
    "SC-003",
    "Pin ChatGPT, then dictate five times from Safari, Notes, TextEdit, Safari, Notes. For each, the app that received the text and the Enter count (e.g. ChatGPT/1,ChatGPT/1,...)",
  ],
  [
    "AC-005 timing",
    "Enter attempt minus t0 for each eligible Enter, in ms (debug log 'Automatic Enter'), all <= 100 on macOS?",
  ],
  [
    "AC-005 Chromium",
    "ChatGPT and Comet (read twice, 150 ms apart): was the full transcript in the field when Enter landed? (yes/no per app)",
  ],
  [
    "AC-005 options",
    "Enter count per option with and without pin: Never/unpinned, Never/pinned, Always/unpinned, Always/pinned, Only in PIN/unpinned, Only in PIN/pinned (expected 0,0,1,1,0,1)",
  ],
  [
    "AC-005 focus",
    "Switching to another app during the 50 ms pause or after insertion: Enter count (expected 0)",
  ],
  [
    "AC-003 closed",
    "Notes pinned and quit, dictate: inserted anywhere? reason shown ('<App> is pinned but not open')? pin kept?",
  ],
  ["AC-003 reopened", "Notes reopened, dictate: text inserted into Notes?"],
  [
    "AC-001 badge",
    "Badge shows the pin mark and VoiceOver reads 'Destination: <name>, pinned'; return off with a pin still shows the badge; menu opens without taking focus?",
  ],
  [
    "AC-002 override",
    "Choose Comet in the menu while pinned: this dictation goes to Comet, the next to ChatGPT; Unpin before insertion goes back to the captured app?",
  ],
  [
    "AC-006 CLI",
    "With Handy running: handy settings destination pin <bundle id> --json, status, unpin answers and badge parity?",
  ],
  [
    "AC-004 settings",
    "Output settings show only Automatic Enter (Never, Always, Only in PIN mode) and the key; arrow keys and Enter change it; Tab order mode then key?",
  ],
];

const parseOptions = (argv: string[]): Options => {
  const value = (name: string): string | null => {
    const index = argv.indexOf(`--${name}`);
    return index >= 0 ? (argv[index + 1] ?? null) : null;
  };
  const binary = value("binary");
  const out = value("out");
  if (binary === null || !path.isAbsolute(binary)) {
    throw new Error("--binary must be an absolute path to the release binary");
  }
  if (out === null) throw new Error("--out <proof directory> is required");
  return {
    binary,
    platform: value("platform") ?? "macos",
    out,
    interactive: argv.includes("--interactive"),
  };
};

const sha256 = (file: string): string =>
  createHash("sha256").update(fs.readFileSync(file)).digest("hex");

/** Run one ignored native harness through cargo and record it. */
const runHarness = (target: string): CommandRecord => {
  const argv = [
    "cargo",
    "test",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--features",
    FEATURES,
    "--test",
    target,
    "--",
    "--include-ignored",
    "--nocapture",
  ];
  const started = Date.now();
  const run = spawnSync(argv[0], argv.slice(1), { encoding: "utf8" });
  return {
    target,
    argv,
    code: run.status,
    ms: Date.now() - started,
    stdout: run.stdout ?? "",
    stderr: run.stderr ?? "",
  };
};

/** Ask the operator each manual check and record the answers verbatim. */
const runProtocol = async (): Promise<Observation[]> => {
  const prompt = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  const observations: Observation[] = [];
  for (const [check, question] of PROTOCOL) {
    const answer = await prompt.question(`[${check}] ${question}\n> `);
    observations.push({ check, question, answer: answer.trim() });
  }
  prompt.close();
  return observations;
};

const main = async (): Promise<number> => {
  const options = parseOptions(process.argv.slice(2));
  fs.mkdirSync(options.out, { recursive: true });
  const environment = {
    platform: options.platform,
    os: `${os.type()} ${os.release()} ${os.arch()}`,
    binary_sha256: sha256(options.binary),
    features: FEATURES,
    recorded_at: new Date().toISOString(),
  };
  const commands = HARNESSES.map(runHarness);
  const observations = options.interactive ? await runProtocol() : [];
  const write = (name: string, data: unknown): void =>
    fs.writeFileSync(
      path.join(options.out, name),
      `${JSON.stringify(data, null, 2)}\n`,
    );
  write("environment.json", environment);
  write("commands.json", commands);
  write("native-observations.json", observations);
  const failed = commands.filter((command) => command.code !== 0);
  for (const command of commands) {
    console.log(`${command.target}: exit ${command.code} in ${command.ms} ms`);
  }
  // The observations are evidence for the maintainer's review, never an automatic PASS.
  console.log(
    `${observations.length} observations recorded in ${options.out}; review them before any claim.`,
  );
  return failed.length === 0 ? 0 : 1;
};

process.exit(await main());
