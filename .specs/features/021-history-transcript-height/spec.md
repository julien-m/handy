---
feature: history-transcript-height
title: Long Transcripts Scroll in the History
priority: P2
status: In Progress
created: 2026-11-03
updated: 2026-11-03
---

# Feature Spec: Long Transcripts Scroll in the History

## Header

- **Feature:** Long Transcripts Scroll in the History
- **Branch:** `dev`
- **Date:** 2026-11-03
- **Status:** In Progress (automated evidence; native check pending)
- **Input:** Julien, 2026-11-03: « la taille dans l'historique […] allez, 10 lignes max, et après il faudrait qu'il y ait un mini scroll bar par transcription. »
- **Feature Number:** 021
- **Extends:** the History settings tab (`HistorySettings.tsx`); long recordings are now transcribed in full ([020](../020-long-dictation-chunks/spec.md)), so a single entry can fill the screen.

## User Scenarios & Testing

### Story 1 — Browse a history holding long dictations `P1`

**As a** Handy user with several-minute dictations, **I want** each history entry to show at most 10 lines, **so that** I can browse the history without scrolling past one long text.

**Independent test:** Open the History tab with a 5-minute transcript: the entry shows 10 lines and a narrow scrollbar; scrolling inside it reveals the rest, and Copy still copies the full text.

```gherkin
Feature: History transcript height
  Scenario: Long transcript
    Given a history entry whose text needs more than 10 lines
    When the user opens the History tab
    Then the entry shows 10 lines with its own narrow scrollbar
    And scrolling inside it shows the rest of the text

  Scenario: Short transcript
    Given a history entry of 3 lines
    Then it shows its 3 lines with no scrollbar
```

## Acceptance Criteria

### AC-001
A transcript in the History tab shows at most 10 lines; longer text scrolls inside its entry with a narrow scrollbar (8 px) that does not cover the text.

### AC-002
A shorter transcript keeps its natural height without a scrollbar; selection, Copy, Retry and the "Transcribing…" and "Transcription failed" states are unchanged.

## Functional Requirements

### FR-001
`HistoryEntryComponent` limits the transcript to `TRANSCRIPT_MAX_LINES` (10) lines (`max-height` in `lh` plus the bottom padding, `overflow-y: auto`); the class `history-text-scroll` in `App.css` narrows its WebKit scrollbar.

## Clarifications

- **How many lines (Julien):** 10 lines maximum.

## Success Criteria

### SC-001
A 5-minute transcript takes no more than 10 lines of the History tab.
