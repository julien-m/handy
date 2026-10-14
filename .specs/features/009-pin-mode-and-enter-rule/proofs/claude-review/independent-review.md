# Relecture indépendante — 009-pin-mode-and-enter-rule

- **Date :** 2026-10-14
- **Relecteur :** Claude Code (claude-opus-5-5), indépendant de l'auteur de la spec et du plan
- **Outil :** `/opt/homebrew/bin/livespec`, `--model claude-opus-5-5 --review-max-chars 180000` pour toutes les commandes
- **Périmètre :** [spec](../../spec.md) (sha256 `24721121…089c`), [plan](../../plan.md) (sha256 `edac3414…d90d2`). La revue de plan couvre 16 lots : 8 lots de contexte (plans et specs 006, spec 007, plan 008) et 8 lots portant les exigences (spec 010, plan 008), suivis d'une synthèse. Elle compare aussi le plan à la 008 livrée dans `dev` (`src-tauri/src/settings_service/`). Aucun `spec.md`, `plan.md` ni code modifié.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Revue de spec | complète, **prête** (round 1) | [`.reviews/spec.json`](../../.reviews/spec.json), context `ca6a71ca…5732` |
| Progression `plan` | READY | `livespec validate … --progression plan` |
| Revue de plan | complète, **prête** (round 1, 16 lots + synthèse, aucun constat bloquant) | [`.reviews/plan.json`](../../.reviews/plan.json), context `7d556754…90d2` |
| Analyze (`--pre-impl`) | exit 0 : 0 CRITICAL, 0 HIGH ; couverture 13/13 ; `semantic_status: covered` | commande en lecture seule |
| Progression `implement` | **READY** | `livespec validate … --progression implement` |
| Voie de livraison | **full** (`criticality_high`, blast radius shared) | `livespec delivery lane --feature 009-pin-mode-and-enter-rule` |

## Constats bloquants

Aucun dans les reçus. En pratique, l'implémentation ne peut pas commencer avant la livraison des 006 et 007, toujours au statut `Planned`. Les étapes 2 et 5 exigent le `DestinationManager` de la 006 et les résultats terminaux de la 007.

## Écarts avec la 008 livrée (avertissements)

- **Opérations live de la CLI impossibles en l'état.** Le `Domain::plan` livré est pur : il reçoit les réglages et rend une décision. `DomainPlan::LiveOnly` ne renvoie qu'une valeur calculée depuis les réglages, et aucun domaine n'accède au `DestinationManager`. `destination pin|unpin|status` exigent donc une extension du framework partagé, que le plan ne décrit pas. Par exemple, une opération live exécutée par le backend routé avec l'`AppHandle`, et `unavailable` hors ligne.
- Le plan impose de réutiliser `LiveSettingsEffects`, qui n'existe pas dans le code livré. Le code expose `LiveEffectsPort`/`TauriEffects` et `tauri_adapter::{write, set, edit, set_prepared, ordered, commit_prepared_edit, repair}`. `commit_mutation` et `read_snapshot` existent bien.
- **Clé `auto_submit`.** La 008 l'expose aujourd'hui, en `get`/`set` générique et dans le domaine `output`. Le plan veut la retirer des descripteurs, tout en voulant que `set auto_submit` réponde `validation_failed` avec un guidage. Dans la 008, une clé inconnue reçoit `not_found` avant toute validation : il faut un descripteur « retiré » et documenter le contrat vu par les agents.
- Les codes d'erreur et l'enveloppe du tableau CLI concordent avec la 008. Les opérations de PIN n'avancent pas la révision des réglages.

## Concordance avec la 006 et la 007

- **007 :** les huit résultats consommés, dont trois seulement autorisent Entrée (« inserted, verified », « insertion attempted, read observed », « typed »), correspondent exactement à AC-012. Les délais (300 ms après la dernière lecture, plafond 8 s, 800 ms pour « not read », Jump Desktop 1,5 s / 3 s), la propriété de la restauration et l'insertion AX d'abord concordent avec AC-009, AC-011 et AC-017. Le plan écarte à juste titre le résumé à six résultats, devenu obsolète, du plan 008.
- **006 :** `InsertionOutcome.method_outcome`, `transcript_ref`, `history_status`, `FrontmostAtEnd` et la croix « Remove destination » (override de la dictée courante seulement) concordent avec le plan et la spec 006.
- **Écart :** afficher le badge dès qu'une épingle existe, même avec « Return to the dictation's application » désactivé, contredit 006 AC-003 (pas de badge quand ce réglage est désactivé). Ce choix est défendable (AC-001 exige une marque d'épingle), mais aucune des deux specs ne l'écrit.
- **010 :** `RecoveryDisposition` (`Insert | RecoverInstead | Cancelled`) est un contrat nouveau, que ni le plan 006 ni la spec 010 ne nomment. Le plan 010 doit le produire avant l'insertion.

## Autres constats

- **Constitution :** l'étape 7 fait des opérateurs de plateforme un prérequis et garde toute preuve OS manquante comme « explicit blocker ». Selon la constitution, la preuve Windows est une *pending release evidence* qui ne bloque pas la complétion sur la branche d'intégration. Le reste du plan rattache bien les preuves Windows à la release (avertissement).
- **Cohérence QE :** la `## QE Analysis` du plan indique *Cross-system* et *Functional/Security*. La `## Quality Engineering` de la spec indique *Shared* et *Functional*, et concorde avec la `## QE Analysis` de la spec. La voie n'en dépend pas : la criticité High impose `full`.
- **Testabilité des AC :** tous les AC sont testables ; la matrice AC-005 est exhaustive (SC-001). Deux ambiguïtés non critiques sont reportées : la visibilité du badge épinglé quand le retour est désactivé, et le sort CLI de `auto_submit`.
- **Next Action :** le plan cite `--model=gpt-6`. Il faudra utiliser le modèle résolu au moment de l'implémentation.

## Recommandations

1. Avant l'étape 3, décrire dans le plan l'extension « opération live » du framework 008 et le contrat de la clé retirée `auto_submit`, puis consigner l'exception de badge dans 009 ou 006 via `spec-refine`.
2. Aligner le contrat `RecoveryDisposition` avec les plans 006 et 010.
3. Respecter l'ordre 006 → 007 → 009 ; la progression `implement` est déjà READY.
