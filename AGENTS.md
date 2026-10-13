# AGENTS.md

This file provides guidance to AI coding assistants working with code in this repository.

## Development Commands

**Prerequisites:**

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) package manager

**Core Development:**

```bash
# Install dependencies
bun install

# Run in development mode
bun run tauri dev
# If cmake error on macOS:
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev

# Build for production
bun run tauri build

# Frontend only development
bun run dev        # Start Vite dev server
bun run build      # Build frontend (TypeScript + Vite)
bun run preview    # Preview built frontend
```

**Linting and Formatting (run before committing):**

```bash
bun run lint              # ESLint for frontend
bun run lint:fix          # ESLint with auto-fix
bun run format            # Prettier + cargo fmt
bun run format:check      # Check formatting without changes
bun run format:frontend   # Prettier only
bun run format:backend    # cargo fmt only
```

**Model Setup (Required for Development):**

```bash
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx
```

For detailed platform-specific build setup, see [BUILD.md](BUILD.md).

## Architecture Overview

Handy is a cross-platform desktop speech-to-text application built with Tauri 2.x (Rust backend + React/TypeScript frontend).

### Backend Structure (src-tauri/src/)

- `lib.rs` - Main entry point, Tauri setup, manager initialization
- `managers/` - Core business logic:
  - `audio.rs` - Audio recording and device management
  - `model.rs` - Model downloading and management
  - `transcription.rs` - Speech-to-text processing pipeline
  - `history.rs` - Transcription history storage
- `audio_toolkit/` - Low-level audio processing:
  - `audio/` - Device enumeration, recording, resampling
  - `vad/` - Voice Activity Detection (Silero VAD)
- `commands/` - Tauri command handlers for frontend communication
- `cli.rs` - CLI argument definitions (clap derive)
- `shortcut.rs` - Global keyboard shortcut handling
- `settings.rs` - Application settings management
- `overlay.rs` - Recording overlay window (platform-specific)
- `signal_handle.rs` - `send_transcription_input()` reusable function
- `utils.rs` - Platform detection helpers

### Frontend Structure (src/)

- `App.tsx` - Main component with onboarding flow
- `components/` - React UI components:
  - `settings/` - Settings UI
  - `model-selector/` - Model management interface
  - `onboarding/` - First-run experience
  - `overlay/` - Recording overlay UI
  - `update-checker/` - App update notifications
  - `shared/`, `ui/`, `icons/`, `footer/` - Shared components
- `hooks/useSettings.ts` - Settings state management hook
- `stores/settingsStore.ts` - Zustand store for settings
- `bindings.ts` - Auto-generated Tauri type bindings (via tauri-specta)
- `overlay/` - Recording overlay window entry point
- `lib/types.ts` - Shared TypeScript type definitions

### Key Architecture Patterns

**Manager Pattern:** Core functionality organized into managers (Audio, Model, Transcription) initialized at startup and managed via Tauri state.

**Command-Event Architecture:** Frontend → Backend via Tauri commands; Backend → Frontend via events.

**Pipeline Processing:** Audio → VAD → Whisper/Parakeet → Text output → Clipboard/Paste

**State Flow:** Zustand → Tauri Command → Rust State → Persistence (tauri-plugin-store)

### Technology Stack

**Core Libraries:**

- `transcribe-cpp` - Local Whisper-family inference (GGML/GGUF) with GPU acceleration
- `transcribe-rs` - ONNX speech recognition (Parakeet, Moonshine, SenseVoice, etc.)
- `cpal` - Cross-platform audio I/O
- `vad-rs` - Voice Activity Detection
- `rdev` - Global keyboard shortcuts
- `rubato` - Audio resampling
- `rodio` - Audio playback for feedback sounds

### Application Flow

1. **Initialization:** App starts minimized to tray, loads settings, initializes managers
2. **Model Setup:** First-run downloads preferred Whisper model (Small/Medium/Turbo/Large)
3. **Recording:** Global shortcut triggers audio recording with VAD filtering
4. **Processing:** Audio sent to Whisper model for transcription
5. **Output:** Text pasted to active application via system clipboard

### Settings System

Settings are stored using Tauri's store plugin with reactive updates:

- Keyboard shortcuts (configurable, supports push-to-talk)
- Audio devices (microphone/output selection)
- Model preferences (Small/Medium/Turbo/Large Whisper variants)
- Audio feedback and translation options

### Single Instance Architecture

The app enforces single instance behavior — launching when already running brings the settings window to front rather than creating a new process. Remote control flags (`--toggle-transcription`, etc.) work by launching a second instance that sends args to the running instance via `tauri_plugin_single_instance`, then exits.

