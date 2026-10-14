# Revue indépendante du plan, round 3 — 002-word-replacements

- **Date :** 2026-10-14
- **Périmètre :** `plan.md` à HEAD `6c20f14` (`feature/002-word-replacements`), après les merges de la 008 puis de la 004 dans `dev`.
- **Rôle :** relecteur indépendant. Je n'ai écrit ni la spec, ni le plan, ni le code. Ni Codex ni le skill `audit` n'ont été utilisés.
- **Agent :** Claude Code, modèle `claude-opus-5-5`, revue native en lecture seule.
- **Outil :** `/opt/homebrew/bin/livespec`, avec `--model claude-opus-5-5 --review-max-chars 180000` sur toutes les commandes de revue, de gate et de voie.
- **Écritures dans le repo :**
  - `.reviews/plan.json` et `.reviews/plan.rounds.json`, écrits par `livespec validate --ingest-review` ;
  - ce rapport.
- **Sources lues :**
  - `livespec/system/review-protocol.md` ;
  - les sections revue et Analyze des `SKILL.md` de spec-plan et spec-feature ;
  - en entier : `spec.md`, `plan.md`, `008-settings-cli/plan.md`, `constitution.md`, `project.md`, `stacks/_default.md` et `testing/strategy.md` ;
  - `review_contract.py`, `review_synthesis.py` et `review_synthesis_transport.py`, pour les règles de validation.

## Diagnostic de départ

`--progression implement` répondait `BLOCKED: analyze_not_ready:incomplete`.

| Reçu | `context_hash` du reçu | `context_hash` préparé | Source changée | Action |
|---|---|---|---|---|
| `.reviews/spec.json` | `d1477804…86ca` | `d1477804…86ca` | aucune | Reçu courant, pas de nouvelle revue de spec |
| `.reviews/plan.json` | `ee2cfa8c…c356` | `d7e6700f…bcf2` | `.specs/features/008-settings-cli/plan.md` | Nouvelle revue de plan |

Seul le plan 008 a changé : il est lié comme contexte (`Inspect`) par le plan 002. `spec.md` et `plan.md` de la 002 ont les mêmes empreintes que lors de la revue précédente. Le diff 008 relu depuis `47e873a` porte sur la bascule coordonnée de tous les writers à l'étape 4, sur l'authentification UID des pairs Unix, sur la politique de preuves natives et sur les commandes de compilation Windows et Linux.

## Préparation et budget

- `--prepare-review plan` : 2 lots, 123 sections, 17 exigences, `errors: []`, soit bien en deçà de la limite de 64 lots. Le plan ne cite aucune autre feature en lien normatif.
- **Lot 1 :** `plan.md` L1–L371, plus la spec, le plan 008 (contexte), la constitution, le projet, la stack et la stratégie de test (prompt de 179 597 caractères).
- **Lot 2 :** `plan.md` L372–L434 (Testing Strategy, QE Analysis, Risks, Next Action), avec le même contexte (prompt de 141 124 caractères).
- **Synthèse :** construite selon `synthesis_prompt` (prompt de 50 866 caractères, sous le budget de 180 000).
- **Vérification des citations :** avant l'ingestion, chaque extrait a été vérifié comme sous-chaîne exacte de sa section. J'ai aussi vérifié que chaque citation source appartient aux sections d'obligation de son exigence, et chaque citation de la synthèse à une citation de lot. Un `validate_review_result` à blanc a renvoyé `complete=true`, `ready=true`, `errors=[]` avant l'ingestion réelle.

## Reçus

| Reçu | Lots | `reviewer_model` | complete | ready | errors | Round | `context_hash` | SHA-256 du fichier |
|---|---|---|---|---|---|---|---|---|
| `.reviews/spec.json` (inchangé) | 1 | `claude-opus-5-5` | true | true | `[]` | 1 | `d1477804…86ca` | `68450368…5a37` |
| `.reviews/plan.json` | 2 + synthèse | `claude-opus-5-5` | true | true | `[]` | 2 (`round_cap_reached: false`) | `d7e6700f…bcf2` | `68e60eb4…4f38` |

Résultat de la revue de plan :

- 17/17 exigences `covered` dans les deux lots et dans la synthèse ;
- 1 ambiguïté non critique ;
- 2 warnings et 5 infos, tous en `carry_over` ;
- 1 périmètre technique approuvé (le staging IPC privé) ;
- aucune résolution de lot nécessaire, car aucun lot ne contenait de problème bloquant ;
- une confiance de 4/5.

## Gates

| Commande (`--model claude-opus-5-5 --review-max-chars 180000`) | Résultat |
|---|---|
| `validate … --progression plan` | `Progression: READY` (exit 0) |
| `validate … --pre-impl --format json` (Analyze) | exit 0 ; `findings: []`, `semantic_status: covered`, 17/17 (100 %), 0 CRITICAL et 0 HIGH |
| `validate … --progression implement` | **`Progression: READY`** (exit 0) |
| `livespec delivery lane --feature 002-word-replacements` | **`Delivery lane: FULL`** ; criticality high, blast radius shared (`criticality_high`) |

Sans `--model` ni `--review-max-chars`, `delivery lane` ajoute `spec_review_not_current`, car le reçu de spec est lié à ce modèle et à ce budget. Avec les deux options, la seule raison est `criticality_high`.

