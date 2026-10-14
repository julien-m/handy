"""Measure synthetic AX fields with validated CF ownership and bounded fixture waits."""

from __future__ import annotations

import ctypes
import os
import signal
import subprocess
import time

from native_ax_bindings import (
    Call,
    FIXTURE_WARMUP_SECONDS,
    MAX_CONVERT_UTF16_UNITS,
    MAX_FIXTURE_TREE_DEPTH,
    NativeBindings,
    REQUEST_TIMEOUT_SECONDS,
    STOPPED_TARGET_SAMPLE_COUNT,
    STOP_STATE_POLL_SECONDS,
    STOP_STATE_TIMEOUT_SECONDS,
    UTF8_ENCODING,
)


class AxProbe:
    """Own CF objects explicitly and set each messaged proxy's timeout.

    Attributes:
        cf: CoreFoundation ABI bindings; owned CF values must be released.
        ax: ApplicationServices ABI bindings for synthetic processes only.
        calls: Content-free observations collected for the current fixture.

    Raises:
        OSError: A required native framework cannot be loaded.
    """

    def __init__(self) -> None:
        self._bindings = NativeBindings()
        self.cf = self._bindings.cf
        self.ax = self._bindings.ax
        self.calls: list[Call] = []

    def _require_type(self, value: int, expected_type: int) -> None:
        if self.cf.CFGetTypeID(value) != expected_type:
            raise RuntimeError("Unexpected CoreFoundation boundary type")

    def string(self, text: str) -> int:
        """Create one owned CFString.

        Args:
            text: Public attribute name or synthetic fixture text.
        Returns:
            Owned CFString pointer; caller must call CFRelease exactly once.
        Raises:
            RuntimeError: Native allocation failed."""
        value = self.cf.CFStringCreateWithCString(None, text.encode(), UTF8_ENCODING)
        if value is None:
            raise RuntimeError("CFString allocation failed")
        return int(value)

    def attribute(self, element: int, name: str) -> tuple[int, int | None]:
        """Read an attribute with a proxy-local timeout and record status.

        Args:
            element: Borrowed, validated AX proxy for the owned fixture.
            name: Public native attribute name.
        Returns:
            Native status and optional owned CF value; caller releases any value.
        Raises:
            RuntimeError: Attribute allocation or timeout setup failed."""
        attribute = self.string(name)
        value = ctypes.c_void_p()
        try:
            status = self.ax.AXUIElementSetMessagingTimeout(
                element, REQUEST_TIMEOUT_SECONDS
            )
            if status != 0:
                raise RuntimeError(f"AX timeout setup failed: {status}")
            start = time.monotonic()
            status = self.ax.AXUIElementCopyAttributeValue(
                element, attribute, ctypes.byref(value)
            )
            self.calls.append(
                Call(name, status, round((time.monotonic() - start) * 1000, 3))
            )
            return status, value.value
        finally:
            self.cf.CFRelease(attribute)

    def text_matches(self, value: int, expected: str) -> bool:
        """Compare public synthetic text locally without emitting it.

        Args:
            value: Borrowed CF value, validated as CFString before conversion.
            expected: Public fixture expectation, compared only in memory.
        Returns:
            Whether the converted value equals the expectation; ownership stays caller's.
        Raises:
            RuntimeError: Type, bounded length, sizing, or conversion is invalid."""
        self._require_type(value, self.cf.CFStringGetTypeID())
        units = self.cf.CFStringGetLength(value)
        if units < 0 or units > MAX_CONVERT_UTF16_UNITS:
            raise RuntimeError("Synthetic string exceeds conversion guard")
        maximum = self.cf.CFStringGetMaximumSizeForEncoding(units, UTF8_ENCODING)
        if maximum < 0:
            raise RuntimeError("CFString UTF-8 sizing failed")
        # Add CoreFoundation's required terminating NUL to its maximum UTF-8 size.
        buffer = ctypes.create_string_buffer(maximum + 1)
        if not self.cf.CFStringGetCString(value, buffer, len(buffer), UTF8_ENCODING):
            raise RuntimeError("CFString conversion failed")
        return buffer.value.decode() == expected

    def find_field(self, element: int, depth: int = 0) -> int | None:
        """Traverse only the fixture's AX tree and retain the text-area proxy.

        Args:
            element: Borrowed validated fixture application or child AX proxy.
            depth: Diagnostic recursion depth, bounded by the fixture limit.
        Returns:
            Optional owned text-area proxy; caller releases a returned proxy.
        Raises:
            RuntimeError: A CF boundary value has an invalid native type."""
        status, role = self.attribute(element, "AXRole")
        if role is not None:
            try:
                if status == 0 and self.text_matches(role, "AXTextArea"):
                    return int(self.cf.CFRetain(element))
            finally:
                self.cf.CFRelease(role)
        if depth >= MAX_FIXTURE_TREE_DEPTH:
            return None
        status, children = self.attribute(
            element, "AXWindows" if depth == 0 else "AXChildren"
        )
        if children is None:
            return None
        try:
            if status != 0:
                return None
            self._require_type(children, self.cf.CFArrayGetTypeID())
            return self._search_children(children, depth)
        finally:
            self.cf.CFRelease(children)

    def _search_children(self, children: int, depth: int) -> int | None:
        for index in range(self.cf.CFArrayGetCount(children)):
            child = self.cf.CFArrayGetValueAtIndex(children, index)
            if child is None:
                raise RuntimeError("Null AX child")
            self._require_type(child, self.ax.AXUIElementGetTypeID())
            found = self.find_field(int(child), depth + 1)
            # Retain the located field before releasing the containing array.
            if found is not None:
                return found
        return None

    # @spec FR-011: AX feasibility probe only — .specs/features/007-smart-paste/spec.md#fr-011
    def write_fixture(self, field: int) -> tuple[Call, float]:
        """Replace the public surrogate-pair selection exactly once.

        Args:
            field: Borrowed, validated synthetic text-area proxy.
        Returns:
            Content-free setter observation and monotonic start instant.
        Raises:
            RuntimeError: CF allocation or timeout setup failed.

        Temporary CF strings are released here; the caller still owns the field."""
        name, replacement = self.string("AXSelectedText"), self.string("synthetic")
        try:
            status = self.ax.AXUIElementSetMessagingTimeout(
                field, REQUEST_TIMEOUT_SECONDS
            )
            if status != 0:
                raise RuntimeError(f"AX timeout setup failed: {status}")
            start = time.monotonic()
            status = self.ax.AXUIElementSetAttributeValue(field, name, replacement)
            return Call(
                "AXSelectedText", status, round((time.monotonic() - start) * 1000, 3)
            ), start
        finally:
            self.cf.CFRelease(name)
            self.cf.CFRelease(replacement)

    def _find_ready_field(self, application: int) -> int | None:
        # Cold AppKit registration may reject the first requests; each retry follows
        # a fresh recorded unavailable result and stops within a one-second warmup.
        deadline = time.monotonic() + FIXTURE_WARMUP_SECONDS
        while time.monotonic() < deadline:
            field = self.find_field(application)
            if field is not None:
                return field
        return None

    def _read_units(self, field: int) -> int:
        status, value = self.attribute(field, "AXValue")
        if value is None:
            raise RuntimeError(f"Synthetic AXValue unavailable: {status}")
        try:
            if status != 0:
                raise RuntimeError(f"Synthetic AXValue unavailable: {status}")
            self._require_type(value, self.cf.CFStringGetTypeID())
            return int(self.cf.CFStringGetLength(value))
        finally:
            self.cf.CFRelease(value)

    def _readbacks(
        self, field: int, start: float, mode: str
    ) -> list[tuple[int, float, bool]]:
        observations: list[tuple[int, float, bool]] = []
        points = [149, 150, 151, 180] if mode == "delayed" else [0]
        for point in points:
            # Measure prescribed monotonic boundary points; actual elapsed is recorded.
            remaining = start + point / 1000 - time.monotonic()
            if remaining > 0:
                time.sleep(remaining)
            status, value = self.attribute(field, "AXValue")
            if value is None:
                raise RuntimeError("Synthetic readback unavailable")
            try:
                if status != 0:
                    raise RuntimeError("Synthetic readback unavailable")
                verified = self.text_matches(value, "AsyntheticZ")
            finally:
                self.cf.CFRelease(value)
            observations.append(
                (point, round((time.monotonic() - start) * 1000, 3), verified)
            )
        return observations

    def _wait_stopped(self, process: subprocess.Popen[str]) -> int:
        deadline = time.monotonic() + STOP_STATE_TIMEOUT_SECONDS
        while time.monotonic() < deadline:
            pid, status = os.waitpid(process.pid, os.WNOHANG | os.WUNTRACED)
            if pid == process.pid:
                if os.WIFSTOPPED(status) and os.WSTOPSIG(status) == signal.SIGSTOP:
                    return status
                if os.WIFEXITED(status) or os.WIFSIGNALED(status):
                    process.returncode = os.waitstatus_to_exitcode(status)
                    raise RuntimeError("Synthetic fixture exited before confirmed stop")
                raise RuntimeError("Unexpected synthetic fixture state")
            if pid != 0:
                raise RuntimeError("Unexpected child PID from waitpid")
            time.sleep(STOP_STATE_POLL_SECONDS)
        raise RuntimeError("Synthetic fixture stop confirmation timed out")

    def _stopped_calls(
        self, process: subprocess.Popen[str], field: int
    ) -> tuple[int, list[Call]]:
        stopped: list[Call] = []
        os.kill(process.pid, signal.SIGSTOP)
        try:
            status = self._wait_stopped(process)
            for _ in range(STOPPED_TARGET_SAMPLE_COUNT):
                _, value = self.attribute(field, "AXValue")
                stopped.append(self.calls[-1])
                if value is not None:
                    self.cf.CFRelease(value)
            return os.WSTOPSIG(status), stopped
        finally:
            if process.poll() is None:
                os.kill(process.pid, signal.SIGCONT)
