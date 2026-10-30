# Workflow evidence — 006

Model observed in runtime turn_context: gpt-6.1-sol.
Before-plan hooks resolved; convention loader applied. User decision supersedes mockup-generation hook and Penflow forward generation at plan stage: UI contract prepared once before spec-ship. No UI certification claimed.

retry_hypothesis="spec review model mismatch" retry_evidence="prepared-spec-gpt.json and independent native reviewer raw result" retry_result="pending"

Penflow inspection: BLOCKED, missing expected-ui-tree.json, code-ir.json and native geometry; current semantic tree covers microphone-priority only. This is a known deferred UI preparation gate, not a 006 proof.

Spec review retry_result="PASS": complete/ready and progression plan READY. Budget increased to 200000 based on 54 default plan batches; complete new context has one 162047-character batch. Source hashes and exact spec reviewer batch prompt unchanged across budgets; genuine existing raw spec response ingested against current full preparation.

retry_hypothesis="Activation-deadline wording wrongly rejects configured paste_delay_ms beyond 1 s" retry_evidence="Independent raw-plan-first.json: AC-002/FR-002 contradictory; new explicit authorization/deadline boundary and 1500 ms regression scenario" retry_result="pending"

Correction 1 retry_result="PASS": independent receipt complete=true ready=true, Analyze semantic covered and progression implement READY. Analyze has LOW AC-004 structural-reference warning from shorthand AC-003/004.
retry_hypothesis="Explicit AC-004 step mapping resolves structural-reference shorthand gap" retry_evidence="Step 5 AC covered line and fresh current independent review/Analyze" retry_result="pending"

Correction 2 retry_result="PASS": current independent receipt complete=true ready=true confidence=4; Analyze 17/17 structural references, semantic covered, no findings; progression implement READY with gpt-6.1-sol and review-max-chars=200000. Finalize verify PASS; application diff empty. Native/UI evidence remains future gate, no app runtime success claimed.

retry_hypothesis="Archive used a stale global registry hash after another planner finalized; stdout transcript lacked required Penflow contract marker" retry_evidence="archive-result.json receipt file_sha256_mismatch:.specs/changelog.md plus contains failure; refresh finalize verify and capture real command summary/status before archive" retry_result="pending"

Archive retry_result="PASS": archive-result.json outcome=success; finalize/review receipts verified; archive.run accepted.
