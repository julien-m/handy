# .specs — Handy

> Specification registry for Handy. All artifacts produced by LiveSpec are indexed here.
>
> Last updated: 2026-10-13

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
| 005 | Custom AI Prompts | Planned | 2026-10-12 | 2026-10-12 | [spec](features/005-custom-ai-prompts/spec.md) |
| 006 | Destination Application | Planned | 2026-10-12 | 2026-10-13 | [spec](features/006-destination-application/spec.md) |
| 007 | Smart Paste | In Progress | 2026-10-12 | 2026-10-13 | [spec](features/007-smart-paste/spec.md) |
| 008 | Settings CLI | Planned | 2026-10-12 | 2026-10-12 | [spec](features/008-settings-cli/spec.md) |
| 009 | PIN Mode and Enter Rule | Planned | 2026-10-12 | 2026-10-13 | [spec](features/009-pin-mode-and-enter-rule/spec.md) |
| 010 | Recovery Bubble | Planned | 2026-10-12 | 2026-10-13 | [spec](features/010-recovery-bubble/spec.md) |
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
| 2026-10-13 | Feature | Smart Paste: record native feasibility and explicit cache/late-write limits; settings integration awaits 008 |
| 2026-10-13 | Feature | [Feature 007] Smart paste plan rewritten from 2026-10-13 spec — 9 steps, 4 diagrams; AX-first and receipt-only restoration; native/design release gates |
| 2026-10-13 | Feature | [Feature 010] Plan refined: current 007 AX/readback contract and explicit AC-006 native acceptance |
| 2026-10-13 | Feature | [Feature 009] Plan refined: verified AX success for Enter, uncertain suppression, LAST-read restoration and current-only destination cross — plan only |
| 2026-10-13 | Feature | [Feature 006] Plan refined: one-click destination removal, eight smart-paste outcomes and owner-bound restoration tests |
| 2026-10-13 | Feature | [Feature 010] Plan refined: uncertain recovery, direct AX verified handoff, read-confirmed last-read restoration and Copy guard |
| 2026-10-12 | Feature | Plan refined: Recovery Bubble — smart paste outcomes and deliberate Copy/changeCount guard replace retired clipboard-window integration; Windows reliable paste retained |
| 2026-10-12 | Feature | [Feature 009] Plan refined: align Enter with smart paste outcomes, shared output smart_paste configuration and changeCount restoration guard; preserve Windows reliable paste |
| 2026-10-12 | Feature | Smart Paste — 9 implementation steps, 4 diagrams, 008 output domain, bounded snapshots and truthful outcomes |
| 2026-10-12 | Feature | [Feature 006] Plan refined: smart-paste terminal results replace retired clipboard-window integration; Windows reliable paste preserved |
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
