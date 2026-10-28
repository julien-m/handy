# Revue indépendante, round 2 — 006-destination-application

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14. Branche `dev`, HEAD `18ae22c` (correction du plan 006 : `853d2d5`).
- Outil : `/opt/homebrew/bin/livespec`, avec `--model claude-opus-5-5 --review-max-chars 180000` sur chaque commande.
- Je n'ai modifié ni `spec.md`, ni `plan.md`, ni le code. Aucun cargo.
- Sources : `spec.md` `4e6ed7e0…0a875a1` (inchangée depuis le round 1), `plan.md` `2142ca18…bcca1a`.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Revue de spec | inchangée depuis le round 1 : le reçu reste courant (complete, ready) | [`.reviews/spec.json`](../../.reviews/spec.json) |
| Clarify (`--progression plan`) | READY | [progression-plan-r2.txt](progression-plan-r2.txt) |
| Préparation de la revue de plan | **refusée** : `paired_review_batch_budget_exceeded` (plus de 64 lots ; aucun lot produit), `context_hash` `e026afce…ac1043` | sortie de `--prepare-review plan` |
| Revue de plan, ingestion | non faite : aucune préparation à soumettre ; je n'ai pas fabriqué de reçu | — |
| Analyze (`--pre-impl`) | exit 1 : 1 HIGH (« Stale, invalid or incomplete semantic review ») ; couverture structurelle 17/17 | [analyze-pre-impl-r2.json](analyze-pre-impl-r2.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:incomplete` | [progression-implement-r2.txt](progression-implement-r2.txt) |
| `delivery lane` | **full** (`criticality_high`, rayon `shared`) | [delivery-lane-r2.json](delivery-lane-r2.json) |

## Constat bloquant : la revue de plan ne peut pas être préparée

La ligne 40 du plan dit : « Inspect [008's spec](../008-settings-cli/spec.md), [its implementation map](../008-settings-cli/implementation.md) and … ». LiveSpec classe un lien comme contexte seulement quand « Inspect » le précède immédiatement (`_CONTEXT_PREFIX` dans `validator/semantic/review_files.py`). Le second lien est donc normatif.

`implementation.md` de la 008 tire alors toute sa fermeture. Sa phrase « Read [the functional source] … ; inspect [current progress] … [execution history] » contient « Read » ; les liens de cette phrase sont donc normatifs eux aussi. La fermeture comprend :

- `plan.md`, `progress.md` et `logs/2026-10-13.md` de la 008, et quatre rapports d'audit de la 008 ;
- `.specs/changelog.md`, trois `.specs/tool-defects/…` et `.specs/ship.md`.

Elle ajoute environ 400 000 caractères d'obligations normatives. L'appariement spec × plan dépasse alors 64 lots.

Deux effets :

1. Au budget imposé de 180 000, la revue de plan est impossible. Le plan ne peut donc pas atteindre READY.
2. `.specs/ship.md` est exclu de git (`.git/info/exclude`). Sur un clone neuf, la préparation échoue aussi sur `unresolved_reference:unread_source:.specs/ship.md:FileNotFoundError`, que j'ai reproduit sur une copie sans ce fichier.

Une simulation sur une copie jetable a confirmé la correction. Avec « Inspect [008's spec](…). Inspect [its implementation map](…). Inspect the delivered code … », la préparation passe à **6 lots** (1,06 M caractères), et la 008 entre comme contexte.

La correction appartient à l'auteur du plan (`spec-refine 006 plan`). Elle relance ensuite la revue de plan au round 2.

## Bloquants du round 1

| Round 1 | État au round 2 |
| --- | --- |
| Preuves natives Windows érigées en gates | **Corrigé.** Constitution reprise (l. 66) ; l'étape 1 n'est gatée que sur macOS (l. 651, 691) ; Windows est en *pending release evidence* avec la commande exacte (l. 656, 676) |
| Refus « plateforme non supportée » en `unavailable`/5 | **Corrigé (D4).** `validation_failed`/2, `details.reason = "unsupported_platform"`, décidé dans le validateur pur, hors ligne comme en ligne (l. 119, 212, 722 ; contrat) |
| Noms de la 008 livrée | **Corrigé.** `LiveSettingsEffects` déclaré inexistant (l. 53) ; écriture UI par `tauri_adapter::write` ; tous les noms cités existent dans `dev` (`RoutedBackend`, `sequence::write_change`, `protocol::contract_schema`, `checked_schema_matches_the_actual_dto_generator`, `validate_context`, `OUTPUT_KEYS`) |
| `DestinationContext` et correspondance avec `PasteOutcome` | **Corrigé.** Section « DestinationContext and RecoveryDisposition » et table des huit valeurs de `method_outcome` |
| Classification QE alignée sur la spec | Corrigé (High, Shared) |

## Concordance des contrats

- **D2/D3/D4 avec le code 008.** `DomainPlan::LiveOnly(Value)`, `DomainCall`, `DomainRequest`, `ValidationContext {prompt_ids, updates_allowed}`, `ErrorDetails` (5 champs) et `SettingsBackend::{commit_change, is_live}` existent tels que le plan les décrit. `LiveContext::commit_change` se branche bien sur `RoutedBackend::commit_change` → `sequence::write_change`. `DomainPlan` ne dérive aucun trait : y ajouter `Box<dyn LiveOperation>` ne casse rien. Les noms concordent avec `.specs/ship.md` (« Contrat D2/D3 »).
- **006 ↔ 007.** `DestinationContext { dictation_id, destination_generation, source, target: Option<ResolvedTarget> }` et `bundle_id()` répondent au besoin de la 007 (« PID, bundle ID, destination generation »). `method_outcome` reprend les huit valeurs de `PasteOutcome.kind` sans les transformer, et `transaction_id` est porté.
- **006 ↔ 009 ↔ 010.** `RecoveryDisposition { Pending, Insert, Recover, Cancelled }`, `set_disposition` monotone, `RecoveryPort::before_insertion → PreInsertionVerdict { Insert, Recover(RecoveryReason) }` et `InsertionOutcome` (13 champs) sont identiques dans le plan 006, son contrat, `.specs/ship.md`, le plan 009 et le plan 010.

## Avertissements (non bloquants)

- **Constructions de `ValidationContext`.** Le plan en annonce trois en production (`operations::set`, `LiveSettings::validate_ui_value`, `domains::output`). Il en existe une quatrième, la revalidation sous verrou dans `settings_service/edit.rs:98`, plus environ 28 dans `src-tauri/tests/`. Le compilateur les signalera. Prévoir `PlatformCapabilities::current()` pour `edit.rs` et une aide de test.
- **Précédence du refus de plateforme.** `domains::execute` appelle `backend.read()?` avant `plan`. Un store hors ligne inutilisable donne donc `unavailable` avant `unsupported_platform`. Cas marginal à documenter ou à tester.
- **Côté 007** (hors périmètre de cette revue, à signaler à son correcteur) :
  - le plan 007 cite encore `LiveSettingsEffects` et `commit_mutation` (l. 35, 153) ;
  - il décrit `DestinationContext` sans le cas `target: None`, que le plan 006 traite (la 007 saute alors l'AX direct) ;
  - le nom `WorkspacePort` existe dans les deux features, avec deux rôles différents.
- **Question ouverte de `.specs/ship.md` (005 ↔ 006).** Une ligne d'historique sans audio, en cas d'échec IA, doit-elle valoir `history_status = saved` ? La décision revient à Julien. Le plan 006 garde la sauvegarde conditionnée au WAV.
- **Référence non versionnée.** Le plan désigne `.specs/ship.md` comme source des noms D2/D3 (l. 725). Ce fichier n'est pas versionné : les noms font foi dans le plan lui-même.

## Prochaines étapes

1. `spec-refine 006 plan` : rendre contextuels les liens vers la 008 à la ligne 40, avec un « Inspect » devant chaque lien.
2. Refaire la revue de plan (`--prepare-review plan` doit donner environ 6 lots), puis Analyze et `--progression implement`.
3. Ensuite seulement, refaire les revues de plan des 009 et 010, dont le plan 006 est une source.
