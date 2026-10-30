# Audit indépendant — 008-settings-cli, étape 4 (périmètre B)

- Dépôt : `/Users/julienm/projects/handy`, branche `feature/008-settings-cli`
- Commits audités : `3f26053` (code), `459a793` (doc). Copie figée : `git archive 459a793`, dans `audit-008b/src`.
- Périmètre : commandes UI simples, modifications ciblées, effets en direct, frontend, tests, écart `selected_microphone`. R1 à R4, le démarrage et la suppression des écrivains directs relèvent de l'autre auditeur.
- Auditeur en lecture seule. Aucune commande lancée dans le dépôt, aucun commit. Les sondes ont été ajoutées uniquement dans la copie (`src-tauri/tests/audit_probe.rs`).

## Verdict : **FAIL**

Le rapport compte 2 constats majeurs, 8 mineurs et 5 nits. Aucun blocker.

L'essentiel du câblage est correct :
- validation partagée ;
- refus typés à la place des défauts silencieux ;
- utilisateur existant non bloqué ;
- ordre effet → événement → succès ;
- événement limité à la révision et aux noms de clés ;
- 26 locales traduites ;
- commandes de lecture infaillibles.

Deux exigences explicites du plan pour l'étape 4 restent toutefois non tenues :
1. La transaction n'est pas sérialisée de bout en bout (préparation, commit, effets). L'état natif peut donc diverger de l'état stocké.
2. Les tests P0 de compensation et d'échec après commit sont absents, alors que `progress.md` déclare « Success criteria: Met ».

---

## Constats

### MAJOR-1 — Préparation et effets en direct hors de la section sérialisée : l'état natif peut diverger de l'état stocké

**Où**
- `src-tauri/src/settings_service/live.rs:170-192`. `commit()` prend `commits` puis le relâche à la fin de la fonction, avant tout effet.
- `src-tauri/src/settings_service/tauri_adapter.rs:86-94` et `:172-189`. `finish()` applique les effets après la libération du verrou.
- `src-tauri/src/settings_service/tauri_adapter.rs:124-144`. `set_prepared` exécute `prepare()` avant de prendre le verrou.
- `src-tauri/src/settings_service/effects.rs:267-293`. Chaque effet applique `commit.after` (ou la paire `before`/`after`) de *son* commit, pas l'état publié courant.

**Exigence**
Le plan, section « All surfaces and live effects », impose : « Worker transaction serializes prepare/validate → durable persist → publish snapshot → apply live effects → notify ». Sa section « Failure handling » de l'étape 4 ajoute : « … stale effect is P0 ». Le module `effects.rs` annonce lui-même servir aussi les commandes CLI routées à partir de l'étape 5.

