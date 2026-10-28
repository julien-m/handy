/** Bounded process execution for the opt-in public native fixture. */
import { spawn, type ChildProcessByStdio } from "node:child_process";
import type { Readable } from "node:stream";

type NativeChild = ChildProcessByStdio<null, Readable, Readable>;

// Diagnostics are small; bound runaway compiler/native output before buffering.
const MAX_PROCESS_OUTPUT_BYTES = 1_000_000;

/** Owns bounded stdout/stderr buffers and the reaped direct child's exit status. */
export interface ProcessResult {
  stdout: Buffer;
  stderr: Buffer;
  exitStatus: number;
}

interface CaptureState {
  stdout: Buffer[];
  stderr: Buffer[];
  bytes: number;
  failure: Error | null;
}

// Diagnostic trace contains only owned process identities, phases, and timings.
function tracePhase(
  phase: string,
  event: string,
  pid?: number,
  code?: number | null,
): void {
  process.stderr.write(
    `${JSON.stringify({ phase, event, pid, code, monotonic_ms: performance.now() })}\n`,
  );
}

function stopChild(child: NativeChild, phase: string): void {
  // Children inherit the wrapper group; Cargo's outer timeout can kill all of
  // them. Here only the owned direct child is signalled and reaped on close.
  try {
    if (!child.kill("SIGKILL"))
      process.stderr.write(
        `${phase}: child could not be signalled for teardown\n`,
      );
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    process.stderr.write(`${phase}: child teardown failed: ${detail}\n`);
  }
}

function collectOutput(
  state: CaptureState,
  destination: Buffer[],
  chunk: Buffer,
  child: NativeChild,
  phase: string,
): void {
  state.bytes += chunk.length;
  if (state.bytes > MAX_PROCESS_OUTPUT_BYTES) {
    state.failure = new Error(
      `${phase} exceeded the bounded diagnostic output size`,
    );
    stopChild(child, phase);
  } else destination.push(chunk);
}

function finishCapture(
  state: CaptureState,
  code: number | null,
): ProcessResult {
  if (state.failure) throw state.failure;
  if (code === null) throw new Error("Child terminated without an exit status");
  return {
    stdout: Buffer.concat(state.stdout),
    stderr: Buffer.concat(state.stderr),
    exitStatus: code,
  };
}

function observeChild(
  child: NativeChild,
  state: CaptureState,
  timer: ReturnType<typeof setTimeout>,
  phase: string,
): Promise<ProcessResult> {
  return new Promise((resolve, reject) => {
    observeOutput(child, state, phase);
    child.on("error", (error) => {
      clearTimeout(timer);
      reject(new Error(`${phase} could not launch: ${error.message}`));
    });
    child.on("exit", (code) => tracePhase(phase, "exit", child.pid, code));
    // close follows direct-child reaping and draining both output streams.
    child.on("close", (code) => {
      tracePhase(phase, "close", child.pid, code);
      clearTimeout(timer);
      try {
        resolve(finishCapture(state, code));
      } catch (error) {
        reject(error);
      }
    });
  });
}

function observeOutput(
  child: NativeChild,
  state: CaptureState,
  phase: string,
): void {
  child.stdout.on("data", (chunk: Buffer) =>
    collectOutput(state, state.stdout, chunk, child, phase),
  );
  child.stderr.on("data", (chunk: Buffer) =>
    collectOutput(state, state.stderr, chunk, child, phase),
  );
}

/**
 * Starts one child in the inherited group, buffers bounded output, and reaps it.
 * Timeout or output overflow signals that owned child with SIGKILL.
 * @param command Nonempty executable-and-arguments tuple.
 * @param timeoutMs Positive finite safety deadline in milliseconds.
 * @param phase Diagnostic name included in errors.
 * @returns Owned stdout/stderr buffers and the direct child's exit status.
 * @throws Invalid-input, launch, timeout, output-overflow, or signal-exit errors.
 */
export async function runCommand(
  command: readonly [string, ...string[]],
  timeoutMs: number,
  phase: string,
): Promise<ProcessResult> {
  if (!command.length || typeof command[0] !== "string" || !command[0].length)
    throw new Error(`${phase}: executable must be a nonempty string`);
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0)
    throw new Error(`${phase}: timeout must be positive and finite`);
  const child = spawn(command[0], command.slice(1), {
    stdio: ["ignore", "pipe", "pipe"],
  });
  tracePhase(phase, "start", child.pid);
  const state: CaptureState = {
    stdout: [],
    stderr: [],
    bytes: 0,
    failure: null,
  };
  const timer = setTimeout(() => {
    state.failure = new Error(
      `${phase} exceeded its ${timeoutMs} ms safety deadline`,
    );
    stopChild(child, phase);
  }, timeoutMs);
  return await observeChild(child, state, timer, phase);
}

/**
 * Checks an already captured child result without starting or signalling processes.
 * @param result Reaped child status and bounded output buffers.
 * @param phase Diagnostic name included in a failure.
 * @returns Nothing when the exit status is zero.
 * @throws An error containing the status and bounded stderr when it is nonzero.
 */
export function requireSuccess(result: ProcessResult, phase: string): void {
  if (result.exitStatus !== 0)
    throw new Error(`${phase} failed (${result.exitStatus}): ${result.stderr}`);
}
