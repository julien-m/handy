# .specs — Handy

> Specification registry for Handy. All artifacts produced by LiveSpec are indexed here.
>
> Last updated: 2026-10-15

---

## System Files

| Document | Description |
|---|---|
| [spec-system.md](spec-system.md) | Universal spec rules (read first) |
| [constitution.md](constitution.md) | Architecture principles |
| [project.md](project.md) | Project profile (vision, users, constraints) |
| [stacks/_default.md](stacks/_default.md) | Current tech stack |
| [testing/strategy.md](testing/strategy.md) | Testing strategy |
| [changelog.md](changelog.md) | Global changelog |
| [roadmap.md](roadmap.md) | Feature backlog (MVP / Post-MVP / Future) |

---

## Design

| Document | Description |
|---|---|
| [design/](design/) | UI mockups and screen references |
| [design/changelog.md](design/changelog.md) | Design change history |

---

## Features

<!-- readme:features:start -->
| # | Feature | Status | Created | Updated | Spec |
|---|---|---|---|---|---|
| 001 | Microphone Priority | Planned | 2026-10-09 | 2026-10-12 | [spec](features/001-microphone-priority/spec.md) |
| 002 | Word Replacements | Planned | 2026-10-12 | 2026-10-12 | [spec](features/002-word-replacements/spec.md) |
| 003 | Recognition Vocabulary | Planned | 2026-10-12 | 2026-10-12 | [spec](features/003-recognition-vocabulary/spec.md) |
| 004 | Multiple Dictation Shortcuts | Planned | 2026-10-12 | 2026-10-12 | [spec](features/004-multiple-dictation-shortcuts/spec.md) |
| 005 | Custom AI Prompts | Planned | 2026-10-12 | 2026-10-14 | [spec](features/005-custom-ai-prompts/spec.md) |
| 006 | Destination Application | Planned | 2026-10-12 | 2026-10-14 | [spec](features/006-destination-application/spec.md) |
| 007 | Smart Paste | In Progress | 2026-10-12 | 2026-10-14 | [spec](features/007-smart-paste/spec.md) |
| 008 | Settings CLI | In Progress | 2026-10-12 | 2026-10-13 | [spec](features/008-settings-cli/spec.md) |
| 009 | PIN Mode and Enter Rule | In Progress | 2026-10-12 | 2026-10-15 | [spec](features/009-pin-mode-and-enter-rule/spec.md) |
| 010 | Recovery Bubble | Planned | 2026-10-12 | 2026-10-14 | [spec](features/010-recovery-bubble/spec.md) |
| 011 | Dictation Field Return | Planned | 2026-10-15 | 2026-10-15 | [spec](features/011-dictation-field-return/spec.md) |
<!-- readme:features:end -->


---

## Architecture Decisions

<!-- readme:decisions:start -->
| ADR | Decision | Date | Status |
|---|---|---|---|
| [ADR-001](stacks/decisions/ADR-001-handy-app-stack.md) | React + Vite as web application stack | 2026-10-09 | Active |
| [ADR-002](stacks/decisions/ADR-002-testing-strategy.md) | Browser evidence testing strategy | 2026-10-09 | Active |
| [Observed stack](stacks/decisions/ADR-bootstrap-f36751c3f59c.md) | Observed stack retention | 2026-10-09 | Accepted |
<!-- readme:decisions:end -->

---

## Recent Activity

> Latest entries from [changelog.md](changelog.md).

<!-- readme:activity:start -->
| Date | Type | Description |
|---|---|---|
| 2026-10-15 | Feature | [Feature 009] Implementation in progress: session pin, single Automatic Enter authority and destination CLI behind destination-return (automated evidence in four feature combinations; macOS native proofs pending with Julien) |
| 2026-10-15 | Plan | [Feature 011] Plan created: Dictation Field Return — 6 implementation steps, 4 diagrams (input read after 006's session commit, focus restored after activation, verified in 006's guard before 010's verdict) |
| 2026-10-15 | Spec | [Feature 011] Spec created: Dictation Field Return — 4 stories, 8 AC, 5 FR (the dictation's input remembered at start, focused, raised and verified before the paste, fallback to 006/010) |
| 2026-10-14 | Feature | [Features 004, 005, 006, 007, 008] UI hardening: shortcut draft exit and one-shot record requests (004), return-to-app row hidden while its gate is closed (006), Reload failure kept in the conflict alert and history limit saves tested (008), indicator window bounded by its monitor, menu timers cleaned, closed-gate test compared with dev (005/006/007) |
| 2026-10-14 | Feature | [Feature 010] Plan refined: permission_missing aligned on the spec (no keystroke sent, so no possible-partial-insertion detail) |
| 2026-10-14 | Feature | [Feature 010] Spec refined: focus in another process inserts (Key Entities), history claim conditional on a successful save, no 'some text' detail for permission_missing, destination-return gate recorded, a new dictation closes the bubble even before 4 s |
| 2026-10-14 | Feature | [Feature 009] Spec refined: AC-003 reasons split into pinned_app_not_running and activation_failed, destination-return gate recorded; plan keeps dev's 50 ms pause before Enter |
| 2026-10-14 | Feature | [Feature 010] Plan refined: round 4 review ingested, 010 behind 006's destination-return gate (closed build keeps dev's paste-error), before_insertion synchronous in the first guard check with a ~50 ms AX budget, names aligned on delivered 006/007 |
| 2026-10-14 | Feature | [Feature 009] Plan refined: round 4 review ingested, 009 behind 006's destination-return gate (closed build keeps dev's auto_submit, migration never rewrites it), Enter keyed by dictation and generation, status limited to the pinned app |
| 2026-10-14 | Feature | [Feature 007] Smart paste wired into dictation through 006's insertion contract (006 guard context, method_outcome, disposition, Enter and late-read hooks), behind the smart-paste-dispatch gate closed by default; dev (004, 006) merged; native AC-016 matrix pending with Julien |
<!-- readme:activity:end -->

