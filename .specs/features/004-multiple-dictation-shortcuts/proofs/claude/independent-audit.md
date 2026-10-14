# Audit indépendant — 004 Multiple Dictation Shortcuts (noyau)

- **Date :** 2026-10-13
- **Auditeur :** Claude (claude-opus-5-5), en lecture seule, non auteur du code
- **Périmètre :** commits `1e4617c`, `7f31bd6`, `8a4a614` contre `git merge-base HEAD dev` (`967eb38`), branche `feature/004-multiple-dictation-shortcuts`
- **Référentiel :** `spec.md` (AC-001 à AC-006, FR-001 à FR-005, SC-001 à SC-003), `plan.md`, `progress.md`, `.specs/constitution.md`, `AGENTS.md`, `.conventions/index.md`
- **Exclu par consigne :** ce qui dépend de 008 (CLI `handy settings shortcuts`, service de révision, ré-inscription à chaud depuis un autre processus, verrou `shortcut_session_busy`). La décision produit sur les conflits hérités (conservés et seulement signalés) est appliquée.

## Verdict : **PASS**

Aucun blocker ni major. Le noyau respecte le modèle 1 à 4 raccourcis, la compatibilité ascendante et descendante, la sémantique d'activation, et applique le vérificateur de conflits partagé à toutes les additions et modifications. Les attentes des 36 tests existants du coordinateur sont inchangées (SC-001). Rien dans le noyau n'empêche d'intégrer 008 proprement : `plan_edit` est pur, et les codes de `ShortcutEditError` correspondent au contrat CLI.

Restent 7 constats minor et 7 nits, listés ci-dessous. Le seul écart réel à l'AC-004 (sondage OS des actions inactives, constat m1) est attribué à tort à 008.

## Constats

### Minor

