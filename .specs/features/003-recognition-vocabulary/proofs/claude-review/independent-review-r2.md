# Revue indépendante, round 2 — 003-recognition-vocabulary

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`.
- Base relue : corrections `9f0ddd1`. Outil : `/opt/homebrew/bin/livespec` (dev, `livespec` `71f1e0d`), `--model claude-opus-5-5 --review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo.
- Lu en entier :
  - la [spec](../../spec.md) et le [plan](../../plan.md) ;
  - le [rapport du round 1](independent-review.md) ;
  - la [constitution](../../../../constitution.md) ;
  - le contrat D2/D3 de `.specs/ship.md` et la section « Step 0 » du [plan 006](../../../006-destination-application/plan.md).
- Contrôlé dans le code 008 livré (`src-tauri/src/settings_service/`) : `domains/mod.rs`, `sequence.rs`, `tauri_adapter.rs`, `validation.rs`, `input.rs`, `render.rs`, plus `lib.rs`.

## Reçus

| Revue | Résultat | `context_hash` | Reçu |
| --- | --- | --- | --- |
| Spec | Inchangée depuis le round 1 (même hash de contexte), reçu courant : complete, **ready** ; pas de nouvelle revue | `c5e679cf…7fa557966` | [`.reviews/spec.json`](../../.reviews/spec.json), sha256 `462ea1c2…97bfe5` |
| Plan | **non préparable** : `paired_review_batch_budget_exceeded`, `unresolved_reference:external_contract_not_loaded:…/whisper-large-v3.md` | `0b8205af…a09d1ce7` (préparation refusée) | aucun ; [`.reviews/plan.json`](../../.reviews/plan.json) reste celui du round 1 (`89d07013…283f09`), périmé |

Sources : `spec.md` `e50af3db…ab125567f`, `plan.md` `24b84ab0…a64d3b1`.