**Scénarios concrets**
1. **VAD (préparé).** T1 (UI) choisit `earshot` : `prepare` installe le recorder earshot. T2 (CLI routée à l'étape 5, ou tout autre écrivain concurrent) choisit `silero` : `prepare` installe silero. T2 commite (rév. 8, silero), puis T1 commite (rév. 9, earshot). Le disque dit `earshot`, le recorder natif est `silero`. Aucun effet ne corrige l'écart, car `set_prepared` passe `skip=[key]`. Même schéma pour `selected_channel` (`commands/audio.rs:355-378`) et pour `change_binding` (`shortcut/mod.rs:177-212`).
2. **Implémentation clavier (effet post-commit avec paire avant/après).** A commite `tauri→handy_keys` (rév. 6), B commite `handy_keys→tauri` (rév. 7). Si l'effet de B s'exécute avant celui de A, `switch_keyboard_implementation(handy_keys→tauri)` désenregistre une implémentation qui n'est pas active. Ensuite, l'effet de A, `tauri→handy_keys`, laisse HandyKeys actif alors que le disque dit `tauri`.
3. **Effets « valeur » (thème, logs, overlay, tray, autostart).** Deux commits concurrents appliqués dans le désordre laissent la valeur du commit le plus ancien.

**Atteignabilité aujourd'hui.** Les commandes synchrones s'exécutent sur le thread principal et l'UI désactive le contrôle pendant `isUpdating(key)`. La course n'est donc pas atteignable avec une seule fenêtre et une seule clé. Elle l'est en revanche :
- entre commandes `async` (`update_microphone_mode`, `set_selected_microphone`, `set_selected_channel`, `change_vad_backend_setting`) et réparations de managers ;
- et dès l'étape 5, entre UI et CLI sur la même clé, ce qui est précisément le cas visé par le plan.

**Correctif**
- Sérialiser préparation, commit et effets dans une même section ordonnée. Le plus simple : un `Mutex` d'« application » pris par `set`/`edit`/`set_prepared`/`repair*` autour de `prepare → commit → finish`. Une réparation imbriquée appelée depuis un effet, par exemple `persist_binding_resets` ou `persist_tauri_fallback` dans `switch_keyboard_implementation`, doit alors passer par une variante sans reprise du verrou, ou être différée après l'effet.
- À défaut : rendre chaque effet idempotent, sur le modèle « réconcilier vers l'état publié courant » plutôt que « appliquer `commit.after` ». La paire avant/après du clavier doit alors venir de l'état natif réel, et non du snapshot.
- Ajouter un test multi-thread qui intercale deux `set_prepared` sur la même clé et vérifie l'égalité natif = disque.

### MAJOR-2 — Les tests P0 exigés par l'étape 4 manquent ; la doc déclare le gate atteint

**Où**
- `src-tauri/tests/settings_live.rs` (3 tests).
- `src-tauri/src/settings_service/live_tests.rs`.
- `src-tauri/src/settings_service/effects.rs:363-412` (uniquement des tests d'inventaire).
- `.specs/features/008-settings-cli/progress.md`, checkpoint de l'étape 4 : « Success criteria: Met for the automated Step 4 gate ».

**Exigence.** L'étape 4 du plan demande :
- « New tests verify … compensation and post-commit failure behavior » ;
- « P0 tests require … failed preparation leave both disk and revision unchanged, and injected effect failure never yields a success with stale shortcuts » ;
- et, en Execution evidence, « manager mutation race tests, UI event/read ordering tests ».

**Constat.** Aucun test n'exerce :
- `tauri_adapter::set_prepared`, ni sa compensation sur refus ;
- `compensate_binding` (`shortcut/mod.rs:277`) ;
- `finish`, ni l'ordre effet → événement → retour ;
- `SettingsError::committed_effect_failed` (`errors.rs:174`, aucun appelant de test : `grep` vide hors `tauri_adapter.rs`) ;
- les réparations de managers sous concurrence (`clear_missing_selection`, `persist_auto_selection`, `persist_default_microphone_after_fallback`) ;
- l'ordre événement/lecture côté UI au-delà de la fonction pure `isCurrentRevision`.

**Correctif.** Extraire de `tauri_adapter` une couche testable sans `AppHandle`, par exemple un trait `LiveEffects` injecté avec un effet factice faillible et un port natif factice. Tester ensuite :
- (a) préparation refusée : disque et révision inchangés, aucun événement ;
- (b) commit refusé : compensation appelée, état natif restauré ;
- (c) effet en échec : `internal_error` + `outcome: committed` + `current_revision` + `field`, événement émis, valeur relue ;
- (d) deux réparations de managers concurrentes.

Corriger également la phrase « Success criteria: Met » de `progress.md` tant que ces tests manquent.

### MINOR-1 — L'UI fait un rollback local même quand l'issue est `committed`, et peut restaurer une valeur plus ancienne qu'un rafraîchissement déjà appliqué

**Où.** `src/stores/settingsStore.ts:345-353`. `updateSetting`, branche `catch` : le rollback vers `originalValue` est inconditionnel.

**Scénario 1 (effet en échec).** `update_microphone_mode(true)` : la valeur est commitée (rév. N), l'effet `update_mode` échoue et `settings-changed{N}` est émis. Ensuite :
- si la réponse du rafraîchissement (rév. N, `always_on=true`) arrive avant la réponse en erreur de la commande, le `catch` remet `false` sans changer `revision` ;
- aucun autre rafraîchissement ne suit, donc l'UI affiche `false` alors que le disque dit `true` ;
- le message affiché est bien « enregistré mais non appliqué », mais l'interrupteur dit l'inverse.

Dans l'ordre habituel (événement, puis réponse de la commande, puis réponse du rafraîchissement), l'état se corrige, mais rien ne garantit cet ordre.

**Scénario 2 (refus).** La CLI (étape 5) commite `A=x` (rév. 7) et la refresh UI l'applique. Une modification UI de A en vol est ensuite refusée : le rollback remet la valeur capturée avant `x`.

**Correctif.** Dans le `catch` :
- si `detail.outcome === "committed"` (ou `"unknown"`), ne pas faire de rollback et appeler `refreshSettings()` ;
- sinon, faire le rollback uniquement si `get().revision` n'a pas avancé depuis le début, et faire un `refreshSettings()` sinon.

Ajouter un test du store avec des `commands` simulés.

### MINOR-2 — Écart UI/CLI sur la sentinelle `"default"` (`selected_microphone`, `clamshell_microphone`, `selected_output_device`)

**Où.** `src-tauri/src/settings_service/validation.rs:61-79`. Seul `validate_ui` convertit `"default"`/`"Default"` en `null` ; `validate_cli` stocke la chaîne telle quelle.

**Preuve.** Sonde `selected_microphone_default_sentinel_diverges_between_ui_and_cli` : `ui stores null, cli stores "default"`. Le test de parité à 25 cas exclut cette clé.

**Impact**
- **macOS / Windows.** `Some("default")` donne `DesiredMicrophone::Selected("default")` (`managers/audio.rs:464-466`). Aucun périphérique ne porte ce nom : à la prochaine ouverture du flux (always-on ou prochain enregistrement), le gestionnaire bascule vers le micro par défaut, puis `persist_default_microphone_after_fallback` commite `null`. On obtient une révision et un événement inattendus, et entre-temps `get` renvoie `"default"`.
- **UI.** `refreshSettings` ne normalise que `null → "Default"`. `"default"` en minuscules ne correspond à aucune option et le sélecteur reste vide.
- **Linux (ALSA).** `"default"` peut être un vrai nom de PCM. L'UI ne peut alors pas sélectionner ce périphérique (elle le convertit en `null`), alors que la CLI le peut.
- **Contrat.** C'est une violation de SC-001 / AC-003 (« même valeur stockée »). Elle n'est pas atteignable aujourd'hui (le `set` CLI est désactivé jusqu'à l'étape 5), mais elle doit être tranchée avant d'activer l'étape 5.

**Correctif.** Faire produire `null` au frontend pour « Default » sur les trois clés (`settingsStore.ts:129-146`, ainsi que `clamshell_microphone`, qui transmet aujourd'hui `null` comme chaîne). Supprimer ensuite la sentinelle de `validate_ui`, pour que `"default"` reste un nom littéral sur les deux surfaces. À défaut, appliquer la même normalisation dans `validate_value`, c'est-à-dire pour les deux ports. Ajouter le cas au test de parité.

### MINOR-3 — Écriture durable sur le thread principal, et sliders qui commitent à chaque événement d'entrée

**Où**
- Toutes les commandes `#[tauri::command] pub fn` synchrones de `shortcut/mod.rs` (par exemple `:634`, `:646`), ainsi que `set_log_level`, `set_selected_output_device`, `set_clamshell_microphone`, `set_model_unload_timeout`. Les commandes Tauri 2 sans `async` s'exécutent sur le thread principal.
- `VolumeSlider.tsx:16-18` : `onChange` appelle `updateSetting` à chaque événement `input`. Même chose pour `WordCorrectionThreshold`, `PasteDelay`, `HoldThreshold` et `RecordingBuffer` via `Slider`.

**Mesure.** Sonde `durable_ui_commit_latency`, sur ce Mac (APFS, build debug) : médiane **12,2 ms**, p90 13,3 ms, max 26,2 ms par commit (fichier temporaire, deux `sync_all`/F_FULLFSYNC, rename). Ajouter l'attente possible du verrou inter-processus, jusqu'à l'échéance de 5 s.

Avant le commit, `store.set` était en mémoire. Un glisser de slider produit maintenant des dizaines de commits par seconde sur le thread principal : saccades, inflation de révisions et une rafale d'événements et de rafraîchissements. Le plan demande : « UI native command adapters become async workers where needed; never wait for a worker holding the transaction lock from the main thread ».

**Correctif**
- Passer les commandes de réglages en `#[tauri::command(async)]` ou en `spawn_blocking`, sauf les effets qui exigent le thread principal, que l'on y redispatche via `run_on_main_thread`.
- Débouncer ou committer au relâchement (`onPointerUp`/`onChangeEnd`) pour les sliders.

### MINOR-4 — Le compensateur s'exécute aussi sur une issue `unknown`

**Où.** `tauri_adapter.rs:137-142`. `Err(error) => { compensate(); … }` sans distinguer `Outcome::Unknown`, alors que `LiveSettings::commit` (`live.rs:177-181`) a déjà relu et republié le disque, qui peut contenir la nouvelle valeur.

**Scénario.** VAD ou canal : la persistance renvoie `unavailable`/`unknown` après le remplacement (R1 de l'étape 3). Le snapshot republié indique `earshot`, mais `compensate()` remet `silero` en natif. Les deux divergent jusqu'au prochain changement de la clé.

**Correctif.** En cas d'issue `Unknown`, ne pas compenser à l'aveugle : réconcilier le natif sur `live.snapshot()`, c'est-à-dire réappliquer l'effet de la valeur publiée.

### MINOR-5 — Pas de réconciliation après un effet en échec (écart au plan, documenté comme limite)

**Où.** `tauri_adapter.rs:172-189` et `implementation.md`, « Known limits ».

**Exigence.** Le plan demande : « keep committed data authoritative and reconcile live state before accepting another affected mutation ».

**Écart.** Rien ne réconcilie. Resoumettre la même valeur est un no-op (`changed` vide, donc aucun effet) : on ne peut pas réessayer. Exemple : `always_on_microphone=true` commité, mais `update_mode` échoue (périphérique occupé). Le mode natif reste `OnDemand` tant que l'utilisateur ne repasse pas par `false` puis `true`.

**Correctif.** Mémoriser les clés « committed-not-applied » et réessayer leur effet avant le prochain commit, ou dès qu'une même valeur est resoumise. Sinon, faire valider explicitement l'écart au plan par l'orchestrateur.

### MINOR-6 — Le remplacement `--debug` est perdu quand `debug_mode` ou `log_level` change

**Où.** `effects.rs:331-336` (`apply_logging`) comparé à `lib.rs:1011-1024`.

**Avant.** Basculer `debug_mode` ne modifiait pas `FILE_LOG_LEVEL`. Désormais, chaque modification de `debug_mode` (Cmd/Ctrl+Shift+D) ou de `log_level` remet le niveau des logs fichier au `log_level` stocké et la diffusion webview au `debug_mode` stocké. Lancé avec `--debug` (Trace), un simple basculement du mode debug rabaisse les logs. Le commentaire « same as before the settings service » est inexact pour `debug_mode`.

**Correctif.** Conserver le remplacement d'exécution, par exemple via un `AtomicBool CLI_DEBUG` consulté par `apply_logging`, et documenter le comportement.

### MINOR-7 — La parité UI/CLI ne couvre ni les vraies commandes Tauri ni les arguments typés

**Où.** `tests/settings_live.rs:78-112`. Le chemin « UI » testé est `LiveSettings::commit_ui_value`, pas les commandes Tauri.

**Écart.** Pour les commandes à argument typé (`LogLevel`, `ShortcutActivation`, `ModelUnloadTimeout`, `ChineseScript`, `VadBackend`, accélérateurs, `u64`/`f32`), une valeur invalide est refusée par la désérialisation IPC de Tauri, avant le validateur partagé. On obtient une chaîne brute et non un `SettingsCommandError`, donc un code et un message différents de la CLI. Le frontend retombe alors sur `errors.settings.generic`.

**Couverture.** 25 cas sur environ 17 clés, alors que SC-001 vise 100 % des 51 scalaires. Le cas `"default"` (MINOR-2) est absent.

**Correctif**
- Étendre la table à chaque scalaire réglable, ou la générer depuis les descripteurs (valeur valide, valeur hors domaine, type invalide).
- Pour les commandes typées, accepter un `serde_json::Value` ou une `String` et laisser le validateur partagé refuser. Sinon, documenter que l'IPC typé est un garde-fou supplémentaire et hors périmètre d'AC-003.

### MINOR-8 — La documentation `459a793` surestime l'état

**Où.** `progress.md`, checkpoint de l'étape 4 (« Success criteria: Met ») et `implementation.md`, section « Step 4 all-writer cutover ».

**Écarts**
- La ligne « Prepare and compensate » n'est étayée par aucun test (voir MAJOR-2).
- La ligne UI affirme « rolls back and toasts », sans préciser que le rollback s'applique aussi à `committed` (voir MINOR-1).
- L'écart avec le plan sur la réconciliation n'est signalé que comme limite : il n'apparaît ni comme déviation du plan, ni comme élément à valider.

**Correctif.** Aligner le texte une fois MAJOR-1, MAJOR-2 et MINOR-1 traités.

### NIT-1 — Un échec de `changed_keys` après le commit durable n'est pas signalé `committed`

`live.rs:182`. Si `serde_json::to_value(AppSettings)` échouait après la persistance, on renverrait `internal_error` sans `outcome: committed` ni publication du snapshot. Le snapshot publié serait alors en retard sur le disque. Ce cas est théorique. Correctif : publier d'abord, puis calculer les clés, et marquer `committed` en cas d'échec.

### NIT-2 — `delete_model` efface la sélection sans relire la condition sous le verrou

`commands/models.rs:75-89`. La condition `selected_model == model_id` est évaluée hors verrou, et l'édition vide `selected_model` sans condition. Une sélection concurrente d'un autre modèle serait effacée. Il faut faire comme `clear_missing_selection` et revérifier dans la closure.

### NIT-3 — Le sélecteur d'accélération exécute la seconde écriture même si la première a échoué

`AccelerationSelector.tsx:118-130`. L'ordre est correct : appareil puis GPU, ou sortie de GPU puis effacement de l'appareil, et `auto` + appareil est bien un point fixe de la normalisation (vérifié dans `settings.rs:1112-1139`). Mais `updateSetting` avale l'erreur : si l'appareil échoue (persistance), l'accélérateur `gpu` part quand même, est refusé (`gpu_device_required`) et l'utilisateur voit deux toasts. Correctif : que `updateSetting` renvoie un booléen, et s'arrêter au premier échec.

### NIT-4 — Un modèle vide pour Apple Intelligence serait refusé

`edit.rs:146-159` et `settings.rs:861-866` : `ensure_post_process_defaults` réécrit un modèle vide en modèle par défaut pour `apple_intelligence`. `require_stable` refuserait donc `change_post_process_model_setting("apple_intelligence", "")` avec `canonical_value`. Aucun chemin UI ne le fait aujourd'hui. À garder en tête pour un futur domaine CLI.

### NIT-5 — Erreur trompeuse quand HandyKeys échoue à l'initialisation

`shortcut/mod.rs:376-400` et `:579-592`. L'effet commite d'abord `handy_keys`, puis `persist_tauri_fallback` recommite `tauri` et l'effet renvoie une erreur. L'UI reçoit `committed` avec la révision du *premier* commit, et le toast « enregistré mais non appliqué » est trompeur puisque la valeur a été rétablie à `tauri`. Correctif : renvoyer `unavailable` avec `current_revision` à jour, ou un motif dédié.

---

## Points vérifiés conformes

1. **Commandes simples.** Les scalaires passent tous par `tauri_adapter::set_value` → `LiveSettings::commit_ui_value` → `validate_ui`, avec revalidation sous verrou (`edit.rs:123-140`) et les mêmes descripteurs que `validate_cli`. Les anciens remplacements silencieux sont supprimés et deviennent des refus `validation_failed` : `paste_method`, `sound_theme`, `theme`, `overlay_position` (y compris `"none"`, que l'UI n'envoie pas : `ShowOverlay.tsx` n'émet que `top`/`bottom`), `overlay_style`, `typing_tool`, `clipboard_handling`, `auto_submit_key`, `keyboard_implementation`, `recording_retention_period`. Les bornes des descripteurs correspondent aux contrôles UI (`VolumeSlider`, `PasteDelay`, `RecordingBuffer`, `WordCorrectionThreshold`, `HoldThreshold`, `HistoryLimit`).
2. **Utilisateur existant.** Sonde `legacy_invalid_values_do_not_block_the_first_write_of_another_key` sur un store pré-révision contenant `paste_method=invalid_method`, `theme=sepia`, `overlay_position=none`, `keyboard_implementation=rdev`, `log_level=verbose`, `sound_theme=theremin`, `history_limit=5000` et `paste_delay_ms=0`. La lecture donne les défauts récupérés (`recovered=true`) et la migration overlay donne `overlay_style=none`. La normalisation est persistée une fois (rév. 1) et la première écriture d'une autre clé (`audio_feedback`) commite en rév. 2, avec seulement cette clé modifiée. Les valeurs hors bornes héritées sont conservées : seule la clé modifiée est revalidée. Les resoumettre est refusé, comme en CLI.
3. **Accélération.** L'ordre d'écriture est valide dans les quatre transitions (voir NIT-3 pour la seule réserve).
4. **Modifications ciblées sous verrou** (closures pures, réexécutables) :
   - prompts (`id` généré hors closure) ;
   - fournisseurs, URL de base `custom` seulement ;
   - clé API (aucun log de valeur ; `SecretMap` a un `Debug` expurgé ; l'événement ne porte que des noms de clés) ;
   - modèle ;
   - sélection et désélection automatiques ;
   - repli micro (condition relue dans la closure) ;
   - repli clavier ;
   - réinitialisations de raccourcis (seules les entrées concernées).
5. **Effets.** Aucun effet n'est perdu par rapport à `merge-base..459a793` : thème (et `theme-changed`), overlay (position, style et cache), logs, autostart, menu et visibilité du tray, rechargement ORT, nouvel essai du GPU transcribe, raccourci de post-traitement (et `reconcile_fallback`), implémentation clavier, mode, appareil et canal du micro, VAD, nettoyage de l'historique (limite et rétention). Les clés sans effet sont lues au moment de l'usage (test `every_declared_runtime_effect_is_applied_or_read_at_use`). Effets → `settings-changed{revision, keys}` → retour : l'ordre est respecté, et l'événement est émis même en cas d'échec d'effet.
6. **Frontend.**
   - Erreurs typées : `SettingsCommandError` avec `code`, `message`, `field`, `reason`, `outcome` et `current_revision`.
   - Clés traduites `errors.settings.*` (8 clés) présentes et réellement traduites dans les 26 locales : 0 copie anglaise hors `en`, `check:translations` OK.
   - Les rafraîchissements de révision inférieure sont ignorés. Pour deux modifications rapides suivies d'un événement en retard, les réponses de `get_settings_view` portent la révision courante au moment du traitement, donc la plus récente gagne. Il reste un scintillement possible : une réponse à révision égale écrase la valeur optimiste.
   - `get_settings_view` et `get_default_settings` renvoient des types non-`Result` : les bindings n'ont pas d'enveloppe `Result`, et un repli sur les défauts à la révision 0 existe avant l'installation du service.
7. **Tests existants.** Ils prouvent bien la parité décision/valeur sur 25 cas, +8 révisions sans perte de champ pour 8 écritures concurrentes en processus, et la clôture de l'inventaire des écrivains. Ils ne prouvent ni la compensation, ni l'échec après commit, ni l'ordre UI (voir MAJOR-2).

## Tests exécutés (copie figée, `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`)

| Commande | Résultat |
|---|---|
| `cargo test --test settings_live` | ok, 3 passed |
| `cargo test --test settings_writers` | ok, 4 passed |
| `cargo test --test settings_transactions` | ok, 42 passed, 1 ignored |
| `cargo test --test settings_validation` | ok, 21 passed |
| `cargo test --lib` | ok, 299 passed, 16 ignored |
| `cargo test --test audit_probe` (sondes de l'auditeur, copie uniquement) | ok, 4 passed : utilisateur existant, écart `"default"`, no-op f32, latence de commit (médiane 12,2 ms) |
| `bun install` | ok |
| `bun run lint` | exit 0 |
| `bunx tsc --noEmit` | exit 0 |
| `bun run test:settings-errors` | « all assertions passed » |
| `bun run check:translations` | exit 0, 25 langues complètes (+ `en`) |

Journaux : `audit-008b/logs/`.