---

*Maintained automatically by LiveSpec commands. Do not remove section markers.*

<!-- finalize:spec-specify:2026-10-12:efcdfaa5 -->

<!-- finalize:spec-specify:2026-10-12:eba866f3 -->

<!-- finalize:spec-specify:2026-10-12:d5c7ff88 -->

<!-- finalize:spec-specify:2026-10-12:bf4ea80d -->

<!-- finalize:spec-specify:2026-10-12:e57cf592 -->

<!-- finalize:spec-specify:2026-10-12:aa9ea82f -->

<!-- finalize:spec-specify:2026-10-12:2ddb4fdd -->

<!-- finalize:spec-specify:2026-10-12:04b5e2f9 -->

<!-- finalize:spec-specify:2026-10-12:f1da6064 -->

<!-- finalize:spec-specify:2026-10-12:d83a0c90 -->

<!-- finalize:spec-plan:2026-10-12:4dabd6e8 -->

<!-- finalize:spec-plan:2026-10-12:7f8d42a3 -->

<!-- finalize:spec-plan:2026-10-12:218e2fcd -->

<!-- finalize:spec-plan:2026-10-12:cbfc6dff -->

<!-- finalize:spec-plan:2026-10-12:ea4a26ff -->

<!-- finalize:spec-plan:2026-10-12:86a3d72b -->

<!-- finalize:spec-plan:2026-10-12:688a7be7 -->

<!-- finalize:spec-plan:2026-10-12:1f19f822 -->

<!-- finalize:spec-plan:2026-10-12:5e13c1dd -->

<!-- finalize:spec-plan:2026-10-12:5e51aa59 -->

<!-- finalize:spec-plan:2026-10-12:70754b4a -->

<!-- finalize:spec-plan:2026-10-12:12d2f055 -->

<!-- finalize:spec-plan:2026-10-12:32171392 -->

<!-- finalize:spec-refine:2026-10-12:3471b770 -->

<!-- finalize:spec-plan:2026-10-12:7b940570 -->

<!-- finalize:spec-plan:2026-10-12:134acf63 -->

<!-- finalize:spec-plan:2026-10-12:307d8e77 -->

<!-- finalize:spec-plan:2026-10-12:6e7fa9b3 -->

<!-- finalize:spec-plan:2026-10-12:1cac4499 -->

<!-- finalize:spec-plan:2026-10-13:dd714906 -->

<!-- finalize:spec-plan:2026-10-13:53c4754c -->

<!-- finalize:spec-plan:2026-10-13:183e40c6 -->

<!-- finalize:spec-plan:2026-10-13:09d4dd98 -->

<!-- finalize:spec-plan:2026-10-13:17322c07 -->

<!-- finalize:spec-feature:2026-10-13:72e02927 -->

<!-- finalize:spec-plan:2026-10-13:e428ba3d -->

<!-- finalize:spec-implement:2026-10-13:61bf0cb4 -->

<!-- finalize:spec-implement:2026-10-13:9b1995bc -->

<!-- finalize:spec-plan:2026-10-13:ff08720c -->

<!-- finalize:spec-plan:2026-10-13:1953f656 -->

<!-- finalize:spec-refine:2026-10-14:d424af86 -->

<!-- finalize:spec-refine:2026-10-14:b0be88bc -->

<!-- finalize:spec-refine:2026-10-14:0feee4a1 -->

<!-- finalize:spec-refine:2026-10-14:8325a713 -->

<!-- finalize:spec-refine:2026-10-14:f275f226 -->

<!-- finalize:spec-refine:2026-10-14:61308064 -->

<!-- finalize:spec-refine:2026-10-14:86e43d44 -->

<!-- finalize:spec-refine:2026-10-14:2190dbf9 -->

<!-- finalize:spec-refine:2026-10-14:d7cad1e7 -->

<!-- finalize:spec-refine:2026-10-14:e2c204ed -->

<!-- finalize:spec-refine:2026-10-14:48c67c3f -->

<!-- finalize:spec-refine:2026-10-14:19f333f0 -->

<!-- finalize:spec-refine:2026-10-14:fb255547 -->

<!-- finalize:spec-refine:2026-10-14:b5422949 -->

<!-- finalize:spec-refine:2026-10-14:8ecc2640 -->

<!-- finalize:spec-refine:2026-10-14:3d66e47e -->

<!-- finalize:spec-specify:2026-10-15:5625200d -->

<!-- finalize:spec-plan:2026-10-15:67c12fe0 -->

<!-- finalize:spec-implement:2026-10-15:851e4c61 -->