**m1 — Pas de contrôle OS pour un raccourci de post-processing modifié pendant que la fonction est désactivée ; ce report n'est pas dû à 008.**
- Fichiers : `src-tauri/src/shortcut/edits.rs:102-104`, `src-tauri/src/shortcut/mod.rs:143-149`, `src-tauri/src/shortcut/mod.rs:1101-1105`.
- Scénario : backend Tauri, post-processing désactivé. L'utilisateur enregistre pour « Transcribe with Post-Processing » une combinaison déjà prise globalement par une autre application. `apply_shortcut_edit` saute l'inscription (`is_statically_registered` vaut faux) et sauvegarde. Plus tard, l'activation du post-processing appelle `let _ = register_shortcut(...)`, qui échoue sans rien dire : le raccourci est stocké, affiché, mais ne fonctionne pas, et l'utilisateur n'en est pas informé. Avant 004, `change_binding` tentait l'inscription et remontait le refus de l'OS. C'est donc une petite régression du retour à l'utilisateur, en échange d'une correction réelle : le raccourci ne devient plus actif alors que le post-processing est désactivé.
- Conformité : le comportement « non inscrit tant que désactivé » est conforme au cas limite de la spec (« Post-processing disabled: its shortcuts are unregistered but kept »). En revanche, le plan (§ Registration transactions) exige un sondage inscription puis désinscription avant sauvegarde. `progress.md` le classe « Waiting for 008 », alors qu'il ne dépend ni des révisions, ni du verrou, ni de l'IPC.
- Correctif proposé : dans `apply_shortcut_edit`, avec Tauri et une action inactive autre que `cancel` (et `cancel` hors Linux si on veut aller jusqu'au plan), inscrire temporairement chaque raccourci ajouté puis le désinscrire avant `write_settings`, et retourner `RegistrationFailed` en cas de refus. À défaut, retirer cet item de la liste « Waiting for 008 » et le noter comme écart assumé.

**m2 — Avec Tauri, modifier un raccourci en conflit hérité désinscrit le raccourci de l'autre action.**
- Fichiers : `src-tauri/src/shortcut/edits.rs:130-136` ; `src-tauri/src/shortcut/tauri_impl.rs` (`unregister_shortcut` désinscrit par valeur de `Shortcut`).
- Scénario : store hérité avec `transcribe = "alt+space"` et `transcribe_with_post_process = "option+space"`, une seule combinaison pour l'OS. Au démarrage, une seule action obtient l'inscription, par exemple le post-processing. L'utilisateur remplace `alt+space` par `f5` sur Transcribe. Le delta désinscrit `Shortcut(alt+space)`, c'est-à-dire l'inscription du post-processing : son raccourci ne répond plus jusqu'au redémarrage. Le défaut existait déjà avec l'ancien `change_binding` (même désinscription par valeur). Il contredit toutefois la promesse de l'AC-002 : les conflits hérités « keep their current registration and behavior unchanged ». HandyKeys n'est pas touché, car ses tables sont indexées par `(action, shortcut)`.
- Correctif proposé : avant de désinscrire un raccourci retiré, vérifier si un autre élément de l'inventaire, inscrit statiquement, se normalise à la même combinaison. Dans ce cas, ne pas désinscrire, ou ré-inscrire l'élément survivant avec sa propre identité juste après. Ajouter un test sur le calcul du delta.

**m3 — La réparation au changement d'implémentation ne déduplique que les chaînes identiques et ne vérifie aucun conflit.**
- Fichiers : `src-tauri/src/shortcut/action_shortcuts.rs:270-287`, `src-tauri/src/shortcut/mod.rs:520-571`.
- Scénario : passage de HandyKeys à Tauri avec `transcribe = ["option_left+space", "alt+space"]`. Le premier raccourci est invalide pour Tauri et revient au défaut `option+space`. L'alternative `alt+space` est conservée, car elle diffère en tant que chaîne. Résultat : deux entrées équivalentes dans la même action. La seconde échoue à l'inscription (« already in use », seulement journalisé), et l'interface affiche ensuite un avertissement de conflit que le checker n'aurait jamais laissé créer. Le défaut réparé peut aussi entrer en collision avec une autre action. Le plan demandait de fusionner l'alternative égale au défaut réparé et de refuser la bascule si le défaut entre en conflit.
- Correctif proposé : dans `repair_for_implementation`, comparer les alternatives au premier raccourci via `normalize_shortcut(…, destination)` plutôt qu'en chaîne. Faire passer le défaut réparé par `find_conflict` sur l'inventaire des autres actions et, en cas de conflit, refuser la bascule ou la signaler. Ajouter le test correspondant à `action_shortcuts/tests.rs`.

**m4 — Les Behavioral AC « recorder » et « Add » ne sont que partiellement réalisés.**
- Fichiers : `src/components/settings/shortcut-list.tsx:121-136` et `:177-191`, `src/components/settings/shortcut-chip.tsx:355-372`.
- Scénario : la capture soumet aussitôt la combinaison, sans étape de validation. Il n'existe donc ni état visuel `empty`, `valid` ou `invalid` du recorder, ni bouton Add `enabled` ou `disabled` selon la validité, ni « retry » explicite après un refus OS : il faut réenregistrer. Les états `idle`, `loading`, `error` et `success` existent sur la liste (`data-state`). Après une erreur, l'état reste `error` jusqu'à la soumission suivante.
- Conformité : l'écart est documenté (passe UI Penflow repoussée par la décision du 2026-10-12, Step 6 « Partial »). Il ne bloque pas le noyau, mais la spec liste ces scénarios.
- Correctif proposé : lors de la passe UI, ajouter une prévalidation backend en lecture seule (par exemple une commande `check_shortcut_candidate` qui réutilise `check_candidate`), afficher l'état du recorder, exposer un bouton « Réessayer » sur `registration_failed`, et ramener l'état à `idle` au prochain focus ou après un délai.

**m5 — Chemin hérité `change_binding`/`reset_binding` (raccourci Cancel) : refus non traduits, ou ignorés.**
- Fichiers : `src-tauri/src/shortcut/edits.rs:173-186`, `src-tauri/src/shortcut/mod.rs:284-310`, `src/stores/settingsStore.ts:429-441`.
- Scénario 1 : sous une locale autre que l'anglais, l'utilisateur donne à Cancel une combinaison de Transcribe. `change_binding` refuse désormais le conflit, ce qui est conforme à l'AC-004. Le toast affiche « Échec de la définition du raccourci : Conflicts with 'option+space' of Transcribe », dont le motif et le nom de l'action restent en anglais (`binding.name`).
- Scénario 2 : Cancel vaut `f5`, `escape` a été ajouté à Transcribe, puis l'utilisateur clique Reset sur Cancel. `reset_binding` refuse (le défaut `escape` entre en conflit) avec `success: false`. `settingsStore.resetBinding` ignore la réponse : rien ne s'affiche. Avant 004, ce reset réussissait toujours.
- Jugement sur le changement assumé « `change_binding` refuse un conflit » : conforme à la spec (AC-004 inclut cancel), sans régression fonctionnelle. Le message n'est simplement pas traduit.
- Correctif proposé : passer le raccourci Cancel sur `edit_binding_shortcuts` (replace/reset) et `describeEditError`, ou faire porter à `BindingResponse` le `ShortcutEditError` typé. Faire lever `resetBinding` du store quand `success` est faux, afin que l'appelant affiche le toast.

**m6 — Couverture de tests absente sur deux chemins sensibles du noyau.**
- Fichiers : `src-tauri/src/shortcut/edits.rs:119-169` (delta et rollback), `src-tauri/src/secure_input.rs:385-455` (plan des shadows Carbon).
- Scénario : aucun test automatisé n'exerce `apply_registration_delta` ni `restore_registrations`, que ce soit pour le refus d'une addition, la restauration des retraits ou les éléments conservés intacts. `cargo test --lib secure_input` exécute 0 test, alors que `plan_fallback_binding` et `carbon_equivalent` sont des fonctions pures (macOS) qui portent désormais l'identité de plusieurs raccourcis. Une régression de rollback ou d'identité des shadows ne serait détectée que par les preuves natives, encore en attente.
- Correctif proposé : extraire le calcul `removed`/`added` dans une fonction pure testée (cas Replace, Reset avec défaut déjà présent, promotion du premier). Ajouter des tests `#[cfg(target_os = "macos")]` sur `plan_fallback_binding` : identité conservée après élargissement des côtés, modificateur seul `Immune`, `fn` `Uncovered`.

**m7 — Les avertissements de conflits hérités ne se rafraîchissent qu'au changement du binding de la liste.**
- Fichier : `src/components/settings/shortcut-list.tsx:117-119`.
- Scénario : sur la page General, Transcribe porte un avertissement de conflit hérité avec Cancel. L'utilisateur change Cancel (recorder distinct sur la même page) et lève ainsi le conflit. L'avertissement reste affiché sur Transcribe jusqu'à un remontage ou une modification de Transcribe. Si `getShortcutConflicts` échoue, l'erreur part seulement dans la console et les avertissements disparaissent sans aucun signal.
- Correctif proposé : dépendre de l'ensemble des `bindings`, ou écouter `settings-changed`, et conserver la dernière liste connue en cas d'échec.

### Nit

- **n1** — `src-tauri/src/secure_input.rs:172-178` : `dedup_ids` déduplique chaque liste séparément. Une action dont une alternative est couverte et une autre non apparaît à la fois dans `covered` et `uncovered` du diagnostic. Documenter cette sémantique, ou dériver un statut par action (le pire gagne).
- **n2** — `src/components/settings/debug/KeyboardImplementationSelector.tsx:43-44` : le toast « Keyboard shortcuts were incompatible and reset to defaults » devient inexact quand une alternative est supprimée plutôt que réinitialisée. Ajuster le libellé (« reset or removed »).
- **n3** — `src/components/settings/shortcut-chip.tsx:367-369` (à vérifier en natif) : en capture HandyKeys, la webview reçoit encore les touches. Si la puce a le focus clavier, Tab la quitte, `onBlur` déclenche `cancelRecording`, et un raccourci contenant Tab ne peut plus être enregistré au clavier. La capture Tauri n'est pas touchée, car elle appelle `preventDefault`. Correctif possible : intercepter `keydown` pendant la capture HandyKeys, comme le fait la variante Tauri.
- **n4** — Preuves commitées volumineuses : trois copies de `conventions.stdout` de 2,2 Mo (environ 161 000 lignes) dans `proofs/claude/commit-*/`. Conserver le reçu JSON et un résumé plutôt que la sortie complète.
- **n5** — `.specs/conventions-gates.yaml` : la mise à jour de `constitution_sha256` dans `1e4617c` relève de l'entretien LiveSpec, pas de la feature. Elle est inoffensive mais mériterait son propre commit.
- **n6** — `src-tauri/src/shortcut/edits.rs:42` : `warn!` journalise chaque refus, y compris les refus prévus (conflit, limite). Un `debug!` ou `info!` suffirait pour une validation utilisateur ordinaire.
- **n7** — `progress.md`, ligne « Waiting for 008 » : l'item « Tauri OS probe for inactive actions » est à requalifier (voir m1). L'item « Migration of the remaining direct writers » est correct.

## Points vérifiés sans constat

- **Modèle (AC-001, AC-002, FR-001, FR-004)** : `current_binding` reste le premier raccourci. `additional_bindings` est additif, omis quand il est vide (store mono-raccourci identique octet pour octet, testé), et ignoré par un lecteur plus ancien (testé avec la forme héritée figée). La borne de 4 est appliquée dans `plan_add` et côté UI. Le dernier raccourci est irretirable (`LastShortcut`). La promotion au retrait du premier ne ré-inscrit rien, car l'identité est le texte du raccourci et non sa position. Le reset vers le défaut unique est vérifié contre les conflits ; les entrées conservées ne sont pas ré-inscrites. Les autres actions sont inchangées. Cancel refuse les alternatives (`AlternativesUnsupported`).
- **Activation (AC-003, FR-002, SC-001)** : scénarios déroulés sur la machine à états de production. Toggle : B arrête A, le Stop reprend les métadonnées de départ et `actions.rs` ignore `shortcut_str`. Push-to-talk : B ignoré en appui comme au relâchement, et B n'annule pas le relâchement différé de A. Hold-or-toggle : B ignoré pendant la phase maintenue et arrête en phase verrouillée. Appuis simultanés : le second est absorbé par le debounce global de 30 ms. Relâchements dans le désordre : seul le relâchement du propriétaire est différé, avec 50 ms de grâce inchangée. Bascule de mode en cours d'enregistrement : identique au comportement mono-raccourci. Appui mémorisé pendant le traitement : propriété de l'action en toggle, du raccourci en modes maintenus, tap déjà verrouillé compris. Les déclencheurs externes gardent la parité. Le diff des 36 tests existants ne montre que l'ajout du champ `shortcut_id`, sans aucune attente modifiée. 14 tests A/B ont été ajoutés.
- **Conflits (AC-004, FR-003, SC-002)** : normalisation par le parseur du backend actif, avec alias (`alt`/`option`, `cmd`/`command`), ordre des modificateurs, côtés repliés pour HandyKeys, et touches canoniques (`Key` de handy-keys est un enum, ses alias `esc`/`escape` convergent). Chevauchement de modificateurs seuls détecté dans les deux sens, au sein d'une action comme entre actions. Cancel et le post-processing désactivé font partie de l'inventaire. L'action propriétaire est nommée (`owning_action`) et traduite dans la liste. Le refus OS de Tauri intervient avant `write_settings`, avec rollback. Un conflit hérité non concerné ne bloque pas une modification valide, et un remplacement à l'identique reste un no-op.
- **Changements de comportement assumés** :
  - `change_binding` refuse désormais un conflit : conforme à l'AC-004 (« every shortcut of every action, including cancel »). Pas de régression fonctionnelle ; la limite de traduction est traitée en m5.
  - Un raccourci de post-processing désactivé n'est plus inscrit à la modification : conforme au cas limite de la spec, et cohérent avec le démarrage, qui ne l'inscrivait déjà pas. Cela corrige une incohérence d'avant 004. Le seul coût, la perte du contrôle OS, est traité en m1.
- **Secure Input** : chaque alternative a son shadow Carbon, qui garde le `shortcut_id` d'origine même avec des modificateurs élargis. Le relâchement d'une session maintenue reste donc reconnu quand Secure Input s'active pendant l'appui. La réconciliation se fait après chaque modification.
- **Code de l'UI** : aucun texte en dur ; ESLint i18n réussit. Les 11 clés `settings.general.shortcut.list.*` sont présentes et traduites dans les 26 locales : aucune n'est identique à l'anglais hors `en`, et les placeholders `{{action}}`, `{{shortcut}}`, `{{limit}}` et `{{error}}` sont cohérents partout. Les noms d'action existent dans toutes les locales. Les contrôles indisponibles restent focusables (`aria-disabled`) et donnent leur raison (`title`, `aria-label`). L'ordre de tabulation est : lignes, puis ajout, puis reset. Quitter la capture (perte de focus ou clic extérieur) n'écrit rien.
- **`bindings.ts`** : il est cohérent avec une génération par Specta (doc-comments Rust repris). Les deux nouvelles commandes sont sans `Result` et n'ont aucun `as any`. Les refus métier arrivent dans `response.error` et sont affichés. Les échecs d'IPC remontent par exception jusqu'au `catch` de `submit` (message générique et `console.error`). Seul `getShortcutConflicts` avale son erreur (m7).
- **Robustesse et conventions** : aucun `unwrap`, `expect`, `panic!`, `todo!` ou `unreachable!` ajouté en production (le seul `console.log` est dans le script de test). Pas de `any` en TypeScript. Les stubs non-macOS de `secure_input.rs` ne touchent pas aux types modifiés. Clippy ne signale rien dans les fichiers touchés. `livespec conventions verify` : PASS, 0 nouveau constat.
- **Périmètre** : le code applicatif reste dans la feature. `package.json` gagne le script `test:shortcut-list`. Hors feature, on ne trouve que la documentation LiveSpec et les preuves (n4, n5).

## Tests et vérifications exécutés

Tous lancés le 2026-10-13 dans le worktree, avec `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target` (cache réutilisé, aucun profil supplémentaire). Les journaux sont dans le scratchpad de la session (`repro-004/`).

| Commande | Résultat |
|---|---|
| `cargo test --lib` (complet) | **PASS** — 340 réussis, 0 échec, 16 ignorés |
| `cargo test --lib shortcut::` | PASS — 43 réussis |
| `cargo test --lib transcription_coordinator::` | PASS — 50 réussis (36 existants avec attentes inchangées, plus 14 `multi_shortcut_tests`) |
| `cargo test --lib secure_input` | 0 test (voir m6) |
| `cargo fmt --check` | **PASS** (exit 0) |
| `cargo clippy --all-targets` (cache réutilisé, 12 s) | exit 0 ; 5 avertissements, tous hors des fichiers de la feature ; identiques à la référence `dev` (`proofs/claude/dev-baseline/clippy.stderr`, diff vide) |
| `bun run lint` | **PASS** (exit 0) |
| `bun run test:keyboard` | **PASS** — « keyboard: all assertions passed » |
| `bun run test:shortcut-list` | **PASS** — « shortcut-list-model: all assertions passed » |
| `bun run check:translations` | **PASS** — 25 langues complètes |
| `bunx --no-install tsc --noEmit` | **PASS** (exit 0) |
| `livespec conventions verify --feature repo --json` | **PASS** — `new_count: 0`, 401 avertissements tous déjà dans la baseline ; reçu `.specs/conventions/runs/20261013T153409Z/receipt.json`, créé par la commande elle-même et non suivi par git |

Non exécuté, hors de portée d'un audit en lecture seule sans interaction physique : les preuves natives listées dans `progress.md` (matrice de touches physiques, refus OS par une autre application, Secure Input, retour à la version précédente, Windows et Linux, latence). Elles restent des preuves de release en attente et ne sont pas déclarées réussies ici.