## Gates

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r2.txt](progression-plan-r2.txt) |
| Analyze (`--pre-impl`) | exit 1 ; 1 HIGH : « Stale, invalid or incomplete semantic review » ; couverture structurelle 15/15 | [analyze-pre-impl-r2.json](analyze-pre-impl-r2.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:incomplete` | [progression-implement-r2.txt](progression-implement-r2.txt) |
| `delivery lane` | **light** (criticité medium, rayon shared, aucune raison de voie complète) | [delivery-lane-r2.json](delivery-lane-r2.json) |

Le round 1 avait obtenu READY sur `implement`. La correction du plan a cassé la préparation de sa revue ; l'implémentation est donc de nouveau bloquée.

## Blocages : la revue de plan ne peut pas être préparée

1. **Budget.**
   - Le contexte normatif atteint 607 650 caractères, dont 59 857 pour le plan, à cause du graphe documentaire 008/006 (`implementation.md`, `progress.md`, `plan.md`, journaux et audits de la 008 ; `ship.md` ; `plan.md` et `spec.md` de la 006 ; `changelog.md`).
   - L'appariement de `review_batches.build_batches` donne 65 lots à 180 000, pour un plafond de 64.
   - Cause : ligne 38 du plan, le lien `[008's foundation plan](../008-settings-cli/plan.md) is historical context only` n'est pas précédé de `Inspect`. Il est donc normatif et importe toute la fermeture des liens de la 008.
2. **Contrat externe.**
   - Ligne 88 du plan, `Inspect the [Whisper model documentation](https://raw.githubusercontent.com/…/whisper-large-v3.md)` : le mot `the` sépare `Inspect` du lien. Le lien est donc lu comme un contrat normatif externe, que l'outil refuse de charger.
- Je n'ai contourné ni le budget ni le plan.
- Correction vérifiée sur une copie jetable de `.specs/` (hors dépôt) : citer les documents 008 en chemins inline et écrire `Inspect [Whisper model documentation](…)` suffit. La revue tient alors en **1 lot** de 111 050 caractères, sans erreur.

## Vérifications demandées

### Points du round 1 : traités

- **Index par ligne.** Les issues indexées hors du schéma 008 passent par D3 : `details.items` avec `ErrorItem::new(index, code)` (au plus 100) et les variantes `term_empty`, `term_too_long`, `term_line_break`, `term_forbidden_character` et `term_duplicate`.
- **`context_budget_unverified`.** Il n'est plus `unavailable`/5 mais `internal_error`/1 avec `details.reason`, et la release reste bloquée tant qu'il peut apparaître.
- **Statut en direct.** `vocabulary status` devient une opération live D2.
- **Export.** Il se fait sur stdout seulement, et un fichier s'obtient par redirection. L'AC-007 demande seulement « one term per line » : c'est conforme.
- **Criticité QE.** Le plan s'aligne sur la spec (Medium, Shared, UX puis Regression), ce qui donne la voie *light*. C'est cohérent, même si j'avais recommandé High au round 1.
- **Plafond de 223 tokens.** Il est désormais justifié pour Whisper par la règle du moteur (`n_text_ctx / 2 − 1`). Les autres mécanismes n'ont plus de plafond applicatif : le budget mesuré est consigné à l'étape 1.
- **Preuves Windows/Linux.** Elles sont en *pending release evidence* (étape 6, QE, Testing Strategy).

### Noms D2/D3 et code 008 : conformes

- D2 :
  - `DomainPlan::Live(Box::new(VocabularyStatus))`, `LiveOperation::run(context, call)`, `LiveContext::read` ;
  - `managed_state::<Arc<TranscriptionManager>>` : `lib.rs` enregistre bien un `Arc<TranscriptionManager>` ;
  - `LiveAnswer::Read` ;
  - hors ligne, refus `unavailable`/5 avec `details` vide.
- D3 : `SettingsError::with_items`, `ErrorItem::new`, `MAX_ERROR_ITEMS`, `ErrorReason::ContextBudgetUnverified`.
- 008 :
  - `sequence::write_change`, `SettingsBackend::commit_change`, `commit_edit_with_probe`, `input::read_source`, `View::Export`, `ValidationReason::VocabularyDomain` ;
  - l'alias `delete` → `remove` via `domains::canonical_action`.
- `tauri_adapter::write_change` est l'extension additive annoncée dans `ship.md`.

### Dépendances : déclarées

- 008 mergée.
- Étape 0 de la 006 en prérequis explicite de l'étape 4, rappelée dans le résumé et la Next Action.
- Les étapes 1 à 3 n'utilisent ni D2 ni D3.

## Constats restants (non bloquants)

1. **Statut hors ligne sans décision dans la spec.**
   - Au round 1, le plan renvoyait hors ligne le statut réussi `unknown_until_model_loads`. Il renvoie maintenant le refus `unavailable`/5.
   - Contrairement à la 001 (Clarification AC-008 du 2026-10-14), la spec 003 n'enregistre pas cette décision D2. L'AC-007 dit seulement que le CLI « offers … status ».
   - Recommandation : consigner la décision dans les Clarifications de la spec via `spec-refine 003`, pour qu'elle reste la source de vérité.
2. **Étiquette « Rule D4 ».**
   - Le plan nomme « Rule D4 » le refus de `unavailable`/5 pour `context_budget_unverified`. Or, dans `ship.md`, D4 désigne la plateforme non supportée (`validation_failed`/2).
   - Le principe est le même (jamais exit 5 pour un état durable), mais l'étiquette prête à confusion. Info.
3. **Interface.** L'étape 5 attend la passe Penflow commune avant `spec-ship`.

## Recommandations

1. Dans le plan :
   - citer `008-settings-cli/plan.md` et `implementation.md` en chemins inline ;
   - écrire `Inspect [Whisper model documentation](…)` ;
   - puis refaire la revue de plan, Analyze et `--progression implement`.
2. Enregistrer dans la spec la décision D2 sur `vocabulary status` hors ligne (`spec-refine 003`).
