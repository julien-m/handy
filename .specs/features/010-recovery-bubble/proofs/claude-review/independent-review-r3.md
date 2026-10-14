# Revue indépendante, round 3 — 010-recovery-bubble

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`, base relue `2178406` (correction 010).
- Outil : `/opt/homebrew/bin/livespec` (`livespec` `71f1e0d`), avec `--model claude-opus-5-5 --review-max-chars 180000` sur chaque commande.
- Je n'ai modifié ni `spec.md`, ni `plan.md`, ni les contrats, ni le code. Aucun cargo.
- Sources : `spec.md` `f5e9438a…08cee4e5` (inchangée), `plan.md` `de44c762…05ca4e23`, `contracts/recovery-contract.md` `9a9ec676…f731e87d`.
- Contexte inspecté : plan 006 `cd795f2b…c7247`, contrat 006 `0675b52c…aac88`, spec 007 et spec 009.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r3.txt](progression-plan-r3.txt) |
| Préparation de la revue de plan | **18 lots** (9 lots de contexte, 9 lots portant les 15 exigences), aucune erreur ; `context_hash` `db49fe1d…d047381` | sortie de `--prepare-review plan` |
| Revue de plan | 18 lots relus chacun par un relecteur natif indépendant, puis synthèse (transport de 130 132 caractères, sous le budget) ; **complete, ready** | [`.reviews/plan.json`](../../.reviews/plan.json), sha256 `ce3a1158…a4f77c2d` |
| Conclusions finales | 15/15 `covered` ; 0 ambiguïté ; 0 bloquant ; 13 avertissements et 5 infos reportés | reçu |
| Tour de revue | round 2 sur 2 corrections possibles, `round_cap_reached: false` | [`.reviews/plan.rounds.json`](../../.reviews/plan.rounds.json) |
| Analyze (`--pre-impl`) | exit 0 ; aucun constat ; couverture structurelle 15/15 | [analyze-pre-impl-r3.json](analyze-pre-impl-r3.json) |
| `--progression implement` | **READY** | [progression-implement-r3.txt](progression-implement-r3.txt) |
| `delivery lane` | **full** (`criticality_high`, rayon `shared`) | [delivery-lane-r3.json](delivery-lane-r3.json) |

## Portées signalées par les lots et résolues en synthèse

Cinq entrées de portée non approuvée figuraient au registre des lots. La synthèse les a résolues sur le fond, avec citations :

- **Mise à jour « may have been pasted after all »** (lots 012 et 014). 007 AC-012 la demande explicitement : la bulle de récupération doit l'annoncer. La 010 met seulement à jour l'explication en place, sans seconde bulle, sans changer le résultat et sans Entrée.
- **Avertissement d'historique non sauvegardé** (lots 012 et 018). AC-005 conditionne la mention « gardé dans l'historique » à la réussite de la sauvegarde. L'explication doit rester véridique en cas d'échec, et la 006 exige que le texte reflète le statut réel. Le libellé reste à fixer (avertissement reporté).
- **Échec de Copy qui garde la bulle** (lot 014). Gestion d'erreur technique justifiée : fermer la bulle perdrait le texte.

## Bloquants du round 2 : traités

| Round 2 | État au round 3 |
| --- | --- |
| Lien normatif vers le plan 008 (préparation refusée, plus de 64 lots) | **Corrigé.** Documents 008 cités en chemins `code` ; 18 lots, sans `.specs/ship.md` |
| Écart de raison d'épingle avec la 009 | **Corrigé.** `pinned_app_not_running` → « <App> is pinned but not open » ; `activation_failed` → « Could not bring <App> forward » pour une épingle lancée non activée. Identique dans les plans 006, 009 et 010 |
| `.specs/ship.md` cité comme source des contrats | **Corrigé.** Contrats cités depuis le plan 006 seulement |

## Concordance des contrats

- **010 ↔ 006.**
  - Repris tels quels : `InsertionOutcome` (13 champs), `RecoveryDisposition`, `set_disposition`, `RecoveryPort::before_insertion → PreInsertionVerdict { Insert, Recover(RecoveryReason) }`, ainsi que les raisons `app_closed | activation_failed | insertion_failed | pinned_app_not_running`.
  - La 010 n'appelle jamais `set_disposition` ni `claim_enter`.
- **010 ↔ 007.** `PasteDiagnostic { dictation_id, transaction_id, late_read_observed }` est identique dans le plan 007, le plan 010 et le contrat 010. Les huit valeurs de `method_outcome` sont reprises : quatre déclenchent la récupération, et `clipboard_handed_over` n'est ni un succès ni un échec.
- **010 ↔ 009.** Le découpage des raisons d'épingle suit 010 AC-003. La spec 009 AC-003 garde une raison unique « pinned app unavailable » : écart de libellé côté spec 009, non bloquant (voir le rapport 009).
- **Noms 008.** `SettingDescriptor` (lignes de `descriptor_data.rs`), `SettingsMutation`, `validation::{validate_cli, validate_ui}`, `PublicSettings`, `Revision`, `SettingsError`, `tauri_adapter::write` et `OUTPUT_KEYS` existent dans `src-tauri/src/settings_service/`.
  - Inexactitude sans effet : la ligne 37 range `commit_mutation` parmi les noms « announced but never shipped ». Or `SettingsRepository::commit_mutation` existe (`transaction.rs:58`), et le plan 006 le liste comme livré. La 010 ne l'utilise pas ; seule la phrase est à corriger.

## Reports à l'implémentation (`carry_over`, non bloquants)

- **Minimum de 4 s face à une nouvelle dictée.** La 010 ferme la bulle à la dictée suivante, comme le veut sa spec. Le plan et le contrat 006 disent encore que la 010 hérite du minimum de 4 s ; il faut une décision datée côté 006.
- **Surfaces Windows et Linux.** La section surface qualifie le repli qui vole le focus de « failed gate ». Il faut la reformuler en *pending release evidence* : seul macOS conditionne.
- **Ordre et appartenance :**
  - fixer qui refait la vérification de premier plan et d'annulation après la classification (jusqu'à 150 ms) ;
  - dans la table de décision, la ligne « no selected target » passe avant « not editable » ;
  - l'étape 3 doit attendre les étapes 0 et 3 de la 006 (`OUTPUT_KEYS`, descripteurs, schéma) ;
  - l'étape 4 ne doit pas dupliquer, dans `actions.rs`, la transcription et le statut d'historique que la 006 possède.
- **Portée et contrats :**
  - classification effectuée même réglage désactivé ;
  - « 007 clipboard executor » non défini, et aucun chemin Copy hors smart paste ;
  - diagnostic tardif reçu avant l'accusé de présentation ;
  - détail « Some text may have reached application » affiché aussi pour `permission_missing` (tension au niveau de la spec) ;
  - Constitution Check sans les règles de preuves natives et de portée minimale.
- **Infos :**
  - `Cancelled` attribué au texte vide (la 006 dit `skipped`) ;
  - cas pré-insertion implicites dans SC-001 ;
  - timers de la 007 retestés ;
  - moment de la comparaison visuelle ;
  - « existing boolean » ;
  - script `test:playwright`.

## Constats bloquants restants

Aucun.
