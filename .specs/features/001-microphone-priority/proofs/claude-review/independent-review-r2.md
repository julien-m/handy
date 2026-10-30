# Revue indépendante, round 2 — 001-microphone-priority

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`.
- Base relue : corrections `9ac7c4d`. Outil : `/opt/homebrew/bin/livespec` (dev, `livespec` `71f1e0d`), `--model claude-opus-5-5 --review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo.
- Lu en entier :
  - la [spec](../../spec.md) et le [plan](../../plan.md) ;
  - le [rapport du round 1](independent-review.md) ;
  - la [constitution](../../../../constitution.md) ;
  - le contrat D2/D3 de `.specs/ship.md` et la section « Step 0 » du [plan 006](../../../006-destination-application/plan.md).
- Contrôlé dans le code 008 livré (`src-tauri/src/settings_service/`) : `sequence.rs`, `tauri_adapter.rs`, `validation.rs`, `descriptor_data.rs`, `effects.rs`, `native_changes.rs`, `domains/mod.rs`, `transaction.rs`, `normalization.rs`, plus `settings.rs` et `lib.rs`.

## Reçus

| Revue | Résultat | `context_hash` | Reçu |
| --- | --- | --- | --- |
| Spec (refaite, périmée depuis `9ac7c4d`) | 1 lot, complete, **ready** ; 1 ambiguïté non critique, 2 infos | `6e8643c4…6098beb255` | [`.reviews/spec.json`](../../.reviews/spec.json), sha256 `3310ce48…167fa8` |
| Plan | **non préparable** : `paired_review_batch_budget_exceeded` | `96ebe2c1…48bd70400d` (préparation refusée) | aucun ; [`.reviews/plan.json`](../../.reviews/plan.json) reste celui du round 1 (`94c33969…f9ca03`), périmé |

Sources : `spec.md` `4d3d351d…5805f`, `plan.md` `570639f8…9c07b`.

