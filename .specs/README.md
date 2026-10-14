# .specs — Handy

> Specification registry for Handy. All artifacts produced by LiveSpec are indexed here.
>
> Last updated: 2026-10-14

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
| 006 | Destination Application | Planned | 2026-10-12 | 2026-10-14 | [spec](features/006-destination-application/spec.md) |
| 007 | Smart Paste | Planned | 2026-10-12 | 2026-10-13 | [spec](features/007-smart-paste/spec.md) |
| 008 | Settings CLI | In Progress | 2026-10-12 | 2026-10-13 | [spec](features/008-settings-cli/spec.md) |
| 009 | PIN Mode and Enter Rule | Planned | 2026-10-12 | 2026-10-14 | [spec](features/009-pin-mode-and-enter-rule/spec.md) |
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
| 2026-10-14 | Feature | [Feature 009] Plan refined: 008 sources cited as code paths (plan review preparable in 20 batches), explicit AC-006 (13/13), DestinationManager::claim_enter, pinned_app_not_running and activation_failed aligned with 010 |
| 2026-10-14 | Feature | [Feature 006] Plan refined: 008 sources cited as code paths (plan review preparable in 1 batch), four ValidationContext constructions, DestinationManager::claim_enter for 009, pinned_app_not_running reason |
| 2026-10-14 | Feature | [Feature 005] Plan refined: plan review preparable again, Step 2 ordered, offline prompt list justified |
| 2026-10-14 | Feature | [Feature 003] Plan refined: plan review preparable again, durable-error rule renamed, offline status justified |
| 2026-10-14 | Feature | [Feature 001] Plan refined: plan review preparable again, 006 step 0 before Step 2 |
| 2026-10-14 | Feature | [Feature 010] Plan refined: one Enter veto with 009, method outcome from 006, late-read update |
| 2026-10-14 | Feature | [Feature 005] Spec and plan refined: truthful failure notice, merged 008 names, live prompt list |
| 2026-10-14 | Feature | [Feature 009] Spec and plan refined: pinned badge rule, D2 live pin operations, retired auto_submit, single Enter veto |
| 2026-10-14 | Feature | [Feature 001] Spec and plan refined: literal "default" migration, live microphone list, full 2-second attempts |
| 2026-10-14 | Feature | [Feature 006] Plan refined: settings framework Step 0 (D2–D4) and merged 008 names |
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

<!-- finalize:spec-plan:2026-10-13:e428ba3d -->

<!-- finalize:spec-implement:2026-10-13:61bf0cb4 -->

<!-- finalize:spec-implement:2026-10-13:9b1995bc -->

<!-- finalize:spec-plan:2026-10-13:ff08720c -->

<!-- finalize:spec-plan:2026-10-13:1953f656 -->

<!-- finalize:spec-refine:2026-10-14:b0be88bc -->

<!-- finalize:spec-refine:2026-10-14:0feee4a1 -->

<!-- finalize:spec-refine:2026-10-14:8325a713 -->

<!-- finalize:spec-refine:2026-10-14:f275f226 -->
