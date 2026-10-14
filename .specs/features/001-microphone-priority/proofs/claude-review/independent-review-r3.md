# Revue indépendante, round 3 — 001-microphone-priority

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`.
- Base relue : corrections `b772239`. Outil : `/opt/homebrew/bin/livespec`, `--model claude-opus-5-5 --review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo, pas de Codex.
- Revue de plan : lot préparé par `--prepare-review plan`, envoyé en entier à un relecteur natif indépendant (sous-agent Claude, même modèle), qui a rendu le JSON brut. J'ai contrôlé ses citations contre les sections préparées avant l'ingestion.
- Contrôlé à la main :
  - la section « Step 0 » et « Settings framework extension » du plan 006 (`.specs/features/006-destination-application/plan.md`, état `c352eac`) ;
  - le code 008 livré : `settings_service/{domains/mod.rs,sequence.rs,tauri_adapter.rs,native_changes.rs,validation.rs,errors.rs,effects.rs,transaction.rs,normalization.rs,edit.rs}`, `settings.rs` et `lib.rs`.

## Reçus

| Revue | Résultat | `context_hash` | Reçu |
| --- | --- | --- | --- |
| Spec | Inchangée depuis le round 2 ; reçu courant complete, **ready** | `6e8643c4…6098beb255` | [`.reviews/spec.json`](../../.reviews/spec.json), sha256 `3310ce48…167fa8` |
| Plan | 1 lot de 123 139 caractères, sans erreur ; **complete, ready** ; 19/19 `covered` ; round 2 sur 2, plafond non atteint | `de909c63…e4a89094` | [`.reviews/plan.json`](../../.reviews/plan.json), sha256 `acefc00f…8f0111` |

Sources : `spec.md` `4d3d351d…5805f`, `plan.md` `a85aad5e…e7d17`. Bundle brut ingéré (hors dépôt) : sha256 `734717ff…a2ef`.

## Gates

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r3.txt](progression-plan-r3.txt) |
| Analyze (`--pre-impl`) | exit 0 ; aucun constat ; couverture structurelle 19/19, sémantique `covered` | [analyze-pre-impl-r3.json](analyze-pre-impl-r3.json) |
| `--progression implement` | **READY** | [progression-implement-r3.txt](progression-implement-r3.txt) |
| `delivery lane` | **full** (criticité high, rayon shared) | [delivery-lane-r3.json](delivery-lane-r3.json) |

## Vérifications demandées

### Constats du round 2 : traités

- **Revue non préparable (bloquant).** Corrigé : les documents 008 et 006 sont cités en chemins inline, plus aucun lien vers `.specs/ship.md` (non suivi). La revue tient en 1 lot.
- **Ordre de l'étape 2.** Corrigé : son prérequis nomme « 006 step 0 merged » (D3 `ErrorReason`, `ErrorItem`, `with_items`, `with_reason`), le résumé et la Next Action suivent.
- **Budget de l'AC-003 hors Clarifications de la spec.** Non traité dans la spec (inchangée). Le relecteur le relève de nouveau comme ambiguïté non critique (`deadline-skips-configured-choices`). Non bloquant.
- **Clarification Penflow mal formée, passe UI commune.** Inchangés (info).

### Noms D2/D3 : conformes au plan 006 et au code 008

- D2, recopiés dans le résumé du plan, identiques au plan 006 : `DomainPlan::Live(Box<dyn LiveOperation>)`, `LiveOperation::run(&self, context: &dyn LiveContext, call: &LiveCall)`, `LiveCall { dry_run, expected }`, `LiveAnswer::{Read(Value), Committed { read, data }}`, `LiveContext::{app, read, commit_change}`, `managed_state::<T>(context)` ; hors ligne, refus live-only `unavailable`/5 avec `details` vide.
- D3 : `ErrorDetails.reason: Option<ErrorReason>`, `ErrorDetails.items: Vec<ErrorItem>`, `ErrorItem::new(index, code)`, `MAX_ERROR_ITEMS = 100`, `SettingsError::with_reason`, `with_items`. Identiques au plan 006.
- 008, présents dans le code : `DomainPlan::LiveOnly` et sa branche `Unavailable` dans `domains::execute`, `sequence::write_change` (pub(crate)), `tauri_adapter::{ordered, commit_prepared_edit, set_prepared}`, `SettingsBackend::commit_change`, `commit_edit_with_probe`, `SettingsRepository::initialize`, `normalization::normalize`, `settings::apply_settings_migrations`, `validation::editable_descriptor`, `LiveEffect::MicrophoneDevice`, `RuntimeEffect::AudioInput`, `PublicSettings`.
- `lib.rs` enregistre bien `Arc<AudioRecordingManager>` (`managed_state::<Arc<AudioRecordingManager>>`).
- À ajouter par la 001, absents aujourd'hui comme prévu : `ValidationReason::MicrophonesDomain` (l'owner `microphones` tombe aujourd'hui sur `UiOnly`) et `tauri_adapter::write_change`.

### Dépendances : déclarées

- 008 mergée (`e9f96df`).
- Étape 0 de la 006 : prérequis des étapes 2 et 5, rappelée dans le résumé et la Next Action.
- `tauri_adapter::write_change` : ajouté par la première de 001 et 003 livrée.

## Constats restants (non bloquants, `carry_over` de la revue de plan)

1. **Avertissement, ordre.** L'étape 2 déplace `RuntimeEffect::AudioInput` vers `microphone_priority` et passe `selected_microphone` en lecture seule, alors que le résolveur n'arrive qu'aux étapes 3–4 et que la commande `set_selected_microphone` n'est retirée qu'à l'étape 5. Entre les deux, l'effet déclaré n'est lu nulle part et l'ancien sélecteur écrit une clé en lecture seule. Préciser l'état intermédiaire.
2. **Avertissement, couverture.** Perte du micro pendant un enregistrement always-on : la réouverture du flux always-on après la fin de l'enregistrement n'est ni énoncée ni testée.
3. **Avertissement, testabilité.** Les commandes de build et de tests automatisés Windows/Linux (exigées par la constitution, distinctes des preuves natives en attente) ne sont pas nommées.
4. **Info.** En pratique, une seule tentative configurée à 2 s pleines précède le défaut (pas deux) : l'oracle des tests doit suivre la règle de démarrage.
5. **Info.** L'étape 2 attend aussi le spike natif de l'étape 1, ce qui sérialise du travail indépendant.
6. **Ambiguïté non critique** `deadline-skips-configured-choices` (voir ci-dessus).

Portée supplémentaire : 5 éléments techniques, tous justifiés (plafond de workers d'ouverture, détails D3, champs de `list`, noms blancs refusés, relâchement push-to-talk après fin automatique).

## Constats bloquants restants

Aucun.
