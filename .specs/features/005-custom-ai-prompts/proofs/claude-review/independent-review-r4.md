# Revue indépendante, round 4 — 005-custom-ai-prompts

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev` (`f52e022`).
- Base relue : plan `ce3a000` (historique sans WAV, règle portée avec la 006). Outil : `/opt/homebrew/bin/livespec`, `--model claude-opus-5-5 --review-max-chars 180000` partout.
- Aucune modification de `spec.md`, de `plan.md` ni du code. Pas de cargo.
- Lu en entier :
  - la [spec](../../spec.md) et le [plan](../../plan.md) ;
  - le contexte préparé (95 sections : constitution, project, stack, testing strategy) ;
  - le [rapport du round 2](independent-review-r2.md).
- Contrôlé dans `dev` :
  - `settings_service/domains/{live_operation.rs,mod.rs}`, `error_reason.rs`, `errors.rs`, `platform_capability.rs` ;
  - `sequence.rs`, `tauri_adapter.rs`, `native_changes.rs` ;
  - `shortcut/live_change.rs` ;
  - `managers/{history_store.rs,history.rs}`, `actions.rs` ;
  - `PostProcessingSettings.tsx`.

## Reçus

| Revue | Résultat | `context_hash` | Reçu |
| --- | --- | --- | --- |
| Spec | à jour, non refaite (`--progression plan` READY, `spec.md` inchangé `3213b0c5…36b189f9`) | `dd7f4418…331b768f3` | [`.reviews/spec.json`](../../.reviews/spec.json), sha256 `96925688…4fe223b1` |
| Plan | 1 lot de 128 113 caractères ; **complete, ready**, round 2/2 de corrections, `round_cap_reached: false` ; 3 avertissements, 3 infos, 2 ambiguïtés non critiques, 2 portées techniques | `61c1eed9…2518b8c229d` | [`.reviews/plan.json`](../../.reviews/plan.json), sha256 `c8a3e98c…583241cc9` ; JSON brut : [claude-r4-plan-bundle.json](../../.reviews/plan-run/claude-r4-plan-bundle.json) ; ingestion : [plan-ingest-r4.json](plan-ingest-r4.json) |

Sources : `plan.md` `0145ca5f…7b30` (sha256).

## Gates

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r4.txt](progression-plan-r4.txt) |
| Analyze (`--pre-impl`) | exit 0 ; aucun constat ; 17/17 ; `semantic_status: covered` | [analyze-pre-impl-r4.json](analyze-pre-impl-r4.json) |
| `--progression implement` | **READY** | [progression-implement-r4.txt](progression-implement-r4.txt) |
| `delivery lane` | **full** (criticité high, rayon shared) | [delivery-lane-r4.json](delivery-lane-r4.json) |

## Prompt fait seulement d'espaces : non bloquant

- **Le constat.** AC-001 fixe le texte entre 1 et 8000 caractères, sans parler de trim. Le plan refuse en plus un texte fait seulement d'espaces (`prompt_text_empty`).
- **Pourquoi ce n'est pas une portée métier non approuvée :**
  - l'UI actuelle a déjà ce comportement : Save est désactivé si `!draftText.trim()` (`PostProcessingSettings.tsx`, lignes 341 et 408) ;
  - les scénarios de la spec (« Given the prompt name or text is empty », Save désactivé) se lisent naturellement comme couvrant ce cas ;
  - la règle ne retire rien à l'utilisateur : un prompt vide n'a pas d'usage, et AC-005 traite déjà un résultat fait seulement d'espaces comme un échec.
- **Ce que j'ai consigné.** Une ambiguïté **non critique** sur AC-001 (`whitespace-only-prompt-text`) et une info, toutes deux dans le `carry_over`.
- **À faire au prochain `spec-refine`.** Écrire dans AC-001 « text non-empty after trimming, 1–8000 characters », ou retirer la règle du plan.

## Cohérence avec la 006 livrée dans `dev`

- **D2 / D3 / D4.** Les noms livrés sont exactement ceux du plan :
  - `DomainPlan::Live(Box<dyn LiveOperation>)` ;
  - `LiveOperation::run(&self, &dyn LiveContext, &LiveCall)`, avec `LiveCall { dry_run, expected }` ;
  - `LiveAnswer::Read` ;
  - `LiveContext::{app, read, commit_change}` et `managed_state::<T>` (`unavailable` si absent) ;
  - `PlatformCapabilities::check` et `DomainRequest.platform` ;
  - `ErrorReason` / `ErrorItem::new` ;
  - `SettingsError::{with_reason, with_items, unsupported_platform(Option<&str>)}`.

  L'offline `Live` donne bien `unavailable` avec des détails vides. `ValidationReason::PromptsDomain` et la ligne `post_process_prompts` (Array, domaine `prompts`) existent.
- **Historique.** La migration 5 (`NULLABLE_AUDIO`, reconstruction transactionnelle, ids et `AUTOINCREMENT` conservés) est dans `managers/history_store.rs`. `actions.rs` enregistre `file_name: wav_saved.then_some(file_name)` via `destination::history::save_dictation`, après le post-traitement et avant l'insertion. La règle d'historique sans WAV du plan est donc déjà livrée.

  Deux écarts mineurs (info) :
  - l'étape 7 parle encore de reconstruire la table dans `managers/history.rs` ;
  - le modèle de données a une colonne `audio_available`, alors que la 006 déduit l'absence d'audio de `file_name` NULL.

  La 005 n'a plus qu'à ajouter une migration 6 pour ses colonnes de métadonnées.

## Constats (non bloquants, à traiter avant ou pendant l'étape 3)

1. **La 004 est mergée dans `dev`** (squash `a229f8d`). Le plan dit encore qu'elle ne l'est pas : résumé, prérequis des étapes 3 et 8, Next Action. Le prérequis est rempli ; seul le texte est périmé.
2. **Les noms des changements préparés ont changé avec la 004** :
   - `tauri_adapter::commit_prepared_edit` n'existe plus ; il est remplacé par `tauri_adapter::edit_prepared(app, key, apply, native)` ;
   - `sequence::write_change_prepared` existe déjà, avec la signature `(live, port, (key, change), expected, native)` ;
   - la préparation `bindings` est déjà dans `native_changes` ;
   - le CLI routé ne prépare qu'un `DomainPlan::LiveChange`, via `commit_live_change`, refusé hors ligne avec `instance_required`. `RoutedBackend::commit_change` d'un `DomainPlan::Change` commite sans préparation ;
   - désactiver l'IA a sa propre préparation, `post_process_enabled`.

   L'intention du plan reste faisable : suppression simple hors ligne, préparée et compensée dans l'instance. Il faut toutefois étendre `RoutedBackend::commit_change` (ou ajouter une variante), et non créer une nouvelle fonction. Il faut aussi étendre `post_process_enabled` aux raccourcis de prompt.
3. **Bail de quiescence de la 004.** Pendant une session de dictée, tout changement de raccourcis préparé est refusé : `unavailable`, `shortcut_session_busy`. C'est vrai aussi pour `post_process_enabled`. Le plan prévoit au contraire que désactiver l'IA, ou supprimer un prompt qui a des raccourcis, réussit pendant un enregistrement. Les règles de course et les cas de `prompt_sessions.rs` doivent s'aligner sur ce refus.
4. **Ambiguïté non critique reportée.** Le canal de l'avis d'échec de 4 s quand le style d'overlay vaut `none` n'est toujours pas défini.
5. **Interface.** L'étape 6 attend la passe Penflow commune.

Aucun constat bloquant. Les portées hors spec sont techniques et justifiées :
- le worker Apple Intelligence borné (`provider_busy`) ;
- les colonnes de métadonnées de l'historique.

## Recommandation

`implement` est READY sur la voie **full**. Avant d'attaquer l'étape 3, un `spec-refine 005 plan` léger devrait :
- retirer « 004 not merged » ;
- réaligner l'étape 3 sur `edit_prepared`, sur le `write_change_prepared` existant et sur la préparation `post_process_enabled` ;
- intégrer le refus `shortcut_session_busy` dans le modèle de session ;
- simplifier l'étape 7 (migration 6 seule).

Ce refine invalidera la revue de plan, qu'il faudra alors refaire.
