/** Strict schema validation and provenance for the native AppKit characterization. */
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { runCommand, requireSuccess } from "./smart-paste-native-process.js";

// Safety bound for short local Git/toolchain metadata calls.
const METADATA_TIMEOUT_MS = 10_000;
// Private frozen scope keeps consumers from changing the evidence hash inputs.
const inputPaths: readonly string[] = Object.freeze([
  "scripts/smart-paste-native-proof.ts",
  "scripts/smart-paste-native-process.ts",
  "scripts/smart-paste-native-report.ts",
  "src-tauri/tests/smart_paste_native.rs",
]);

/** Source, command, and toolchain identities bound to one native capture. */
export interface NativeProvenance {
  captured_at: string;
  command: string;
  command_argv: string[];
  source_head: string;
  worktree_diff_sha256: string;
  worktree_diff_scope: readonly string[];
  harness_sha256: string;
  producer_input_sha256: Record<string, string>;
  toolchain: { bun: string; swift: string };
}

interface NativeReport {
  schema_version: 1;
  os: string;
  general_clipboard_touched: boolean;
  native_surface: string;
  current_host_only_requested: boolean;
  remote_current_host_only_effect: string;
  callbacks_during_worker_hold: boolean;
  lazy_fulfillment_preserves_change_count: boolean;
  markers_eager: boolean;
  snapshot_roundtrip_equal: boolean;
  prekey_callback_qualifies: boolean;
  first_postkey_callback_count: number;
  all_reads_received_full_fixture: boolean;
  cache_limitation_observed: boolean;
  prekey_cached_after_keydown: boolean;
  repeated_postkey_read_cached: boolean;
  prekey_qualifying_callback_count: number;
  prekey_then_postkey_callback_count: number;
  repeated_postkey_callback_count: number;
  callback_milliseconds: number[];
  key_down_milliseconds: number;
  prekey_callback_milliseconds: number[];
  prekey_publication_key_down_milliseconds: number;
  actual_key_posting: string;
  snapshot_deadline_helper: string;
  read_receipt_limit: string;
}

const booleanFields = [
  "general_clipboard_touched",
  "current_host_only_requested",
  "callbacks_during_worker_hold",
  "lazy_fulfillment_preserves_change_count",
  "markers_eager",
  "snapshot_roundtrip_equal",
  "prekey_callback_qualifies",
  "all_reads_received_full_fixture",
  "cache_limitation_observed",
  "prekey_cached_after_keydown",
  "repeated_postkey_read_cached",
];
const stringFields = [
  "os",
  "native_surface",
  "remote_current_host_only_effect",
  "actual_key_posting",
  "snapshot_deadline_helper",
  "read_receipt_limit",
];
const countFields = [
  "first_postkey_callback_count",
  "prekey_qualifying_callback_count",
  "prekey_then_postkey_callback_count",
  "repeated_postkey_callback_count",
];
const timeFields = [
  "key_down_milliseconds",
  "prekey_publication_key_down_milliseconds",
];
const arrayFields = ["callback_milliseconds", "prekey_callback_milliseconds"];
const finiteNonnegative = (item: unknown): item is number =>
  typeof item === "number" && Number.isFinite(item) && item >= 0;
const allowed = new Set([
  "schema_version",
  ...booleanFields,
  ...stringFields,
  ...countFields,
  ...timeFields,
  ...arrayFields,
]);

/**
 * Reads the supplied object to validate every required field and reject unknown fields.
 * @param value External parsed JSON to validate; the object is not modified.
 * @returns A type predicate that is false for missing, unknown, or mistyped fields.
 * @throws An error if an externally supplied object's property accessors throw.
 */
export function isNativeReport(value: unknown): value is NativeReport {
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return false;
  // Non-null, non-array object checks permit record-like reads; each value stays
  // unknown until the field validators below establish its precise type.
  const report = value as Record<string, unknown>;
  return (
    report.schema_version === 1 &&
    Object.keys(report).every((key) => allowed.has(key)) &&
    booleanFields.every((key) => typeof report[key] === "boolean") &&
    stringFields.every(
      (key) => typeof report[key] === "string" && report[key].length > 0,
    ) &&
    countFields.every(
      (key) => finiteNonnegative(report[key]) && Number.isInteger(report[key]),
    ) &&
    timeFields.every((key) => finiteNonnegative(report[key])) &&
    arrayFields.every(
      (key) =>
        Array.isArray(report[key]) && report[key].every(finiteNonnegative),
    )
  );
}

/**
 * Computes a digest in memory without filesystem or process side effects.
 * @param bytes UTF-8 string or raw Buffer input.
 * @returns The lowercase SHA-256 hexadecimal digest.
 * @throws A crypto-provider error if SHA-256 initialization or hashing fails.
 */
export const hash = (bytes: string | Buffer): string =>
  createHash("sha256").update(bytes).digest("hex");
const shellQuote = (value: string): string =>
  `'${value.replaceAll("'", "'\\''")}'`;

async function hashSourceDiff(
  diff: Buffer,
  untracked: Buffer,
): Promise<string> {
  // Hash only the producer inputs; generated receipts cannot hash themselves.
  const digest = createHash("sha256").update(diff);
  for (const path of untracked.toString().trim().split("\n").filter(Boolean))
    digest
      .update("\0untracked\0")
      .update(path)
      .update("\0")
      .update(await readFile(path));
  return digest.digest("hex");
}

async function metadataCommand(
  command: readonly [string, ...string[]],
  phase: string,
) {
  const result = await runCommand(command, METADATA_TIMEOUT_MS, phase);
  requireSuccess(result, phase);
  return result.stdout;
}

async function readMetadata() {
  return await Promise.all([
    metadataCommand(["git", "rev-parse", "HEAD"], "source head"),
    metadataCommand(
      ["git", "diff", "--binary", "HEAD", "--", ...inputPaths],
      "producer diff",
    ),
    metadataCommand(
      [
        "git",
        "ls-files",
        "--others",
        "--exclude-standard",
        "--",
        ...inputPaths,
      ],
      "producer untracked inputs",
    ),
    metadataCommand(["swiftc", "--version"], "Swift toolchain"),
  ]);
}

/**
 * Reads producer files and starts bounded Git/Swift metadata children to bind a capture.
 * @returns Source hashes, UTC capture time, exact invocation, and toolchain identities.
 * @throws Filesystem, child launch/timeout/exit, crypto, or missing Bun-runtime errors.
 */
export async function collectProvenance(): Promise<NativeProvenance> {
  const capturedAt = new Date().toISOString();
  const commandArgv = [process.execPath, ...process.argv.slice(1)];
  const bunVersion = process.versions.bun;
  if (!bunVersion)
    throw new Error("Native characterization requires the Bun runtime");
  const [head, diff, untracked, swiftVersion] = await readMetadata();
  return {
    captured_at: capturedAt,
    command: commandArgv.map(shellQuote).join(" "),
    command_argv: commandArgv,
    source_head: head.toString().trim(),
    worktree_diff_sha256: await hashSourceDiff(diff, untracked),
    worktree_diff_scope: inputPaths,
    harness_sha256: hash(await readFile(inputPaths[0])),
    producer_input_sha256: await hashProducerInputs(),
    toolchain: {
      bun: bunVersion,
      swift: swiftVersion.toString().trim(),
    },
  };
}

async function hashProducerInputs(): Promise<Record<string, string>> {
  return Object.fromEntries(
    await Promise.all(
      inputPaths.map(
        async (path): Promise<readonly [string, string]> => [
          path,
          hash(await readFile(path)),
        ],
      ),
    ),
  );
}
