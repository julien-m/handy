# Revue indépendante, round 3 — 009-pin-mode-and-enter-rule

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`, base relue `2178406` (correction 009 : `243088e`).
- Outil : `/opt/homebrew/bin/livespec` (`livespec` `71f1e0d`), avec `--model claude-opus-5-5 --review-max-chars 180000` sur chaque commande.
- Je n'ai modifié ni `spec.md`, ni `plan.md`, ni le code. Aucun cargo.
- Sources : `spec.md` `35eb4a9d…ec47679d` (revue de spec du round 2, toujours courante), `plan.md` `31cb7f07…9125e80a`. Contexte inspecté : plan 006 `cd795f2b…c7247`, spec 006, spec 007, spec 010.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r3.txt](progression-plan-r3.txt) |
| Préparation de la revue de plan | **20 lots** (10 lots de contexte, 10 lots portant les 13 exigences), aucune erreur ; `context_hash` `b32944cf…c521a7` | sortie de `--prepare-review plan` |
| Revue de plan | 20 lots relus chacun par un relecteur natif indépendant, puis synthèse (transport de 142 254 caractères, sous le budget) ; **complete, non ready** | [`.reviews/plan.json`](../../.reviews/plan.json), sha256 `e17d0a93…62a47abf` |
| Conclusions finales | 12 `covered` ; AC-003 `ambiguous` (non critique) | reçu |
| Tour de revue | round 1 de ce nouvel état (le round précédent, sur un autre état, était ready) ; `round_cap_reached: false` | [`.reviews/plan.rounds.json`](../../.reviews/plan.rounds.json) |
| Analyze (`--pre-impl`) | exit 1 : 1 HIGH (portée métier non approuvée, ci-dessous) ; couverture structurelle 13/13 | [analyze-pre-impl-r3.json](analyze-pre-impl-r3.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:blocked` | [progression-implement-r3.txt](progression-implement-r3.txt) |
| `delivery lane` | **full** (`criticality_high`, rayon `shared`) | [delivery-lane-r3.json](delivery-lane-r3.json) |

## Constat bloquant restant : l'indice `retired_key`

Le plan (section « One setting and idempotent migration », étape 3) ajoute :

- `descriptor::RETIRED_KEYS = [("auto_submit", "automatic_enter")]` ;
- `ErrorReason::RetiredKey` (`"retired_key"`) et `SettingsError::retired_key(replacement)` ;
- `details.field = "automatic_enter"` sur le refus de `auto_submit`.

Le refus de `auto_submit` en `not_found`/4 découle bien d'AC-004 (« No other automatic Enter or auto-submit control remains »). L'indice, lui, est une valeur publique nouvelle du contrat CLI (schéma, `settings-cli.md`, README). Aucune AC, FR ni clarification de la 009 ne la demande, et la 008 ne prévoit pas de clés retirées.

La constitution est explicite : « a plan mechanism that the spec does not require … is not built », et « Specify changed behavior before implementing it ».

Un lot l'a classé en portée métier bloquante. Quatre autres l'ont classé en portée technique. J'ai tranché sur le fond en synthèse : c'est un comportement observable, pas une nécessité technique, donc il bloque.

Deux corrections possibles, au choix de l'auteur :

1. retirer l'indice et garder un `not_found`/4 simple ;
2. faire approuver l'indice dans la spec : une clarification datée de la 009 qui demande ce comportement. Cela demande un `spec-refine` puis une nouvelle revue de spec.

Ce constat est apparu à ce round. Au round 2, le relecteur l'avait noté comme correction d'un bloquant du round 1, sans le confronter à la règle de portée minimale.

## Bloquants du round 2 : traités

| Round 2 | État au round 3 |
| --- | --- |
| Analyze HIGH, AC-006 sans référence de tâche | **Corrigé.** Références AC écrites une par une ; couverture structurelle 13/13 |
| Lien normatif vers la 008 (préparation impossible ou à 60 lots) | **Corrigé.** Documents 008 cités en chemins `code`, étape 0 de la 006 nommée dans le texte ; 20 lots, sans `.specs/ship.md` |
| Lecture atomique de la disposition | **Corrigé.** `DestinationManager::claim_enter(dictation_id, destination_generation) -> Result<EnterClaim, EnterRefusal>`, identique au plan et au contrat 006 |
| Raison d'épingle alignée avec la 010 | **Corrigé dans les plans.** `pinned_app_not_running` (épingle non lancée) et `activation_failed` avec `destination_source = pinned` (épingle lancée mais non activée en 1 s), sans sous-type libre |

## Question de l'auteur : AC-003 et sa raison unique

Jugement : **acceptable, non bloquant**. La synthèse conclut `ambiguous` avec une ambiguïté **non critique**, reportée à l'implémentation.

