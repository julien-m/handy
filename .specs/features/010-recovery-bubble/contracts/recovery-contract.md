# Recovery Bubble Local Contract

Inspect [the technical plan](../plan.md) for decision, reason, threading, testing and dependency context. Local Tauri/native commands and a settings descriptor only; no HTTP endpoint/OpenAPI.

| Boundary | Contract |
|---|---|
| 006 input | InsertionOutcome v1 unchanged, opaque transcript_ref resolved by trusted native manager; fields include outcome/session/cancel identity, target/source/status/reason, actual history outcome, completed_at |
| Recovery instance | Native immutable full text, unique bubble_id plus session_id/cancel_generation, phase, typed reason, target name and history_status; no public verified flag |
| Surface snapshot | Targeted preview/reason/history metadata, current bubble identity, actions/capabilities; no full text app-wide broadcast |
| Action | Copy/Close/explanation/drag request with exact current bubble identity; stale closed instance rejected, never affects successor |
| 007 method outcome | Identity-bound terminal adapter alongside unchanged 006 v1: inserted, verified / insertion attempted, read observed / not read / permission missing / uncertain / typed / clipboard handed over / insertion failed; direct AX-only readback verification, wait for terminal read/no-read result before recovery/Enter |
| 007 AX chain | 007 tries AXSelectedText and compares reread with expected replacement before clipboard (outside special-app strategies); unchanged permits fallback, unexpected change → uncertain with no fallback; 010 never retries insertion; current 007 direct AX budget ≤250 ms including ≤150 ms readback, separate from 010 classification; 007 owns destination-PID guard and clipboard handling |
| 007 restoration | macOS smart paste owns read-confirmed restore: 300 ms quiet after LAST post-Cmd+V read (Jump 1.5 s), capped 8 s after FIRST read; no read under 800 ms (Jump 3 s) → not read, NEVER restore for this dictation including late read/quit/new output; on the existing executor check publication changeCount and write without interleaving deliberate Copy; changed count abandons restoration, no window cancellation API |
| Deliberate Copy | Existing clipboard executor writes full plain text; success changes changeCount even for identical text and closes matching instance, so pending smart paste restore abandons; failure/partial write retains text/surface and reports error, original restore guard remains authoritative; existing Windows/Linux Copy paths kept |
| Drag | macOS native plain-text full payload plus preview, only copy; hidden during session; operation none restores only current instance, accepted closes/logs drop accepted without field verification or text |
| 009 Enter | Recovery decision/not read/permission missing/uncertain/insertion failed invalidates session submission before show; only inserted, verified/read observed/typed allow 009 policy, clipboard handed over allows neither recovery nor Enter; final queued Enter checks decision/generation; Copy/drag never auto-submit |
| History | saved/not_saved/failed from actual WAV/save_entry result; kept-in-history explanation only saved, no deletion or new durable guarantee |
| 006 reason presenter | Advertise capability, retain transcript then acknowledge visible ownership; suppress duplicate indicator only on ack; passive ≥4 s and indefinite bubble, explicit 010 closure events immediate |
| Platforms | macOS full bubble; Windows unavailable/activation/method failure without drag; Linux only method failure without drag; unsupported capture uses old method path |
| Settings | show_recovery_bubble_without_text_field boolean, default true, in 008 output domain; frozen at dictation start, no current-bubble dismiss on mutation |
| Method configuration | 007 adds paste_method=smart_paste to 008 output descriptors/Domain through read_snapshot/commit_mutation, refused outside macOS; macOS reliable paste migration owned by 007, Windows reliable paste and Linux methods kept; no clipboard_recovery_* key |
| Settings commands | handy settings output get KEY / output set KEY true\|false --json and generic get/set; same validation/service/revision renderer as 008 |
| JSON/exits | Exactly {ok:true,revision:N,data:{...}} or {ok:false,error:{code,message,details:{...}}}; 0 success, 1 persistence/internal, 2 usage/validation, 3 revision conflict, 4 not found, 5 unavailable; no transcript output |

All UI/native command inputs are typed and identity-validated. Exported preview never becomes drag or Copy source. Recovery state is volatile; crash/restart survival relies solely on successful existing history save. This is a planned contract, not observed runtime proof.
