# Revue indépendante — 007-smart-paste

- Relecteur : Claude (`claude-opus-5-5`), sans part à l'écriture de la spec, du plan ni du code. Date : 2026-10-14. Worktree `handy-wt/007-smart-paste`, branche `feature/007-smart-paste`, base `b0b65cb`.
- Outil : `/opt/homebrew/bin/livespec` en mode dev (`/Users/julienm/projects/livespec`, commit `71f1e0d`). `--model claude-opus-5-5 --review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo, pas de build.
- Lu : [spec](../../spec.md), [plan](../../plan.md), [constitution](../../../../constitution.md), [projet](../../../../project.md), [stack](../../../../stacks/_default.md), [stratégie de test](../../../../testing/strategy.md). Les noms 008 cités par le plan (`LiveSettings::commit_scalar`, `SettingsRepository::commit_mutation`, `effects::apply_changed`, `settings::apply_settings_migrations`, `validate_cli`/`validate_ui`, `OutputDomain`, `OUTPUT_KEYS`, `SettingsMutation`, `SettingsSnapshot`, `Revision`, `SettingDescriptor`, `tauri_adapter::write`) existent tous dans `src-tauri/src/settings_service/` et `settings.rs`.

## Situation de départ

`--progression implement` renvoyait `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review;spec_review_model_mismatch:gpt-6.1-sol!=claude-opus-5-5`. Les reçus dataient de la revue Codex. La spec avait changé depuis (décision Concealed/Transient sur AC-009, limite AC-012), puis le plan (historique, noms réels de la 008).

## Reçus

| Revue | Lots | Résultat | `context_hash` | Reçu (sha256) |
| --- | --- | --- | --- | --- |
| Spec | 1 (65 sections, 29 exigences) | complete, **ready**, round 1 (3 avertissements, 2 infos, 1 ambiguïté non critique) | `eae1a798…a3aed4e042` | [`spec.json`](../../.reviews/spec.json) `f4bfdb1e…297bde6` |
| Plan | 1 (100 sections, 29 exigences) | complete, **ready**, round 1 (2 avertissements, 2 infos, 1 ambiguïté non critique) | `45c1b2a4…3d18e565` | [`plan.json`](../../.reviews/plan.json) `9f083786…962616d` |

Sources : `spec.md` `d973643e…829cedd`, `plan.md` `583fc398…a236391`. Aucun lien Markdown normatif vers une autre feature : la préparation tient en un seul lot (spec 60 607 caractères, plan 134 633). Les 29 exigences sont `covered` dans les deux revues, y compris les identifiants retirés AC-001/AC-007, FR-001/FR-004 et SC-001/SC-003, explicitement non réutilisés.

Un premier ingest de la spec a été refusé (`invalid_citation`) : cinq citations de conclusions pointaient vers des sections hors du périmètre de l'exigence. Le validateur n'accepte que les sections rattachées à l'exigence. Elles ont été retirées, sans changer aucune conclusion. Le reçu final reste en round 1.

## Progression

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan.txt](progression-plan.txt) |
| Analyze (`--pre-impl`) | exit 0, aucun constat, 0 CRITICAL/HIGH, couverture structurelle 29/29, sémantique `covered` | [analyze-pre-impl.json](analyze-pre-impl.json) |
| `--progression implement` | **READY** | [progression-implement.txt](progression-implement.txt) |
| `delivery lane` | **full** (raison `qe_section_missing`) | [delivery-lane.json](delivery-lane.json) |

## Question Clarify ouverte (non tranchée)

**AC-012 / AC-013 : message de lecture tardive en mode « copy to clipboard ».** Dans ce mode, le transcript devient du texte brut au « not read ». Une lecture tardive n'appelle donc plus le fournisseur, et la bulle de récupération (010) ne peut pas dire que le texte a peut-être été collé. La spec (Clarifications) et le plan documentent cette limite « to confirm with Julien ». Elle contredit la phrase inconditionnelle d'AC-012 dans ce mode. Ambiguïté marquée **non critique** : l'outcome terminal et l'éligibilité à Enter ne changent pas. Décision attendue de Julien, à enregistrer par le protocole de clarification (`decision:ac-012:late-read-message-in-copy-to-clipboard-mode`).

## Constats (non bloquants, reportés en `carry_over`)

Spec :

1. **800 ms contre 3 s.** AC-012 annonce l'outcome « après la lecture ou le délai de 800 ms », alors qu'AC-009 fixe 3 s pour Jump Desktop. Le plan tranche correctement : une seule source de délai par application, et Enter/recovery attendent 3 s pour Jump.
2. **Décision ouverte** sur la limite du mode copie (ci-dessus).
3. **Cache AppKit.** Le premier signal de lecture suppose qu'AppKit rappelle le fournisseur après une lecture par un gestionnaire de presse-papiers avant la frappe. Ce n'est pas vérifié. Un « not read » erroné reste du côté sûr (rien d'ancien n'est collé), mais l'outcome serait faux. C'est à vérifier dans AC-016.
4. (info) Les types ignorés par la borne 200 ms / 50 MB manquent au presse-papiers restauré, alors que la Story 1 promet un contenu identique. La limite est documentée.
5. (info) Deux `[ASSUMED]` restent, mais ils sont fondés : les lectures avant frappe ne comptent pas (AC-008), et Screen Sharing garde le transcript (décision du 2026-10-13).

Plan :

1. **Métadonnées périmées.** `Next Action` prescrit encore `--model=gpt-6.1-sol`, et le front matter `updated: 2026-10-13` précède la décision historique du 2026-10-14 citée par le plan.
2. **Courses résiduelles.** Trois courses ne sont pas éliminables par le mécanisme : écriture AX appliquée après la garde finale (doublon), copie externe entre la garde `changeCount` et l'écriture (écrasement), frappe différée d'une dictée précédente (contenu périmé). Le plan en fait des gates de release fondés sur les preuves natives, ce qui respecte la tolérance zéro de SC-007, SC-008 et AC-016. La release dépend donc des mesures.
3. (info) **Ordre.** L'étape 1 conditionne le câblage de production à `--progression implement` READY et aux spikes natifs. L'étape 8 conditionne le code UI à la gate Penflow partagée. Or le code est déjà implémenté jusqu'à l'étape 7 alors que ce READY n'est obtenu que maintenant. L'écart est à consigner dans les preuves d'implémentation.
4. (info) Après une écriture AX possible, une valeur illisible en fin de fenêtre donne `uncertain`. AC-017 ne définit que l'illisibilité avant écriture. Ce choix conservateur est cohérent avec la règle « jamais deux fois ».

Portée additionnelle : uniquement technique et justifiée. Elle comprend le helper de snapshot en processus séparé, le refus de démarrer si un presse-papiers non vide n'a aucun type sauvegardable, la garde AX à 1 000 000 d'unités UTF-16 avec un timeout de 20 ms par appel, le marqueur `smart_paste_migration_version` et les bornes d'items et de types. Les ajouts métier (insertion AX d'abord, non-restauration Concealed/Transient, migration macOS) sont couverts par des décisions de Julien.

## Voie de livraison

`livespec delivery lane` renvoie **full** avec `criticality: null` et la raison `qe_section_missing` (« spec.md has no 'Quality Engineering' section »). La spec contient bien une section QE, mais intitulée `## QE Analysis` (« Criticality: High; Blast radius: Every macOS paste… »). La voie full correspond de toute façon à cette criticité. Renommer la section, via `spec-refine`, permettrait à l'outil de lire la classification au lieu de tomber par défaut en voie full.
