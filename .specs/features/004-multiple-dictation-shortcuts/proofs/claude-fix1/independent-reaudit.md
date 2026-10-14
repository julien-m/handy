# Re-audit indépendant — 004 Multiple Dictation Shortcuts (correctifs du premier audit)

- **Date :** 2026-10-13
- **Auditeur :** Claude (claude-opus-5-5), en lecture seule, non auteur du code
- **Périmètre :** commits `3dee540`, `c30b1b6`, `99ffd46`, `d8cdb5e` et `21a4aed` (HEAD `21a4aedf7fc1e9bec936fb5f57a22fb532404a30`), qui corrigent les constats m1, m2, m3, m5, m6, m7, n2, n4 et n7 de [l'audit précédent](../claude/independent-audit.md)
- **Référentiel :** `spec.md` (AC-002, AC-004 en priorité), `progress.md`, `.specs/constitution.md`, `AGENTS.md`, `.conventions/index.md` (règles générales, Rust et TypeScript)
- **Hors périmètre, comme dans le premier audit :** ce qui dépend de 008 (révisions, CAS, compensation d'un échec de persistance, CLI, verrou de session).

## Verdict : **PASS**

Aucun blocker ni major. Les neuf constats visés sont corrigés, et chaque correction est vérifiée par lecture du code, par les tests du dépôt et par des reproductions supplémentaires. Restent un constat minor, de la même famille que m2 mais sur un autre chemin (désactivation du post-processing), et six nits.

| Constat d'origine | État vérifié |
|---|---|
| m1 — sondage OS du post-processing désactivé | Corrigé (Tauri, hors cancel, comme documenté) |
| m2 — un conflit hérité perdait son inscription Tauri | Corrigé pour les modifications de raccourcis ; le chemin du post-processing reste exposé (voir **M1**) |
| m3 — réparation au changement d'implémentation | Corrigé |
| m5 — refus de cancel non traduits, reset avalé | Corrigé (la syntaxe invalide reste brute, voir **N5**) |
| m6 — tests du rollback et des shadows | Corrigé (11 tests de delta, 5 tests de shadows) |
| m7 — avertissements de conflits périmés | Corrigé |
| n2, n4, n7 | Corrigés |

## Constats

### Minor

**M1 — Désactiver le post-processing désinscrit encore une inscription Tauri partagée avec Transcribe (même cause que m2, autre chemin).**
- Fichier : `src-tauri/src/shortcut/mod.rs:1143` (`change_post_process_enabled_setting`, branche `else` : `let _ = unregister_shortcut(&app, binding);`).
- Scénario : backend Tauri, store hérité `transcribe = "alt+space"` et `transcribe_with_post_process = "option+space"`, post-processing activé. Au démarrage, l'une des deux actions détient l'unique inscription native de la combinaison. L'utilisateur désactive le post-processing. `unregister_shortcut` désinscrit `Shortcut(option+space)` par valeur, c'est-à-dire l'emplacement partagé. Transcribe ne répond plus à `alt+space` jusqu'au redémarrage ou jusqu'au prochain cycle suspend/resume d'un recorder. C'est exactement le défaut m2, que `registration_delta` corrige pour les modifications mais pas ici.
- Gravité : ce comportement existait avant 004 (même désinscription), il n'y a donc pas de régression. `progress.md` range la migration du toggle post-processing dans « Waiting for 008 ». Mais la ligne m2 de la table des dispositions affirme « Fixed », alors que la promesse de l'AC-002 (« keep their current registration and behavior unchanged ») reste rompue sur ce chemin. Et le correctif ne dépend pas de 008.
- Correctif proposé : à la désactivation, planifier `plan_registration_delta(all_of(post), &[], &other_registered(...), same_slot)` puis `apply_registration_delta` sur le `BackendRegistry`. La passation rend alors l'emplacement à Transcribe. Exposer pour cela `other_registered` et `prepare_registrations` dans `edits.rs`, ou une petite fonction `release_action_registrations`. Ajouter le test « retrait de toutes les inscriptions d'une action avec survivant » dans `registration_delta/tests.rs`. À défaut, préciser dans la ligne m2 de `progress.md` que ce chemin reste ouvert, et ajouter le cas aux preuves natives en attente.

### Nit

- **N1 — Un échec de libération du sondage laisse la combinaison capturée jusqu'au redémarrage.** `src-tauri/src/shortcut/tauri_impl.rs:130-137`. Le plugin (`tauri-plugin-global-shortcut` 2.3.2, `unregister`) renvoie l'erreur avant de retirer l'entrée de sa table. Sous macOS, `global-hotkey` a déjà oublié le pointeur Carbon. En cas d'échec, la combinaison reste donc inscrite sans handler : aucune action ne se déclenche, mais la frappe est avalée dans tout le système. Tant que l'app tourne, toute tentative ultérieure d'utiliser cette combinaison dans Handy est refusée avec « already in use ». La modification est bien refusée et rien n'est sauvegardé. Le déclencheur, un échec de `UnregisterEventHotKey`/`UnregisterHotKey`, est quasi impossible. Correctif : en cas d'échec, tenter un second `unregister`, journaliser en `error!` avec la combinaison, et le documenter dans le commentaire de la fonction. Pendant la fenêtre normale du sondage (deux allers-retours sur le thread principal), une frappe ne déclenche rien : l'entrée n'a pas de handler et le `Builder` n'a pas de handler global (`lib.rs:891`). Elle est seulement avalée, ce qui est négligeable.
- **N2 — `hand_over` masque un vrai échec de désinscription.** `src-tauri/src/shortcut/registration_delta.rs:160-162`. Sous macOS, désinscrire un raccourci non inscrit renvoie `Ok`, si bien que la branche `Err` ne couvre en pratique que de vrais échecs. Ils sont pourtant journalisés en `debug!` comme « slot was not registered ». Scénario : `unregister(removed)` échoue, puis `register(survivor)` échoue avec « already in use ». Le raccourci retiré, absent des réglages, continue de déclencher l'action modifiée, et la modification est annoncée comme réussie. Correctif : journaliser en `warn!`, et ne tenter `register(survivor)` qu'après un `unregister` réussi, ou après avoir vérifié `is_registered`.
- **N3 — Deux définitions de « inscrit » qui ne coïncident pas pour un identifiant d'action inconnu.** `src-tauri/src/shortcut/edits.rs:148-160` (`other_registered` parcourt `settings.bindings`, et `is_statically_registered` y renvoie `true` pour tout identifiant inconnu), contre `src-tauri/src/shortcut/mod.rs:155-170` (`startup_registrations` ne parcourt que les bindings par défaut). Un store qui contient un binding inconnu (version plus récente, ou futures actions de la 005) partageant un emplacement avec un raccourci retiré donne cet emplacement à une action sans entrée dans `ACTION_MAP`. Le raccourci ne fait alors rien mais reste capturé. Correctif : filtrer `other_registered` sur les identifiants par défaut, comme le démarrage.
- **N4 — Le premier passage à HandyKeys ne fait pas la réparation m3.** `src-tauri/src/shortcut/mod.rs:464-472`. Quand `initialize_handy_keys_with_rollback` initialise HandyKeys, `handy_keys::init_shortcuts` inscrit les raccourcis tels quels : rien n'est relu avec le parseur de destination, et `reset_bindings` vaut toujours `[]`. Ce comportement est antérieur aux correctifs et sans effet pratique connu, puisque la syntaxe Tauri est lisible par HandyKeys. Mais la phrase « Switch repair reads every entry with the destination parser » de `progress.md` n'est vraie qu'à partir du second passage. Correctif : appliquer `plan_switch_repair` avant `init_shortcuts`, ou nuancer la documentation.
- **N5 — Une syntaxe invalide sur le chemin hérité de cancel reste en anglais, préfixée « Error: ».** `src-tauri/src/shortcut/mod.rs:300` (`InvalidShortcut` renvoyé en `Err(String)` pour compatibilité) et `src/components/settings/shortcut-list-model.ts:89` (`String(error)`). Les refus typés (conflit, OS) sont bien traduits ; seule la syntaxe invalide garde le texte brut du parseur. Correctif : remplir aussi `edit_error` pour `InvalidShortcut` dans une réponse `success: false`, ou utiliser `error.message` plutôt que `String(error)`.
- **N6 — Résidus mineurs de couverture et de dépôt.** Le cas « mouse-based → `Immune` » de `secure_input_shadow.rs:37` n'est pas testé (seul le cas modifier-only l'est). Les trois `conventions.stdout` de 2,2 Mo restent dans l'historique de la branche (`1e4617c`, `7f31bd6`, `8a4a614`) : n4 n'est résolu dans `dev` qu'avec une fusion squash. Correctif : ajouter `plan("mouse4")` ou l'équivalent, et fusionner en squash ou réécrire ces commits avant la fusion.

## Points vérifiés sans constat

### 1. `registration_delta.rs` (m2)
- **Planification** (`plan_registration_delta`) : les survivants sont `after ∩ before` de l'action, plus les inscriptions des autres actions inscrites statiquement, triées par identifiant. Un raccourci retiré qui partage son emplacement avec un survivant passe en `handovers` ; sinon, il passe en `released`. `same_slot` vaut faux sous HandyKeys, dont les tables sont indexées par `(action, shortcut)` ; sous Tauri, c'est `Shortcut::id()`. C'est la même clé que celle de la table du plugin (`shortcuts: HashMap<id, …>`). J'ai vérifié la cohérence avec le vérificateur de conflits : `normalize_tauri` regroupe `SUPER|META`, mais `global-hotkey` 0.8.0 n'analyse pas `meta` comme modificateur (`hotkey.rs:198-219`) ; les alias réels (`alt`/`option`, `cmd`/`command`/`super`, `cmdorctrl`) produisent le même `id`. Il n'existe donc pas de paire « conflit au sens du vérificateur mais emplacements distincts ».
- **Ordre sous Tauri** : les libérations passent d'abord, ce qui permet un remplacement équivalent dans l'emplacement libéré ; puis les ajouts ; les passations en dernier. Un ajout ne peut pas partager l'emplacement d'un survivant, car le vérificateur le refuse (normalisation identique, action propriétaire hors de l'action modifiée, et doublon intra-action refusé). Une passation consiste à désinscrire l'emplacement, quel que soit son détenteur, puis à inscrire le survivant sous sa propre identité. Le résultat ne dépend donc pas de l'action qui a gagné la course au démarrage.
- **Scénarios d'échec reproduits** contre le faux backend du dépôt (un emplacement par combinaison analysée), dans une copie de HEAD (`repro-004b/export`, 5 tests ajoutés, 16/16 PASS) :
  - ajout refusé au milieu d'une modification qui comporte aussi une passation (`[alt+space, f13] → [f5, f13]`, avec le post-processing sur `option+space`) : pour chacun des deux détenteurs possibles, l'emplacement partagé reste à son détenteur, `f13` reste inscrit et `f5` n'est pas inscrit. Les réglages ne changent pas, ce qui est cohérent ;
  - trois prétendants sur une même combinaison (`transcribe`, `transcribe_with_post_process`, plus un troisième binding) : passation au premier survivant trié, quel que soit le détenteur initial ;
  - reset qui libère un raccourci et en passe un autre : les deux opérations sont correctes ;
  - retrait du dernier détenteur (le post-processing quitte la combinaison après que Transcribe l'a quittée) : l'emplacement est libéré, sans passation ;
  - modification pendant la suspension du recorder (toutes inscriptions retirées) : le survivant est inscrit par la passation, et `resume_all_shortcuts` échoue ensuite proprement sur lui avec « already in use », journalisé en `debug`.
- **Désynchronisation après rollback** : le rollback ne concerne que les ajouts et les libérations, et les passations ne s'exécutent qu'en cas de succès. Il ne reste que deux cas résiduels très improbables, un échec du plugin au moment de restaurer ou de passer (N2), et un échec de persistance après l'inscription, ce dernier rattaché à 008. Les réglages ne sont écrits qu'après `prepare_registrations`.

### 2. Sondage OS (m1)
`probe_shortcut` analyse la combinaison, refuse si Handy la détient déjà, l'inscrit sans handler, puis la libère. Il ne s'exécute que sous Tauri, pour une action non inscrite autre que cancel, donc uniquement pour le post-processing désactivé. Un refus devient `RegistrationFailed`, transmis avant `write_settings`. Un échec de libération est une erreur et jamais un succès ; ses conséquences sont décrites en N1. L'absence de sondage pour cancel est documentée sous « Not built, independent of 008 ».

### 3. `implementation_switch.rs` (m3)
Chaque entrée est relue avec `destination_form` (validation puis normalisation de la destination). Une alternative équivalente au défaut réinitialisé est fusionnée ; les autres équivalences sont conservées comme conflits hérités, ce qui est voulu et testé. `introduced_conflicts` ne rapporte que les paires `(action, shortcut)` absentes avant le changement. Le tri par identifiant rend l'ordre d'inscription déterministe. La fonction supprimée de `action_shortcuts.rs` a été entièrement remplacée, et son test a été déplacé. Côté frontend, le seul consommateur (`KeyboardImplementationSelector.tsx`) lit `reset_bindings` et `conflicts`. Le libellé n2 est corrigé, et la nouvelle clé `conflictsKept` est présente et traduite dans les 26 locales.

### 4. m5
`BindingResponse.edit_error` porte le refus typé. `updateBinding` et `resetBinding` lèvent `ShortcutEditFailure`, et `resetBinding` ne l'avale plus. Les deux seuls appelants (`GlobalShortcutInput.tsx:345`, `HandyKeysShortcutInput.tsx:384`) l'interceptent et affichent un toast traduit qui nomme l'action propriétaire, via `settings.general.shortcut.bindings.<id>.name`. Les clés de message existaient déjà dans les 26 locales.

### 5. m7
`useStoredConflicts` dépend de la sérialisation de tous les bindings et de `keyboard_implementation`. Les réponses périmées sont ignorées (`isCurrent`). En cas d'échec, la dernière liste est conservée et `conflictsUnavailable` est affiché (`role="status"`). La clé est traduite dans les 26 locales, jamais identique à l'anglais hors `en`. Le store met bien `bindings` à jour après chaque modification : `editBindingShortcuts` y écrit la réponse, et le reset passe par `refreshSettings`.

### 6. `secure_input_shadow.rs` (m6)
J'ai comparé ligne à ligne avec l'ancien code de `secure_input.rs` (`d8cdb5e`). `carbon_equivalent` est identique. `plan_fallback_binding` est identique à une factorisation près : `immunity()` regroupe les deux branches modifier-only et mouse-based, avec les mêmes messages de journal à l'interpolation près, et les mêmes retours. `is_mouse_key` est identique et reste utilisé par le diagnostic (`secure_input.rs:568`). Le module n'est compilé que sur macOS. Rien ne change pour Windows et Linux.

### 7. `settings-changed` au premier passage à HandyKeys
L'événement n'a qu'un écouteur (`settingsStore.ts:697`), qui appelle `refreshSettings`, une lecture seule par `get_app_settings`. Le seul autre effet est le rafraîchissement des conflits par m7, lui aussi en lecture seule. Aucun écouteur Rust. Aucun chemin ne réémet l'événement, donc pas de boucle. Le coût se limite à un double rafraîchissement, puisque le sélecteur appelle aussi `refreshSettings`. Le chemin d'erreur (HandyKeys qui ne démarre pas) n'émet toujours rien, comme avant. Le changement est donc sans risque, et cohérent avec l'autre branche du changement d'implémentation.

### 8. `bindings.ts`
J'ai régénéré les bindings avec le même `Builder` Specta et les mêmes options d'export (`Typescript::default().bigint(Number)`), via un test injecté dans une copie de HEAD (`repro-004b/bindings.generated.ts`). Le résultat est **identique octet pour octet** au `src/bindings.ts` commité (`diff` vide). `BindingResponse.edit_error: ShortcutEditError | null` et `ImplementationChangeResult.conflicts: StoredShortcutConflict[]` correspondent exactement aux types Rust (`#[serde(tag = "code", rename_all = "snake_case")]`).

### Conventions
Aucun `unwrap`, `expect`, `panic!` ni `todo!` ajouté hors des tests. Les erreurs sont typées et propagées. Les nouvelles fonctions sont courtes et documentées. Aucun `any` en TypeScript. `livespec conventions verify` : `new_count: 0`, et les avertissements des fichiers touchés relèvent tous de la baseline (`doc_coverage` sur des commandes antérieures).

## Tests et vérifications exécutés

Tous lancés le 2026-10-13 sur HEAD `21a4aed`, avec `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`, sans build release. Les journaux sont dans `/private/tmp/claude-501/-Users-julienm-projects-handy/6c63dd2f-66f1-4748-8d50-174f39c2881a/scratchpad/repro-004b/`.

| Commande | Résultat |
|---|---|
| `cargo test --lib` (complet) | **PASS** — 363 réussis, 0 échec, 16 ignorés |
| `cargo test --lib shortcut::` | **PASS** — 61 réussis |
| `cargo test --lib transcription_coordinator::` | **PASS** — 50 réussis |
| `cargo test --lib secure_input` | **PASS** — 5 réussis (0 lors du premier audit) |
| `cargo fmt --check` | **PASS** (exit 0) |
| `bun run lint` | **PASS** (exit 0) |
| `bunx --no-install tsc --noEmit` | **PASS** (exit 0) |
| `bun run test:keyboard` | **PASS** — « keyboard: all assertions passed » |
| `bun run test:shortcut-list` | **PASS** — « shortcut-list-model: all assertions passed » |
| `bun run check:translations` | **PASS** — 25 langues complètes |
| `bun run format:check` (en plus) | **PASS** |
| `livespec conventions verify --feature repo --json`, sur un export de HEAD (`git archive`, avec un lien symbolique vers `node_modules` pour ESLint) | **PASS** — `blockers: []`, `new_count: 0`, 669 entrées de baseline, 400 avertissements tous dans la baseline ; reçu écrit dans l'export uniquement (`.specs/conventions/runs/20261013T162137Z/receipt.json`). Un premier lancement sans `node_modules` donnait FAIL (`node_modules/.bin/eslint: No such file`) : c'est un artefact de l'export, pas un défaut du code |
| Régénération Specta de `bindings.ts` (copie de HEAD) | **Identique** au fichier commité |
| 5 scénarios de delta supplémentaires (copie de HEAD) | **PASS** — 16/16 tests `registration_delta` |

Le worktree est inchangé, à l'exception de ce rapport (`git status` : seuls `pipeline.md`, préexistant, et les reçus `conventions/runs` non suivis, antérieurs à cet audit).

Non exécuté, faute d'interaction physique : les preuves natives en attente listées dans `progress.md`. Elles ne sont pas déclarées réussies ici. Je recommande d'y ajouter le cas de M1 : désactiver le post-processing avec un conflit hérité, puis vérifier que `alt+space` déclenche encore Transcribe.