## Internationalization (i18n)

All user-facing strings must use i18next translations. ESLint enforces this (no hardcoded strings in JSX).

**Adding new text:**

1. Add key to `src/i18n/locales/en/translation.json`
2. Use in component: `const { t } = useTranslation(); t('key.path')`

**File structure:**

```
src/i18n/
├── index.ts           # i18n setup
├── languages.ts       # Language metadata
└── locales/
    ├── en/translation.json  # English (source)
    ├── de/, es/, fr/, ja/, ru/, zh/, ...
    └── ...
```

For translation contribution guidelines, see [CONTRIBUTING_TRANSLATIONS.md](CONTRIBUTING_TRANSLATIONS.md).

## Code Style

**Rust:**

- Run `cargo fmt` and `cargo clippy` before committing
- Handle errors explicitly (avoid unwrap in production)
- Use descriptive names, add doc comments for public APIs

**TypeScript/React:**

- Strict TypeScript, avoid `any` types
- Functional components with hooks
- Tailwind CSS for styling
- Path aliases: `@/` → `./src/`

## CLI Parameters

Handy supports command-line parameters on all platforms for integration with scripts, window managers, and autostart configurations.

**Implementation:** `cli.rs` (definitions), `main.rs` (parsing), `lib.rs` (applying), `signal_handle.rs` (shared logic)

| Flag                     | Description                                                |
| ------------------------ | ---------------------------------------------------------- |
| `--toggle-transcription` | Toggle recording on/off on a running instance              |
| `--toggle-post-process`  | Toggle recording with post-processing on/off               |
| `--cancel`               | Cancel the current operation on a running instance         |
| `--start-hidden`         | Launch without showing the main window (tray icon visible) |
| `--no-tray`              | Launch without system tray (closing window quits the app)  |
| `--debug`                | Enable debug mode with verbose (Trace) logging             |

**Key design decisions:**

- CLI flags are runtime-only overrides — they do NOT modify persisted settings
- Remote control flags work via `tauri_plugin_single_instance`: second instance sends args, then exits
- `send_transcription_input()` in `signal_handle.rs` is shared between signal handlers and CLI

## Debug Mode

Access debug features: `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows/Linux)

## Platform Notes

- **macOS**: Metal acceleration, accessibility permissions required for keyboard shortcuts
- **Windows**: Vulkan acceleration, code signing. Implicit Vulkan layers (overlays, capture hooks) are disabled for the Handy process via `VK_LOADER_LAYERS_DISABLE=~implicit~` set in `main.rs`; opt out with `HANDY_KEEP_VULKAN_IMPLICIT_LAYERS=1` or by setting `VK_LOADER_LAYERS_DISABLE` yourself
- **Linux**: OpenBLAS + Vulkan, limited Wayland support, overlay uses GTK layer shell (disable with `HANDY_NO_GTK_LAYER_SHELL=1`)
- **Nix/NixOS**: the Nix package sets `HANDY_DISABLE_UPDATER=1` to force-disable the self-updater at runtime without touching the persisted setting (self-update can't work against an immutable `/nix/store`)

## Troubleshooting

See the [Troubleshooting](README.md#troubleshooting) section in README.md.

## GitHub workflow for AI coding assistants

**MANDATORY. Before opening any PR, issue, or discussion in this repo: you MUST read the relevant template file and follow it strictly.** That includes sections that look "ceremonial" — checklists, AI Assistance disclosures, "Human Written Description". A generic Summary/Test-plan layout is not acceptable.

- **Opening a PR:** Read [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md). Every section listed there is mandatory. If a section requires a human-written paragraph (e.g. "Human Written Description"), leave a clear TODO placeholder and ask the human contributor to fill it in — do not invent their voice.
- **Opening an issue:** Read [`.github/ISSUE_TEMPLATE/`](.github/ISSUE_TEMPLATE/). Blank issues are disabled; pick the right template (`bug_report.md` for bugs). Feature requests do not belong in issues — they go to [Discussions](https://github.com/cjpais/Handy/discussions) (see `.github/ISSUE_TEMPLATE/config.yml`).
- **Proposing a feature:** Handy is under a feature freeze. New features require community support gathered in [Discussions](https://github.com/cjpais/Handy/discussions) before any PR is opened — see the PR template's "Community Feedback" section.
- **Translations:** Follow [CONTRIBUTING_TRANSLATIONS.md](CONTRIBUTING_TRANSLATIONS.md).
- **Full contributor workflow:** [CONTRIBUTING.md](CONTRIBUTING.md).

**Commits:** Use conventional commit prefixes (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`). Focus the message on _why_, not _what_.

