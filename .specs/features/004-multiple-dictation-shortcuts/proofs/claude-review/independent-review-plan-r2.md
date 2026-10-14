# Revue indépendante du plan, round 2 — 004-multiple-dictation-shortcuts

- **Date :** 2026-10-14
- **Périmètre :** revue de plan à HEAD `d46ee31` (`feature/004-multiple-dictation-shortcuts`), après l'intégration avec 008 (merge `22e0f33`, puis `e2b0ab5` et `41fa07d`). S'y ajoute un jugement sur deux décisions d'implémentation listées dans `progress.md`.
- **Rôle :** relecteur indépendant. Je n'ai écrit ni la spec, ni le plan, ni le code. Je n'ai utilisé ni Codex ni le skill `audit`.
- **Agent :** Claude Code, modèle `claude-opus-5-5`, en revue native en lecture seule.
- **Outil :** `/opt/homebrew/bin/livespec`, avec `--model claude-opus-5-5 --review-max-chars 180000` sur chaque commande.
- **Écritures :**
  - `.reviews/plan.json` et `.reviews/plan.rounds.json`, écrits par `--ingest-review` ;
  - `proofs/claude-review/analyze-r2.json`, la sortie d'Analyze ;
  - ce rapport.
  - `spec.md`, `plan.md` et le code n'ont pas changé. `pipeline.md`, non suivi, existait déjà : je ne l'ai pas commité.
- **Sources lues en entier :**
  - `review-protocol.md` ;
  - `spec.md`, `plan.md`, `contracts/shortcuts-cli.md` et `008-settings-cli/plan.md` ;
  - `.specs/constitution.md`, `project.md`, `stacks/_default.md` et `testing/strategy.md` ;
  - `progress.md`, `implementation.md` et la revue précédente ;
  - dans le code : `shortcut/live_change.rs`, `settings_service/sequence.rs` (chemin de commit préparé), `bindings_change.rs::compensate_changes`, `TranscriptionCoordinator::quiesce`, `edits::set_post_process_enabled` et `PostProcessingToggle.tsx`.

## Pourquoi la revue de plan était périmée

Ni `spec.md` ni `plan.md` n'ont changé depuis le reçu précédent, qui portait le `context_hash` `25e5eb81…`. Deux dépendances de la revue ont changé :

| Source | Ancien SHA-256 | Nouveau SHA-256 | Cause |
|---|---|---|---|
| `contracts/shortcuts-cli.md` | `747d201e…` | `1d3a03a4…` | Contrat réécrit par `41fa07d` : fichiers réels, raisons `instance_required`, `shortcut_session_busy` et `registration_refused`, dry run hors ligne |
| `008-settings-cli/plan.md` | `f95b8b53…` | `863ef278…` | Plan 008 révisé (`50fdd04`, `e3bc314`) : preuves natives en attente, migration de tous les writers |

La revue de spec était encore à jour : `--prepare-review spec` redonne le `context_hash` `81eadd51…` du reçu, qui est complete, ready et produit par `claude-opus-5-5`. Je ne l'ai pas refaite.

## Reçus

| Reçu | Lots | Modèle | complete | ready | errors | Round | `context_hash` | SHA-256 du fichier |
|---|---|---|---|---|---|---|---|---|
| `.reviews/spec.json` (inchangé) | 1 | `claude-opus-5-5` | true | true | `[]` | 1 | `81eadd51…79c1` | `dbfb9595…03c1` |
| `.reviews/plan.json` (nouveau) | 1 (117 sections, 14 exigences) | `claude-opus-5-5` | true | true | `[]` | 1 selon l'ingestion (`round_cap_reached: false`) ; `plan.rounds.json` a 2 entrées | `a0e08fb2…395b` | `113c11a2…300f` |

**Contenu du nouveau reçu de plan :**

- Les 14 exigences (AC-001 à AC-006, FR-001 à FR-005, SC-001 à SC-003) sont `covered`.
- Il contient 1 ambiguïté non critique, désormais résolue par la spec, 3 périmètres techniques approuvés, 4 warnings, 2 infos et une confiance de 4/5.
- Toutes les citations ont été vérifiées avant l'ingestion, comme sous-chaînes exactes de leur section.

## Gates

| Commande | Résultat |
|---|---|
| `validate … --progression plan` | `Progression: READY` (exit 0) |
| `validate … --pre-impl --format json` (Analyze), avant la revue | exit 1 ; 1 HIGH `AN-SEMANTIC-5308cc3e` « Stale, invalid or incomplete semantic review » |
| `validate … --pre-impl --format json` (Analyze), après la revue | exit 0 ; `findings: []`, `semantic_status: covered`, 14/14, 0 CRITICAL et 0 HIGH ([sortie](analyze-r2.json)) |
| `validate … --progression implement`, avant la revue | `BLOCKED: analyze_not_ready:incomplete` |
| `validate … --progression implement`, après la revue | **`Progression: READY`** (exit 0) |
| `livespec delivery lane --feature 004-multiple-dictation-shortcuts --model claude-opus-5-5 --review-max-chars 180000` | **`full`**, criticality high, blast radius shared, raison unique `criticality_high` |

