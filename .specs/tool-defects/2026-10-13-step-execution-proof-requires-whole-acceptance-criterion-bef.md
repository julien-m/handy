# Superseded report: Whole acceptance rule misinterpreted as a Step 1 gate

- **Status:** superseded-supervisor-gate-interpretation
- **Date:** 2026-10-13
- **Feature:** 008-settings-cli
- **Phase:** implement
- **Runtime mode:** dev
- **Runtime source:** /Users/julienm/projects/livespec

## Command

```text
livespec test --feature 008-settings-cli --acceptance-mapping .specs/features/008-settings-cli/proofs/current-run-step1/acceptance-mapping.json --ingest-mapping-review .specs/features/008-settings-cli/proofs/current-run-step1/mapping-review-bundle.json
```

## Error

```text
exit 1: "complete": true, "ready": false, "errors": []; AC-005 disposition missing. Step 1 assertion evidence cannot certify the whole AC-005; execution verification requires bool(certified) even when required acceptance scope is empty.
```

## Historical proposed recovery — superseded

The original report proposed updating the runtime and restoring Implement. Root subsequently inspected the Step Gate and rejected the inference that a runtime fix was needed; preserve the original command/error as evidence, not an active recovery instruction.

## Historical context: incorrect per-step proof inference

The independent mapping reviewer is correct: occupied passive-probe store preservation and current-session uncertainty do not prove the complete AC-005. Later steps still must implement actual CLI live/private routing, offline validation/locking, deadline/unknown outcome, live effects and UI refresh. Read [the partial mapping](../features/008-settings-cli/proofs/current-run-step1/acceptance-mapping.json), [actual reviewer JSON](../features/008-settings-cli/proofs/current-run-step1/mapping-review-response.json), [review response schema](../features/008-settings-cli/proofs/current-run-step1/mapping-response-schema.json), [review receipt](../features/008-settings-cli/proofs/current-run-step1/acceptance-review.json) and [observed ingest exit](../features/008-settings-cli/proofs/current-run-step1/mapping-command-results.json). Review complete=true/ready=false/errors=[] means a valid review rejecting whole-criterion coverage, not a schema/transport failure. No capture was attempted with that rejected mapping; no new policy-078.1 capture or acceptance certification is claimed.

The generic goal task `task.016.execute_each_plan_step_with_code_targeted_tests` has evidence_kind=execution and requires execution_receipt_path without a narrower acceptance scope. Inspect [the current goal contract](/var/folders/l_/t1s_zytx2fqb0dkzz40d3v7m0000gn/T/livespec-goals/goal-spec-implement-930d1379.contract.json). The skill requires per-step proof before advancing, while the trusted proof model certifies complete feature ACs. This leaves Step 1 unable to produce the demanded typed receipt until later AC-005 work exists, even though its planned local primitives pass independently. The complete feature acceptance gate is a separate obligation and remains required.

Read [runtime execution verification](../../../livespec/validator/execution_evidence.py#L257-L285): lines 257–285 require a verified mapping, certify complete criteria, reject missing required IDs and return valid only when no gaps and bool(certified). Even an empty required scope cannot validate an execution receipt with zero certified criteria. Read [runtime mapping schema and preparation](../../../livespec/validator/execution_mapping.py#L38-L54): lines 38–54 define strict fields including nonempty bindings and prepare the complete spec/exact tests/mapping; no separate per-step semantic proof scope is exposed. These are informative read-only source pointers, not modifications to the LiveSpec checkout.

Inspect [GREEN](../features/008-settings-cli/proofs/current-run-step1/green2.stdout), [actual bridge](../features/008-settings-cli/proofs/current-run-step1/bridge.json), [check summary](../features/008-settings-cli/proofs/current-run-step1/summary.json) and [independent source quality review](../features/008-settings-cli/proofs/current-run-step1/code-quality-review.json): 20 passed, 0 failed, 0 ignored; clippy/types/lint/format exit 0; PASS within Step 1 partial scope. After-step audit was NOT RUN because Step 1 is not Done. Windows/Linux, older-binary and second-account observations stay pending release evidence; they are not this tool blocker. Approved console selection is retained, with native console-flash behavior pending.

Preserve whole-criterion integrity and existing source hashes/raw-review binding. A runtime repair must support trustworthy per-step execution evidence without substituting it for complete acceptance certification. Do not mark AC-005 covered, rewrite the approved criterion, invent a receipt, bypass review or implement later steps before this step gate is satisfiable. Consumer changes stop here; no other repository, global configuration or system settings were modified.

## Root correction — current authority

Root inspected spec-implement Step Gate lines 303–318: a step is gated by targeted checks, transverse checks, independent source reviews, checkpoint and audit. The generic `task.016.execute_each_plan_step_with_code_targeted_tests` execution receipt is aggregate goal proof after Step 7; it does not demand whole-AC certification after each individual plan step. The observed mapping refusal is correct acceptance policy, not a Cargo runner failure or demonstrated runtime bug. The historical analysis above misapplied final goal acceptance semantics to Step 1 and is superseded.

Keep the actual reviewer result complete=true/ready=false/errors=[] with AC-005 missing. Do not execute with that rejected mapping or claim a certified AC. Step 1 local actual-bridge 20/20, clippy/types/lint/format exit 0 and partial-scope source review PASS remain valid local observations. Root restores pipeline state through the official CLI; authoritative Step 1 is In Progress pending renewed audit including untracked files, and Steps 2–7 stay Todo until that gate accepts Step 1. No Step 1 Done, Step 2 start, archive success, commit or feature delivery is claimed here.

Final feature closure still requires current independent mapping reviews, complete whole-AC certification, trusted Cargo policy-078.2 execution capture and all applicable macOS native evidence. This correction preserves those gates; it neither weakens criteria nor fabricates a step/acceptance receipt. Read [the authoritative checkpoint](../features/008-settings-cli/progress.md) before continuing. The known branch-metadata audit minor remains for the root to resolve through its authorized spec metadata workflow.
