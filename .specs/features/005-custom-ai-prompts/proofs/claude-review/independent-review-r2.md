# Revue indépendante, round 2 — 005-custom-ai-prompts

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`.
- Base relue : corrections `639530e`. Outil : `/opt/homebrew/bin/livespec` (dev, `livespec` `71f1e0d`), `--model claude-opus-5-5 --review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo.
- Lu en entier :
  - la [spec](../../spec.md) et le [plan](../../plan.md) ;
  - le [rapport du round 1](independent-review.md) ;
  - la [constitution](../../../../constitution.md) ;
  - le contrat D2/D3/D4 et les « Écarts signalés » de `.specs/ship.md` ;
  - la section « Step 0 » du [plan 006](../../../006-destination-application/plan.md).
- Contrôlé dans le code 008 livré (`src-tauri/src/settings_service/`) : `sequence.rs`, `tauri_adapter.rs`, `native_changes.rs`, `validation.rs`, `domains/mod.rs`, `effects.rs`, plus `settings.rs`.

## Reçus

| Revue | Résultat | `context_hash` | Reçu |
| --- | --- | --- | --- |
| Spec (refaite, périmée depuis `639530e`) | 1 lot, complete, **ready** ; 1 ambiguïté non critique, 1 avertissement, 2 infos | `dd7f4418…331b768f3` | [`.reviews/spec.json`](../../.reviews/spec.json), sha256 `96925688…223b1` |
| Plan | **non préparable** : `paired_review_batch_budget_exceeded` | `557a08df…773c09a151` (préparation refusée) | aucun ; [`.reviews/plan.json`](../../.reviews/plan.json) reste celui du round 1 (`480dacd0…3c961c1`), périmé |

Sources : `spec.md` `3213b0c5…36b189f9`, `plan.md` `a8cabacf…fb51f9b88`.

