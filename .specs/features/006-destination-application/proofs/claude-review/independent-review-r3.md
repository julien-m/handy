# Revue indépendante, round 3 — 006-destination-application

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`, base relue `2178406` (correction 006 : `c352eac`).
- Outil : `/opt/homebrew/bin/livespec` (`livespec` `71f1e0d`), avec `--model claude-opus-5-5 --review-max-chars 180000` sur chaque commande.
- Je n'ai modifié ni `spec.md`, ni `plan.md`, ni les contrats, ni le code. Aucun cargo.
- Sources : `spec.md` `4e6ed7e0…0a875a1` (inchangée), `plan.md` `cd795f2b…c7247`, `contracts/destination-contract.md` `0675b52c…aac88`.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Préparation de la revue de plan | **1 lot** (159 021 caractères), aucune erreur, `context_hash` `c262a362…cb960` | sortie de `--prepare-review plan` |
| Revue de plan (lot unique, relecteur natif indépendant) | **complete, ready** ; 18/18 `covered` ; 0 ambiguïté ; 0 bloquant ; 3 avertissements, 2 infos, 2 portées techniques justifiées | [`.reviews/plan.json`](../../.reviews/plan.json), sha256 `90ba7593…af06a77` |
| Tour de revue | round 2 sur 2 corrections possibles, `round_cap_reached: false` | [`.reviews/plan.rounds.json`](../../.reviews/plan.rounds.json) |
| Clarify (`--progression plan`) | READY | [progression-plan-r3.txt](progression-plan-r3.txt) |
| Analyze (`--pre-impl`) | exit 0 ; aucun constat ; couverture structurelle 17/17 | [analyze-pre-impl-r3.json](analyze-pre-impl-r3.json) |
| `--progression implement` | **READY** | [progression-implement-r3.txt](progression-implement-r3.txt) |
| `delivery lane` | **full** (`criticality_high`, rayon `shared`) | [delivery-lane-r3.json](delivery-lane-r3.json) |

Le `context_hash` complet de la préparation et du reçu est `c262a3626b1935648dd79b336ac1df986685a0f0806a57494317960910ccb960`.

## Bloquant du round 2 : traité

- La ligne 40 cite désormais la spec et la carte d'implémentation de la 008 en chemins `code`. La 008 n'entre plus dans la fermeture normative : la préparation tient en un lot, sans dépendre de `.specs/ship.md` (exclu de git).
- Le tableau de décisions ne désigne plus `.specs/ship.md` comme source des noms D2/D3 : ils font foi dans le plan et son contrat.

## Avertissements du round 2

| Round 2 | État |
| --- | --- |
| Constructions de `ValidationContext` | **Traité.** Quatre constructions de production, dont `edit::revalidate_mutation` (`edit.rs:98`), plus les 29 constructions de `src-tauri/tests/` |
| Lecture atomique de la disposition pour 009 | **Traité.** `DestinationManager::claim_enter(dictation_id, destination_generation) -> Result<EnterClaim, EnterRefusal>` sous le verrou de `set_disposition`, avec tests (accordé une fois, refus `Stale`/`Disposition`/`AlreadyClaimed`, `Recover` ultérieur sans révocation) |
| Raison d'épingle | **Traité.** `pinned_app_not_running` ajouté par la 009 à l'énumération de la 006 ; `activation_failed` pour une épingle lancée mais non activée |
| Précédence du refus de plateforme (`read()?` avant `plan`) | Non repris ; reste un cas marginal non bloquant |

## Concordance des contrats

- **006 ↔ 007.** `DestinationContext { dictation_id, destination_generation, source, target: Option<ResolvedTarget { pid, identity }> }` et `bundle_id()` couvrent le besoin de la 007 (« PID, bundle ID, destination generation »). `method_outcome` reprend les huit valeurs de `PasteOutcome.kind`, `transaction_id` est porté. Le plan 007 cite encore `LiveSettingsEffects` (inexistant) ; c'est à corriger côté 007.
- **006 ↔ 009 ↔ 010.** `RecoveryDisposition { Pending, Insert, Recover, Cancelled }`, `set_disposition` monotone, `claim_enter`, `RecoveryPort::before_insertion → PreInsertionVerdict { Insert, Recover(RecoveryReason) }`, `InsertionOutcome` (13 champs) et les raisons `app_closed | activation_failed | insertion_failed | pinned_app_not_running` sont identiques dans le plan 006, son contrat, le plan 009 et le plan 010. `PasteDiagnostic { dictation_id, transaction_id, late_read_observed }` est identique dans les plans 007 et 010 et le contrat 010.
- **D2/D3 et code 008.** J'ai vérifié dans `src-tauri/src/settings_service/` les noms livrés que le plan cite : `DomainPlan::{Read, Change, LiveOnly}`, `ChangeKind::{Scalar, Apply}`, `ValidationContext`, `edit::revalidate_mutation`, `SettingsRepository::{read_snapshot, commit_mutation, initialize}`, `tauri_adapter::{write, set, edit, repair, set_prepared, ordered}`, `OUTPUT_KEYS: [&str; 10]`, `ErrorDetails.field: Option<&'static str>`. Les noms nouveaux (`LiveOperation`, `LiveContext`, `managed_state`, `PlatformCapability`, `ErrorReason`, `ErrorItem`) n'existent pas encore : l'étape 0 les crée.

## Reports à l'implémentation (`carry_over`, non bloquants)

1. **Moment de construction du contexte.** Le plan construit `DestinationContext` après `paste_delay_ms`, mais le délai et la garde vivent dans l'adaptateur (`clipboard.rs`). Fixer l'ordre : délai, garde, `before_insertion`, `set_disposition(Insert)`, puis dispatch.
2. **Portée de l'étape 0.** Les opérations live (D2) et `details.items` (D3) ne servent aucune exigence de la 006 ; le relecteur les a acceptées comme portée technique décidée par l'orchestrateur pour 001, 003, 005 et 009. La traçabilité vers FR-006 seule est trompeuse.
3. **Ordonnancement.** Les étapes 2 et 3 (gestionnaire de session, clé de réglage et CLI) attendent l'étape 1, donc la géométrie UI validée. Le travail non UI pourrait avancer avant.
4. **Info.** La liste de champs de `DestinationSession` et l'ER omettent `enter_claimed`. `clipboard_handed_over` laisse la disposition à `Insert` : `claim_enter` accorderait, et seul le filtre de la 009 sur `method_outcome` supprime l'Entrée (ce que fait le plan 009).
5. **Info sans objet.** Le « `false` littéral » de la ligne de descripteur est le champ `nullable` de la macro `inventory!` (vérifié dans `descriptor_data.rs`), pas la valeur par défaut.

## Constats bloquants restants

Aucun.
