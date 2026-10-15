/**
 * Native proof orchestration of feature 010 (recovery bubble), to run with the maintainer on a
 * release binary built with `destination-return` and `smart-paste-dispatch` open
 * (`bun run tauri build --bundles app --features destination-return,smart-paste-dispatch`).
 * It runs the ignored read-only native harness (`recovery_native.rs`: bounded classification
 * of the application in front, 006's foreground read unchanged afterwards), which never
 * activates an application, types or touches the clipboard, and records every command (argv,
 * exit code, duration, stdout/stderr) in a proof directory with the OS and the binary's
 * sha256. With `--interactive`, it then walks the operator through the manual protocol of the
 * plan (Steps 1, 6 and 7: first click without activation, field classification matrix
 * including a Spotlight panel, native drag into TextEdit, Notes, ChatGPT and Comet, VoiceOver,
 * presentation latency, Copy against smart paste restoration, no Enter) and records each
 * observation the operator types. The script itself never shows a bubble: the operator
 * dictates with Handy. No transcript or window title is recorded, only the safe test phrase
 * the operator chose and the observations.
 *
 * Usage:
 *   bun scripts/recovery-native-proof.ts --binary <absolute release binary>
 *     --platform macos|windows|linux --out <dir> [--interactive]
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

/** Ignored native harness (read-only: no key, text, clipboard or activation). */
const HARNESSES = ["recovery_native"];
const FEATURES = "destination-return,smart-paste-dispatch";

/** Manual protocol of the macOS native gate; each line is one recorded observation. */
const MACOS_PROTOCOL: [string, string][] = [
  [
    "AC-003 first click",
    "With TextEdit in front, a recovery bubble shown: click Copy, Close and Why? once each. Did each act on the first click, and did TextEdit stay frontmost with its key window (yes/no per control)?",
  ],
  [
    "AC-003 VoiceOver",
    "VoiceOver names of the drag handle, Copy, Close and Why? and the reason line; can each be activated with the VoiceOver cursor?",
  ],
  [
    "AC-002 matrix",
    "Classification per destination (bubble or insertion): Finder window list, Finder desktop, Safari page without a focused field, TextEdit document, Notes note, a search field, an unknown custom control",
  ],
  [
    "AC-002 Spotlight",
    "Spotlight open over Finder, then over TextEdit (destination stays frontmost): bubble or insertion? (accepted limit of 2026-10-15)",
  ],
  [
    "AC-002 timing",
    "recovery_native worst sample (<= 60 ms over 30 runs) and any UI stall seen during dictation into a slow application",
  ],
  [
    "AC-004 drop",
    "Drag the bubble into TextEdit, Notes, ChatGPT, Comet: full transcript received? (TextEdit and Notes must be complete; record the others as observed)",
  ],
  [
    "AC-004 refusal",
    "Drop on the desktop and press Escape during a drag: the bubble reappears unchanged? Hidden while dragging? Modifier keys never change copy?",
  ],
  [
    "AC-004 stale",
    "Start a new dictation while dragging: the bubble closes for good and the late drop does not reopen it?",
  ],
  [
    "AC-001 reasons",
    "Notes quit during a dictation, a pinned app not running, an app not brought forward, smart paste not read: reason line and explanation as specified, no Enter, no paste-error toast?",
  ],
  [
    "AC-001 latency",
    "Decision to visible samples from the debug log 'Recovery bubble N presented ... after the recovery decision' (record all; above 300 ms is a QE finding)",
  ],
  [
    "AC-005 Copy",
    "Copy while a smart paste restoration is pending: the transcript stays on the clipboard after the quiet period; the bubble closes; Copy failure leaves it open",
  ],
  [
    "AC-005 lifetime",
    "Bubble left 60 s, then the screen locked and unlocked: still there? A new dictation closes it at once?",
  ],
  [
    "AC-007 settings",
    "Settings > Advanced > Output: the toggle 'Show the bubble when no text field is focused' with Tab and Space, VoiceOver announcement, CLI parity (handy settings output get/set)",
  ],
];

/** Release protocol off macOS (pending release evidence, never a gate). */
const OTHER_PROTOCOL: [string, string][] = [
  [
    "AC-006 reduced bubble",
    "Destination unavailable, activation and method failure (Windows) / method failure only (Linux X11, Wayland, HANDY_NO_GTK_LAYER_SHELL=1): Copy, Close, Why? without drag, first click without focus theft?",
  ],
  [
    "AC-006 screen reader",
    "Narrator (Windows) or Orca (Linux) names and activation of Copy, Close and Why?",
  ],
  [
    "AC-006 overlay none",
    "Overlay style none: the bubble still appears for a method failure?",
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
const runProtocol = async (
  protocol: [string, string][],
): Promise<Observation[]> => {
  const prompt = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  const observations: Observation[] = [];
  for (const [check, question] of protocol) {
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
  const macos = options.platform === "macos";
  const commands = macos ? HARNESSES.map(runHarness) : [];
  const protocol = macos ? MACOS_PROTOCOL : OTHER_PROTOCOL;
  const observations = options.interactive ? await runProtocol(protocol) : [];
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
