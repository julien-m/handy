# N12 — smart paste crash, `CFRelease(NULL)` on the main thread

## Root cause

- `src-tauri/src/smart_paste/macos_cf.rs:45` (before the fix): `CfOwned::from_create_rule` was `(!reference.is_null()).then_some(Self(reference))`. `bool::then_some` takes its argument eagerly, so `Self(NULL)` was built and then dropped on the `None` path, and `CfOwned::drop` called `CFRelease(NULL)`. CoreFoundation aborts on that (`EXC_BREAKPOINT`, "CFRelease() called with NULL").
- Crashing call: `select_source_on_main` (`src-tauri/src/smart_paste/macos_keyboard.rs:99`), the job that `MacInputSources::select_source` posts to the main queue through `main_thread::run_on_main`. The chain is `coordinator` → `keyboard::switch_for_paste` (AC-011: current layout `com.apple.keylayout.French` is neither ABC nor US) → `select_source("com.apple.keylayout.ABC")` → `TISCreateInputSourceList(filter, false)`. That call returns NULL because ABC is not an enabled input source on this Mac, so `from_create_rule(NULL)` releases NULL.
- Proof from the crashed binary itself (UUID matches the `.ips`), in [disassembly-select-source.txt](disassembly-select-source.txt). Frame 2 (`0x2edf98`) is the return from `bl 0x1002ed410`. That function calls `CFStringCreateWithCharacters`, `CFDictionaryCreate` with `kTISPropertyInputSourceID`, `TISCreateInputSourceList`, `CFArrayGetCount` and `TISSelectInputSource`, which is `select_source_on_main`. At `0x1002ed4c8`, `cbz x0` (list NULL) branches to `0x1002ed508`/`0x1002ed50c`, then `bl _CFRelease` with `x0 = x23 = NULL` at `0x1002ed510`. The return address `0x2ed514` is frame 1 of the crash. The two other null paths show the same pattern (`0x1002ed460`, `0x1002ed518`).
- Enabled input sources (read-only `defaults read com.apple.HIToolbox`): French layout only, with no ABC or US layout.

## Fix

- `CfOwned` now stores a `NonNull<c_void>`, built with `NonNull::new(..).map(Self)`. A null `CfOwned` cannot be represented, so its drop can never call `CFRelease(NULL)`. A null Create/Copy result still returns `None`, and the callers keep reporting their error (`SelectFailed`, `SourceUnavailable`, `PostFailed`…). AC-011's fallback (try US, then send Cmd+V without switching) now runs as specified.
- `dispatch.rs`: the synthetic outcomes are no longer built (and logged) before the submit. `no actor reply` is logged only for a timeout or a dropped reply channel. A stopped actor logs `actor stopped, insertion refused`.

## Evidence

- Before the fix: [before-fix-test.log](before-fix-test.log). `a_null_create_rule_result_is_none_and_never_released` exits with 133 (SIGTRAP, the same CF abort). [before-fix-log-test.log](before-fix-log-test.log): the 3 log tests fail, and a delivered reply logs `no actor reply` twice, which is the native symptom.
- After the fix: [test-lib-features.log](test-lib-features.log) shows 1056 passed. Also in this folder: `fmt-check.log`, `clippy-features.log`, `clippy-default.log` (`-D warnings`, all targets) and `conventions-repo.summary.json` (PASS, 0 new; full report at its `receipt_path`).

## Still to validate natively (with Julien)

- A smart-paste dictation into Chrome on a French-only layout: no crash. Cmd+V is sent without switching (ABC and US are not selectable), and the paste lands with the French layout's V key.
- The same with ABC enabled: switch, paste, then restore French.