## Gates

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r2.txt](progression-plan-r2.txt) |
| Analyze (`--pre-impl`) | exit 1 ; 1 HIGH : « Stale, invalid or incomplete semantic review » ; couverture structurelle 19/19 | [analyze-pre-impl-r2.json](analyze-pre-impl-r2.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:incomplete` | [progression-implement-r2.txt](progression-implement-r2.txt) |
| `delivery lane` | **full** (criticité high, rayon shared) | [delivery-lane-r2.json](delivery-lane-r2.json) |

## Blocage : la revue de plan ne peut pas être préparée à 180 000 caractères

- `--prepare-review plan` renvoie zéro lot et l'erreur `paired_review_batch_budget_exceeded`.
- Le contexte normatif chargé atteint 615 288 caractères, dont 56 542 pour le plan. Le reste vient du graphe documentaire 008/006 :
  - 008 : `implementation.md`, `progress.md`, `plan.md`, `logs/2026-10-13.md`, les quatre audits sous `proofs/` et `contracts/settings-cli.md` ;
  - 006 : `plan.md`, `spec.md` et le contrat ;
  - le reste : `ship.md`, `changelog.md` et trois `tool-defects`.
- J'ai reproduit l'appariement avec `review_batches.build_batches` : 75 lots à 180 000, pour un plafond `MAX_REVIEW_BATCHES` de 64.
- Cause : ligne 23 du plan, le lien `[008's plan](../008-settings-cli/plan.md) is historical context` n'est pas précédé de `Inspect`. Il est donc normatif et importe toute la fermeture des liens de la 008.
- Je ne l'ai pas contourné : budget inchangé et aucune modification du plan. L'implémentation reste bloquée.
- Correction vérifiée sur une copie jetable de `.specs/` (hors dépôt) :
  - en écrivant ce lien `Inspect [008's plan](…)`, la préparation passe, mais avec 22 lots ;
  - en citant les deux documents 008 en chemin (code inline) plutôt qu'en lien, la revue tient en **1 lot** de 121 550 caractères.
- Il faut ensuite refaire la revue de plan (round 2), Analyze et la progression.

## Vérifications demandées

### Bloquants du round 1 : corrigés

- **Preuves Windows/Linux en gates.** Corrigé partout :
  - étape 1 : macOS seul ; Windows (WASAPI) et Linux en *pending release evidence*, avec la commande exacte ;
  - étape 2 : « Step 1 native feasibility established on macOS » ;
  - étape 7 et la ligne « P0 audio ownership » du QE : même règle.
- **Écarts 008 du round 1.** Tous traités :
  - `LiveSettingsEffects` remplacé par `effects.rs` ;
  - écritures UI par `tauri_adapter::write_change` ;
  - `selected_microphone` en lecture seule, avec le refus `MicrophonesDomain` ;
  - migration : seul `null` donne `[]` ;
  - `list` devient une opération live.

### Noms D2/D3 (plan 006, `ship.md`) et code 008 : conformes

- D2 :
  - `DomainPlan::Live(Box::new(MicrophonesList))` ;
  - `domains::live_operation::LiveOperation::run(context, call)` ;
  - `LiveContext::read` ;
  - `managed_state::<Arc<AudioRecordingManager>>` : `lib.rs` enregistre bien un `Arc<AudioRecordingManager>` ;
  - `LiveAnswer::Read` ;
  - hors ligne, refus `unavailable`/5 avec `details` vide.
- D3 : `ErrorReason::MicrophonesDomain`, `MicrophoneNameEmpty`, `MicrophoneNameDuplicate`, `ErrorItem::new(index, code)`, `with_reason`, `with_items`.
- Noms 008 qui existent dans le code : `sequence::write_change` (pub(crate)), `SettingsBackend::commit_change`, `commit_edit_with_probe`, `SettingsRepository::initialize`, `normalization::normalize`, `settings::apply_settings_migrations(&mut settings, payload)`, `validation::editable_descriptor`, `LiveEffect::MicrophoneDevice`, `RuntimeEffect::AudioInput`, `PublicSettings`.
- `tauri_adapter::write_change` n'existe pas encore. C'est l'extension additive annoncée dans `ship.md` (001 ou 003, la première livrée l'ajoute).

### Dépendances : déclarées

- 008 mergée (`e9f96df`).
- Étape 0 de la 006 en prérequis explicite de l'étape 5, et rappelée dans le résumé et la Next Action.

### Décisions Clarify AC-006 et AC-008 : cohérentes entre spec et plan

- **AC-006** :
  - `null` → `[]` ; toute valeur non nulle, y compris `"default"`, → `[nom exact]` ;
  - fixtures prévues dans `microphone_settings.rs`.
- **AC-008** :
  - `list` en live, `unavailable`/5 hors ligne, `get microphone_priority` lisible hors ligne ;
  - `set-priority` fonctionne en ligne comme hors ligne ;
  - `selected_microphone` en lecture seule, refusé avec `validation_failed`/2 et `details.reason = "microphones_domain"`, tout comme `set microphone_priority`.

## Constats restants (non bloquants)

1. **Budget AC-003 sans trace dans la spec.**
   - Le plan applique une « orchestrator decision 2026-10-14 » : chaque tentative garde ses 2 s, et une tentative ne démarre que s'il reste 4 s, pour réserver le créneau du défaut.
   - Cette règle respecte le texte de l'AC-003, mais la spec ne l'enregistre pas dans ses Clarifications. Ma revue de spec la reporte comme ambiguïté non critique (`attempt-budget-allocation`).
   - À consigner par `spec-refine 001`.
2. **Ordre de l'étape 2.**
   - L'étape 2 ajoute les variantes D3 « once 006 step 0 is merged », mais son prérequis ne cite pas l'étape 0 de la 006.
   - À déclarer, ou bien déplacer ces variantes à l'étape 5.
3. **Clarifications.** La puce de décision Penflow porte toujours un identifiant vide et une clé égale à tout le paragraphe (info, déjà signalée au round 1).
4. **Interface.** Les états visuels et les IDs de test attendent la passe Penflow commune avant `spec-ship`.

## Recommandations

1. Dans le plan, citer `008-settings-cli/plan.md` et `implementation.md` en chemins inline, ou au minimum écrire `Inspect [008's plan](…)`. Puis refaire la revue de plan, Analyze et `--progression implement`.
2. Enregistrer la règle de budget de l'AC-003 dans les Clarifications de la spec (`spec-refine 001`).
3. Ajouter « 006 step 0 merged » au prérequis de l'étape 2, ou déplacer les variantes D3 à l'étape 5.
