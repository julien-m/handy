"""Regenerate pinned Unicode tables with staged writes and rollback on errors."""

import hashlib
import json
import os
import tempfile
import urllib.error
import urllib.request
from builtins import ExceptionGroup
from pathlib import Path

ROOT = Path(__file__).parent
UNICODE_VERSION = "17.0.0"
BASE = f"https://www.unicode.org/Public/{UNICODE_VERSION}/ucd/"
LICENSE_URL = "https://www.unicode.org/license.txt"
LICENSE_SHA256 = "e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96"
NETWORK_TIMEOUT_SECONDS = 30
MAX_DOWNLOAD_RETRIES = 2
TRANSIENT_HTTP_STATUSES = {408, 429, 500, 502, 503, 504}
PINNED = {
    "CaseFolding.txt": "ff8d8fefbf123574205085d6714c36149eb946d717a0c585c27f0f4ef58c4183",
    "UnicodeData.txt": "2e1efc1dcb59c575eedf5ccae60f95229f706ee6d031835247d843c11d96470c",
    "Scripts.txt": "9f5e50d3abaee7d6ce09480f325c706f485ae3240912527e651954d2d6b035bf",
    "PropList.txt": "130dcddcaadaf071008bdfce1e7743e04fdfbc910886f017d9f9ac931d8c64dd",
    "ScriptExtensions.txt": "ec2107e58825a1586acee8e0911ce18260394ac8b87e535ca325f1ccbeb06bc6",
}


def fetch(url: str) -> bytes:
    """Download official development data; performs network I/O only.

    Args:
        url: Official Unicode source URL.
    Returns:
        Response bytes after at most two transient-failure retries.
    Raises:
        URLError, TimeoutError: A permanent error or exhausted transient retries.
    """
    # Retry fresh transport failures only; content checksum failures never retry.
    for attempt in range(MAX_DOWNLOAD_RETRIES + 1):
        try:
            with urllib.request.urlopen(
                url, timeout=NETWORK_TIMEOUT_SECONDS
            ) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            if (
                error.code not in TRANSIENT_HTTP_STATUSES
                or attempt == MAX_DOWNLOAD_RETRIES
            ):
                raise
        except (urllib.error.URLError, TimeoutError):
            if attempt == MAX_DOWNLOAD_RETRIES:
                raise
    raise RuntimeError("Download retry loop exhausted without a result")


def property_rows(text: str) -> list[tuple[int, int, str]]:
    """Decode UCD property records without side effects.

    Args:
        text: UTF-8-decoded property file content.
    Returns:
        Inclusive start/end ranges with property names.
    Raises:
        ValueError: A record has malformed fields or hexadecimal values.
    """
    rows = []
    for line in text.splitlines():
        value = line.split("#", 1)[0].strip()
        if not value:
            continue
        interval, property_name = value.split(";", 1)
        endpoints = interval.strip().split("..")
        rows.append(
            (int(endpoints[0], 16), int(endpoints[-1], 16), property_name.strip())
        )
    return rows


def letter_digit_points(text: str) -> set[int]:
    """Expand UnicodeData Letter and Decimal_Number ranges without side effects.

    Args:
        text: UTF-8-decoded UnicodeData content.
    Returns:
        Letter and decimal-number scalar values.
    Raises:
        ValueError: A record has malformed fields or hexadecimal values.
    """
    points: set[int] = set()
    start = 0
    for line in text.splitlines():
        code, name, category, *_ = line.split(";")
        point = int(code, 16)
        if name.endswith(", First>"):
            start = point
            continue
        if category.startswith("L") or category == "Nd":
            first = start if name.endswith(", Last>") else point
            points.update(range(first, point + 1))
    return points


def coalesce(points: set[int]) -> list[tuple[int, int]]:
    """Compress scalar values into sorted ranges without side effects.

    Args:
        points: Unicode scalar values to compress.
    Returns:
        Inclusive, non-overlapping intervals.
    """
    result: list[tuple[int, int]] = []
    for point in sorted(points):
        if result and result[-1][1] + 1 == point:
            result[-1] = (result[-1][0], point)
        else:
            result.append((point, point))
    return result


def _property_points(text: str, properties: set[str]) -> set[int]:
    return {
        point
        for start, end, prop in property_rows(text)
        if prop in properties
        for point in range(start, end + 1)
    }


def mark_points(text: str) -> set[int]:
    """Collect General_Category Mn, Mc and Me scalars without side effects.

    Args:
        text: UTF-8-decoded UnicodeData content.
    Returns:
        Combining mark scalar values; UnicodeData lists no mark as a range.
    Raises:
        IndexError, ValueError: A record has malformed fields or hexadecimal values.
    """
    return {
        int(row.split(";")[0], 16)
        for row in text.splitlines()
        if row.split(";")[2] in {"Mn", "Mc", "Me"}
    }


def _cjk_points(
    sources: dict[str, str], letters: set[int], marks: set[int]
) -> set[int]:
    cjk = _property_points(
        sources["Scripts.txt"], {"Han", "Hiragana", "Katakana", "Hangul"}
    )
    # Shared kana Letter/Mark characters include prolonged marks and combining dakuten.
    for start, end, scripts in property_rows(sources["ScriptExtensions.txt"]):
        if {"Hira", "Kana", "Hang"}.intersection(scripts.split()):
            cjk.update(
                point
                for point in range(start, end + 1)
                if point in letters or point in marks
            )
    return cjk