<!-- cc-hub:project-rules:start -->

Generated by cc-hub from canonical `.agent-sync/rules`. Do not edit this block manually.

## Shared Project Agent Rules

### commands

Source: `commands.md`

<!-- @spec(FR-004) -->
<!-- @spec(FR-010) -->

# LiveSpec Commands — Référence

> **Chargé à la demande par `livespec-routing.md`.** Ne pas charger ce fichier par défaut.

## Table — Intention → Commande

| Intention utilisateur                                             | Commande à proposer                                                 |
| ----------------------------------------------------------------- | ------------------------------------------------------------------- |
| "Ajoute / implémente / rajoute X" (une feature)                   | `/spec-feature "<description>"`                                     |
| "Ajoute plusieurs features" / "ship N features"                   | `/spec-ship` (interactif) ou `/spec-ship --count N`                 |
| "Corrige X" / "fix the UI" / écart mockup ↔ code                 | `/spec-fix <feature>`                                               |
| "Teste l'interface" / "vérifie le rendu"                          | `/spec-test <feature>` (avec flags device/target)                   |
| "Vérifie que tout est aligné" / spec ↔ code                      | `/spec-check`                                                       |
| "Diagnostic santé projet" / stale mappings, tests, hooks, runners | `/spec-doctor`                                                      |
| "Explique comment marche X" / "pourquoi a-t-on choisi Y"          | `/spec-explain "<question ou feature>"`                             |
| "Spécifie une feature" (sans coder)                               | `/spec-specify "<description>"`                                     |
| "Plan technique pour X"                                           | `/spec-plan <feature>`                                              |
| "Implémente depuis le plan"                                       | `/spec-implement <feature>`                                         |
| "Qu'est-ce qu'on devrait faire ensuite ?"                         | `/spec-propose`                                                     |
| "Statut du projet" / "où en est-on"                               | `/spec-status`                                                      |
| "Change la stack" / "ADR"                                         | `/spec-stack change "..."`                                          |
| "Raffine la spec / le plan"                                       | `/spec-refine <feature> [plan]`                                     |
| "Vérifie l'outillage / les credentials"                           | `/spec-preflight`                                                   |
| "Lance le projet LiveSpec" (first-time setup)                     | `/spec-init`                                                        |
| "Mets à jour LiveSpec"                                            | `/spec-migrate`                                                     |
| "Crée une user journey" / "bootstrap journeys anciennes features" | `/spec-journey create` ou `/spec-journey bootstrap --from-existing` |
| Lifecycle hooks (before/after une commande)                       | `/spec-hooks <command> --create before\|after`                      |
| Coverage playground                                               | `/spec-play-coverage`                                               |
| Vérifier la sortie d'une commande                                 | `/spec-verify-output <command>`                                     |
| Refresh des conventions projet                                    | `/spec-refresh-conventions`                                         |
| Synchroniser les events brainstorm vers LiveSpec                  | `/spec-refresh-from-brainstorm`                                     |

## Commandes — détail

### /spec-check

Vérifie l'alignement spec ↔ code et produit un rapport d'écarts.
**Usage** :

