# Revue indépendante, round 2 — 010-recovery-bubble

- Relecteur : Claude (`claude-opus-5-5`), indépendant de l'auteur de la spec et du plan. Date : 2026-10-14. Branche `dev`, HEAD `18ae22c` (la correction du plan 010 est ce commit).
- Outil : `/opt/homebrew/bin/livespec`, avec `--model claude-opus-5-5 --review-max-chars 180000` à chaque commande.
- Rien n'a été modifié dans `spec.md`, `plan.md` ni le code. Pas de cargo.
- Sources : `spec.md` `f5e9438a…e4e5` (inchangée depuis le round 1), `plan.md` `46e67713…0f3c`, `contracts/recovery-contract.md`.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Revue de spec | Spec inchangée, le reçu du round 1 reste courant (complete, ready) | [`.reviews/spec.json`](../../.reviews/spec.json) |
| Clarify (`--progression plan`) | READY | [progression-plan-r2.txt](progression-plan-r2.txt) |
| Préparation de la revue de plan | **Refusée** : `paired_review_batch_budget_exceeded` (plus de 64 lots, aucun lot produit), `context_hash` `46a12f39…5d3` | sortie de `--prepare-review plan` |
| Revue de plan, ingestion | Non faite : rien n'a été préparé, aucun reçu fabriqué | — |
| Analyze (`--pre-impl`) | exit 1 : 1 HIGH (revue sémantique de plan périmée ou incomplète) ; couverture structurelle 15/15 | [analyze-pre-impl-r2.json](analyze-pre-impl-r2.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:incomplete` | [progression-implement-r2.txt](progression-implement-r2.txt) |
| `delivery lane` | **full** (`criticality_high`, rayon `shared`) | [delivery-lane-r2.json](delivery-lane-r2.json) |

## Constat bloquant : la revue de plan ne peut pas être préparée

À la ligne 37, le plan écrit : « Inspect [008's implementation record](…) and the merged … as the framework reference; [008's plan](../008-settings-cli/plan.md) is historical context ». LiveSpec ne classe un lien en contexte que s'il suit immédiatement « Inspect », « Informative: » ou « Non-normative: » (`_CONTEXT_PREFIX`, `validator/semantic/review_files.py`). Le lien vers le plan 008 devient donc **normatif**, alors que l'auteur le voulait historique ; sa note de la ligne 39 (« These inspected documents do not import their entire obligation inventories ») ne change rien.

Ce lien entraîne `implementation.md`, `progress.md`, les journaux et quatre audits de la 008, ainsi que `.specs/changelog.md`, les `tool-defects` et `.specs/ship.md`. L'appariement dépasse alors 64 lots.

Sur un clone neuf, `.specs/ship.md` est absent (fichier exclu de git) : la préparation échouerait aussi sur `unresolved_reference`.

Simulation sur une copie jetable : avec « Inspect [008's plan](…) as historical context » et les liens 008 des plans 006 et 009 rendus contextuels, la préparation produit **33 lots** (4,57 M caractères). Toutes les sources externes à la 010 y deviennent du contexte.

La revue lit aussi le plan 006 comme contexte, et un changement d'octets dans un contexte invalide le reçu. Il faut donc corriger le plan 006 avant de refaire la revue de la 010.

## Bloquants du round 1

| Round 1 | État au round 2 |
| --- | --- |
| Étape 1 : relevés Windows et Linux exigés avant l'UI | **Corrigé.** Seule la preuve macOS conditionne les étapes ; un échec Windows ou Linux constaté plus tard est un constat de release (l. 366, 370) |
| Étape 7 : BLOCKED sur toute preuve OS manquante | **Corrigé.** Preuves Windows et Linux en *pending release evidence* avec la commande exacte ; AC-006 prouvée par les tests automatisés de politique de plateforme (l. 426-430, 463, 481, 525) |
| `method_outcome` comme source unique (pas d'adaptateur 007 séparé) | **Corrigé** (l. 125, 526 ; contrat) |
| Contrat de récupération partagé avec la 009 | **Corrigé.** La 010 implémente seulement `RecoveryPort::before_insertion` → `PreInsertionVerdict { Insert, Recover(RecoveryReason) }`, n'appelle jamais `set_disposition` et ne publie aucun autre veto (l. 99-100, 123, 528) |
| Diagnostic de lecture tardive de la 007 | **Corrigé.** `PasteDiagnostic { dictation_id, transaction_id, late_read_observed }` met à jour la bulle corrélée par `dictation_id` et le `transaction_id` de l'`InsertionOutcome`, sans second résultat, sans Entrée et sans changer la disposition (l. 127, 527) |
| Noms de la 008 jamais livrés | **Corrigé.** Usage de `descriptor_data.rs`, `OUTPUT_KEYS`, `validate_cli`/`validate_ui` et `tauri_adapter::write`, tous présents dans `dev` (l. 37, 151, 529) |

## Concordance des contrats

- **010 ↔ 006.** Les 13 champs d'`InsertionOutcome`, `RecoveryDisposition`, `set_disposition`, `RecoveryPort` et `PreInsertionVerdict` sont identiques dans le plan 006, son contrat, `.specs/ship.md` et le contrat 010.
- **010 ↔ 007.** Huit valeurs de `method_outcome`, dont quatre déclenchent la récupération. `clipboard_handed_over` n'est ni un succès ni un échec. `PasteDiagnostic` porte les mêmes trois champs dans le plan 007 (l. 147). La 007 a annoncé que la 010 serait le consommateur de « may have been pasted after all », et la 010 tient cet engagement.
- **010 ↔ 009 : écart de raison, à corriger avant l'étape 2 de la 010.**
  - La table de la 010 distingue deux cas (l. 97-98, 497-498) : une épingle non lancée reçoit « 009's pinned-app reason » (« is pinned but not open ») ; une application lancée mais non activée reçoit `activation_failed` (« Could not bring <App> forward »).
  - Le plan 009 émet une raison unique `pinned app unavailable` pour l'application fermée comme pour l'expiration du délai d'activation. Il garde un « diagnostic subtype » qu'il ne nomme pas (plan 009 l. 101, 530).
  - Il faut fixer avec la 009 le nom de la variante et le transport du sous-type. Sans cela, une épingle lancée mais hors délai serait affichée « not open ».
- **Réglage.** `show_recovery_bubble_without_text_field` est un booléen ajouté par une ligne de descripteur et une entrée `OUTPUT_KEYS`, sans effet `effects.rs`. Il est compatible avec la 008 livrée et avec les longueurs de tableau prévues par la 006 (11 clés, puis +1).

## Avertissements (non bloquants)

- Ligne 23 : `.specs/ship.md`, non versionné, est cité comme l'endroit où les contrats sont fixés. Ces contrats font déjà foi dans le plan 006 ; mieux vaut le citer lui.
- Le spike natif de l'étape 1 (l. 366) garde la clause « their adapters are written and compiled » pour Windows et Linux. C'est conforme à la constitution (compilation sur chaque cible).

## Prochaines étapes

1. `spec-refine 010 plan` :
   - ligne 37 : « Inspect [008's plan](…) as historical context » ;
   - aligner la raison d'épingle avec la 009.
2. Corriger d'abord le plan 006, puis préparer la revue de plan 010 (environ 33 lots), l'ingérer, lancer Analyze et `--progression implement`.