def build_records(sources: dict[str, str]) -> list[str]:
    """Extract full default folding and pinned properties without side effects.

    Args:
        sources: Checked source filenames and decoded UCD contents.
    Returns:
        Compact folding, whitespace, category and script records.
    Raises:
        KeyError, ValueError: A required source is missing or malformed.
    """
    records = [
        f"# Unicode {UNICODE_VERSION}; F=full C+F folding, W=White_Space, L=Letter/Nd, "
        "M=Mn/Mc/Me, J=Join_Control, K=CJK"
    ]
    for line in sources["CaseFolding.txt"].splitlines():
        fields = line.split("#", 1)[0].split(";")
        if len(fields) > 2 and fields[1].strip() in {"C", "F"}:
            records.append(f"F;{fields[0].strip()};{fields[2].strip()}")
    for label, points in _property_tables(sources):
        records.extend(f"{label};{start:X};{end:X}" for start, end in coalesce(points))
    return records


def _property_tables(sources: dict[str, str]) -> list[tuple[str, set[int]]]:
    letters = letter_digit_points(sources["UnicodeData.txt"])
    marks = mark_points(sources["UnicodeData.txt"])
    white = _property_points(sources["PropList.txt"], {"White_Space"})
    # ZWNJ/ZWJ join letters inside one word (Persian, Urdu, joined sequences).
    joiners = _property_points(sources["PropList.txt"], {"Join_Control"})
    cjk = _cjk_points(sources, letters, marks)
    return [
        ("W", white),
        ("L", letters),
        ("M", marks),
        ("J", joiners),
        ("K", cjk),
    ]


def _checked_download(url: str, expected: str) -> bytes:
    data = fetch(url)
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError(f"Unicode checksum mismatch: {url}")
    return data


def _prepare_outputs() -> dict[str, bytes]:
    sources = {
        name: _checked_download(BASE + name, expected).decode("utf-8")
        for name, expected in PINNED.items()
    }
    license_data = _checked_download(LICENSE_URL, LICENSE_SHA256)
    generated = ("\n".join(build_records(sources)) + "\n").encode()
    provenance = {
        "version": UNICODE_VERSION,
        "sources": [
            {"url": BASE + name, "sha256": expected}
            for name, expected in PINNED.items()
        ],
        "generated_sha256": hashlib.sha256(generated).hexdigest(),
        "license": {"url": LICENSE_URL, "sha256": LICENSE_SHA256},
        "regenerate": "python3 src-tauri/src/word_replacements/unicode_generate.py",
    }
    return {
        "unicode-data.txt": generated,
        "unicode-license.txt": license_data,
        "unicode-sources.json": (json.dumps(provenance, indent=2) + "\n").encode(),
    }


def _stage_file(destination: Path, data: bytes) -> Path:
    descriptor, temporary = tempfile.mkstemp(
        prefix=f".{destination.name}.", dir=destination.parent
    )
    staged = Path(temporary)
    try:
        with os.fdopen(descriptor, "wb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        if (
            hashlib.sha256(staged.read_bytes()).digest()
            != hashlib.sha256(data).digest()
        ):
            raise ValueError(f"Staged Unicode output mismatch: {destination.name}")
        return staged
    except (OSError, ValueError):
        staged.unlink(missing_ok=True)
        raise


def _restore_outputs(
    root: Path, previous: dict[str, bytes | None], replaced: list[str]
) -> None:
    failures: list[Exception] = []
    # Restore every successfully replaced destination, even if another rollback fails.
    for name in reversed(replaced):
        staged = None
        try:
            data = previous[name]
            if data is None:
                root.joinpath(name).unlink(missing_ok=True)
            else:
                staged = _stage_file(root / name, data)
                os.replace(staged, root / name)
        except (OSError, ValueError) as error:
            failures.append(error)
        finally:
            if staged is not None:
                staged.unlink(missing_ok=True)
    if failures:
        raise ExceptionGroup("Unicode output rollback failed", failures)


def _publish_outputs(root: Path, outputs: dict[str, bytes]) -> None:
    previous = {
        name: (root / name).read_bytes() if (root / name).exists() else None
        for name in outputs
    }
    staged: dict[str, Path] = {}
    replaced: list[str] = []
    try:
        # Stage and validate the entire set before replacing any destination.
        for name, data in outputs.items():
            staged[name] = _stage_file(root / name, data)
        for name, temporary in staged.items():
            os.replace(temporary, root / name)
            replaced.append(name)
    except (OSError, ValueError):
        _restore_outputs(root, previous, replaced)
        raise
    finally:
        for temporary in staged.values():
            temporary.unlink(missing_ok=True)


def main() -> None:
    """Download checked inputs and publish three generated files in ROOT.

    Returns:
        None.
    Raises:
        URLError, TimeoutError, ValueError: Download or source verification failed.
        OSError, ExceptionGroup: Staging, replacement or rollback failed.
    Side Effects:
        Performs network I/O and writes sibling temporary and generated files.
        Each replacement is atomic; caught late failures restore prior files.
        This does not guarantee crash atomicity across multiple files.
    """
    _publish_outputs(ROOT, _prepare_outputs())


if __name__ == "__main__":
    main()
