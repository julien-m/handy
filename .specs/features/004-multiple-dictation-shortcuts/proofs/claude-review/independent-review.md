# Revue indépendante des specs — 004-multiple-dictation-shortcuts

- **Date :** 2026-10-13
- **Périmètre :** `spec.md` et `plan.md` à HEAD `cf93910a465daada0f483dee5ae374f35651c00e` (`feature/004-multiple-dictation-shortcuts`), et le comportement du toggle post-processing introduit par `cf93910`.
- **Rôle :** relecteur indépendant. Je n'ai écrit ni la spec, ni le plan, ni le code. Ni Codex ni le skill `audit` n'ont été utilisés.
- **Agent :** Claude Code, modèle `claude-opus-5-5` (celui de `.specs/semantic/config.yaml`), revue native en lecture seule.
- **Outil :** `/opt/homebrew/bin/livespec` (mode dev), avec `--model claude-opus-5-5 --review-max-chars 180000` sur toutes les commandes.
- **Écritures dans le repo :**
  - `.reviews/spec.json`, `.reviews/plan.json`, `.reviews/spec.rounds.json` et `.reviews/plan.rounds.json`, écrits par `livespec validate --ingest-review` ;
  - ce rapport.
  - Le fichier `pipeline.md`, non suivi, existait avant cette revue. Je ne l'ai ni modifié ni commité.
- **Sources lues :**
  - `livespec/system/spec-system.md`, `review-protocol.md` et `delivery-review.md` ;
  - les sections revue et Analyze des `SKILL.md` de spec-feature, spec-specify et spec-plan ;
  - `spec.md`, `plan.md` et `contracts/shortcuts-cli.md` ;
  - `proofs/claude/independent-audit.md` et `proofs/claude-fix1/independent-reaudit.md` ;
  - le diff de `cf93910` (`shortcut/mod.rs`, `shortcut/edits.rs`, `PostProcessingToggle.tsx`), `Sidebar.tsx` et `PostProcessingSettings.tsx`.

## État des reçus

Avant cette revue, la gate répondait `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review;spec_review_model_mismatch:gpt-6!=claude-opus-5-5`. L'ancien reçu de spec venait de `gpt-6`, et la spec avait changé depuis.

| Reçu | Lots | `reviewer_model` | complete | ready | errors | Round | `context_hash` | SHA-256 du fichier |
|---|---|---|---|---|---|---|---|---|
| `.reviews/spec.json` | 1 (57 sections, 14 exigences) | `claude-opus-5-5` | true | true | `[]` | 1 | `81eadd51…79c1` | `dbfb9595…03c1` |
| `.reviews/plan.json` | 1 (116 sections, 14 exigences) | `claude-opus-5-5` | true | true | `[]` | 1 | `25e5eb81…424c` | `6c2a6314…346f` |

- **Revue de spec :** les 14 exigences sont `covered`, avec 1 ambiguïté non critique, 2 warnings, 4 infos et une confiance de 4/5.
- **Revue de plan :** les 14 exigences sont `covered`, avec 1 ambiguïté non critique (la même), 2 warnings et 2 infos. Deux périmètres techniques sont approuvés : le refus `shortcut_session_busy` pendant une session, et les sondages OS de post-processing et de cancel. La confiance est de 4/5.
- **Vérification des citations :** chaque citation a été vérifiée avant l'ingestion, comme sous-chaîne exacte de sa section et dans le périmètre de ses obligations.

## Gates et progression

| Commande (`--model claude-opus-5-5 --review-max-chars 180000`) | Résultat |
|---|---|
| `validate … --progression plan` | `Progression: READY` (exit 0) |
| `validate … --pre-impl --format json` (Analyze) | exit 0 ; `findings: []`, `semantic_status: covered`, 14/14, 0 CRITICAL et 0 HIGH |
| `validate … --progression implement` | **`Progression: READY`** (exit 0) |
| `livespec delivery lane --feature 004-multiple-dictation-shortcuts` | **`Delivery lane: FULL`** ; criticality high, blast radius shared (`criticality_high`) |

## Question : un raccourci refusé par l'OS empêche-t-il d'activer le post-processing ?

**Réponse : ce comportement n'est pas couvert par la spec, et il va contre son intention.**

- **AC-004** applique le refus de l'OS à « the change », c'est-à-dire à un raccourci candidat qu'on ajoute ou modifie. Activer le post-processing n'est pas un candidat.
- **Les Edge Cases** disent : « Post-processing disabled: its shortcuts are unregistered but kept, as today for its single shortcut ». Le Header dit : « Keep the existing activation behavior ». **AC-002** garde le comportement des raccourcis stockés.
  - Aujourd'hui, et avant `cf93910`, activer le post-processing réussit toujours. L'échec d'inscription est ignoré (`let _ = register_shortcut`).
- **Le plan se contredit.** `plan.md:76` dit « retain the current post-processing/cancel enabled-state rules ». `plan.md:338` fait passer le toggle par la transaction « register before durable persistence », dont un refus laisse les réglages inchangés.
- **Ce que fait le code** depuis `cf93910` :
  - `change_post_process_enabled_setting` renvoie `RegistrationFailed` dès qu'une seule des quatre inscriptions possibles est refusée. Le réglage n'est pas sauvegardé, et le toggle revient à off avec un toast.
  - Or l'éditeur des raccourcis de post-processing est dans `PostProcessingSettings`, et la section n'apparaît dans `Sidebar.tsx:63` que si `post_process_enabled` est vrai.
  - Un utilisateur dont le raccourci de post-processing stocké est pris par une autre application ne peut donc ni activer la feature, ni corriger ce raccourci depuis l'UI. Ses seules issues : libérer la combinaison dans l'autre application, changer d'implémentation clavier depuis Debug, ou passer par la CLI, pas encore livrée.

