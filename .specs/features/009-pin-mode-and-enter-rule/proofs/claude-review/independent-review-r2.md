# Revue indépendante, round 2 — 009-pin-mode-and-enter-rule

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14. Branche `dev`, HEAD `18ae22c` (correction 009 : `388f5a5`).
- Outil : `/opt/homebrew/bin/livespec`, avec `--model claude-opus-5-5 --review-max-chars 180000` sur chaque commande.
- Je n'ai modifié ni `spec.md`, ni `plan.md`, ni le code. Aucun cargo.
- Sources : `spec.md` `35eb4a9d…7679d`, `plan.md` `fdceee75…ca0d3`.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Revue de spec (périmée depuis `388f5a5`) | **complete, ready**, round 2 ; 1 lot ; 13/13 `covered` ; 0 ambiguïté ; 1 avertissement et 2 infos reportés | [`.reviews/spec.json`](../../.reviews/spec.json) sha256 `36cc45f9…9a065`, `context_hash` `4eeb5592…a1e` |
| Clarify (`--progression plan`) | READY | [progression-plan-r2.txt](progression-plan-r2.txt) |
| Préparation de la revue de plan | 60 lots, 8,75 M caractères, `context_hash` `ceb52fb4…5a9a` | sortie de `--prepare-review plan` |
| Revue de plan, ingestion | **non faite.** Le reçu serait périmé avant tout usage (voir plus bas) | — |
| Analyze (`--pre-impl`) | exit 1 : **2 HIGH**, revue sémantique périmée et **AC-006 sans référence de tâche** ; couverture 12/13 (92,31 %) | [analyze-pre-impl-r2.json](analyze-pre-impl-r2.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:incomplete` | [progression-implement-r2.txt](progression-implement-r2.txt) |
| `delivery lane` | **full** (`criticality_high`, rayon `shared`) | [delivery-lane-r2.json](delivery-lane-r2.json) |

## Revue de spec

La seule modification normative est l'exception de badge (AC-001 et Clarifications, décision du 2026-10-14). Elle tranche l'ambiguïté non critique du round 1 : avec une épingle active, le badge s'affiche même si le réglage est désactivé ; sans épingle, 006 AC-003 s'applique sans changement. La spec 006 n'est pas modifiée.

Reports au plan (`carry_over`) :

- **Avertissement.** Le QE P1 « Native checks pinning ChatGPT and Comet » ne distingue pas les plateformes. Seul macOS conditionne `Implemented` ; Windows est une preuve de release en attente.
- **Info.** La raison unique d'AC-003 diffère de la table de la 010 (voir plus bas).
- **Info.** Le prédicat de badge est à implémenter dans la seule fonction de la 006.

## Constats bloquants

1. **Analyze HIGH, AC-006 sans référence de tâche.** Le commit `388f5a5` a remplacé la seule référence explicite, « FR-001/003; AC-006 » (ligne P0 real CLI), par « FR-001/003; AC-004/006 » (l. 493). Le parseur structurel ne lit pas « AC-004/006 » comme AC-006 ; la couverture passe de 13/13 au round 1 à 12/13. Correction : écrire « AC-004, AC-006 » ou citer AC-006 dans une étape.
2. **Revue de plan à refaire sur une base stable.** Je ne l'ai pas lancée, pour trois raisons :
   - **Lien normatif involontaire.** La ligne 25, « Inspect [008's spec](…), [its implementation map](…) … », rend le second lien normatif (LiveSpec ne retient comme contexte que le lien qui suit immédiatement « Inspect »). Il entraîne `implementation.md`, `plan.md`, `progress.md` et les journaux de la 008, quatre audits, `.specs/changelog.md`, trois `tool-defects`, ainsi que `.specs/ship.md`, exclu de git. Sur un clone neuf, la préparation échoue sur `unresolved_reference:unread_source:.specs/ship.md`.
   - **Plan 006 à corriger.** Le plan 006 est une source normative de cette revue (lien « [006's Step 0] », l. 25). Il doit changer, puisque sa propre revue est refusée (`paired_review_batch_budget_exceeded`). Tout reçu 009 produit maintenant deviendrait périmé.
   - **Plan 009 à corriger.** Le constat 1 impose de toute façon une modification de `plan.md`, qui périme aussi le reçu.

   Une revue de 60 lots consommerait un round de correction pour un état normatif déjà condamné. Simulation sur une copie jetable : avec les liens 008 rendus contextuels dans les plans 006 et 009, la préparation passe à 30 lots.

## Bloquants et avertissements du round 1

| Round 1 | État au round 2 |
| --- | --- |
| Opérations live de la CLI impossibles avec `LiveOnly` | **Corrigé.** D2 de l'étape 0 de la 006 : `DomainPlan::Live(Box<dyn LiveOperation>)`, `managed_state::<Arc<DestinationManager>>`, `LiveAnswer::Read`, aucune écriture de révision, dry-run sans effet (l. 133) |
| `LiveSettingsEffects` inexistant | **Corrigé.** Liste des noms livrés (l. 25), tous présents dans `dev` (`operations::known_key`, `tauri_adapter::{write, set, edit, repair}`, `OUTPUT_KEYS`, `apply_settings_migrations`) |
| Clé `auto_submit` : `validation_failed` impossible avant `not_found` | **Corrigé.** `not_found`/4 + `details.reason = "retired_key"` + `details.field = "automatic_enter"` via `descriptor::RETIRED_KEYS`, consulté par `operations::known_key` et `domains::output` seulement pour une clé non descripteur (l. 109, 151) |
| Badge avec retour désactivé | **Corrigé dans la spec** (AC-001) ; le plan étend `badge_visible` (l. 95) |
| Deux contrats de veto d'Entrée | **Corrigé.** `RecoveryDisposition` unique de la 006 ; `RecoverInstead` retiré (l. 117, 540) |
| Étape 7 : preuves OS manquantes en « explicit blocker » | **Corrigé.** Windows, Linux et ancien binaire en *pending release evidence* avec commandes exactes (l. 459, 497, 511) |
| Précédence des erreurs CLI | Fixée : usage → plateforme (D4, 2) → absence d'instance (5) → recherche (4), hors ligne comme en ligne (l. 136-141) |

## Concordance des contrats

- **009 ↔ 006.** Le plan 009 reprend tels quels `RecoveryDisposition`, `set_disposition` monotone, `InsertionOutcome.method_outcome` et `DestinationContext`. Les trois résultats éligibles (`inserted_verified`, `read_observed`, `typed`) et les cinq suppressifs concordent avec la table du plan 006 et avec 007 AC-012.
- **Écart d'API (avertissement).** La 009 lit la disposition « under the same lock as `Pending → Consumed` ». Or la 006 n'expose que `set_disposition` et ne nomme aucune lecture atomique. La 009 doit ajouter à `DestinationManager` une méthode de réclamation sous le verrou de session, par exemple `claim_enter(dictation_id, generation, …)`, et la nommer dans son plan.
- **Écart de raison 009 ↔ 010 (à corriger avant l'étape 2 de la 010).**
  - La 009 émet une raison unique `pinned app unavailable` pour l'application fermée comme pour l'activation hors délai, avec un « diagnostic subtype » non nommé (l. 101, 530).
  - La 010 attend « 009's pinned-app reason » pour une épingle **non lancée** (« is pinned but not open ») et `activation_failed` pour une application lancée mais non activée (« Could not bring <App> forward », plan 010 l. 97-98, 497-498).
  - Avec le plan 009 actuel, une application épinglée lancée mais hors délai serait affichée « not open », ce que la 010 interdit.

  Il faut fixer le nom de la variante et le transport du sous-type dans `InsertionOutcome.reason`.
- **D2/D3 et code 008.** `ErrorDetails.field` est un `&'static str` : la valeur `"automatic_enter"` convient. Après la 009, le schéma énumère les clés de descripteur dans `field`, et `automatic_enter` en fait partie. `SettingsError::new(SettingsErrorCode::NotFound)` est le refus livré actuel de `known_key`.

## Prochaines étapes

1. `spec-refine 009 plan` :
   - remplacer « AC-004/006 » par une référence explicite à AC-006 ;
   - placer un « Inspect » devant chaque lien vers la 008 (l. 25) ;
   - nommer la lecture atomique de la disposition ;
   - fixer la raison d'épingle et son sous-type avec la 010.
2. Après la correction du plan 006 : préparer la revue de plan 009 (environ 30 lots), l'ingérer, puis lancer Analyze et `--progression implement`.