- Le comportement exigé par AC-003 est entièrement couvert :
  - aucune insertion ni repli ;
  - transcription conservée selon la 006 ;
  - épingle conservée ;
  - retour vers l'application quand elle se relance.
- Seul le libellé diverge. AC-003 donne la raison unique « pinned app unavailable », et le plan la porte par deux raisons typées.
- Ce découpage est cohérent avec la spec 010 :
  - son AC-003 prévoit les deux phrases « <App> is pinned but not open » et « Could not bring <App> forward » ;
  - ses Key Entities séparent déjà « pinned app unavailable » de « activation not confirmed ».
- Les deux specs se contredisent donc légèrement. Le plan tranche dans le sens de la 010, sur une décision d'orchestrateur notée seulement dans le plan.
- Recommandation : au prochain `spec-refine 009`, inscrire cette correspondance dans les Clarifications de la spec ou reformuler AC-003. Ce n'est pas une condition de passage.

## Concordance des contrats

- **009 ↔ 006.** `RecoveryDisposition`, `set_disposition` monotone, `claim_enter` et ses trois refus, `InsertionOutcome.method_outcome` et `DestinationContext` sont repris tels quels. Les trois résultats éligibles (`inserted_verified`, `read_observed`, `typed`) et les cinq suppressifs concordent avec la table de la 006 et 007 AC-012.
- **009 ↔ 010.** Les raisons `pinned_app_not_running` et `activation_failed` sont identiques dans les plans 006, 009 et 010 et le contrat 006.
- **Écarts restants (avertissements) :**
  - la croix « Remove destination » est décrite « FrontmostAtEnd with source `badge` », alors que la 006 donne `source = FrontmostAtEnd` à cette route. Sans effet sur l'Entrée (seul `Pinned` qualifie `pin_only`), mais les tests de la 009 doivent viser la valeur de la 006 ;
  - `resolved_target` de la 009 vaut `None` pour `FrontmostAtEnd`, alors que `DestinationContext.target` de la 006 porte l'application de premier plan lue à la garde. Il faut renommer le champ ou aligner son sens ;
  - le résumé dit que la 009 « étend » `RecoveryDisposition` ; elle ne fait que la lire via `claim_enter`.
- **D2/D3 et code 008.** Noms livrés vérifiés dans `src-tauri/src/settings_service/` :
  - `validation::{validate_cli, validate_ui, ValidationContext, SettingsMutation}` ;
  - `domains::{Domain, DomainPlan, DomainRequest, DomainChange, ChangeKind, production}`, `OutputDomain`, `OUTPUT_KEYS` ;
  - `operations::{SettingsBackend, execute}` (et `known_key`), `route::route` ;
  - `tauri_adapter::{write, set, edit, repair}` (alias `live_settings` dans `shortcut/mod.rs`), `LiveSettings` ;
  - `SettingsRepository::initialize`, `normalization::normalize` → `settings::apply_settings_migrations`, `public::PublicSettings` ;
  - `ErrorDetails.field: Option<&'static str>`.

  Les noms D2/D3 (`DomainPlan::Live`, `LiveOperation`, `LiveContext`, `managed_state`, `ErrorReason`, `PlatformCapability`) sont ceux que fixe l'étape 0 de la 006.

## Reports à l'implémentation (`carry_over`, non bloquants)

- La ligne « Full acceptance » des Resolved Test Commands inclut la preuve native Windows sans la marquer *pending release evidence*. Les étapes 1, 5 et 7 et la constitution disent l'inverse ; il faut corriger la ligne.
- La Constitution Check n'a pas de ligne pour la règle des preuves natives ni pour la portée minimale.
- Il manque `destination/context.rs` dans la liste de fichiers de l'étape 2.
- Le verrou de lecture brute « chord held » ne doit pas s'appliquer au smart paste : la 007 seule détecte les lectures.
- `enter_insertion.rs` teste des internes de la 007.
- Le retrait du délai de 50 ms du chemin classique n'est pas justifié.
- Trois issues ne sont pas fixées :
  - le démarrage avec un `automatic_enter` invalide ;
  - l'effet de `--expected-revision` sur `pin`/`unpin` ;
  - les transcriptions Linux à l'étape 5.
- Infos :
  - les diagrammes n'ont pas d'appel `claim_enter` ni de refus ;
  - des prérequis manquent : 008, étapes 2 et 3 de la 006, livraison de la 007 ;
  - la ligne de test CLI abrège le refus de plateforme ;
  - l'indice Linux `pin_only` relève d'AC-007 de la 006.

## Prochaines étapes

1. `spec-refine 009 plan` : retirer l'indice `retired_key` (ou l'approuver d'abord dans la spec), et corriger au passage la source `badge` de la croix et la ligne « Full acceptance ».
2. Refaire la revue de plan (20 lots), puis Analyze et `--progression implement`.
