---
feature: settings-backups
title: Settings Survive Another Handy Version
priority: P1
status: In Progress
created: 2026-11-07
updated: 2026-11-07
---

# Feature Spec: Settings Survive Another Handy Version

## Header

- **Feature:** Settings Survive Another Handy Version
- **Branch:** `dev`
- **Date:** 2026-11-07
- **Status:** In Progress (automated evidence; native check with Julien pending)
- **Input:** Julien, 2026-11-07, after an older Handy rewrote his settings and his word replacement rules were lost: « je ne veux plus jamais que ça arrive que je perde tous mes paramètres, parce qu'on installe une nouvelle version. »
- **Feature Number:** 023
- **Extends:** [008 Settings CLI](../008-settings-cli/spec.md) (settings service).
- **Processing:** entirely local.

## Context

On 2026-11-07 an older Handy build, still in `/Applications`, started on the same `settings_store.json`. It did not know the recent settings (word replacements, smart capitalization, smart spacing, smart paragraphs, paste verification, automatic Enter) and rewrote the file without them, keeping the revision counter. The current build then read the missing settings as their defaults, and the word replacement rules were lost: no copy existed.

## User Scenarios & Testing

### Story 1 — An older version cannot erase my settings `P1`

**As a** Handy user, **I want** my settings back after another Handy version rewrote them, **so that** installing or opening a version never costs me my rules and options.

**Independent test:** commit settings, rewrite the store without some of them (as an older build does), start again: the settings are back and the rewritten file is kept aside.

```gherkin
Feature: Settings backups
  Scenario: An older build rewrote the settings
    Given Handy committed my settings at revision 95
    When another program rewrites the store at revision 95 without some settings
    Then the next start restores the latest copy at revision 96
    And the rewritten file is kept in settings_backups

  Scenario: A store from a newer Handy
    Given the store was written by a newer settings schema
    When this Handy starts
    Then it reads the settings and never rewrites the store
```

## Acceptance Criteria

### AC-001
After each committed change, a private copy of the store exists in `settings_backups/` next to `settings_store.json`; the 20 newest copies are kept.

### AC-002
At startup, when the store's revision is lower than the latest copy's, or equal with a different `settings` payload, the latest copy is restored at the next revision, the rewritten store is kept as `overwritten-<seconds>.json`, and a warning is logged.

### AC-003
A store whose `settings_schema_version` is newer than this build's is read but never rewritten.

## Functional Requirements

### FR-001
`settings_service::backups::keep` copies every generation the service persists (startup normalization, live and offline commits) and prunes the oldest. Copies and their folder are private (0600/0700 on Unix) because the store can hold API keys. A failed copy is logged and never fails the commit.

### FR-002
`SettingsRepository::initialize` compares the store with the latest copy before normalizing and restores the copy when the store was rewritten by another program (`backups::restoration`). A store edited by hand while Handy is stopped is treated the same way; the edited file stays in `settings_backups/`. The restore happens only once the rewritten file is set aside; when it cannot be, startup writes nothing (no restore, no normalization) and reports a recovery, so the next start can still restore once the folder is writable.

### FR-003
`store::document` marks a store with a newer `settings_schema_version` as not writable, so every write is refused instead of dropping settings this build does not know.

## Success Criteria

### SC-001
Installing, opening or reopening any Handy version from this one on never loses a setting: the worst case is a restore from the copy of the last committed change.