**Recommandation :**

1. Revenir à la sémantique « as today » : l'activation sauvegarde toujours `post_process_enabled = true`.
2. Garder le delta de `cf93910` pour les emplacements partagés : la passation d'emplacement corrige M1 du re-audit, et elle doit rester.
3. Inscrire les raccourcis acceptés par l'OS et signaler chaque raccourci refusé, par un toast et par un avertissement sur sa ligne, comme les conflits hérités. Le raccourci reste ainsi modifiable.
4. Réserver le refus d'AC-004 aux ajouts et modifications de raccourcis.

Si le produit veut au contraire le refus, il faut une décision Clarify explicite, qui rende l'éditeur accessible quand le post-processing est désactivé. Dans les deux cas, la décision doit être enregistrée (ambiguïté `ac-004:post-processing-enable-with-os-refusal`, présente dans les deux reçus) et le plan aligné (`plan.md:76` contre `plan.md:338`).

## Autres constats

| # | Sév. | Emplacement | Constat | Correctif proposé |
|---|---|---|---|---|
| W1 | warning | `spec.md` Key Entities (l. 316) contre AC-004 (l. 186) | **Contradiction interne de la spec.** La table des entités annonce « overlap warning », alors qu'AC-004 et Story 3 rejettent le chevauchement modifier-only. Le plan tranche dans le bon sens (`plan.md:98`), mais le texte reste contradictoire. | Remplacer par « overlap (rejection) », et ajouter « legacy conflict warning » si c'est ce qui est visé. |
| W2 | warning | `plan.md:104`, `contracts/shortcuts-cli.md` ; `spec.md` Story 4 et AC-006 | **Mutations CLI limitées à une instance en cours.** Le plan rend toutes les mutations live-only : sans instance, add, remove et reset renvoient `unavailable`/5. La spec vise le provisioning de plusieurs machines et ne dit rien d'un refus hors ligne. C'est justifié pour le sondage OS Tauri, moins pour HandyKeys. | Écrire la politique dans AC-006, ou autoriser une écriture hors ligne validée au prochain démarrage. |
| I1 | info | `spec.md` QE Analysis (l. 330, 335) et Quality Engineering (l. 340) ; `plan.md:430` | **Cohérence QE.** Criticité High et risque Regression sont identiques partout ; la voie est donc full. Le blast radius diffère : texte libre dans QE Analysis, « Shared » dans Quality Engineering et « Shared/Cross-system » dans le plan. « Missing evidence: No native runtime test » est périmé, car des preuves natives macOS existent dans `progress.md`. Quality Engineering omet le P1 et le budget de 10 ms, sans contradiction. | Aligner le vocabulaire sur Shared et rafraîchir Missing evidence. |
| I2 | info | `spec.md` AC-006 | « F8 envelope codes » n'est pas défini ; il s'agit de l'enveloppe de 008. | Écrire « 008 envelope codes ». |
| I3 | info | `spec.md` FR-004 | FR-004 ne référence qu'AC-005, alors que la dernière suppression et la limite de quatre sont définies dans AC-001. Le plan compense. | Ajouter AC-001. |
| I4 | info | `spec.md:15-17`, `plan.md:468` | Métadonnées périmées : « specification only », Status Planned, et Next Action avec `--model=gpt-6`. | Les rafraîchir à la finalisation. |

Les AC sont tous testables. Le seul comportement non décidé est celui du toggle ci-dessus.

## Recommandations

1. Trancher le comportement du toggle post-processing dans Clarify, de préférence « as today, refus signalé », puis corriger `change_post_process_enabled_setting` et `PostProcessingToggle.tsx` en conséquence. Ajouter la preuve native déjà listée dans `progress.md` (« refused late activation »), avec l'oracle choisi.
2. Corriger W1 et la politique hors ligne (W2) par `spec-refine`, puis rafraîchir les deux reçus.
3. Aligner la QE (I1). La voie reste full.
4. Garder `--review-max-chars 180000` sur toutes les gates, car les reçus sont liés à ce budget.

## Empreintes

| Objet | SHA-256 |
|---|---|
| `spec.md` | `9e68d9ad6871cc48385c57b9e89b5cb6caf8bd115043eac269a564c9be90b350` |
| `plan.md` | `07e9b2fede5b0a64d06f636f1a30ba3163b2ae63016e8084d67bbf23cae4d1d5` |
| `contracts/shortcuts-cli.md` | `747d201eb64258971e56ebc353f5ff1657d5f67df118548c080a8cc0e6f76ad2` |
| `008-settings-cli/plan.md` (contexte) | `f95b8b53075ed8a3619f5843b3ac626efc03f9a6cb3806d9b29980ed4ca0356d` |
| `.reviews/spec.json` | `dbfb959557c4a3c3124cd29346692ca96b29c9716eabd5dc001bf5a1264203c1` |
| `.reviews/plan.json` | `6c2a63148c1b07ea86cbe3bc90487eeefb1e60421aa08d15dee2f0767125346f` |
| `.reviews/spec.rounds.json` | `d87ab70170a71becb3a46c6d336749d517fb2c77bfbf9c2934da03ed9381cb81` |
| `.reviews/plan.rounds.json` | `f6b6fc01ee90e73dc7c2689b7096ecda81c7a7ededeb3f7ffb0536ed8602167d` |
| `context_hash` spec | `81eadd514383f5beb570124c78d873a243fd43a0b0acc92699cf0668fd3579c1` |
| `context_hash` plan | `25e5eb81dac8bf92cdbcd152519d13fb5d5cdac8b13711f86da400f23cea424c` |

Les réponses brutes sont conservées dans `raw_results` des reçus.
