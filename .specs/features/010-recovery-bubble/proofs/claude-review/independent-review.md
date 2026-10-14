# Relecture indépendante — 010-recovery-bubble

- **Date :** 2026-10-14
- **Relecteur :** Claude Code (claude-opus-5-5), indépendant de l'auteur de la spec et du plan
- **Outil :** `/opt/homebrew/bin/livespec`, `--model claude-opus-5-5 --review-max-chars 180000` pour toutes les commandes
- **Périmètre :** [spec](../../spec.md) (sha256 `f5e9438a…e4e5`), [plan](../../plan.md) (sha256 `597f728c…e18fe2`) et [contrat local](../../contracts/recovery-contract.md). La revue de plan couvre 18 lots : 9 lots de contexte (plan et contrat 006, spec 007, plan 008) et 9 lots portant les exigences (spec 009, plan 008), suivis d'une synthèse. Elle compare aussi le plan à la 008 livrée dans `dev`. Aucun `spec.md`, `plan.md` ni code modifié.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Revue de spec | complète, **prête** (round 1) | [`.reviews/spec.json`](../../.reviews/spec.json), context `25476774…7f34` |
| Progression `plan` | READY | `livespec validate … --progression plan` |
| Revue de plan | complète, **non prête** (round 1 ; 2 constats bloquants ; 15/15 exigences `covered`) | [`.reviews/plan.json`](../../.reviews/plan.json), context `cd00d943…4447` |
| Analyze (`--pre-impl`) | exit 1 : 1 HIGH (« Semantic review has unresolved blocking findings »), couverture 15/15 | commande en lecture seule |
| Progression `implement` | **BLOCKED** `analyze_not_ready:blocked` | `livespec validate … --progression implement` |
| Voie de livraison | **full** (`criticality_high`, blast radius shared) | `livespec delivery lane --feature 010-recovery-bubble` |

## Constats bloquants (plan)

1. **Étape 1.** Elle impose de relever le comportement Windows et Linux (clic, focus) avant le travail d'UI de production, et arrête le câblage UI et natif sur un échec. Le changelog du plan rend les preuves natives Windows et Linux « mandatory ».
2. **Étape 7.** Elle déclare BLOCKED toute preuve OS manquante et refuse la certification de livraison tant qu'elle manque.

Selon la constitution (décision du 2026-10-13), les preuves Windows et Linux sont des *pending release evidence* : elles ne bloquent jamais les étapes d'implémentation, la complétion sur la branche d'intégration ni le merge dans `dev`. Seule la preuve native macOS est exigée avant `Implemented`.
*Correction proposée :* `spec-refine 010 plan` : la preuve macOS conditionne les étapes 2 à 7 ; les vérifications Windows (bulle réduite) et Linux (échec de méthode seul, X11, Wayland, layer-shell) deviennent des *pending release evidence* avec la commande exacte.

## Écarts avec la 008 livrée

- Le plan impose les noms du plan 008, dont `LiveSettingsEffects`, absent du code livré (qui expose `LiveEffectsPort`/`TauriEffects` et `tauri_adapter::{write, set, edit, set_prepared, repair}`). L'impact est faible : le seul réglage, `show_recovery_bubble_without_text_field`, est un booléen scalaire que le domaine `output` et `validate_cli` livrés acceptent sans changer le framework.
- Les codes d'erreur, les sorties et l'enveloppe JSON du contrat concordent avec la 008.

## Concordance avec la 006, la 007 et la 009

- **007 :** huit résultats, dont quatre déclenchent la récupération (`not read`, `permission missing`, `uncertain`, `insertion failed`) et `clipboard handed over` ni succès ni échec ; propriété de la restauration ; Copy qui change le `changeCount` ; budget AX de 250 ms (dont 150 ms de relecture). Tout cela concorde avec 007 AC-009, AC-012, AC-017 et AC-018.
- **Écart avec la 007 :** la 007 attend que la bulle annonce « may have been pasted after all » après une lecture tardive. Le plan 010 n'a qu'une explication statique et ne consomme pas ce diagnostic.
- **Écart avec la 006 :** le plan consomme `InsertionOutcome` v1 « inchangé », sans `method_outcome`, et ajoute un adaptateur 007 séparé. Or le plan 006 actuel porte déjà `method_outcome` avec les huit résultats. Deux canaux pour la même information risquent de diverger.
- **Écart avec la 009 :** le veto d'Entrée (« revokes automatic submission ») et le `RecoveryDisposition` du plan 009 décrivent le même mécanisme sous deux contrats. Il faut une seule définition, décidée avant le dispatch. Le comportement concorde avec 009 AC-005.
- **Raisons :** la table canonique distingue « <App> is pinned but not open » (application épinglée fermée) de « Could not bring <App> forward » (application lancée mais non activée). Cela tranche correctement l'ambiguïté relevée dans la spec ; la 009 doit transmettre le sous-type fermée/délai dépassé.

## Autres constats

- **Cohérence QE :** la `## QE Analysis` du plan (High, *shared/cross-system*, Data) est proche de la `## Quality Engineering` de la spec (High, *Shared*, Data puis Regression). La `## QE Analysis` et la `## Quality Engineering` de la spec concordent entre elles.
- **Testabilité des AC :** tous les AC sont testables. Une ambiguïté non critique est reportée (raison affichée pour une application épinglée lancée mais non activable, tranchée par le plan). Story 4 rend la conservation dans l'historique inconditionnelle, alors qu'AC-005 la conditionne à un enregistrement réussi : AC-005 fait foi.
- **Dépendances :** l'étape 4 exige les 006, 007 et 009 livrées ; la 010 vient en dernier.
- **Next Action :** le plan cite `--model=gpt-6`.

## Recommandations

1. Lancer `spec-refine 010 plan` pour ramener les preuves Windows et Linux au statut *pending release evidence* (étapes 1 et 7, changelog), puis refaire une revue de plan indépendante (round 2), Analyze et la progression `implement`.
2. Au même moment, faire de `method_outcome` (006) la source unique, définir le contrat de décision de récupération partagé avec la 009, et décider du traitement du diagnostic de lecture tardive de la 007.