**Note sur `delivery lane` :** sans `--model` ni `--review-max-chars`, la commande ajoute la raison `spec_review_not_current`, parce qu'elle prend alors le budget par défaut. La voie reste `FULL`. Il faut passer ces deux options pour que la voie se fonde sur le reçu à 180 000 caractères.

## Constats de la revue de plan

| # | Sév. | Emplacement | Constat | Correctif proposé |
|---|---|---|---|---|
| W1 | warning | `plan.md:76` contre `plan.md:338-341` | **Toggle post-processing et refus de l'OS.** Ce constat n'a pas changé depuis le round 1. La ligne 76 garde les règles actuelles. Step 3 fait passer le toggle par la transaction « register before persistence », avec « No saved rejection ». La spec tranche dans le sens de la ligne 76 (Edge Cases « as today ») : l'ambiguïté `ac-004:post-processing-enable-with-os-refusal` est résolue dans le reçu. Le code suit la spec. | À la prochaine révision du plan, écrire dans Step 3 : le toggle est toujours sauvegardé et chaque refus est signalé. |
| W2 | warning | `plan.md:108`, `plan.md:341` contre le contrat (ligne `persistence_failed`) | **Échec de compensation.** C'est nouveau, car le contrat a été réécrit. Le plan prévoit `internal_error` avec un diagnostic « degraded » et un dispatch bloqué. Le contrat dit au contraire que les inscriptions préparées « sont annulées », et sa ligne `internal_error` ne couvre que l'échec après commit. Plan et contrat décrivent donc deux issues différentes. | Retenir une seule version (voir le jugement 2 ci-dessous). |
| W3 | warning | `plan.md:104` ; spec, Story 4 | **Mutations CLI limitées à une instance en cours.** Ce constat n'a pas changé : refuser une écriture hors ligne restreint ce que la spec annonce (provisionner plusieurs machines). | Écrire la politique dans AC-006 par `spec-refine`. |
| W4 | warning | `plan.md:384` contre `constitution.md` (décision du 2026-10-13) | **Preuves natives en attente.** C'est nouveau. Le plan dit « Missing platform/native/design evidence stays BLOCKED for delivery » et place Windows et Linux parmi les gates. La constitution dit que ces preuves restent « pending release evidence » et ne bloquent jamais la fin de la feature ni le merge dans `dev`. Le plan 008 a été aligné, le plan 004 non. `progress.md` applique déjà correctement la constitution. | Reformuler la gestion d'échec de Step 7 : seule la preuve macOS bloque Implemented. |
| I1 | info | `plan.md:430` | Le blast radius est noté « Shared/Cross-system », alors que la spec dit Shared. La voie n'en est pas affectée. | Écrire Shared. |
| I2 | info | `plan.md:1-16`, `:338`, `:466-468` | Métadonnées périmées : statut Planned et Next Action avec `--model=gpt-6`. Les fichiers planifiés (`shortcut/registry.rs`, `settings_service/live.rs`, cibles `shortcuts_registry`, `shortcuts_transactions`, `shortcuts_lifecycle` et `shortcuts_migration`) ont été remplacés par `live_change.rs` et `quiescence.rs`, que nomme le contrat. | Rafraîchir le plan à sa finalisation. |

**Périmètres techniques approuvés :**

1. Le refus `shortcut_session_busy` de toute mutation du registre pendant une session, explicitement étendu au toggle post-processing.
2. Les sondages Tauri des actions inactives.
3. L'état dégradé après un échec de compensation. C'est un mécanisme du plan que la spec n'exige pas.

## Jugement sur les deux décisions d'implémentation

### 1. Toggle post-processing refusé pendant une dictée : conforme

- **AC-003 :** en push-to-talk et dans la phase tenue, « only the release of the shortcut that started the session ends it ». Désactiver le post-processing désinscrit ses raccourcis : une session lancée par un de ces raccourcis et tenue perdrait son relâchement, et l'enregistrement resterait bloqué. Refuser protège donc une exigence de la spec.
- **Ce n'est pas une régression de spec :**
  - Le Header « Keep the existing activation behavior » porte sur le déclenchement des dictées, pas sur le moment où un réglage s'applique.
  - L'Edge Case « Post-processing disabled: its shortcuts are unregistered but kept » reste vrai, simplement hors session.
  - Aucune exigence n'impose que le toggle s'applique en pleine session.
- **Le plan le couvre :**
  - `plan.md:92` soumet toute « Registry mutation » au bail de quiescence et prévoit le refus `unavailable`/`shortcut_session_busy`.
  - `plan.md:338` fait passer le toggle par la même transaction.
  - Le reçu le classe en périmètre technique approuvé.
- **Le code est cohérent :**
  - `live_change::post_process` ouvre d'abord la fenêtre `open(&app, "post_process_enabled")`, puis refuse avant toute inscription ou écriture.
  - Le toggle UI ne change pas d'état et affiche le message traduit `errors.settings.shortcutSessionBusy`.