Analyze n'a modifié aucun fichier : `git status` ne montrait que les deux reçus `.reviews/plan*.json`.

## Constats

| # | Sév. | Emplacement | Constat | Correctif proposé |
|---|---|---|---|---|
| W1 | warning | `spec.md` AC-007 ; `plan.md` « Matcher and approximate correction protection » | Le plan laisse la suppression des fillers et la normalisation modifier le texte remplacé, après l'étape approximative protégée. C'est conforme à l'ordre des étapes d'AC-007, mais plus étroit que la phrase « without AI post-processing, the inserted text includes them too ». Reconduit du round précédent. | Qualifier AC-007 à la prochaine révision de spec. Aucun changement du plan. |
| W2 | warning | `constitution.md` « Native platform evidence » ; `plan.md` « Resolved Test Commands » | **Nouveau.** La constitution exige que le code compile et passe ses tests automatisés sur macOS, Windows et Linux. Le plan 008 révisé nomme les commandes `cargo check` et `cargo test` pour `x86_64-pc-windows-msvc` et `x86_64-unknown-linux-gnu`. Le plan 002 n'en nomme aucune, alors qu'il ajoute des dialogues de fichiers natifs, un export atomique par renommage et du staging IPC. | Reprendre ces deux commandes de la 008 dans Resolved Test Commands et dans les prérequis de l'étape 7. Les observations natives restent des preuves de release en attente. |
| I1 | info | `plan.md` « Validation, exchange and identity » | Un export valide de plus de 1 000 000 d'octets ne peut pas être réimporté. Le plan documente ce risque résiduel et garde des fixtures SC-002 importables. Reconduit. | Qualifier AC-005 dans la spec. |
| I2 | info | `spec.md` AC-002 ; `plan.md` « Validation, exchange and identity » | **Nouveau.** Le plan lit « line break » comme CR/LF seulement : U+0085, U+2028 et U+2029 restent permis. Le round-trip tient, puisque le format découpe sur LF, mais cette lecture est implicite. | L'écrire dans les tests du validateur, ou interdire aussi ces séparateurs. |
| I3 | info | `plan.md` « Matcher… » ; ambiguïté AC-006 | Aucune fixture ne fixe un ZWNJ ou un ZWJ voisin d'une correspondance côté CJK. | Ajouter une fixture avant/après de chaque côté. |
| I4 | info | `008/plan.md` « All surfaces and live effects » ; `plan.md` Step 5 | **Nouveau.** La 008 impose des commandes UI asynchrones, qui n'attendent jamais le verrou de transaction depuis le thread principal. L'étape 5 de la 002 ajoute des commandes de commit et de dialogue sans rappeler cette règle. | Le préciser dans l'étape 5. |
| I5 | info | `plan.md` QE Analysis | Le vocabulaire diffère de la spec : « Shared/Cross-system » et « Functional » dans le plan, Shared et Data dans la spec. La voie n'est pas affectée. Reconduit. | Reprendre les termes de la spec. |
| I6 | info | `plan.md` Next Action | La section nomme `--model=gpt-6.1-sol` et présente l'implémentation comme à venir. Reconduit. | Rafraîchir à la finalisation. |

**Ambiguïté non critique (AC-006, `ac-006:cjk-side-join-control`).** Elle demande si un ZWNJ ou un ZWJ voisin d'une correspondance côté CJK la bloque. La lecture littérale de la spec dit oui : ce sont des caractères de mot, et l'exemption CJK ne couvre que les caractères de base. Ni la spec ni le plan ne le disent explicitement. Hors reçu : `matcher.rs:249` bloque désormais les join controls côté CJK (`!(data.is_combining_mark(character) || data.is_join_control(character))`). Le code suit donc la lecture littérale, et il ne reste qu'à l'écrire dans la spec et dans une fixture.

Aucune contradiction entre la spec et le plan. Aucune nouvelle portée métier. Le plan 008 révisé reste compatible avec les interfaces que cite la 002 : `commit_mutation`, `read_snapshot`, `LiveSettingsEffects`, le trait Domain avec ses alias, la trame de 1 MiB, l'échéance de 5 s, la table des codes de sortie, `input.rs`/`render.rs` et la bascule de propriété du store à l'étape 4.

## Hors périmètre du reçu

`AGENTS.md` décrit l'API livrée par la 008 : `settings_service::tauri_adapter::{write, set, edit, set_prepared, repair}` et `native_changes`. Le plan 002 nomme les interfaces de conception (`commit_mutation`, `LiveSettingsEffects`) et laisse l'étape 1 se lier aux « actual 008 interfaces ». Ce n'est pas une contradiction, mais l'implémenteur doit suivre les noms réels.

## Empreintes

- `plan.md` : `4c854f3b…5000`
- `spec.md` : `33ad7724…35c5`
- `008-settings-cli/plan.md` : `863ef278…4eec`
- `constitution.md` : `f65acb99…2067`
- `.reviews/plan.json` : `68e60eb4ed7e70980e41d4438008d3b6086c472275ffc5105d424c0a76414f38`
- `.reviews/plan.rounds.json` : `314788bfe9411308b8760973de9661334eb352d182b38c27d6286f41bf63701f`
