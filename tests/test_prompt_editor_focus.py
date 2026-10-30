"""Direct browser regression for LiveSpec bugfix receipts; browser evidence only.

Use installed Chrome and the project-local Playwright Python package. This does
not certify focus in a native Tauri/WebKit window.
"""
from collections.abc import Iterator
import os
import socket
import subprocess
import time
import urllib.request
from pathlib import Path

import pytest
from playwright.sync_api import sync_playwright


@pytest.fixture(scope="module")
def editor_url() -> Iterator[str]:
    """Serve the real editor harness in an isolated Vite process."""
    root = Path(__file__).resolve().parents[1]
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    url = f"http://127.0.0.1:{port}/tests/harness/prompt-editor.html?webkitFocus"
    with open(os.devnull, "w") as output:
        server = subprocess.Popen(
            ["bunx", "vite", "--host", "127.0.0.1", "--port", str(port), "--strictPort"],
            cwd=root, stdout=output, stderr=output,
        )
        try:
            deadline = time.monotonic() + 10
            while True:
                try:
                    with urllib.request.urlopen(url, timeout=1):
                        break
                except OSError:
                    if server.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError("Isolated Vite server did not become ready")
                    time.sleep(0.05)
            yield url
        finally:
            server.terminate()
            server.wait(timeout=5)


@pytest.mark.acceptance
def test_save_returns_focus_without_focusout(editor_url: str) -> None:
    """Saving rescues actual DOM focus even when focusout cannot reach the section."""
    with sync_playwright() as driver:
        browser = driver.chromium.launch(channel="chrome", headless=True)
        try:
            page = browser.new_page()
            page.goto(editor_url)
            name = page.get_by_test_id("prompt-settings-editor-form-name")
            name.fill("Meeting notes (short)")
            save = page.get_by_test_id("prompt-settings-editor-save")
            save.focus()
            page.keyboard.press("Enter")
            page.get_by_test_id("prompt-settings-editor-saved").wait_for(state="visible")
            page.evaluate(
                "() => new Promise(resolve => "
                "requestAnimationFrame(() => requestAnimationFrame(resolve)))"
            )
            active = page.evaluate(
                "() => ({tag: document.activeElement?.tagName, "
                "text: document.activeElement?.textContent})"
            )
            assert active == {"tag": "H2", "text": "Edit prompt"}
            assert page.evaluate("() => window.harness.backend.updates") == 1
        finally:
            browser.close()
