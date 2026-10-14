#!/usr/bin/env python3
"""Measure AX operations using only disposable, offscreen synthetic AppKit fields."""

from __future__ import annotations

import argparse
from dataclasses import asdict
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import tempfile

from native_ax_bindings import (
    Call,
    FixtureResult,
    NativeRun,
    FIXTURE_WARMUP_SECONDS,
    REQUEST_TIMEOUT_SECONDS,
)
from native_ax_probe import AxProbe

# Diagnostic fixtures should exit promptly; escalate after two seconds, then reap.
FIXTURE_SHUTDOWN_TIMEOUT_SECONDS = 2
# Ten seconds bounds fixture GUI registration without requiring any interaction.
FIXTURE_STARTUP_TIMEOUT_SECONDS = 10
# Local metadata commands are small; ten seconds fails closed on a broken tool.
METADATA_TIMEOUT_SECONDS = 10
# Swift fixture compilation gets one minute, including compiler startup/cache work.
SWIFT_COMPILE_TIMEOUT_SECONDS = 60


# Generate small, oversized, and delayed offscreen AppKit fields. Each process announces
# readiness, accepts AX inspection while its main loop runs, and remains alive until
# the Python owner terminates and reaps it; no activation, keys, or clipboard APIs.
SWIFT_FIXTURE = r"""import AppKit
class DelayedTextView: NSTextView {
    override func setAccessibilitySelectedText(_ selectedText: String?) {
        let range = selectedRange()
        // Delay one ms beyond the reviewed 150 ms readback window.
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.151) { [self] in
            insertText(selectedText ?? "", replacementRange: range)
        }
    }
}
let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let window = NSWindow(contentRect: NSRect(x: -5000, y: -5000, width: 320, height: 100),
    styleMask: [.titled], backing: .buffered, defer: false)
let isDelayed = CommandLine.arguments.contains("delayed")
let frame = NSRect(x: 10, y: 10, width: 300, height: 70)
let text: NSTextView = isDelayed ? DelayedTextView(frame: frame) : NSTextView(frame: frame)
// The large fixture is one UTF-16 unit beyond the reviewed preparation guard.
text.string = CommandLine.arguments.contains("large") ? String(repeating: "x", count: 1000001) : "A😀Z"
text.setSelectedRange(NSRange(location: 1, length: 2))
window.contentView?.addSubview(text)
window.makeFirstResponder(text)
window.orderFrontRegardless()
app.finishLaunching()
// Let the fixture main loop start before announcing readiness; no product timing claim.
DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
    print("READY")
    fflush(stdout)
}
app.run()
"""


def _stop_fixture(process: subprocess.Popen[str]) -> None:
    """Resume before terminate, then kill and reap if graceful exit times out."""
    if process.poll() is None:
        os.kill(process.pid, signal.SIGCONT)
        process.terminate()
        try:
            process.wait(timeout=FIXTURE_SHUTDOWN_TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=FIXTURE_SHUTDOWN_TIMEOUT_SECONDS)
    if process.stdout is not None:
        process.stdout.close()


def _field_observations(
    probe: AxProbe, field: int, mode: str
) -> tuple[int, Call | None, bool | None, list[tuple[int, float, bool]]]:
    units = probe._read_units(field)
    if mode == "large":
        return units, None, None, []
    setter, start = probe.write_fixture(field)
    delayed = probe._readbacks(field, start, mode)
    return units, setter, delayed[-1][2], delayed


def _fixture_result(
    probe: AxProbe, process: subprocess.Popen[str], field: int, mode: str
) -> FixtureResult:
    units, setter, verified, delayed = _field_observations(probe, field, mode)
    stop_signal, stopped = (
        probe._stopped_calls(process, field) if mode == "small" else (None, [])
    )
    return FixtureResult(
        mode,
        process.pid,
        True,
        units,
        setter,
        verified,
        list(probe.calls),
        delayed,
        stopped,
        stop_signal,
    )


def _observe_fixture(
    probe: AxProbe, process: subprocess.Popen[str], mode: str
) -> FixtureResult:
    """Read/write only this fixture; large fields are classified before allocation."""
    application = probe.ax.AXUIElementCreateApplication(process.pid)
    if application is None:
        raise RuntimeError("AX application allocation failed")
    field: int | None = None
    try:
        field = probe._find_ready_field(int(application))
        if field is None:
            raise RuntimeError(f"Synthetic AXTextArea unavailable: {probe.calls}")
        return _fixture_result(probe, process, field, mode)
    finally:
        if field is not None:
            probe.cf.CFRelease(field)
        probe.cf.CFRelease(application)


# @spec FR-010: Native fixture evidence only — .specs/features/007-smart-paste/spec.md#fr-010
def _run_fixtures(probe: AxProbe, binary: Path) -> list[FixtureResult]:
    """Run only fresh synthetic processes with bounded startup and cleanup."""
    results: list[FixtureResult] = []
    for mode in ["small", "large", "delayed"]:
        process = subprocess.Popen(
            [str(binary), mode], stdout=subprocess.PIPE, text=True
        )
        try:
            if (
                process.stdout is None
                or not select.select(
                    [process.stdout], [], [], FIXTURE_STARTUP_TIMEOUT_SECONDS
                )[0]
            ):
                raise RuntimeError("Synthetic fixture startup timed out")
            if process.stdout.readline().strip() != "READY":
                raise RuntimeError("Synthetic fixture startup failed")
            probe.calls.clear()
            results.append(_observe_fixture(probe, process, mode))
        finally:
            _stop_fixture(process)
    return results


