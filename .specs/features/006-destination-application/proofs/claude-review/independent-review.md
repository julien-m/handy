# Revue indépendante — 006-destination-application

- Relecteur : Claude (`claude-opus-5-5`), sans part à l'écriture de la spec ni du plan. Date : 2026-10-14. Branche `dev`, HEAD `0fb75ae` (008 mergée).
- Outil : `/opt/homebrew/bin/livespec` en mode dev (`.specs/livespec.lock` → `/Users/julienm/projects/livespec`, commit `71f1e0d`). Budget `--review-max-chars 180000` sur toutes les commandes.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo, pas de build.
- Lu : [spec](../../spec.md), [plan](../../plan.md), [contrat de destination](../../contracts/destination-contract.md), [constitution](../../../../constitution.md), [spec 008](../../../008-settings-cli/spec.md), [implementation 008](../../../008-settings-cli/implementation.md), [spec 007](../../../007-smart-paste/spec.md) et [plan 007](../../../007-smart-paste/plan.md) (sections contrat de sortie, réglages, étape 7), et le code 008 livré : `settings_service/{errors.rs,validation.rs,native_changes.rs,tauri_adapter.rs,domains/mod.rs,domains/output.rs}`, `README.md` (codes de sortie).

## Reçus

| Revue | Lots | Résultat | `context_hash` | Reçu (sha256) |
| --- | --- | --- | --- | --- |
| Spec | 1 | complete, **ready**, round 1 | `9877a016…6ef997447` | [`spec.json`](../../.reviews/spec.json) `0654051d…f5dd67` |
| Plan | 2 + synthèse | complete, **not ready** (3 bloquants), round 1 | `7b2ac5da…2281a11ac` | [`plan.json`](../../.reviews/plan.json) `199e5aff…977036` |

Sources : `spec.md` `4e6ed7e0…0a875a1`, `plan.md` `06b6535d…196d282`, `contracts/destination-contract.md` `c9cdc584…a08590`. La première ingestion de la revue de spec a été refusée pour citations hors sections de l'exigence (`invalid_citation`) ; elle a été corrigée sans changer le jugement, au même état normatif (un seul round enregistré).

