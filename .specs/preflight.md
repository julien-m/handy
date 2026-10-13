# Preflight — Handy

> Checks required before autonomous implementation of the program features (001–010). Regenerated on 2026-10-13 from the stack, BUILD.md, AGENTS.md and the feature plans.

## Tooling

### Bun
- **binary:** `bun`
- **verify:** `bun --version`
- **install:** `curl -fsSL https://bun.sh/install | bash`
- **severity:** critical
- **source:** stack (_default.md)

### Frontend dependencies
- **binary:** `bun`
- **verify:** `test -d node_modules/@tauri-apps`
- **install:** `bun install`
- **severity:** critical
- **source:** stack (_default.md)

### Rust toolchain
- **binary:** `cargo`
- **verify:** `cargo --version && rustc --version`
- **install:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **severity:** critical
- **source:** stack (_default.md)

### CMake
- **binary:** `cmake`
- **verify:** `cmake --version`
- **install:** `brew install cmake`
- **severity:** critical
- **source:** stack (_default.md) — required by transcribe-cpp-sys build script (verified by cargo check on 2026-10-13)

### Xcode Command Line Tools
- **binary:** `xcrun`
- **verify:** `xcode-select -p && swift --version`
- **install:** human
- **severity:** critical
- **source:** manual

### Silero VAD model
- **binary:** `test`
- **verify:** `test -s src-tauri/resources/models/silero_vad_v4.onnx`
- **install:** `mkdir -p src-tauri/resources/models && curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx`
- **severity:** critical
- **source:** manual

### LiveSpec CLI
- **binary:** `livespec`
- **verify:** `livespec --help`
- **install:** human
- **severity:** critical
- **source:** manual

### Penflow CLI
- **binary:** `penflow`
- **verify:** `~/projects/penflow/.venv/bin/penflow --help`
- **install:** `cd ~/projects/penflow && python3 -m venv .venv && .venv/bin/pip install -e ".[dev]"`
- **severity:** critical
- **source:** spec (UI features 001–007, 009, 010)

### pen.dev application
- **binary:** `open`
- **verify:** `test -d ~/Applications/Pen.app || test -d /Applications/Pen.app`
- **install:** human
- **severity:** warning
- **source:** spec (UI features 001–007, 009, 010)

## Authentication

No application authentication or deployment is required to build and test Handy locally.

## Tokens

None required. AI post-processing tests (005) use fake LLM clients; manual checks with a real provider use the tester's own key entered in Handy's settings, never stored by the implementation pipeline.

<!-- preflight:custom:start -->
<!-- Add manual checks here. Use the same ### format as above. Set source: manual -->
<!-- preflight:custom:end -->