- `/spec-check` — toutes les features
- `/spec-check feature-name` — une feature ciblée
- `/spec-check --tree-only` — structure seule
- `/spec-check --quality feature` — qualité tests/code uniquement
- `/spec-check --visual-status` — dashboard gouvernance visuelle
- `/spec-check --surfaces` — détection de surface drift
- `/spec-check --pre-impl feature` — analyse pré-implémentation read-only (rapport `## Specification Analysis Report`, sort 1 si CRITICAL/HIGH, n'écrit ni `checks/` ni changelog ni `src/`)

### /spec-doctor

Audit santé projet : orchestre `livespec validate --coherence` et détecte mappings stale, tests manquants, runners non inclus, hooks non appliqués, lifecycle ambigu et orphelins visuels.
**Usage** :

- `/spec-doctor` — audit compact
- `/spec-doctor --format json` — sortie machine
- `/spec-doctor --strict` — warnings promus en erreurs
- `/spec-doctor --fix-plan` — plan de cleanup read-only

### /spec-explain

Documentation vivante — comprend comment une feature marche, ou répond à une question naturelle sur l'historique du projet.
**Usage** :

- `/spec-explain notifications` — par nom de feature
- `/spec-explain 004-notifications` — par identifiant
- `/spec-explain "how do notifications work?"` — question naturelle
- `/spec-explain "why did we choose Supabase over Firebase?"` — historique de décisions
- `/spec-explain "what changed in the auth feature last month?"` — diff temporel

### /spec-feature

Pipeline complet : specify → clarify → plan → review → analyze → implement → test → commit.
**Usage** :

- `/spec-feature "User can filter search results by date range"` — pipeline interactif
- `/spec-feature "Add CSV export" --auto` — pipeline auto, zéro pause
- `/spec-feature --resume csv-export` — reprend un pipeline interrompu
- `/spec-feature "Real-time notifications" --mono` — implémentation mono-agent
- `/spec-feature "Payment" --branch --priority P1` — flags specify

### /spec-fix

Corrige les écarts détectés par `/spec-check` — corrections fonctionnelles et visuelles.
**Usage** :

- `/spec-fix` — auto-détecte feature, fixe tous les gaps
- `/spec-fix feature-name` — fixe tous les gaps d'une feature
- `/spec-fix feature-name --visual` — uniquement écarts visuels/design
- `/spec-fix feature-name --fr FR-003` — un FR ciblé
- `/spec-fix feature-name --ac AC-002` — un AC ciblé
- `/spec-fix feature-name --dry-run` — montre sans modifier
- `/spec-fix feature-name --resume` — reprend une session
- `/spec-fix --all` — toutes les features avec gaps

### /spec-hooks

Affiche, crée ou édite les lifecycle hooks d'une commande.
**Usage** :

- `/spec-hooks` — liste globale
- `/spec-hooks command-name` — hooks actifs d'une commande
- `/spec-hooks command-name --create before` — crée un hook before
- `/spec-hooks command-name --create after --global` — hook global après
- `/spec-hooks command-name --edit before --local` — édite hook local

### /spec-implement

Auto-implémente depuis un plan existant — analyze, code, test, map.
**Usage** :

- `/spec-implement` — auto-sélection depuis la roadmap
- `/spec-implement feature-name` — feature ciblée

### /spec-init

Initialise LiveSpec dans un projet via un brainstorm conversationnel 3 phases.
**Usage** :

- `/spec-init` — projet vierge
- `/spec-init --from-code` — reverse-engineering depuis un codebase existant

### /spec-migrate

Met à jour LiveSpec — compare la version projet vs repo et applique les migrations en attente.
**Usage** :

- `/spec-migrate`

### /spec-journey

Crée, édite, bootstrap, impact-check, liste, inspecte et lance des User Journeys v2 globales.
**Usage** :

- `/spec-journey create` — création interactive depuis intention libre
- `/spec-journey bootstrap --from-existing` — candidates pour anciennes features/projets
- `/spec-journey edit <journey-id>` — édition gouvernée avec décision/changelog
- `/spec-journey impact` — analyse des journeys impactées
- `/spec-journey run` — exécution compiled-only
- `/spec-journey list`
- `/spec-journey inspect <journey-id>`

### /spec-plan

Génère un plan technique avec diagrammes (sequence, state, ER).
**Usage** :

- `/spec-plan` — auto-détecte la feature courante
- `/spec-plan feature-name` — feature ciblée

### /spec-play-coverage

Ouvre le playground de coverage avec données grep live.
**Usage** :

- `/spec-play-coverage`

### /spec-preflight

Vérifie outillage, auth, credentials avant exécution autonome.
**Usage** :

- `/spec-preflight` — read-only
- `/spec-preflight --fix` — auto-install / réparation

### /spec-propose

Analyse le contexte et propose la/les prochaine(s) feature(s) à construire.
**Usage** :

- `/spec-propose` — une proposition
- `/spec-propose --count 3` — 3 features rankées par priorité
- `/spec-propose --role admin` — focus rôle admin
- `/spec-propose --mvp` — uniquement MVP-critical
- `/spec-propose --auto` — sans prompt d'action

### /spec-refine

Raffine des artefacts spec existants via conversation guidée.
**Usage** :

- `/spec-refine` — menu interactif
- `/spec-refine project` — artefacts projet
- `/spec-refine NNN` — spec d'une feature par numéro
- `/spec-refine feature-name` — spec d'une feature par nom
- `/spec-refine NNN plan` — plan d'une feature
- `/spec-refine feature-name plan` — plan d'une feature par nom

### /spec-refresh-conventions

Initialise ou rafraîchit les conventions projet depuis la stack LiveSpec (mode verbeux).
**Usage** :

- `/spec-refresh-conventions`

### /spec-refresh-from-brainstorm

Synchronise les deltas `brainstorm/lifecycle/` vers `.specs/` via un Impact Report interactif.
**Usage** :

- `/spec-refresh-from-brainstorm`

### /spec-ship

Autopilot batch : ship plusieurs features depuis la roadmap, bout-en-bout.
**Usage** :

- `/spec-ship` — sélection interactive (tier ou count)
- `/spec-ship --tier mvp` — toutes les features MVP
- `/spec-ship --count 3` — les 3 prochaines de la roadmap
- `/spec-ship --resume` — reprend après une feature bloquée
- `/spec-ship --tier mvp --mono` — mode mono-agent (ressources légères)

### /spec-specify

Crée une nouvelle feature spec (user stories, Mermaid, AC, FR).
**Usage** :

- `/spec-specify "User can receive real-time notifications"` — depuis idée libre
- `/spec-specify "..." --priority P1` — avec priorité
- `/spec-specify "As a designer, I want to bid on jobs"` — user story format

### /spec-stack

Affiche la stack courante, analyse l'impact d'un changement, crée des ADR.
**Usage** :

- `/spec-stack` — stack courante
- `/spec-stack change "we need Edge deployment now"` — change un composant
- `/spec-stack change "switch database to Firebase"`
- `/spec-stack decisions` — toutes les ADR
- `/spec-stack impact "switch from Supabase to Planetscale"` — dry run d'impact

### /spec-status

Vue factuelle de la roadmap et des features.
**Usage** :

- `/spec-status` — vue complète
- `/spec-status --roadmap` — roadmap uniquement
- `/spec-status --features` — table features
- `/spec-status --json` — sortie machine
- `/spec-status --json --roadmap` — combinaisons

### /spec-test

Audit coverage, génère les tests manquants, exécute la suite, vérifie la fidélité visuelle.
**Usage** :

- `/spec-test` — sélection interactive
- `/spec-test feature-name` — phases 0-5 pour une feature
- `/spec-test --all` — toutes les features implémentées
- `/spec-test --audit-only` — coverage matrix uniquement (pas de génération/exécution)
- `/spec-test --no-generate` — exécute les tests existants sans en générer
- Devices/targets pertinents — passer via la sélection interactive ou les flags surface (web, iphone, etc.)

### /spec-verify-output

Vérifie la sortie d'une commande contre ses expectations.
**Usage** :

- `/spec-verify-output <command>` — vérif basique
- `/spec-verify-output <command> --scenario "<flags>"` — un scénario précis
- `/spec-verify-output <command> --run <path>` — depuis un run existant
- `/spec-verify-output <command> --json` — sortie machine
- `/spec-verify-output <command> --feature <name>` — scopé feature
- `/spec-verify-output <command> --preview --save` — preview + sauvegarde

### routing

Source: `routing.md`

# LiveSpec Routing

**S'applique quand `.specs/` existe à la racine du cwd.**

Si présent : ce projet utilise [LiveSpec](https://github.com/julien-m/livespec). Toute demande d'action sur le projet (ajout, correction, test, exploration, spec, plan, vérification) doit être **routée vers la commande `/spec-*` correspondante** plutôt qu'exécutée directement.

## Procédure

1. Détecter `.specs/` dans le cwd
2. Identifier l'intention utilisateur (ajouter / corriger / tester / spécifier / planifier / vérifier / expliquer / autre)
3. **Read** `livespec-commands.md` pour mapper intention → commande + paramètres usuels
4. **Proposer** la commande `/spec-*` avec les paramètres pertinents avant d'agir
5. Sur confirmation utilisateur → invoquer la commande via le Skill tool
6. Aucune commande ne couvre l'intention → exécuter normalement, signaler explicitement l'absence de routage

## Lecture préalable

Avant toute commande `spec-*`, **Read** `.specs/spec-system.md` (rappel CLAUDE.md projet).

## Chargement

- `livespec-routing.md` : chargé dès que `.specs/` est détecté.
- `livespec-commands.md` : **chargé à la demande uniquement**, à l'étape 3 ci-dessus.

## Exceptions

- Question / lecture / debug rapide → pas de routage forcé
- Mode Ask actif → proposition seule, jamais d'exécution
- Commande `/spec-*` déjà demandée explicitement par l'utilisateur → pas de re-proposition

## Synchronisation

Le check `.checks/livespec-routing-sync.md` (bloquant via `/audit`) garantit la bijection entre `.agent-sync/skills/spec-*` et les entrées de `livespec-commands.md`. Toute désync bloque le commit.

<!-- cc-hub:project-rules:end -->

<!-- livespec:start -->

## LiveSpec

Read [the specification system](.specs/spec-system.md) before any spec command or code modification.
Use `$spec-init` (Codex) or `/spec-init` (Claude) for initialization.

<!-- livespec:end -->
