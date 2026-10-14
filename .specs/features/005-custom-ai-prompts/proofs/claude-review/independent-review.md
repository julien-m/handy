# Relecture indépendante — 005-custom-ai-prompts

- **Date :** 2026-10-14
- **Relecteur :** Claude Code (claude-opus-5-5), indépendant de l'auteur de la spec et du plan
- **Outil :** `/opt/homebrew/bin/livespec`, `--model claude-opus-5-5 --review-max-chars 180000` pour toutes les commandes
- **Périmètre :** [spec](../../spec.md) (sha256 `cb9ca8a7…7e692`), [plan](../../plan.md) (sha256 `fbd27bee…62a3`) ; comparaison avec la 008 livrée dans `dev` (code `src-tauri/src/settings_service/`, [implementation 008](../../../008-settings-cli/implementation.md)), la [007](../../../007-smart-paste/spec.md) et la [006](../../../006-destination-application/spec.md). Aucun fichier `spec.md`, `plan.md` ni code modifié.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Revue de spec | complète, **prête** (round 1) | [`.reviews/spec.json`](../../.reviews/spec.json), context `f88be3f0…94fb` |
| Progression `plan` | READY | `livespec validate … --progression plan` |
| Revue de plan | complète, **non prête** (round 1 ; 1 conclusion `contradictory`, 2 constats bloquants) | [`.reviews/plan.json`](../../.reviews/plan.json), context `fe0d7ba1…9600` |
| Analyze (`--pre-impl`) | exit 1 : 1 HIGH (AC-005), couverture structurelle 17/17 | sortie JSON non persistée (commande en lecture seule) |
| Progression `implement` | **BLOCKED** `analyze_not_ready:blocked` | `livespec validate … --progression implement` |
| Voie de livraison | **full** (`criticality_high`, blast radius shared) | `livespec delivery lane --feature 005-custom-ai-prompts` |

## Constats bloquants (plan)

1. **AC-005 — contradiction spec/plan sur le message d'échec.** La spec exige que l'indicateur annonce « le traitement a échoué et l'original a été inséré ». Le plan n'affiche ce texte que si l'insertion est vérifiée (seule l'insertion directe AX « inserted, verified » de la 007 le permet) et accepte de laisser AC-005 BLOCKED. Le critère devient inatteignable dans la plupart des cas.
   *Correction proposée :* `spec-refine 005` pour formuler le message à partir du résultat d'insertion de 006/007 (par exemple « AI processing failed; original text sent »), puis rafraîchir le plan.
2. **Constitution — preuves Windows/Linux rendues bloquantes.** Les étapes 1, 5 et 8 exigent les spikes et mesures natives Windows et Linux avant le câblage UI, et font de leur échec un blocage d'AC-003. La décision du 2026-10-13 (constitution) les classe en *pending release evidence*, qui ne bloque jamais l'implémentation, la complétion sur la branche d'intégration ni le merge dans `dev`.
   *Correction proposée :* `spec-refine 005 plan` : seule la preuve macOS conditionne les étapes ; les vérifications Windows et Linux (X11, Wayland, layer-shell désactivé) deviennent *pending release evidence* avec la commande exacte.

## Écarts avec la 008 livrée (avertissements)

- Le plan nomme `LiveSettingsEffects` et « prepare native assignment changes » avant le commit d'un domaine. Le code livré expose `LiveEffectsPort`/`TauriEffects` et `tauri_adapter::{write, set, edit, set_prepared, ordered, commit_prepared_edit, repair}`. Les changements de domaine (`DomainChange` Scalar|Apply → `sequence::write_change`) n'appliquent les effets qu'**après** le commit : il n'existe pas d'étape de préparation ni de compensation pour une édition de domaine. La suppression d'un prompt avec raccourcis via la CLI nécessite donc une extension du framework, ou l'acceptation d'un effet post-commit.
- `domains::canonical_action` convertit `delete`→`remove` et `edit`→`update` **avant** de chercher l'action. Un domaine `prompts` qui déclarerait `delete` refuserait `prompts delete` (`usage_error`). Il faut déclarer `remove` et `update`.
- `Domain::plan` est pur, sans I/O. La disponibilité d'Apple Intelligence (port injecté) ne peut pas être sondée dans `prompts list` ; `LiveOnly` ne fournit que des valeurs dérivées des réglages.
- Les codes d'erreur (`validation_failed`, `limit_exceeded`, `shortcut_conflict` → 2, `revision_conflict` → 3, `not_found` → 4, `unavailable` → 5, `persistence_failed`/`internal_error` → 1) et l'enveloppe JSON concordent avec la 008 livrée.

## Autres constats

- **Dépendance 004 :** l'étape 3 exige le vérificateur partagé de la 004, encore au statut `Planned`. AC-006 et la partie raccourcis des étapes 3 et 6 attendent sa livraison.
- **Cohérence QE :** la `## QE Analysis` du plan indique un blast radius *Shared/Cross-system/External* et un risque *Data/Functional*. La `## Quality Engineering` de la spec, ajoutée après le plan, indique *Shared* et *Data puis Security*. La voie n'en dépend pas : la criticité High impose `full`. La `## QE Analysis` et la `## Quality Engineering` de la spec concordent entre elles.
- **Testabilité des AC :** tous les AC sont testables. Deux ambiguïtés non critiques ont été reportées : le libellé « original was inserted », et le canal de la notification d'échec quand le style d'overlay vaut `none`.
- **Étape 6 (UI) :** elle attend la passe Penflow commune, différée par décision utilisateur avant `spec-ship`.
- **Next Action :** le plan cite `--model=gpt-6.1-sol`. Il faudra utiliser le modèle résolu au moment de l'implémentation.

## Recommandations

1. Lancer `spec-refine 005` sur AC-005 (libellé aligné sur les résultats d'insertion de 006/007), puis `spec-refine 005 plan` pour : le gating macOS seul ; les points d'entrée 008 réellement livrés (canoniques `update`/`remove`, préparation des domaines, readiness CLI) ; la classification QE copiée depuis la spec.
2. Refaire une revue de plan indépendante (round 2), puis Analyze et la progression `implement`.
3. Livrer la 004 avant l'étape 3 de la 005.
