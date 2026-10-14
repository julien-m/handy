"""Typed fixture DTOs and native ABI setup; no AX operation or product dispatch."""

from __future__ import annotations

import ctypes
from dataclasses import dataclass
from pathlib import Path

# Reviewed plan: request at most 20 ms on every messaged proxy.
REQUEST_TIMEOUT_SECONDS = 0.020
# CoreFoundation's documented UTF-8 encoding identifier.
UTF8_ENCODING = 0x08000100
# Only the synthetic fixture tree is searched; eight levels bound diagnostic traversal.
MAX_FIXTURE_TREE_DEPTH = 8
# Cold AppKit registration gets a bounded one-second fixture-only warmup.
FIXTURE_WARMUP_SECONDS = 1.0
# Three samples expose scheduler variation without claiming a latency distribution.
STOPPED_TARGET_SAMPLE_COUNT = 3
# Reviewed AX preparation guard: never convert more than one million UTF-16 units.
MAX_CONVERT_UTF16_UNITS = 1_000_000
# Require a kernel-confirmed stopped fixture within one second before timeout sampling.
STOP_STATE_TIMEOUT_SECONDS = 1.0
# Yield for a millisecond between nonblocking, child-specific state observations.
STOP_STATE_POLL_SECONDS = 0.001


@dataclass(frozen=True)
class Call:
    """Status and wall-clock timing for one bounded AX request.

    Attributes:
        operation: Public attribute/action name; never field content.
        status: Native AX error code returned by the operation.
        elapsed_ms: Observed monotonic duration, including scheduler overhead.
    """

    operation: str
    status: int
    elapsed_ms: float


@dataclass(frozen=True)
class FixtureResult:
    """Public fixture measurements; no field text is returned or logged.

    Attributes:
        mode: Small, oversized, or delayed synthetic field.
        pid: PID of the disposable process owned by the runner.
        field_found: Whether the fixture's AX text-area proxy was found.
        initial_utf16_units: Native length measured before text conversion.
        setter: Setter status/timing, absent for the oversized fixture.
        expected_readback_matches: Local comparison with public expected text.
        calls: Attribute-call status/timing observations, with no content.
        delayed_readbacks: Planned/actual elapsed milliseconds and match flags.
        stopped_target_calls: Calls made only after kernel-confirmed SIGSTOP.
        stopped_target_signal: Observed stop signal, absent if no stop was sampled.
    """

    mode: str
    pid: int
    field_found: bool
    initial_utf16_units: int | None
    setter: Call | None
    expected_readback_matches: bool | None
    calls: list[Call]
    delayed_readbacks: list[tuple[int, float, bool]]
    stopped_target_calls: list[Call]
    stopped_target_signal: int | None


@dataclass(frozen=True)
class NativeRun:
    """Own temporary producer paths until measurements and hashes are serialized.

    Attributes:
        fixture_source: Swift source within the runner-owned temporary directory.
        binary: Compiled executable; retained until its digest is serialized.
        compile_exit_status: Successful compiler exit status after bounded compilation.
        results: Public fixture observations after every child has been reaped.
    """

    fixture_source: Path
    binary: Path
    compile_exit_status: int
    results: list[FixtureResult]


class NativeBindings:
    """Own loaded CoreFoundation/ApplicationServices bindings for one probe.

    Attributes:
        cf: CoreFoundation library with explicit native ABI signatures.
        ax: ApplicationServices library with explicit native ABI signatures.
    Raises:
        OSError: A native framework cannot be loaded.
    """

    def __init__(self) -> None:
        self.cf = ctypes.CDLL(
            "/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation"
        )
        self.ax = ctypes.CDLL(
            "/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices"
        )
        self._configure_ax_bindings()
        self._configure_cf_types()
        self._configure_cf_strings()
        self._configure_cf_arrays()

    def _configure_ax_bindings(self) -> None:
        self.ax.AXIsProcessTrusted.restype = ctypes.c_bool
        self.ax.AXUIElementCreateApplication.argtypes = [ctypes.c_int]
        self.ax.AXUIElementCreateApplication.restype = ctypes.c_void_p
        self.ax.AXUIElementSetMessagingTimeout.argtypes = [
            ctypes.c_void_p,
            ctypes.c_float,
        ]
        self.ax.AXUIElementSetMessagingTimeout.restype = ctypes.c_int
        self.ax.AXUIElementCopyAttributeValue.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.POINTER(ctypes.c_void_p),
        ]
        self.ax.AXUIElementCopyAttributeValue.restype = ctypes.c_int
        self.ax.AXUIElementSetAttributeValue.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.c_void_p,
        ]
        self.ax.AXUIElementSetAttributeValue.restype = ctypes.c_int
        self.ax.AXUIElementGetTypeID.restype = ctypes.c_ulong

    def _configure_cf_types(self) -> None:
        self.cf.CFRelease.argtypes = [ctypes.c_void_p]
        self.cf.CFRetain.argtypes = [ctypes.c_void_p]
        self.cf.CFRetain.restype = ctypes.c_void_p
        self.cf.CFGetTypeID.argtypes = [ctypes.c_void_p]
        self.cf.CFGetTypeID.restype = ctypes.c_ulong
        self.cf.CFStringGetTypeID.restype = ctypes.c_ulong
        self.cf.CFArrayGetTypeID.restype = ctypes.c_ulong

    def _configure_cf_strings(self) -> None:
        self.cf.CFStringCreateWithCString.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_uint,
        ]
        self.cf.CFStringCreateWithCString.restype = ctypes.c_void_p
        self.cf.CFStringGetLength.argtypes = [ctypes.c_void_p]
        self.cf.CFStringGetLength.restype = ctypes.c_long
        self.cf.CFStringGetCString.argtypes = [
            ctypes.c_void_p,
            ctypes.c_void_p,
            ctypes.c_long,
            ctypes.c_uint,
        ]
        self.cf.CFStringGetCString.restype = ctypes.c_bool
        self.cf.CFStringGetMaximumSizeForEncoding.argtypes = [
            ctypes.c_long,
            ctypes.c_uint,
        ]
        self.cf.CFStringGetMaximumSizeForEncoding.restype = ctypes.c_long

    def _configure_cf_arrays(self) -> None:
        self.cf.CFArrayGetCount.argtypes = [ctypes.c_void_p]
        self.cf.CFArrayGetCount.restype = ctypes.c_long
        self.cf.CFArrayGetValueAtIndex.argtypes = [ctypes.c_void_p, ctypes.c_long]
        self.cf.CFArrayGetValueAtIndex.restype = ctypes.c_void_p
