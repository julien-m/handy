# Changelog - 011-dictation-field-return

## 2026-10-15 — [Spec Update]: Feature specification created

- **Type:** Spec Update
- **Spec modified:** Yes (created — all sections)
- **Code modified:** None
- **AC impacted:** AC-001 through AC-008 (all defined)
- **Review:** not run by this command; the orchestrator runs the independent spec review (prepared context ready)
- **Author:** Claude Code (spec-specify)
- **Summary:** 4 stories, 8 AC, 5 FR, 4 SC from Julien's request of 2026-10-15 and his validated technical frame: the focused input of the destination (006) or pinned (009) application is remembered at dictation start and given focus back, raised and verified before the paste; any failed stage falls back to 006/010; behind the `destination-return` gate, no new setting, no UI.

<!-- finalize:spec-specify:2026-10-15:5625200d -->

## 2026-10-15 — [Plan]: Technical plan generated

- **Type:** Feature
- **Spec modified:** Lifecycle metadata only (Planned through finalize); normative body unchanged
- **Code modified:** None (plan.md created)
- **AC impacted:** None (pre-implementation)
- **Review:** not run by this command; the orchestrator runs the independent plan review (prepared context ready)
- **Author:** Claude Code (spec-plan)
- **Summary:** 6 implementation steps, 2 sequence diagrams, 1 state diagram, 1 logical ER diagram; capture on a worker right after 006's session commit, restore after activation on the insertion worker, verification inside 006's guard before 010's verdict, budgets 50/50/10 ms with per-element AX timeouts, Windows UIA adapter as pending release evidence, deltas listed for 006 and 010.

<!-- finalize:spec-plan:2026-10-15:67c12fe0 -->

## 2026-10-15 — [Spec Update]: Corrected after spec review round 1

- **Type:** Spec Update
- **Spec modified:** Yes (Stories 1–3, AC-001, AC-002, AC-004, AC-005, AC-006, AC-007, AC-008, FR-001, FR-002, FR-004, FR-005, Key Entities, Clarifications, QE Analysis, Quality Engineering, Screens, Edge Cases, SC-001, SC-002, new SC-005); plan realigned (summary, capture, budgets, lifetime, attempt and guard decision table, 010 articulation, Windows adapter, diagrams, steps, tests, QE, risks, deltas)
- **Code modified:** None
- **AC impacted:** AC-001, AC-002, AC-004, AC-005, AC-006, AC-007, AC-008
- **Decisions:** `fallback-after-partial-return` (FR-004) and `caret-after-focus-restore` (AC-002), validated by Julien (maintainer) on 2026-10-15 and traced in Clarifications: Handy notes the focused element before its attempt and inserts through 006/010 only when the attempt changed nothing, otherwise pastes nothing and shows 010's bubble with the existing reason "Insertion failed" (or 006's `paste-error` when the bubble cannot be presented); the selection range (position and length only) is remembered at the start, restored and verified before the paste, and an unreadable or unverified range never leads to a paste into the input
- **Other corrections (round-1 warnings):** cancellation and paste method "none" drop the start reference unused instead of claiming no read; 50 ms start deadline aligned with 010's per-element timeout, 50 ms attempt and 10 ms verification, start path never waits (SC-005); Windows UI Automation calls bounded with the same deadlines and a `TextPattern` range; Chromium Accessibility exposure stated as a precondition of SC-001 verified by native proof, with the safe fallback otherwise; focus-then-raise order checked natively
- **Review:** round 1 spec receipt not ready (1 blocking, 4 warnings); round 2 to run on this version
- **Author:** Claude Code (spec-refine, orchestrated correction)

## 2026-10-15 — [Spec Update]: Corrected after spec review round 2

- **Type:** Spec Update
- **Spec modified:** Yes (Story 3 statement, independent test, Gherkin and diagram; AC-002, AC-004, AC-007, AC-008, FR-004, Key Entities, QE Analysis, Edge Cases, SC-002, SC-004); plan realigned (attempt steps and `unsettled` flag, guard table precedence, Windows text input and expired writes, diagrams, ER, Steps 4 and 5 tests, privacy tests)
- **Code modified:** None
- **AC impacted:** AC-002, AC-004, AC-007, AC-008
- **Corrections (faithful to Julien's decisions of 2026-10-15, never paste without a restored and verified caret, never paste elsewhere than the element noted before the attempt when the attempt changed something):** AC-004 now has two non-overlapping branches and excludes from (a) the remembered input already focused when noted with its range not restored or verified, which goes to (b) as the plan's table already did; Story 3 states that an unreadable range at the start gives the bubble even without an application switch; a focus or range write without an answer in time counts as a change (AC-002, AC-007); AC-008 lists every field of the Remembered input entity; SC-004 also checks that the range never reaches logs; a Windows text input is defined through UI Automation (`ControlType` Edit or Document with `ValuePattern` or `TextPattern`)
- **Review:** round 2 spec receipt not ready (1 blocking, 3 warnings, 2 info); round 3, the last authorized, to run on this version
- **Author:** Claude Code (spec-refine, orchestrated correction)

## 2026-10-15 — [Feature]: Dictation field return implemented behind the destination-return gate (automated evidence; native proofs pending with Julien)

- **Type:** Feature
- **Spec modified:** No (status Planned → In Progress through finalize; requirements unchanged)
- **Code modified:** field model, port, bounded start read, return attempt, guard verification and lifetime in `src-tauri/src/destination/{field,field_capture,field_attempt,field_check,field_lifetime}.rs`; macOS adapter `destination/macos_field.rs`; Windows UI Automation adapter `destination/{windows_field,windows_uia,bounded_worker,uia_mapping}.rs` and two `windows` crate features; 006 integration (`destination/{app,context,guard,insertion,manager,menu,mod,outcome,pin}.rs`: `field_return_failed`, `with_field`, `freeze_with_field`, drop rules); 010 integration (`recovery/{classifier,decision,service,platform/macos_ax}.rs`: shared editable rule and AX helpers, verdict without query for a verified or diverted return, "Insertion failed" with "nothing was sent"); tests `src-tauri/tests/destination_field_*.rs`; native harness `scripts/field-return-native-proof.ts`; native checklist section 011
- **AC impacted:** AC-001 to AC-008 partial (automated evidence only), SC-002 to SC-005 automated, SC-001 pending (native)
- **Author:** Claude Code (spec-implement, claude-opus-5-5)
- Commits on `feature/011-dictation-field-return`: `ffc9ebdb`, `9809ff61` and the documentation commit of this entry.
- Gate closed (default build): identical to `dev`, no focused element is ever read; tested in the four feature combinations.
- Differences from the plan (code is the reference): port held by the destination manager, primitives plus shared bounded sequences, `freeze_with_field`, guard safety net for a diverted return, integration tests over one scripted accessibility world, capture budget from the request with late answers refused, Windows identity by runtime id; listed in `implementation.md`.
- Native macOS proofs (required before Implemented) and the Windows release evidence are listed in `progress.md` and `.specs/native-proofs-checklist.md` (section 011); none is claimed. Deltas for 006 and 010 await their own `/spec-refine`.

<!-- finalize:spec-implement:2026-10-15:28770870 -->