def _metadata(command: list[str]) -> str:
    return subprocess.check_output(
        command, text=True, timeout=METADATA_TIMEOUT_SECONDS
    ).strip()


def _producer_identity(
    source: Path,
) -> dict[str, str | bool | list[str] | dict[str, str]]:
    sources = [
        source,
        source.with_name("native_ax_probe.py"),
        source.with_name("native_ax_bindings.py"),
    ]
    paths = [str(path.relative_to(Path.cwd())) for path in sources]
    tracked_diff = subprocess.check_output(
        ["git", "diff", "--binary", "HEAD", "--", *paths],
        timeout=METADATA_TIMEOUT_SECONDS,
    )
    # Include sources because untracked producers do not appear in git diff.
    inputs = tracked_diff + b"\0".join(path.read_bytes() for path in sources)
    return {
        "source_head": _metadata(["git", "rev-parse", "HEAD"]),
        "worktree_diff_sha256": hashlib.sha256(inputs).hexdigest(),
        "worktree_hash_scope": paths,
        "producer_source_sha256": {
            path: hashlib.sha256(Path(path).read_bytes()).hexdigest() for path in paths
        },
        "receipt_excluded_from_worktree_hash": True,
        "probe_source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
    }


def _runtime_identity(run: NativeRun) -> dict[str, str | int]:
    return {
        "fixture_source_sha256": hashlib.sha256(
            run.fixture_source.read_bytes()
        ).hexdigest(),
        "compiled_binary_sha256": hashlib.sha256(run.binary.read_bytes()).hexdigest(),
        "toolchain": _metadata(["swiftc", "--version"]),
        "compile_exit_status": run.compile_exit_status,
        "probe_exit_status": 0,
        "platform": _metadata(["sw_vers"]),
    }


def _proof_constraints() -> dict[str, bool | int | list[str] | dict[str, str]]:
    return {
        "trusted": True,
        "requested_proxy_timeout_ms": round(REQUEST_TIMEOUT_SECONDS * 1000),
        "discovery_warmup_budget_ms": round(FIXTURE_WARMUP_SECONDS * 1000),
        "limitations": [
            "Proxy timeout is requested, not a strict wall-clock guarantee.",
            "No total-250ms or application compatibility or universal exactly-once claim.",
            "Late asynchronous writes can remain unchanged after the readback window and apply later.",
            "No clipboard, activation or key events were used; every fixture was reaped.",
        ],
        "primary_documentation": {
            "url": "https://developer.apple.com/documentation/applicationservices/1459345-axuielementsetmessagingtimeout",
            "sdk_header": "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks/ApplicationServices.framework/Versions/A/Frameworks/HIServices.framework/Versions/A/Headers/AXUIElement.h",
            "lines": "387-402",
        },
    }


def _compile_and_run(probe: AxProbe, directory: Path) -> NativeRun:
    source, binary = directory / "fixture.swift", directory / "fixture"
    source.write_text(SWIFT_FIXTURE)
    compiled = subprocess.run(
        ["swiftc", str(source), "-o", str(binary)],
        capture_output=True,
        text=True,
        timeout=SWIFT_COMPILE_TIMEOUT_SECONDS,
        check=True,
    )
    return NativeRun(source, binary, compiled.returncode, _run_fixtures(probe, binary))


def _write_receipt(output: Path, source: Path, run: NativeRun) -> None:
    """Persist typed provenance and observations without logging field contents."""
    relative = source.relative_to(Path.cwd())
    receipt = {
        "feature": "007-smart-paste",
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "exact_command": ["python3", str(relative), "--out", str(output)],
        **_producer_identity(source),
        **_runtime_identity(run),
        **_proof_constraints(),
        "fixtures": [asdict(result) for result in run.results],
    }
    output.write_text(json.dumps(receipt, indent=2) + "\n")


def main() -> int:
    """Bound fixture work, serialize source-bound observations, and reap children.

    Returns:
        Zero after the fresh source-bound receipt was written successfully.
    Raises:
        RuntimeError: Permission, fixture startup, native boundary, or observation failed.
        OSError: Native framework loading, process creation, or receipt writing failed.
        subprocess.SubprocessError: Compilation or metadata exceeded its bounded deadline.

    Owns the temporary directory and all children; every child is terminated and reaped.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    output = parser.parse_args().out
    probe = AxProbe()
    if not probe.ax.AXIsProcessTrusted():
        raise RuntimeError("AX permission missing; no prompt requested")
    with tempfile.TemporaryDirectory(prefix="handy-ax-spike-") as directory:
        run = _compile_and_run(probe, Path(directory))
        _write_receipt(output, Path(__file__).resolve(), run)
    print(
        json.dumps(
            {"receipt": str(output), "fixtures": len(run.results), "status": "observed"}
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
