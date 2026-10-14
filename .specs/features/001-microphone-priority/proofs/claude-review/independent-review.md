# Revue indépendante — 001-microphone-priority

- Relecteur : Claude (`claude-opus-5-5`), sans part à l'écriture de la spec ni du plan. Date : 2026-10-14. Branche `dev`, revue sur la base de `bd9c854` (008 mergée en `e9f96df`).
- Outil : `/opt/homebrew/bin/livespec` en mode dev (`/Users/julienm/projects/livespec`, commit `71f1e0d`). Budget `--review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo, pas de build.
- Lu : [spec](../../spec.md), [plan](../../plan.md), [constitution](../../../../constitution.md), [plan 008](../../../008-settings-cli/plan.md) (contexte de la revue), [implementation 008](../../../008-settings-cli/implementation.md), et le code 008 livré : `settings_service/{domains/mod.rs,descriptor_data.rs,errors.rs,validation.rs,tauri_adapter.rs}`.

## Reçus

| Revue | Lots | Résultat | `context_hash` | Reçu (sha256) |
| --- | --- | --- | --- | --- |
| Spec | 1 | complete, **ready**, round 1 | `82e5b650…ababc025f` | [`spec.json`](../../.reviews/spec.json) `7f8d1d3d…56482f` |
| Plan | 1 | complete, **not ready** (1 bloquant), round 1 | `287cd4db…cdd8d7c7` | [`plan.json`](../../.reviews/plan.json) `94c33969…f9ca03` |

Sources : `spec.md` `d949baee…6f41226`, `plan.md` `604643ec…3f441d`.

## Progression

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan.txt](progression-plan.txt) |
| Analyze (`--pre-impl`) | exit 1, 1 HIGH : revue de plan avec constat bloquant ; couverture structurelle 19/19 | [analyze-pre-impl.json](analyze-pre-impl.json) |
| `--progression implement` | **BLOCKED** `analyze_not_ready:blocked` | [progression-implement.txt](progression-implement.txt) |
| `delivery lane` | **full** (criticité high, rayon shared) | [delivery-lane.json](delivery-lane.json) |

## Constat bloquant pour l'implémentation

**Preuves natives Windows/Linux érigées en gates, contraire à la constitution.**

- L'étape 1 prouve la propriété des flux sur « macOS/Windows/Linux », et l'étape 2 exige « Step 1 native feasibility established ».
- L'étape 7 laisse toute preuve native de plateforme manquante « explicitly BLOCKED ».
- La constitution (décision du 2026-10-13, postérieure au plan daté du 2026-10-12) range ces observations en *pending release evidence*, qui ne bloquent ni les étapes, ni la complétion sur la branche d'intégration, ni le merge dans `dev`.

Correction : garder la preuve native macOS comme gate d'Implemented, la compilation et les tests automatisés Windows/Linux comme gates, et les spikes et acceptations natifs Windows/Linux comme lignes *pending* avec leur vérification exacte (modèle du plan 008).

## Écarts entre le plan et la 008 livrée

| Plan 001 | 008 livrée (`e9f96df`) | Effet |
| --- | --- | --- |
| `microphones list` via le trait `Domain` : catalogue (énumération hors ligne par un worker) et entrée effective en direct | `Domain::plan(request, &AppSettings)` est pur (« no I/O and no live effect ») ; `LiveOnly(Value)` est calculé depuis les réglages ; l'instance répond depuis l'instantané publié | Le cadre doit être étendu (contexte d'exécution ou port de lecture : catalogue, état audio). Le plan 008 annonçait que les domaines suivants n'auraient pas à changer le cadre. À expliciter. |
| Étendre `LiveSettingsEffects` | N'existe pas : `effects.rs` derrière `LiveSettings` ; effet `AudioInput` déjà déclaré pour `selected_microphone`/`clamshell_microphone` | Renommer ; brancher l'invalidation du cache de résolution sur l'effet du nouveau descripteur |
| Écrivain UI « same SettingsMutation » | Écritures UI par `settings_service::tauri_adapter` ; une liste est un `ChangeKind::Apply` / `tauri_adapter::edit` | Nommer ces points d'entrée aux étapes 2, 4 et 5 |
| « Retire legacy selected-microphone writer » | `selected_microphone` est réglable en générique (`descriptor_data.rs` : non read-only, domaine `microphones`) | Après migration, `handy settings set selected_microphone …` écrirait une clé ignorée : la rendre read-only ou la rediriger vers le domaine |
| Migration « null or legacy default sentinel → [] » | Décision 008 : le défaut système est `null`, un `default` littéral est un nom de périphérique (PCM ALSA sous Linux), et les stores anciens n'ont jamais contenu la sentinelle | Mapper seulement `null` vers `[]`, ou trancher le cas du littéral (ambiguïté non critique AC-006) |
| Codes et exits 0/1/2/3/4/5, enveloppe, révision, CAS | Identiques | Conforme |
| Refus de liste par le `set` générique | Refus existants par `ValidationReason::{ShortcutsDomain,VocabularyDomain,PromptsDomain}` | Il faudra une raison typée pour le domaine `microphones` (et les traductions UI) |

## Contradictions spec ↔ plan

- **Budget par tentative (AC-003)** : la spec fixe un timeout de 2 s par tentative et de 6 s au total. Le plan réserve la dernière tranche de 2 s au défaut système et répartit le reste entre les candidats. Une tentative peut donc disposer de moins de 2 s ; avec quatre candidats, un périphérique qui ouvre en 1,5 s peut échouer. Cela tranche une vraie tension de la spec, mais l'interprétation doit être acceptée (avertissement).
- **AC-006** : le plan reprend la règle « null/default → liste vide » de la spec, qui heurte la décision 008 sur `default` (voir plus haut).
- Hors cela, le plan couvre les 19 exigences : migration à la présence de la clé, préfixe scellé et terminé une seule fois, pas d'épissure, clamshell, brouillon/CAS/Reload, noms exacts AC-008.

## AC non testables ou fragiles

- Les états visuels des traits (loading, error, success, dragging, drop-target, empty, invalid, valid, enabled, disabled) dépendent de la passe Penflow du programme ; les IDs sémantiques/de test restent `[NEEDS CLARIFICATION]`.
- AC-003 « audio spoken before readiness is not captured » et la terminaison bornée avec cpal ne sont démontrables qu'en natif : le plan fait de leur infaisabilité un blocage de release, ce qui est honnête.
- AC-008 hors ligne : l'« effective device » n'existe pas sans instance. Le plan répond `null`/offline, testable seulement une fois l'extension du cadre définie.

## `## Quality Engineering` ↔ `## QE Analysis`

Les deux sections de la spec concordent (High, Shared, Regression, mêmes gates P0) ; seul le `## QE Analysis` liste les gates P1 et les attentes non fonctionnelles. Le `## QE Analysis` du plan concorde aussi (High, Shared settings/audio pipeline, Regression). La section Clarifications contient une puce de décision altérée par l'outil de clarification (identifiant vide, clé de décision égale au paragraphe Penflow) : à nettoyer, sans effet fonctionnel.

## Recommandations

1. Corriger le plan via `spec-refine 001 plan` :
   - placer les preuves natives Windows/Linux en *pending release evidence* ;
   - décrire l'extension du cadre de domaines 008 (contexte de lecture catalogue/état en direct) avec ses tests ;
   - nommer `tauri_adapter` et `effects.rs` ;
   - rendre `selected_microphone` read-only après migration.

   Puis refaire la revue de plan (round 2).
2. Trancher via `spec-refine 001` le sens de `default` dans AC-006 et la règle d'allocation du budget de l'AC-003.
3. Faire la passe Penflow du programme avant l'étape 6 (UI).