## Progression

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan.txt](progression-plan.txt) |
| Analyze (`--pre-impl`) | exit 1, 1 HIGH : revue sémantique de plan avec constats bloquants ; couverture structurelle 17/17 | [analyze-pre-impl.json](analyze-pre-impl.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:blocked` | [progression-implement.txt](progression-implement.txt) |
| `delivery lane` | **full** (criticité high, rayon shared) | [delivery-lane.json](delivery-lane.json) |

Blocage rapporté sans contournement : il faut corriger le plan puis refaire la revue de plan (round 2 sur 2 corrections possibles).

## Constats bloquants pour l'implémentation

1. **Preuves natives Windows érigées en gates, contraire à la constitution.** Étape 1 : « Both native platforms prove intended APIs… » ; l'étape 2 dépend de l'étape 1 ; étape 7 : « no … missing platform proof » ; QE : « Native feasibility P0/P1 … on both platforms ». La constitution (décision 2026-10-13) range ces observations en *pending release evidence* qui ne bloquent ni les étapes, ni la complétion sur la branche d'intégration, ni le merge dans `dev`. Sur un hôte macOS seul, l'implémentation s'arrêterait à l'étape 2. Correction : reprendre la formulation du plan 008 (compilation et tests automatisés Windows en gate, observation native Windows en attente avec la vérification exacte).
2. **Contrat de refus « plateforme non supportée » incompatible avec la 008 livrée.** Le plan et le contrat prévoient `unavailable`/exit 5 avec `reason/platform` dans `details`. Or, dans la 008 livrée :
   - `ErrorDetails` est fermé (`current_revision`, `outcome`, `field`, `offset`, `issues` ; le schéma JSON a `additionalProperties: false`) : il n'y a pas de champ `reason` ni `platform` ;
   - le README définit l'exit 5 comme « busy, starting, older, or the store cannot be used », avec « read again before retrying » ; un agent réessaierait un refus permanent ;
   - le précédent livré pour un refus d'environnement (`updates_allowed` → `ValidationReason::UpdaterDisabled`) est `validation_failed`/2 ;
   - la 007 refuse `smart_paste` hors macOS en `validation_failed`/2.

   Correction : un contrat unique avec la 007 (par exemple `validation_failed`/2 avec un nouveau `ValidationReason` porté par `field`), ou une extension explicite du schéma et du README de la 008.

## Écarts entre le plan et la 008 livrée

| Plan 006 | 008 livrée (HEAD `0fb75ae`) | Effet |
| --- | --- | --- |
| Réutiliser `LiveSettingsEffects` | N'existe pas : effets dans `effects.rs` derrière `LiveSettings` (un test vérifie que chaque effet de descripteur est appliqué ou lu à l'usage) | Renommer ; déclarer l'effet « lu à l'usage » de la nouvelle clé |
| Commande UI « same SettingsMutation and pure validation/capability guard » | Écritures UI via `settings_service::tauri_adapter::{write,set,edit,set_prepared,repair}` (AGENTS.md interdit tout autre écrivain) | Nommer `tauri_adapter::write`, ou `set_prepared` + `native_changes` si la capacité est vérifiée avant le commit |
| Garde de capacité partagée UI/CLI | `ValidationContext {prompt_ids, updates_allowed}` sans capacité ; `Domain::plan(request, &AppSettings)` pur ; hors ligne, aucune instance ne peut lire le premier plan | Il faut décider comment la CLI hors ligne évalue « frontmost illisible » (statique : Linux seulement ?) |
| Ajout au domaine `output` | `OUTPUT_KEYS: [&str; 10]` fixe, tout autre clé → `not_found` ; 63 descripteurs inventoriés ; parité UI/CLI générée (SC-001 de 008) ; `contracts/settings-cli.schema.json` énumère les clés de `field` | Étendre l'inventaire, la parité, le schéma de contrat, `settings-cli.md` et le README |
| Lecture « value plus support » | `output get` renvoie la valeur projetée par `PublicSettings` | Forme de donnée nouvelle à spécifier (capacité dans `data`) |
| Exits 0/1/2/3/4/5, enveloppe unique | Identiques (`SettingsErrorCode::exit_code`) | Conforme |
| Révision, CAS, no-op, verrou | `commit_mutation`, `read_snapshot`, `SettingsRepository`, `SettingsEdit` | Conforme sur le fond |

## Interface avec la 007 (étape 7 du smart paste)

- La 007 attend un `DestinationContext` (PID, bundle ID, `destination_generation`) de 006/009. Son AX direct (AC-017) n'agit que si l'élément focalisé appartient au processus de destination. Le plan 006 définit seulement un `InsertionPort` qui enveloppe une garde, sans dire quel contexte il transmet à la méthode, ni pour le cas `FrontmostAtEnd` (destination résolue au moment de l'insertion). **À ajouter au contrat de destination** avant l'étape 7 de la 007.
- Noms divergents : `session_id`/`cancel_generation` (006) contre `dictation_id`/`destination_generation`/`transaction_id` (007 `PasteOutcome`). La 006 décrit les huit résultats en prose (« inserted, verified ») ; la 007 utilise des valeurs de fil (`inserted_verified`, `read_observed`, `not_read`, `permission_missing`, `uncertain`, `typed`, `clipboard_handed_over`, `insertion_failed`). Les huit valeurs, leur mappage 009/010 et la règle « enqueue ≠ résultat terminal » concordent.
- Propriété de l'affichage « Text placed on the clipboard for the remote screen » : la 006 dit l'afficher, la 007 dit l'émettre vers l'indicateur. Préciser qui le présente, pour éviter un double affichage.
- Code de refus de plateforme divergent (voir constat bloquant 2).
- Le plan 007 cite lui aussi `LiveSettingsEffects` : même écart de nommage avec la 008 livrée.

## Contradictions spec ↔ plan

- AC-005, overlay `none` : la spec confie la raison à la bulle 010 ; le plan livre la 006 sans raison visible dans ce cas jusqu'à la 010 (limitation explicite, consignée comme ambiguïté non critique). Ce n'est pas bloquant, mais l'AC-005 ne peut pas être certifiée entière à la livraison de la 006.
- Classification QE : la spec (`## Quality Engineering`) dit rayon **Shared**, risque principal **Functional**. Le `## QE Analysis` du plan dit **Cross-system** et **Security/Functional**. La criticité High concorde. La voie de livraison suit la spec (`shared` → full via criticité).
- Tests P0 de la 006 sur les seuils internes de la 007 (299/300 ms, 799/800 ms, cap 8 s, Jump Desktop) : la 006 ne possède aucun minuteur de presse-papiers et sort avant la 007. Avertissement de périmètre (constitution, portée minimale).

## AC non testables à la livraison de la 006

- AC-005 : la remise du transcript à la 010 et la raison en overlay `none` restent sans consommateur.
- AC-006 : la transmission du résultat à 009 (Enter) et à 010 n'est vérifiable qu'en contrat de producteur, avec des consommateurs simulés ; le type `InsertionOutcome` de la spec omet aussi « inserted, verified ».
- Scénarios `has_overlay` (« closed », « open », « focused ») : états visuels non définis, maquettes différées ; à porter par la passe Penflow du programme.
- Mesures non liées à un AC : capture < 50 ms et journalisation (QE seulement).

## `## Quality Engineering` ↔ `## QE Analysis` (spec)

Ces deux sections concordent sur la criticité (High), les dimensions et les gates P0. Le `## Quality Engineering` omet les gates natifs P1 et les attentes non fonctionnelles (50 ms, 1 s, 200 ms, journaux) que liste le `## QE Analysis`. Pas d'incohérence ; c'est le plan qui diverge (voir plus haut).

## Recommandations

1. Corriger le plan (via `spec-refine 006 plan`) :
   - placer les preuves natives Windows en *pending release evidence* ;
   - fixer un contrat de refus de plateforme commun avec la 007 et compatible avec le schéma 008 ;
   - nommer les points d'entrée livrés de la 008 ;
   - définir `DestinationContext` et la table de correspondance avec `PasteOutcome` de la 007.

   Puis refaire la revue de plan (round 2).
2. Aligner la classification QE du plan sur la spec.
3. Planifier la passe Penflow du programme avant `spec-implement 006` : l'étape 1 exige déjà une géométrie approuvée, et `penflow-contract status` est BLOCKED (certified false).
4. Réduire les tests 006 de la logique 007 au mappage de résultats injectés.
