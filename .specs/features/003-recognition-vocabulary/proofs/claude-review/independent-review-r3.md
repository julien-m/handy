# Revue indépendante, round 3 — 003-recognition-vocabulary

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec ni le plan. Date : 2026-10-14, branche `dev`.
- Base relue : corrections `00382f1`. Outil : `/opt/homebrew/bin/livespec`, `--model claude-opus-5-5 --review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo, pas de Codex.
- Revue de plan : lot préparé par `--prepare-review plan`, envoyé en entier à un relecteur natif indépendant (sous-agent Claude, même modèle), qui a rendu le JSON brut. J'ai contrôlé ses citations contre les sections préparées avant l'ingestion.
- Contrôlé à la main :
  - les sections « Step 0 » et « Settings framework extension » du plan 006 (`.specs/features/006-destination-application/plan.md`, état `c352eac`) ;
  - le scénario « Live state without running app » de `.specs/features/008-settings-cli/spec.md` ;
  - le code 008 livré : `settings_service/{domains/mod.rs,sequence.rs,tauri_adapter.rs,validation.rs,errors.rs,input.rs,render.rs}` et `lib.rs`.

## Reçus

| Revue | Résultat | `context_hash` | Reçu |
| --- | --- | --- | --- |
| Spec | Inchangée depuis le round 1 ; reçu courant complete, **ready** | `c5e679cf…7fa557966` | [`.reviews/spec.json`](../../.reviews/spec.json), sha256 `462ea1c2…97bfe5` |
| Plan | 1 lot de 112 997 caractères, sans erreur ; **complete, ready** ; 15/15 `covered` ; round 1 sur 2 | `63fa7e93…b9c0c6e` | [`.reviews/plan.json`](../../.reviews/plan.json), sha256 `9c5b44f3…189e95` |

Sources : `spec.md` `e50af3db…ab125567f`, `plan.md` `be524112…ae936f`. Bundle brut ingéré (hors dépôt) : sha256 `3d87a365…350466`.

## Gates

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan-r3.txt](progression-plan-r3.txt) |
| Analyze (`--pre-impl`) | exit 0 ; aucun constat ; couverture structurelle 15/15, sémantique `covered` | [analyze-pre-impl-r3.json](analyze-pre-impl-r3.json) |
| `--progression implement` | **READY** | [progression-implement-r3.txt](progression-implement-r3.txt) |
| `delivery lane` | **light** (criticité medium, rayon shared, aucune raison de voie complète) | [delivery-lane-r3.json](delivery-lane-r3.json) |

## Vérifications demandées

### Constats du round 2 : traités

- **Revue non préparable (bloquants).** Corrigés :
  - les documents 008 et 006 sont cités en chemins inline, plus aucun lien vers `.specs/ship.md` (non suivi) ;
  - le lien Whisper est écrit `Inspect [Whisper model documentation](…)` et n'est plus lu comme un contrat externe.
  La revue tient en 1 lot, sans erreur.
- **`vocabulary status` hors ligne sans décision dans la spec.** Traité dans le plan, spec inchangée : le refus `unavailable`/5 y est justifié comme le refus live-only déjà livré par la 008. J'ai vérifié les deux appuis : le scénario « Live state without running app » de la spec 008 (exit 5, `unavailable`) et la branche `DomainPlan::LiveOnly(_) => Unavailable` de `domains::execute`. Le relecteur n'en fait pas une ambiguïté. Acceptable.
- **Étiquette « Rule D4 ».** Renommée « durable-error rule », comportement inchangé.

### Noms D2/D3 : conformes au plan 006 et au code 008

- D2, recopiés dans le plan, identiques au plan 006 : `DomainPlan::Live(Box<dyn LiveOperation>)`, `LiveOperation::run(&self, context: &dyn LiveContext, call: &LiveCall)`, `LiveAnswer::Read(Value)`, `LiveContext::read`, `managed_state::<T>(context)` ; hors ligne, refus live-only de la 008.
- D3 : `SettingsError::with_items`, `ErrorItem::new(index, code)`, `MAX_ERROR_ITEMS` = 100, `ErrorReason::ContextBudgetUnverified`. Identiques au plan 006.
- 008, présents dans le code : `sequence::write_change`, `SettingsBackend::commit_change`, `commit_edit_with_probe`, `input::read_source`, `View::Export`, `ValidationReason::VocabularyDomain`, `domains::canonical_action`, `domains::production()`.
- `lib.rs` enregistre bien `Arc<TranscriptionManager>`.

### Dépendances : déclarées

- 008 mergée (`e9f96df`).
- Étape 0 de la 006 : prérequis explicite de l'étape 4 (seule à utiliser D2/D3), rappelée dans le résumé et la Next Action.
- Incohérence mineure (info) : l'étape 2 de la 003 ajoute `tauri_adapter::write_change` sans condition, alors que les plans 001 et 005 le font ajouter par « la première livrée ». Si la 001 est livrée avant, l'étape 2 de la 003 doit réutiliser le wrapper existant.

## Constats restants (non bloquants, `carry_over` de la revue de plan)

1. **Avertissement, conformité.** L'erreur technique `context_budget_unverified` reste atteignable après release pour un modèle hors catalogue à capacité positive sans budget connu, ce qui heurte « exactement un statut ». Préciser son affichage ou prouver que tout modèle chargé reçoit l'un des cinq statuts.
2. **Avertissement, étape manquante.** Aucune étape n'ajoute explicitement les commandes Tauri de prévisualisation du collage en masse et de suppression par occurrence, attendues par l'UI de l'étape 5.
3. **Info, ordre.** L'UI P1 (étape 5) dépend de la parité CLI, donc transitivement de l'étape 0 de la 006.
4. **Info, ordre.** L'étape 2 attend des descripteurs de l'étape 1 dont elle n'a pas besoin.
5. **Info.** Un délai dépassé de la requête de capacité, modèle chargé, donne `unknown_until_model_loads` : ajouter une raison distincte.
6. **Ambiguïtés non critiques** : libellé « not supported » quand le vocabulaire est vide (`not-supported-status-with-empty-vocabulary`) ; persistance de l'enregistrement des termes envoyés (`sent-terms-record-persistence`).

Portée supplémentaire : 5 éléments techniques justifiés (NUL refusé, `context_budget_unverified`, `evidence_kind`, wrapper `write_change`, export sur stdout) et 1 élément métier approuvé par le relecteur, car il applique le cas limite de la spec sur les listes héritées au-delà de 500 termes (suppression seule permise, ajout refusé).

## Constats bloquants restants

Aucun.
