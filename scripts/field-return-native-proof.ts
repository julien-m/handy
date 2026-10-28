/**
 * Native proof orchestration of feature 011 (dictation field return), to run with the
 * maintainer on a release binary built with `destination-return` and `smart-paste-dispatch`
 * open (`bun run tauri build --bundles app --features destination-return,smart-paste-dispatch`).
 * It runs the ignored read-only native harness (`destination_field_native.rs`: bounded start
 * read of the focused input of the application in front, or of `HANDY_FIELD_PID`, the
 * verification reads and 006's foreground read unchanged), which never gives focus, restores a
 * range, raises a window, activates an application, types or touches the clipboard, and records
 * every command (argv, exit code, duration, stdout/stderr) with the OS and the binary's sha256.
 * With `--interactive`, it then walks the operator through the manual matrix of plan Steps 1
 * and 6 (Comet ChatGPT page, Notes, ChatGPT desktop, drafts with the caret inside, changed tab,
 * reloaded page, second window, password field, pinned application in the background, badge
 * change, paste method none, typing in the same field during the recording, forced bubble) and
 * records each observation the operator types. With `--log <Handy log file> --sentinel <text>`
 * it finally checks that the sentinel draft typed in the remembered input never appears in the
 * log (spec SC-004). No transcript, field content or window title is recorded.
 *
 * Usage:
 *   bun scripts/field-return-native-proof.ts --binary <absolute release binary>
 *     --platform macos|windows --out <dir> [--interactive] [--log <file> --sentinel <text>]
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
  log: string | null;
  sentinel: string | null;
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
const HARNESSES = ["destination_field_native"];
const FEATURES = "destination-return,smart-paste-dispatch";

/** Manual matrix of the macOS native gate; each line is one recorded observation. */
const MACOS_PROTOCOL: [string, string][] = [
  [
    "SC-001 Comet exposure",
    "Comet, ChatGPT page, caret inside a draft: does `destination_field_native` (run with Comet in front) report remembered(range_read: true)? (precondition of SC-001; no change of Comet's accessibility mode)",
  ],
  [
    "SC-001 Comet same tab",
    "5 runs: caret inside a draft of the ChatGPT input, dictate, switch to Notes, stop. Text at the caret, draft intact? (record each run; if not exposed, record the 5 safe fallbacks)",
  ],
  [
    "SC-001 Notes",
    "5 runs with a draft in a Notes note, caret inside, switch to TextEdit, stop: text at the caret, draft intact?",
  ],
  [
    "SC-001 ChatGPT desktop",
    "5 runs with a draft in the ChatGPT desktop input, caret inside, switch to Notes, stop: text at the caret, draft intact?",
  ],
  [
    "AC-002 focus effect",
    "Does giving focus back select all or move the caret before the range is restored (Notes, ChatGPT, Comet)? Does focus survive AXRaise (second Notes window)?",
  ],
  [
    "AC-002 second window",
    "Notes: dictate from the editor of window A, bring window B forward, stop: window A raised and text in its editor (or the bubble, never window B)?",
  ],
  [
    "AC-004 changed tab",
    "Comet: dictate from the ChatGPT input, select another Comet tab during the recording, stop: nothing pasted and the bubble 'Insertion failed' with 'nothing was sent'? (a paste into a hidden tab is a finding for Julien)",
  ],
  [
    "AC-004 reloaded page",
    "Comet: dictate from the ChatGPT input, reload the page during the recording, stop: insertion as today (006/010) or the bubble; never a paste elsewhere; which one?",
  ],
  [
    "AC-004 same field typed",
    "Dictate from an input and keep typing in that same input during the recording, then stop: where does the text land, is any typed text replaced? (decision of the r3 spec review ambiguity)",
  ],
  [
    "AC-001 password",
    "Dictate from a password field (Safari login page or System Settings), switch application, stop: nothing remembered (006 behavior), nothing typed into the password field?",
  ],
  [
    "AC-003 pin",
    "Pin Comet with its ChatGPT input focused, bring Notes forward, dictate twice: each text in the ChatGPT input after Comet comes forward?",
  ],
  [
    "AC-003 badge change",
    "Dictate from Comet's input, choose TextEdit from the badge, stop: text in TextEdit as in 006, nothing in Comet?",
  ],
  [
    "AC-006 method none",
    "Paste method none: dictate from Comet's input, switch to Notes, stop: no activation, no focus change, transcript in history only?",
  ],
  [
    "AC-004 forced bubble",
    "Force a diverted case (changed tab): bubble text and explanation, Copy works, no Enter with Automatic Enter Always, no paste-error toast; with the bubble unpresentable (screen locked), one paste-error?",
  ],
  [
    "QE timing",
    "Debug log `field_return … (attempt …, verify …)` and `not_remembered … in …`: worst capture, attempt and verify times; main-thread check() samples (<= 70 ms over 30 runs) and any UI stall",
  ],
];

/** Release protocol off macOS (pending release evidence, never a gate). */
const OTHER_PROTOCOL: [string, string][] = [
  [
    "AC-007 Windows Notepad",
    "Notepad with a draft, caret inside, dictate, switch to Explorer, stop: text at the caret (UI Automation SetFocus and Select), draft intact?",
  ],
  [
    "AC-007 Windows browser",
    "Edge or Chrome text area with a draft: same check; a field without TextPattern shows the bubble, never a paste?",
  ],
  [
    "AC-007 Windows pinned background",
    "Pinned application not in front at the start: nothing remembered (UI Automation exposes only the system focus), 006/009 behavior?",
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
    log: value("log"),
    sentinel: value("sentinel"),
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

/** SC-004: whether the sentinel draft appears in the Handy log (never records the log). */
const sentinelCheck = (
  options: Options,
): { checked: boolean; found: boolean | null } => {
  if (options.log === null || options.sentinel === null) {
    return { checked: false, found: null };
  }
  const text = fs.readFileSync(options.log, "utf8");
  return { checked: true, found: text.includes(options.sentinel) };
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
  const sentinel = sentinelCheck(options);
  write("sentinel-log-check.json", sentinel);
  const failed = commands.filter((command) => command.code !== 0);
  for (const command of commands) {
    console.log(`${command.target}: exit ${command.code} in ${command.ms} ms`);
  }
  // The observations are evidence for the maintainer's review, never an automatic PASS.
  console.log(
    `${observations.length} observations recorded in ${options.out}; review them before any claim.`,
  );
  if (sentinel.found === true) {
    console.log("SC-004 FAILED: the sentinel draft appears in the Handy log");
  }
  return failed.length === 0 && sentinel.found !== true ? 0 : 1;
};

process.exit(await main());