## Gates

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r2.txt](progression-plan-r2.txt) |
| Analyze (`--pre-impl`) | exit 1 ; 1 HIGH : « Stale, invalid or incomplete semantic review » ; couverture structurelle 17/17 | [analyze-pre-impl-r2.json](analyze-pre-impl-r2.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:incomplete` | [progression-implement-r2.txt](progression-implement-r2.txt) |
| `delivery lane` | **full** (criticité high, rayon shared) | [delivery-lane-r2.json](delivery-lane-r2.json) |

## Blocage : la revue de plan ne peut pas être préparée à 180 000 caractères

- Le contexte normatif atteint 617 690 caractères, dont 65 932 pour le plan, à cause du graphe documentaire 008/006 (`implementation.md`, `progress.md`, `plan.md`, journaux et audits de la 008 ; `ship.md` ; `plan.md` et `spec.md` de la 006 ; `changelog.md`).
- L'appariement de `review_batches.build_batches` donne 65 lots à 180 000, pour un plafond de 64.
- Cause : ligne 39 du plan, le lien `[008's plan](../008-settings-cli/plan.md) is historical context` n'est pas précédé de `Inspect`. Il est donc normatif et importe toute la fermeture des liens de la 008.
- Je n'ai contourné ni le budget ni le plan.
- Correction vérifiée sur une copie jetable de `.specs/` (hors dépôt) : en citant les documents 008 en chemins inline, la revue tient en **1 lot** de 123 487 caractères.

## Vérifications demandées

### Bloquants du round 1 : corrigés

1. **Contradiction de l'AC-005.** Résolue par la décision Clarify du 2026-10-14 :
   - l'indicateur dit que l'étape IA a échoué et que le texte original a servi à l'insertion, sans jamais annoncer le succès de l'insertion ;
   - le plan suit la décision (texte source `AI processing failed. The original text was used.`) ;
   - il retire le port de vérification d'insertion ;
   - il fait d'un libellé qui laisse entendre un succès un défaut P0 (étape 5).
   La spec, le plan et la décision sont cohérents.
2. **Preuves Windows/Linux en gates.** Corrigé :
   - étapes 1, 5 et 8 : macOS seul gate ; Windows et Linux (X11, Wayland, layer-shell désactivé) en *pending release evidence*, avec la commande exacte ;
   - même règle dans le QE et la Testing Strategy.

### Avertissements 008 du round 1 : traités

- **Actions canoniques.** Le domaine déclare `remove` et `update` ; `delete` et `edit` passent par `canonical_action`.
- **Préparation des raccourcis.** La suppression passe par la préparation native de la 008 :
  - préparation `bindings` dans `native_changes` ;
  - `tauri_adapter::ordered` et `commit_prepared_edit` côté UI ;
  - nouveau `sequence::write_change_prepared` pour le CLI routé, sur le modèle de `write_scalar_prepared`.
  `write_scalar_prepared`, `PreparedChange`, `prepared_change` et `LiveEffectsPort` existent bien dans le code.
- **Apple Intelligence.** `prompts list` devient une opération live D2 (`PromptsList`). Un override Apple Intelligence hors macOS ARM64 est refusé par D4 : `SettingsError::unsupported_platform(None)`, `validation_failed`/2, `details.reason = "unsupported_platform"`.

### Noms D2/D3/D4 : conformes au plan 006 et à `ship.md`

- D2 :
  - `DomainPlan::Live(Box::new(PromptsList))`, `LiveOperation::run(context, call)`, `LiveContext::read`, `managed_state`, `LiveAnswer::Read` ;
  - `PlatformCapability::AppleIntelligence` via `DomainRequest.platform`.
- D3 : variantes `ErrorReason` propres à la 005, avec des messages fixes.
- D4 : `unsupported_platform`.
- Le plan 006 cite lui-même « 005's Apple Intelligence availability » parmi les consommateurs de D2.

### Dépendances : déclarées

- **004.** Elle n'est pas mergée dans `dev`. Sa fusion est un prérequis explicite de l'étape 3 et des parties raccourcis des étapes 5, 6 et 8. Le plan prévoit l'adaptateur `PromptShortcutRegistry`, sans vérificateur bouchonné.
- **006.** Son étape 0 est un prérequis explicite de l'étape 3, rappelé dans le résumé, le QE et la Next Action.

## Constats restants (non bloquants)

1. **`prompts list` hors ligne, sans décision dans la spec.**
   - Le refus `unavailable`/5 et la lecture par `get post_process_prompts` changent le comportement observable de l'AC-008 sans Clarification dans la spec. La 001 l'a fait pour `microphones list`.
   - Le test indépendant de la Story 6 parle d'un « isolated store ».
   - À consigner par `spec-refine 005`, ou bien répondre hors ligne avec la disponibilité des fournisseurs HTTP (métadonnées pures) et un état inconnu pour Apple Intelligence.
2. **Ordre de l'étape 2.**
   - L'étape 2 utilise `tauri_adapter::write_change`, que `ship.md` attribue à 001/003, sans prévoir de l'ajouter si aucune des deux n'est livrée.
   - Elle cite aussi `commit_prepared_edit` pour la suppression, alors que la préparation `bindings` n'arrive qu'à l'étape 3.
   - Préciser que la 005 ajoute le wrapper s'il manque, et qu'avant l'étape 3 une suppression se commite sans préparation (aucun raccourci de prompt n'existe encore).
3. **Historique et échec WAV (005 ↔ 006).** Ce choix produit est toujours ouvert dans `ship.md`. La 005 garde sa ligne texte seule pour un échec IA, ce qui n'est compatible que si la 006 lit `history_status` dans le résultat réel de `save_entry`. À trancher par Julien avant d'implémenter l'étape 7.
4. **Libellé « inserted » résiduel dans la spec.** Le scénario Timeout, FR-004, SC-001 et les Edge Cases disent encore « is inserted ». Je les lis comme la tentative d'insertion, mais un alignement sur « used for insertion » éviterait toute relecture comme une insertion vérifiée (avertissement de ma revue de spec).
5. **Ambiguïté non critique.** Le canal de l'avis d'échec de 4 s quand le style d'overlay vaut `none` n'est pas défini (reportée depuis le round 1).
6. **Risques.** La ligne « 256×50 menu » parle encore d'« evidence on every indicator platform ». Elle est supplantée par la ligne du 2026-10-14 (info).
7. **Interface.** L'étape 6 attend la passe Penflow commune avant `spec-ship`.

## Recommandations

1. Dans le plan, citer `008-settings-cli/plan.md` et `implementation.md` en chemins inline, puis refaire la revue de plan, Analyze et `--progression implement`.
2. `spec-refine 005` :
   - consigner la décision D2 sur `prompts list` hors ligne ;
   - aligner le libellé « inserted » restant.
3. Préciser l'ordre de l'étape 2 (wrapper `write_change` et suppression sans préparation avant la 004).
4. Faire trancher par Julien la question historique/WAV de `ship.md`.