- **Réserves (non bloquantes) :**
  - Le refus est plus large que le strict nécessaire. Activer pendant une session Transcribe, par exemple, ne menace aucun relâchement. Mais la passation d'un emplacement Tauri partagé peut toucher la touche tenue, et la règle uniforme est plus sûre. C'est acceptable.
  - Le refus couvre aussi `processing`, donc toute la durée d'un post-traitement LLM. C'est un changement visible par l'utilisateur.
  - Le comportement n'est écrit nulle part pour le toggle en particulier. Il faut le nommer dans `plan.md:92` et dans les notes de version.
  - Côté CLI, `handy settings set post_process_enabled` renvoie aussi `unavailable`/`shortcut_session_busy`, avec `field: post_process_enabled`. Le contrat ne décrit le cas qu'avec `field: bindings`, mais il ne couvre que le domaine `shortcuts`.

### 2. Échec de compensation seulement journalisé : conforme à la spec, écart avec le plan

- **Spec :** AC-004 exige seulement que les refus « leave saved shortcuts unchanged ». C'est le cas : le store n'est pas écrit quand le commit est refusé. La spec ne dit rien d'un échec d'annulation dans le runtime.
- **Constitution :** la règle de portée minimale (« a plan mechanism that the spec does not require … is not built ») permet de ne pas construire la gate « degraded ».
- **Plan :** il y a écart avec `plan.md:108` (dispatch bloqué, `internal_error` « runtime-degraded », réconciliation avant la mutation suivante), avec Step 3 (« committed failures and failed compensation explicit ») et avec la ligne Risks de `plan.md:460`.
- **Code :**
  - `compensate_changes` journalise avec `error!` et l'erreur du commit est renvoyée telle quelle, par exemple `persistence_failed`.
  - Pour le toggle, la compensation `toggle_post_process_on_backend` jette même la liste des refus de réinscription, sans la journaliser.
  - Pour le changement d'implémentation, l'échec est journalisé avec `warn!`.
- **Conséquence réelle :** elle est peu probable, car il faut un commit refusé et une annulation qui échoue. Le registre diverge alors du store :
  - un raccourci ajouté mais non sauvegardé reste actif, ou un raccourci retiré n'est pas réinscrit, jusqu'au prochain redémarrage ou à la prochaine réinscription ;
  - l'appelant reçoit `persistence_failed`, dont le contrat affirme que les inscriptions « are undone ». C'est faux dans ce cas.
- **Recommandation :** ce n'est pas bloquant. Il faut aligner les documents au lieu de construire la gate :
  1. inscrire dans le plan la décision de portée minimale (journal plus erreur du commit) ;
  2. corriger la ligne `persistence_failed` du contrat (« undone, or logged when the undo fails ») ;
  3. journaliser les refus de réinscription dans la compensation du toggle.
- **Variante peu coûteuse et plus honnête :** renvoyer `internal_error` avec `details.outcome = "unchanged"` et un drapeau « runtime_degraded », sans gate.

## Recommandations

1. Corriger les documents à la prochaine révision du plan : W1, W2 et W4. Ces corrections sont seulement documentaires et ne changent aucune exigence. Rafraîchir ensuite le reçu de plan.
2. W3 relève de `spec-refine`, si le produit garde la politique « live-only ».
3. Toujours passer `--model claude-opus-5-5 --review-max-chars 180000`, y compris à `delivery lane`.

## Empreintes

| Objet | SHA-256 |
|---|---|
| `spec.md` | `9e68d9ad6871cc48385c57b9e89b5cb6caf8bd115043eac269a564c9be90b350` |
| `plan.md` | `07e9b2fede5b0a64d06f636f1a30ba3163b2ae63016e8084d67bbf23cae4d1d5` |
| `contracts/shortcuts-cli.md` | `1d3a03a47501711daff388d48ad9f7f1f7120b7c62347f1d757dee44f29b47c5` |
| `008-settings-cli/plan.md` | `863ef2789f6e9256d78832d64722f7812104b33c98d5876166f97ffc361d4eec` |
| `.reviews/spec.json` | `dbfb959557c4a3c3124cd29346692ca96b29c9716eabd5dc001bf5a1264203c1` |
| `.reviews/plan.json` | `113c11a2108f09d71096d5fb6a62b67e1a43afd2229454c2fb55c2605df4300f` |
| `.reviews/spec.rounds.json` | `d87ab70170a71becb3a46c6d336749d517fb2c77bfbf9c2934da03ed9381cb81` |
| `.reviews/plan.rounds.json` | `3eedf6c12b8d07b5dcd6e3e94d4584cf4489a4141533b12efc1fa3969565e267` |
| `proofs/claude-review/analyze-r2.json` | `0d16e819f0bac3683d6aa07a842e634657175281c7d7ffbf63db750fc0ed63f3` |
| `context_hash` spec | `81eadd514383f5beb570124c78d873a243fd43a0b0acc92699cf0668fd3579c1` |
| `context_hash` plan | `a0e08fb29ebab5dac10a5396bfe86377febd8319ebaf5814ac262cb6ed3b395b` |

La réponse brute est conservée dans `raw_results` du reçu de plan.
