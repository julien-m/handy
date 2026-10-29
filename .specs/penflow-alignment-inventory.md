# Inventaire des écarts UI ↔ Penflow (features 001 à 011)

Base : `dev` @ `96eb3e1e` (2026-10-17). Lecture seule : aucun fichier du dépôt n'a été modifié.
Sources : `penflow/flow-ui-contract/contract.json` (v2 : 34 écrans, 493 nœuds, 165 actions, 100 transitions), `penflow/pencil-context-map.json` (493 nœuds appariés, du 2026-10-14 15:45 ; il donne les `pencil_node_id` cités plus bas, à revérifier dans pen.dev avant d'éditer), `penflow/style-provenance.md`, les `implementation.md` et `progress.md` des features 001 à 011, `.specs/ship.md`, puis le code `src/` (`data-testid` et `data-instance-id` rendus, `src/i18n/locales/en/translation.json`). Pas de `penflow/README`. `ui.pen` n'a pas été lu.

Chemins de code relatifs à `/Users/julienm/projects/handy/src/`, sauf ceux qui commencent par `src-tauri/`.

## Résumé

**79 écarts.**

| Feature | Écarts | TID absent du contrat | TID différent | Transition | Élément ajouté ou retiré | Texte | Ordre ou placement |
|---|---|---|---|---|---|---|---|
| 001 Microphone priority | 9 | 1 | 3 | 1 | 2 | 1 | 1 |
| 002 Word replacements | 8 | 0 | 1 | 2 | 4 | 0 | 1 |
| 003 Recognition vocabulary | 5 | 2 | 1 | 1 | 1 | 0 | 0 |
| 004 Multiple shortcuts | 8 | 2 | 3 | 0 | 1 | 1 | 1 |
| 005 Custom AI prompts | 12 | 1 | 3 | 3 | 1 | 3 | 1 |
| 006 Destination application | 9 | 0 | 4 | 0 | 4 | 0 | 1 |
| 007 Smart paste | 6 | 1 | 1 | 0 | 3 | 1 | 0 |
| 008 Settings CLI (erreurs d'écriture) | 7 | 1 | 1 | 0 | 0 | 3 | 2 |
| 009 Pin mode and Enter rule | 8 | 1 | 3 | 0 | 3 | 0 | 1 |
| 010 Recovery bubble | 6 | 1 | 3 | 0 | 1 | 1 | 0 |
| 011 Dictation field return | 1 | 0 | 0 | 0 | 0 | 1 | 0 |
| **Total** | **79** | **10** | **23** | **7** | **20** | **11** | **8** |

Le type « TID différent » couvre aussi les modèles d'instance (`instance_id_template`) et les nœuds du contrat que le code rend sous un autre préfixe.

Actions proposées :
- **Aligner le contrat** (cas par défaut) : 51 écarts.
- **Aligner le code** : 2 écarts (002-02, 009-01).
- **Décision de Julien** : 26 écarts, dont 15 relèvent d'une seule décision, D1.

### Décision structurante D1 : les éléments partagés entre écrans d'état

Penflow impose un `test_id` **unique sur tout l'arbre sémantique** : `penflow/semantic_validation.py:264` lève l'erreur `semantic.test_id_duplicate`, et `contract_preparation.py:23` lève `contract.test_id.required_or_duplicate`.

Le contrat dessine chaque état (empty, error, conflict, limit, menu ouvert, épinglé, unavailable…) comme un écran complet, avec ses propres préfixes. Les features 002, 003 et 005 suivent ce découpage : le préfixe de tous les nœuds suit l'écran affiché.

Les features 001, 004, 006, 007, 008, 009 et 010 font autrement. Les éléments partagés gardent le préfixe de l'écran de base, et seuls les éléments propres à l'état portent le préfixe de l'écran d'état. Le contrat ne peut donc pas être aligné sur ce code sans casser l'unicité.

Trois options :
- **(A)** Le code adopte le préfixe de l'écran affiché, comme 002 et 003. Seul le code change ; `ui.pen` reste inchangé.
- **(B)** Le contrat fusionne les écrans d'état en variantes `states` d'un seul écran. Il faut alors supprimer des écrans et des nœuds dans `ui.pen` et dans `pencil-context-map.json`, ce qui est lourd.
- **(C)** Le contrat accepte le mélange de préfixes. Impossible, à cause de l'unicité.

Recommandation : **(A)**. Les 15 écarts concernés sont marqués **D1**.

### Modifications de `ui.pen` nécessaires si les actions proposées sont retenues

Les identifiants de nœud viennent de `pencil-context-map.json`. Un nouveau nœud reçoit le `context` `classification=semantic; semantic_id=<id>; test_id=<test_id>; role=<rôle>`.

1. **006-07 et 010.** Archiver la région `recording-indicator-destination-unavailable` (`CCFAE`, notice `AWLRT`) : passer la révision en `archive`. Le motif est désormais montré par la bulle de récupération.
2. **009-02.** Déplacer `recording-indicator-destination-menu.menu.pin` (`VYFRs`) en premier enfant de `SeRrU`, suivi d'un séparateur décoratif.
3. **009-05.** Ajouter la marque d'épingle dans le badge `R5zj2b` : `semantic_id=recording-indicator-pinned.destination.mark; test_id=recording-indicator-pinned-destination-mark; role=icon`.
4. **009-03 et 009-04, selon la décision.** Ajouter dans `sajPj` l'entrée « Frontmost app at the end » (`recording-indicator-pinned.menu.frontmost-at-end`). Ajouter l'indication « This dictation only » (`recording-indicator-pinned.menu.hint`, `role=text`) sur les applications non épinglées (`M4esgR`).
5. **010-01.** Ajouter une variante d'état de `recovery-bubble` (`uj2qV`) avec un texte après la prévisualisation `ExmAN` : `semantic_id=recovery-bubble.late-read; test_id=recovery-bubble-late-read; role=status`, texte « The application read the clipboard late: the text may have been pasted after all. Check it before copying again. ».
6. **010-05.** Ajouter « Drag into a field » dans la bulle `recovery-bubble-insertion-failed` (`M3XJv`), avant Copy (`h1FSG`) : `recovery-bubble-insertion-failed.drag-handle`, `role=button`.
7. **010-06 et 011-01, facultatif.** Ajouter des variantes de texte :
   - motif « Could not bring ChatGPT forward » (`activation_failed`) ;
   - explication « Nothing was sent to the application. The full transcript is kept in History. », variante de `s6Bb2`.
8. **007-06.** Créer un écran `recording-indicator-hand-over`, copie de `recording-indicator-ai-failed` (`tqJAs`), avec la notice `recording-indicator-hand-over.notice` (`test_id=recording-indicator-hand-over-notice`, `role=alert`) et le texte « Text placed on the clipboard for the remote screen ».
9. **005-09, selon la décision.** Texte de `UI7xv` : « AI processing failed. The original text was used. ».
10. **005-03.** Dans `prompt-settings-invalid.form` (`yB58r`), après « Add API key » (`pI5xF`) : ajouter le champ en ligne `prompt-settings-invalid.form.fix-field` (`role=input`, libellé « API key of OpenAI »).
11. **005-04.** Ajouter les nœuds manquants de chaque mode de l'éditeur : sur `prompt-settings-invalid` (`C2u9q`), le point de terminaison et le résultat du test ; sur `prompt-settings-editor` (`EYkC8`), l'erreur de nom, la notice « not ready » et Fix. Une variante d'état suffit.
12. **005-06.** Texte de `p7nOU` : « Test failed: the provider returned an error ».
13. **005-07.** Raccourcis des réglages au format Handy (« Ctrl + Option + M » hors macOS) dans `L17Wv` et `HvVWb`. Garder les glyphes sur macOS si la maquette est macOS.
14. **004-03.** Déplacer la ligne `shortcut-settings.ai-dictation` (`f6mzXY`) de `shortcut-settings.general` (`Z45qDb`) vers l'écran `prompt-settings` (`pSr0g`), dans un groupe « Post-Processing Hotkey » placé après la bibliothèque (`glRCX`).
15. **004-04.** Textes Handy :

    | Nœuds | Avant | Après |
    |---|---|---|
    | `QIxIL`, `hXbqN`, `WybTK` | « Dictation » | « Transcribe Shortcut » |
    | `h9WeHu` | « Dictation with AI processing » | « Post-Processing Hotkey » |
    | `f5HqxY` | « Cancel » | « Cancel Shortcut » |
    | `mUmTu` | « Activation mode » | « Shortcut Behavior » |

16. **004-05.** Ajouter les nœuds dérivés :
    - `-recording-discard` et `-os-retry` dans `Fj4JW` ;
    - `-shortcut-refused`, `-shortcut-conflict` et `-conflicts-unavailable` en variantes.
17. **004-07.** Add grisé à côté de « Press keys… » dans l'état `recording` de `shortcut-settings.dictation`.
18. **007-02.** Retirer de l'écran macOS « Direct typing » (`kN4bL`) et « External script » (`KSWfk`) du menu `MY1yf`, ou les annoter « Linux/Windows ».
19. **007-04.** Textes en casse Handy :

    | Nœuds | Avant | Après |
    |---|---|---|
    | `dVE1U`, `xGzv7`, `CiDPP` | « Paste method » | « Paste Method » |
    | `BQdW6`, `iY4Rp`, `b8YHh` | « Clipboard handling » | « Clipboard Handling » |
    | `Kl1oW`, `qFdnT`, `RWO3Z` | « Don't modify clipboard » | « Don't Modify Clipboard » |
    | `kN4bL` | « Direct typing » | « Direct » |
    | `KSWfk` | « External script » | « External Script » |

20. **008-02 et 008-03.** `settings-write-errors.save-failed` (`S46r1`) devient un toast d'erreur hors de la carte History. Texte de `PrCcL` : « Couldn't save settings: the settings file could not be written. Nothing was changed. ».
21. **008-04 et 008-05.** Texte de `dttcW` : « Enter a whole number from 0 to 1000. ». Texte de `tfrkx` : « History Limit ».
22. **008-06.** Dans `WmvT5`, ajouter la ligne `settings-write-errors.conflict.reload-failed` (`role=text`) : « The settings could not be reloaded. Try again. ».
23. **003-03 et 003-04.**
    - Ajouter `vocabulary-settings-unsupported.count` dans `nQttn`.
    - Ajouter des variantes `loading`, `list-error` et `error` sur les écrans vocabulaire.
24. **002-03.** Ajouter le bouton Dismiss et le corps du texte dans `replacement-settings-import.unreadable` (`S5d0TF`).
25. **002-04, 002-05 et 002-06.** Notice d'échec de commit, dialogue de garde « Unsaved changes » et dialogue de confirmation de suppression. Il faut aussi des test IDs dans le code.
26. **001-05 et 001-06, selon la décision.**
    - Save inactif (`b3Vw2l`) dans `microphone-settings-error`.
    - Texte du bandeau de conflit (`mvAcL`, titre `iwkAM`) à reformuler.
27. **001-07, selon la décision.** Barre Unsaved changes (`c4cjgJ`) collante ou placée après la ligne closed-lid.
28. **001-03.** Variante `microphone-settings.loading`.
29. **D1.** Aucune modification avec l'option (A). Avec l'option (B), refonte des écrans d'état.

## Inventaire par feature

Légende des actions :
- **contrat** : aligner le contrat sur le code ;
- **code** : aligner le code sur le contrat ;
- **Julien** : décision de Julien.

« Déclaré » signifie que l'écart figure dans `implementation.md`, `progress.md` ou `ship.md`. « Non déclaré » signifie qu'il a été trouvé par comparaison avec le code.

### 001 Microphone priority

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 001-01 | `microphone-settings-empty` | TID différent (D1) | `microphone-settings-empty-{screen,sound,priority,priority-title,add,refresh,in-use,in-use-title,in-use-device,clamshell,clamshell-title,clamshell-select}` (12) | Mêmes éléments en `microphone-settings-*` : `components/settings/microphone-priority/microphone-priority-settings.tsx:48,56`, `priority-editor.tsx:83-84`, `priority-controls.tsx:96,161`, `effective-input.tsx:47-53`, `clamshell-row.tsx:52,83-84`. Seuls `-empty-empty{,-title,-body}` ont le préfixe empty (`priority-list.tsx:31-42`) | Déclaré (implementation.md:82, « Not changed » du round 1) | Julien (D1, reco A) | Aucune avec (A) |
| 001-02 | `microphone-settings-error` | TID différent (D1) | `microphone-settings-error-{screen,sound,priority,priority-title,choices,choice,choice-rank,choice-name,choice-status,choice-move-up,choice-move-down,choice-remove,add,draft,draft-note,discard,save}` (17) | Préfixe de base : `draft-feedback.tsx:104,117,132,138`, `priority-row.tsx:92,114,152-179`, `priority-list.tsx:99`, `priority-controls.tsx:96`. Seuls `-error-{conflict,conflict-title,reload,refresh-error,retry,add-error}` ont le préfixe error | Déclaré (implementation.md:82) | Julien (D1, reco A) | Aucune avec (A) |
| 001-03 | `microphone-settings` | TID absent du contrat | Aucun nœud de chargement | `microphone-settings-loading` (`priority-editor.tsx:48`) | Déclaré (implementation.md:82, « Extra ») | contrat : nœud ou variante `loading` | Variante `microphone-settings.loading` |
| 001-04 | `microphone-settings`, `-error` | TID différent (instance) | `repeat_model.instance_id_template` `microphone-{name}` sur `-choices` | `data-name={name}` sans `data-instance-id` (`priority-row.tsx:180`) | Déclaré (implementation.md:82) | Julien. Le contrat ne peut pas exprimer `data-name`. Reco : le code ajoute `data-instance-id="microphone-<name>"` | Aucune |
| 001-05 | `microphone-settings-error` | Élément (état) | Maquette et nœud `-error-save` : Save actif pendant le conflit | Save inactif (`aria-disabled`, `data-state="disabled"`) jusqu'à Reload ou Discard (`microphone-priority/draft-state.ts:3-4`, `draft-feedback.tsx:117`) | Déclaré (ship.md:184 « À valider avec Julien », conforme AC-007) | Julien (reco : contrat et `ui.pen` sur le code) | `b3Vw2l` à l'état inactif |
| 001-06 | `microphone-settings-error` | Texte | Corps du bandeau dans `ui.pen` : « Your edits are kept… » ; le contrat ne porte que le titre (`-error-conflict-title`) | `settings.sound.microphonePriority.conflict.body` « Your edits are kept. Reload shows the saved order and discards your edits. » (`i18n/locales/en/translation.json:317`) ; Save est désormais inactif | Déclaré (ship.md:184 : « trompeur → maquette + contrat Penflow à aligner ») | Julien : choisir le texte, puis aligner le code et `ui.pen` | `mvAcL` (corps), titre `iwkAM` inchangé |
| 001-07 | `microphone-settings` | Placement | `microphone-settings.draft` suit `sound`, qui ne contient que priority, in-use et closed-lid | Barre sous tout le groupe Sound (Channel, Mute, Audio feedback, Output device et Volume compris), non collante (`components/settings/general/GeneralSettings.tsx:38-51`) | Déclaré (implementation.md:86, à vérifier en vraie fenêtre) | Julien : collante, déplacée après closed-lid, ou acceptée | `c4cjgJ` selon la décision |
| 001-08 | `microphone-settings` | Élément ajouté | Le groupe `sound` ne contient que priority, in-use et clamshell | Lignes Handy existantes dans le même groupe (`GeneralSettings.tsx:41-49`) ; `microphone-settings-screen` imbriqué dans `shortcut-settings-screen` (`GeneralSettings.tsx:26-52`) | Non déclaré (préexistant) | contrat : déclarer les lignes héritées et l'imbrication | Facultatif |
| 001-09 | `microphone-settings-empty` → `microphone-settings` | Transition | Action `microphone-settings-empty.add` sans `success_transition` | Ajouter un micro bascule sur l'écran principal avec un brouillon (`priority-list.tsx`, `use-priority-editor.ts`) | Non déclaré | contrat : transition `empty.add` vers `microphone-settings`, état `default` avec brouillon | Aucune |

Hors contrat, signalé pour Julien : la valeur « None » de Sound s'affiche « Default » dans le sélecteur de debug (implementation.md:85). C'est une décision produit, non comptée.

### 002 Word replacements

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 002-01 | `replacement-settings-import` | Placement | Écran autonome (`jLdv7`) | `replacement-settings-import-screen` rendu dans la racine `${screen}-screen` de la liste, de l'écran vide ou du texte (`components/settings/word-replacements/word-replacements.tsx:41-42`, `import-dialog.tsx:29`) | Non déclaré | contrat : noter la superposition | Aucune |
| 002-02 | `replacement-settings-import` | TID différent (portée) | `replacement-settings-import-dialog` (modal) contient le titre, le fichier, le mode, le résumé, Cancel et Apply | Test ID sur le corps (`import-dialog.tsx:93`) ; titre, Cancel (`:167`) et Apply (`:150`) dans le pied du `Dialog`, hors du nœud | Déclaré (progress.md:198, info « testid of the import dialog » non corrigé) | **code** : porter le test ID sur la racine du `Dialog` | Aucune |
| 002-03 | `replacement-settings-import` | Élément ajouté | `unreadable` n'a qu'un titre | Corps du texte et bouton Dismiss (`import-dialog.tsx:56-75`) | Non déclaré | contrat : nœuds `unreadable.body` et `unreadable.dismiss` ; test ID à ajouter au code | Ajout dans `S5d0TF` |
| 002-04 | `replacement-settings`, `-bulk` | Élément ajouté | Aucune notice d'échec de commit | Alerte d'échec et avertissement « Reload discards » sans test ID (`commit-failure-note.tsx:37-38`, clé `settings.replacements.commit.reloadDiscards`) | Déclaré (progress.md:190 : « a note was chosen over a confirmation ») | contrat : nœud `commit-failure` et test ID dans le code | Ajout d'une variante |
| 002-05 | `replacement-settings` | Élément ajouté | Aucun dialogue « Unsaved changes » | Garde de navigation (`draft-gate.tsx`, `components/unsaved-changes-dialog.tsx`) sans test ID | Déclaré (progress.md:178, Behavioral AC) | contrat : modal et test ID dans le code | Ajout d'une modal |
| 002-06 | `replacement-settings` | Élément ajouté | État `confirm-delete` sans nœud | Dialogue de confirmation sans test ID (`delete-rule-dialog.tsx`) | Non déclaré | contrat : nœud de modal et test ID dans le code | Ajout d'une modal |
| 002-07 | `replacement-settings-empty` → `replacement-settings` | Transition | `empty.add` → état `adding` sur `replacement-settings-empty` | Le formulaire s'ouvre et l'écran devient `replacement-settings` (`word-replacements.tsx:36-37,68-71`, `rules-model.ts` `screenId`) | Non déclaré | contrat : destination `replacement-settings` | Aucune |
| 002-08 | `replacement-settings-import` → `replacement-settings` | Transition | `dialog.apply` → état `imported` sur l'écran import | Le dialogue se ferme et l'écran revient à la liste (`file-store.ts:171-182`) | Non déclaré | contrat : destination `replacement-settings` | Aucune |

### 003 Recognition vocabulary

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 003-01 | `vocabulary-settings`, `-unsupported` | TID différent (instance) | `term-{key}` | `data-instance-id="term-<index stocké>"` (`components/settings/vocabulary/vocabulary-terms.tsx:63`), unique même avec des doublons hérités | Déclaré (implementation.md:165-166, ship.md:156) | contrat : `item_key` `index`, gabarit `term-{index}` | Aucune |
| 003-02 | `vocabulary-settings-paste` → `vocabulary-settings` | Transition | `paste.apply.applied` reste sur `paste`, état `applied` | Apply ferme la boîte, affiche un toast et rend le focus à « Paste a list… » (`components/settings/CustomWords.tsx:254-262,265-273`) | Déclaré (implementation.md:167-169, ship.md:156) | contrat : destination `vocabulary-settings` | Aucune |
| 003-03 | les 4 écrans vocabulaire | TID absent du contrat | Aucun | `<screen>-loading`, `<screen>-list-error` (avec Retry), `<screen>-error` (`vocabulary-notices.tsx:52,85,105`) | Déclaré (implementation.md:170-171) | contrat : nœuds ou variantes par écran | Variantes |
| 003-04 | `vocabulary-settings-unsupported` | TID absent du contrat | `count` seulement sur `vocabulary-settings` (`I0aOa`) | Le compteur s'affiche sur toute liste non vide, donc `vocabulary-settings-unsupported-count` (`CustomWords.tsx:223`, `vocabulary-terms.tsx:306`) | Déclaré (implementation.md:109) | contrat : nœud `unsupported.count` | Ajout dans `nQttn` |
| 003-05 | `vocabulary-settings`, `-unsupported`, `-empty` | Élément (rôle) | `status` : `role` `alert` | `role="status"` (poli) (`vocabulary-status.tsx:41`) | Déclaré (implementation.md:110) | contrat : `role=status` | Contexte de `oYNqU` et de ses équivalents |

### 004 Multiple dictation shortcuts

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 004-01 | `shortcut-settings-conflict` | TID différent (D1) | `shortcut-settings-conflict-{screen,general,dictation,dictation-title,dictation-list,dictation-shortcut,dictation-shortcut-remove,dictation-recording}` | Préfixe de base `shortcut-settings-dictation-*` (`components/settings/shortcut-list-feedback.ts:247-262`). Seuls `-conflict`, `-legacy(-title)`, `-change` et `-os(-title)` ont le préfixe conflict (`:265-269`) | Déclaré (004 progress.md:117) | Julien (D1, reco A) | Aucune avec (A) |
| 004-02 | `shortcut-settings-limit` | TID différent (D1) | `shortcut-settings-limit-{screen,general,dictation,dictation-title,dictation-list,dictation-shortcut,dictation-shortcut-remove,dictation-add}` | Préfixe de base ; seul `-limit-dictation-limit` a le préfixe limit (`shortcut-list-feedback.ts:264`) | Déclaré (progress.md:117) | Julien (D1, reco A) | Aucune avec (A) |
| 004-03 | `shortcut-settings` → `prompt-settings` | Placement et ordre | `general` = dictation, ai-dictation, activation, cancel | Ligne AI sur la page Post Process, seulement si le post-traitement est actif (`components/settings/post-processing/PostProcessingSettings.tsx:191-197`) ; General = dictation, activation, cancel (`GeneralSettings.tsx:32-35`) | Déclaré (progress.md:122) | contrat : déplacer `shortcut-settings.ai-dictation` vers l'écran `prompt-settings` (mêmes test IDs) | Déplacer `f6mzXY` dans `pSr0g` |
| 004-04 | `shortcut-settings`, `-conflict`, `-limit` | Texte | « Dictation », « Dictation with AI processing », « Cancel », « Activation mode » | « Transcribe Shortcut », « Post-Processing Hotkey », « Cancel Shortcut », « Shortcut Behavior » (`translation.json:172,176,180,226`) | Déclaré (progress.md:123 : « Action names … keep Handy's existing texts ») | contrat | `QIxIL`, `hXbqN`, `WybTK`, `h9WeHu`, `f5HqxY`, `mUmTu` |
| 004-05 | `shortcut-settings`, `-conflict` | TID absent du contrat | Aucun | `-recording-discard`, `-os-retry` (Try again et message de l'OS), `-shortcut-refused`, `-shortcut-conflict`, `-conflicts-unavailable` (`shortcut-list-feedback.ts:256-269`, `shortcut-list-rows.tsx:111-114,191,296`, `shortcut-list-alerts.tsx:93`) | Déclaré (progress.md:117,124-125) | contrat | Nœuds dans `Fj4JW` et en variantes |
| 004-06 | `shortcut-settings` | TID absent du contrat | `reset` seulement pour dictation ; `last` seulement pour ai-dictation ; cancel = titre et un bouton | La même liste pour chaque action : `shortcut-settings-dictation-last`, `-ai-dictation-reset`, `-cancel-list`, `-cancel-reset`, `-cancel-recording` (`shortcut-list-rows.tsx:241,265`, `shortcut-list.tsx:186`, `GeneralSettings.tsx:35` en mode `single`) | Partiellement déclaré (progress.md:114 : Cancel passe par la même liste) | contrat | Variantes |
| 004-07 | `shortcut-settings` | Élément (état) | Maquette : Add actif à côté de « Press keys… » | Add inactif tant que l'enregistreur est vide (has_validation) | Déclaré (progress.md:121) | contrat et `ui.pen` | État `recording` de `cl3Ze` : Add grisé |
| 004-08 | `shortcut-settings*` | TID différent (instance) | `shortcut-{keys}` sur `-dictation-list` et `-ai-dictation-list` | Pas de `data-instance-id` sur les lignes (`shortcut-list-rows.tsx:122`, `shortcut-chip.tsx:208`) | Non déclaré | Julien (reco : le code ajoute `data-instance-id`) | Aucune |

### 005 Custom AI prompts

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 005-01 | `prompt-settings` → `prompt-settings-invalid` | Transition | Action `prompt-settings.new` → `prompt-settings-editor` | Un nouveau prompt ouvre le préfixe `prompt-settings-invalid` (`components/settings/post-processing/prompt-draft-model.ts:156-157`) | Déclaré (implementation.md:42, ship.md:172) | contrat : destination `prompt-settings-invalid` | Aucune |
| 005-02 | `prompt-settings-invalid` → `prompt-settings-editor` | Transition | `invalid.save` → état `saved` sur `-invalid` | Après l'enregistrement, l'éditeur repasse en mode édition (`PostProcessingSettings.tsx:219`) | Déclaré (implementation.md:42) | contrat : destination `prompt-settings-editor` | Aucune |
| 005-03 | `prompt-settings-invalid` | Transition et élément ajouté | `invalid.form.fix` = navigation vers `prompt-settings` | Champ en ligne dans l'éditeur pour le fournisseur résolu, `<prefix>-form-fix-field` (`prompt-provider-fix.tsx:108,148`) ; ni navigation ni brouillon perdu | Déclaré (implementation.md:41 : « Penflow should align the contract ») | contrat : `state_change` → `fix-field-open`, nœud `form.fix-field` | Champ après `pI5xF` dans `yB58r` |
| 005-04 | `prompt-settings-editor`, `-invalid` | TID différent (préfixes croisés) | Nœuds dessinés sur un seul écran | En mode new : `prompt-settings-invalid-{form-text-hint,form-endpoint,test-result,test-output,test-meta,delete…}`. En mode edit : `prompt-settings-editor-{form-name-error,form-not-ready(-title),form-fix,test-error(-title)}` (`prompt-editor-fields.tsx:95,106`, `prompt-provider-fix.tsx:52,148,171-174`, `prompt-test-panel.tsx:45-53,83-86`) | Déclaré (implementation.md:42) | contrat : ajouter sur chaque écran les nœuds de l'autre mode (les préfixes restent uniques) | Variantes sur `C2u9q` et `EYkC8` |
| 005-05 | `prompt-settings`, `-editor`, `-invalid` | TID absent du contrat | Aucun | Liste : `prompt-settings-{prompt-status,load-error,loading,empty,reload-error,reload-retry}` (`prompt-list.tsx:58,148,169`, `prompt-library-notice.tsx:28,37`). Éditeur : `<prefix>-{saved,actions-error,deleted-elsewhere,changed-elsewhere,form-text-error,form-provider-error,form-shortcuts-hint,test-retry,test-outdated,delete-dialog,delete-confirm,reload-error,reload-retry,form-shortcut-recorder,form-shortcut-refused,form-shortcut-conflict,form-shortcuts-conflicts-unavailable,form-shortcuts-error}` (`prompt-editor-actions.tsx:39,48`, `prompt-editor.tsx:46,57,97`, `prompt-editor-fields.tsx:140`, `prompt-provider-fields.tsx:183`, `prompt-test-panel.tsx:74,111`, `prompt-delete-dialog.tsx:132,146`, `shortcut-row.tsx:92-101`, `shortcut-list-editor.tsx:140,149`) | Partiellement déclaré (implementation.md:42, « New IDs of this round ») | contrat | Variantes |
| 005-06 | `prompt-settings-invalid` | Texte | « Test failed: provider returned HTTP 401 » | Raison typée « Test failed: the provider returned an error » (`translation.json:717`) ; jamais le message du fournisseur | Déclaré (progress.md:52) | contrat | `p7nOU` |
| 005-07 | `prompt-settings`, `-editor` | Texte | Glyphes « ⌃ ⌥ M », « ⌃ ⌥ S » | Formateur Handy (« Ctrl + Option + M ») dans les réglages ; glyphes sur l'indicateur macOS | Déclaré (progress.md:52,66) | contrat : selon la plateforme de la maquette | `L17Wv`, `HvVWb` |
| 005-08 | `prompt-settings` | Élément ajouté | Écran = traitement, défaut, bibliothèque | Groupes « AI shortcut » et « provider » sous les prompts (`PostProcessingSettings.tsx:191-200`) | Déclaré (progress.md:52) | contrat (lié à 004-03) | Groupes après `glRCX` |
| 005-09 | `recording-indicator-ai-failed` | Texte | « The AI step failed. The original text was used for insertion. » | « AI processing failed. The original text was used. » (`translation.json:1252`) ; le test AC-005 de `src-tauri/src/prompts/dictation.rs` interdit « insert » | Déclaré (progress.md:66, ship.md:155 « à trancher ») | **Julien** : maquette ou spec AC-005 | `UI7xv` selon la décision |
| 005-10 | `recording-indicator-ai-failed` | Placement | Notice sur l'indicateur | Si le style d'overlay est `none` : toast de la fenêtre des réglages, au moins 4 s (`hooks/use-overlay-none-notices.ts`) | Déclaré (progress.md:66, ship.md:155 « peu visible ») | **Julien** | Variante éventuelle |
| 005-11 | `prompt-settings`, `-editor` | TID différent (instance) | `prompt-{id}` sur `prompt-settings-list` ; `shortcut-{keys}` sur `-editor-form-shortcuts` | Pas de `data-instance-id` dans les réglages (`prompt-list.tsx:112`) ; l'indicateur porte bien `prompt-{id}` (`overlay/prompt-pill.tsx:48`) | Non déclaré | Julien (reco : le code ajoute `data-instance-id`) | Aucune |
| 005-12 | `recording-indicator-prompt-menu` | TID différent (D1) | `recording-indicator-prompt-menu-{screen,bar,pill,pill-timer,prompt,prompt-shortcut,destination,destination-name,destination-remove}` | Préfixe de base `recording-indicator-*` (`overlay/indicator-stack.tsx:178,188`, `overlay/RecordingOverlay.tsx:285,325,378`, `overlay/prompt-pill.tsx:217,227`, `overlay/destination-badge.tsx:73,84,124`) ; seul le menu porte `-prompt-menu-menu-*` (`prompt-pill.tsx:21`) | Non déclaré | Julien (D1, reco A) | Aucune avec (A) |

### 006 Destination application

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 006-01 | `recording-indicator-destination-menu` | TID différent (D1) | `recording-indicator-destination-menu-{screen,bar,pill,pill-timer,prompt,prompt-shortcut,prompt-hint,destination,destination-name,destination-remove}` | Préfixe de base (mêmes fichiers que 005-12) ; seul le menu porte `-destination-menu-menu-*` (`overlay/destination-menu.tsx:15`) | Non déclaré | Julien (D1, reco A) | Aucune avec (A) |
| 006-02 | `recording-indicator-no-destination` | TID différent (D1) | `recording-indicator-no-destination-{screen,bar,pill,pill-timer,prompt,prompt-shortcut,prompt-hint,destination,destination-name}` | Préfixe de base ; badge « Frontmost app » (`destination-badge.tsx:73,84`) | Non déclaré | Julien (D1, reco A) | Aucune avec (A) |
| 006-03 | `recording-indicator-destination-menu`, `-pinned` | TID différent (instance) | `app-{bundle_id}` | `data-instance-id="app-<token>"` (`destination-menu.tsx:94`) ; aucun bundle id sur le fil | Déclaré (006 progress.md:102) | contrat : `item_key` `token` | Aucune |
| 006-04 | `recording-indicator*` | Élément (visuel) | Icônes d'application | Monogramme sur `--s-accent-soft`, car `icon_ref` vaut toujours `None` | Déclaré (progress.md:102 ; ship.md:155 « cache d'icônes 006 ») | **Julien** : garder la maquette comme cible (cache d'icônes à faire) ou dessiner le monogramme | Aucune si la cible est gardée |
| 006-05 | `recording-indicator` | Placement | Une barre : prompt, destination, pastille | Forme Live : le badge forme sa propre pastille à côté de la carte ; en compact, le minuteur ne s'affiche qu'à côté d'un badge | Déclaré (progress.md:98) | contrat : variante « Live » | Variante facultative |
| 006-06 | `recording-indicator-destination-menu` | Élément ajouté | Aucun état chargement, erreur ou refus | « loading », « failed » et le retour de refus (`stale`, `unavailable`) sans test ID (`destination-menu.tsx:203-221`) | Déclaré (progress.md:98) | contrat : nœuds et test IDs dans le code | Variantes |
| 006-07 | `recording-indicator-destination-unavailable` | Élément retiré | Écran avec la notice « Notes is closed. Nothing was inserted… » (`recording-indicator-destination-unavailable-notice`) | Notice supprimée par 010 ; le motif passe par la bulle (`overlay/notice-model.ts:1-4`) ; plus aucun rendu | Déclaré (010 implementation.md:69,85) | contrat : retirer l'écran (ou le marquer archivé) et renvoyer vers `recovery-bubble` | Archiver `CCFAE` |
| 006-08 | `output-settings-unavailable` | TID différent (D1) | `output-settings-unavailable-{screen,output,paste-method(-title,-select),clipboard(-title,-select),return-app(-title,-info,-toggle),auto-enter(-title,-mode,-key),no-field-bubble(-title,-toggle)}` (18) | Préfixe de base `output-settings-*` (`components/settings/return-to-app-setting.tsx:15`, `PasteMethod.tsx:60-65`, `ClipboardHandling.tsx:38-42`, `recovery-bubble-toggle.tsx:35`, `advanced/AdvancedSettings.tsx:45,56`) ; l'état reste visible par le bouton inactif et sa raison | Non déclaré | Julien (D1, reco A) | Aucune avec (A) |
| 006-09 | `output-settings` | Élément (visibilité) | La ligne `return-app` est toujours présente | Ligne masquée quand le verrou `destination-return` est fermé (`return-to-app-model.ts:41-46`, `return-to-app-setting.tsx:114`) | Déclaré (implementation.md:39) | contrat : visibilité conditionnelle au verrou | Aucune |

### 007 Smart paste

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 007-01 | `output-settings-paste-methods` | TID différent (D1) | `output-settings-paste-methods-{screen,output,paste-method,paste-method-title,paste-method-select,clipboard,clipboard-title,clipboard-select}` | Préfixe de base `output-settings-paste-method*` (`components/settings/PasteMethod.tsx:60-65`, `ClipboardHandling.tsx:38-42`). Les options (`PasteMethod.tsx:13,66`) et l'alerte (`components/AccessibilityPermissions.tsx:19,72-77`) portent bien le préfixe paste-methods | Déclaré (007 progress.md:18, liste des ids) | Julien (D1, reco A) | Aucune avec (A) |
| 007-02 | `output-settings-paste-methods` | Élément retiré | Options smart, clipboard, direct, script et none | Sur macOS, ni Direct typing ni External script (`paste-method-options.ts:81-95`) | Déclaré (progress.md:18 : « Kept from Handy », amont #1708) | contrat : options selon la plateforme | Retirer ou annoter `kN4bL` et `KSWfk` |
| 007-03 | `output-settings-paste-methods` | TID absent du contrat | Aucun | `-paste-method-clipboard-ctrl-shift-v`, `-paste-method-clipboard-shift-insert` sous Linux (`paste-method-options.ts:30-31`) | Non déclaré | contrat : variante Linux | Facultatif |
| 007-04 | `output-settings*` | Texte | « Paste method », « Clipboard handling », « Don't modify clipboard », « Direct typing », « External script » | « Paste Method », « Clipboard Handling », « Don't Modify Clipboard », « Direct », « External Script » (`translation.json:407,414,416,432,435`) | Non déclaré (textes Handy) | contrat, selon la règle de 004-04 (textes Handy gardés) ; à confirmer par Julien s'il préfère la casse de phrase | Voir le point 19 de la liste `ui.pen` |
| 007-05 | `output-settings` | Élément ajouté | `output` = paste-method, clipboard, return-app, auto-enter, no-field-bubble | Ligne `TypingToolSetting` entre paste method et clipboard (`advanced/AdvancedSettings.tsx:59`), préexistante (Linux) | Non déclaré | contrat : ligne héritée | Aucune |
| 007-06 | (nouvel écran) | Élément ajouté | Pas d'écran de transfert vers Screen Sharing | Notice `recording-indicator-hand-over-notice` « Text placed on the clipboard for the remote screen », au moins 4 s (`overlay/notice-model.ts:42`, `translation.json:1306`) | Déclaré (implementation.md:25 ; ship.md:111, où c'était encore un toast) | contrat : nouvel écran `recording-indicator-hand-over` | Nouvelle région, copie de `tqJAs` |

### 008 Settings CLI : erreurs d'écriture des réglages

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 008-01 | `settings-write-errors` | Placement | Alerte de conflit dans l'écran, au-dessus de History | Alerte globale en haut de la fenêtre (`App.tsx:368`, `components/settings/settings-conflict-alert.tsx:44`) | Non déclaré | contrat : placement global | Facultatif (`WmvT5`) |
| 008-02 | `settings-write-errors` | Placement | `save-failed` = alerte en ligne | Toast d'erreur portant les test IDs (`stores/settingsStore.ts:112,124-133`) | Déclaré (progress.md:14 : « the notice ») | contrat : `role` toast ou status, hors de la carte | `S46r1` en toast |
| 008-03 | `settings-write-errors` | Texte | « Couldn't save settings: the settings file is read-only. Nothing was changed. » | « …the settings file could not be written. Nothing was changed. » (`translation.json:1216`) | Déclaré (progress.md:14) | contrat | `PrCcL` |
| 008-04 | `settings-write-errors` | Texte | « Enter a whole number from 0 to 10000. » | Plage du service, de 0 à 1000 (`components/settings/history-limit-model.ts:6`, `translation.json:874`) | Déclaré (progress.md:14 : « Range text follows the service ») | contrat | `dttcW` |
| 008-05 | `settings-write-errors` | Texte | « History limit » | « History Limit » (`translation.json:871`) | Non déclaré | contrat | `tfrkx` |
| 008-06 | `settings-write-errors` | TID absent du contrat | Aucun | `settings-write-errors-conflict-reload-failed` (`settings-conflict-alert.tsx:23`) | Déclaré (progress.md:15) | contrat | Ligne dans `WmvT5` |
| 008-07 | `settings-write-errors` | TID différent (D1) | `settings-write-errors-screen` | Pas de racine ; `settings-write-errors-history` est rendu dans `output-settings-screen` (`advanced/AdvancedSettings.tsx:82`) | Non déclaré | Julien (D1) : écran d'état sans racine propre | Aucune avec (A) |

### 009 Pin mode and Enter rule

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 009-01 | `recording-indicator-destination-menu` | TID différent | `recording-indicator-destination-menu-menu-pin` | `recording-indicator-pinned-menu-pin` (`overlay/pin-state.ts:10`) | Déclaré (implementation.md:55 ; ship.md:202 « à aligner sur le contrat ») | **code** | Aucune |
| 009-02 | `recording-indicator-destination-menu` | Ordre | `pin` en dernier enfant, après `frontmost-at-end` | Pin au-dessus de la liste, suivi d'un séparateur (`destination-menu.tsx:55-70,211`) | Déclaré (implementation.md:54, Behavioral AC) | contrat : ordre des enfants | Déplacer `VYFRs` en tête de `SeRrU` |
| 009-03 | `recording-indicator-pinned` | Élément ajouté | Menu épinglé sans « Frontmost app at the end » | Entrée toujours rendue (`destination-menu.tsx:153-156`) | Déclaré (ship.md:202 « contrat ou rendu à aligner ») | **Julien** | Ajout dans `sajPj` si le code est gardé |
| 009-04 | `recording-indicator-pinned` | Élément ajouté | Pas d'indication | « This dictation only » sur chaque application non épinglée, `recording-indicator-pinned-menu-hint` (`destination-menu.tsx:76,100`) | Déclaré (implementation.md:54-55 ; ship.md:202 « indication sur toutes les apps vs maquette ») | **Julien** | Ajout sur `M4esgR` si le code est gardé |
| 009-05 | `recording-indicator-pinned` | TID absent du contrat | Aucun | `recording-indicator-pinned-destination-mark` (`overlay/destination-badge.tsx:91`) | Déclaré (implementation.md:55) | contrat | Marque dans `R5zj2b` |
| 009-06 | `recording-indicator-pinned` | TID différent (D1) | `recording-indicator-pinned-{screen,menu,menu-now-label,menu-frontmost,menu-others-label,menu-app,bar,pill,pill-timer,prompt,prompt-shortcut,prompt-hint,destination,destination-name,destination-remove}` | Menu en `recording-indicator-destination-menu-menu-*` (`destination-menu.tsx:15,126-147,201`) et barre au préfixe de base ; seuls Unpin, la marque et l'indication portent le préfixe pinned | Partiellement déclaré (implementation.md:55) | Julien (D1, reco A) | Aucune avec (A) |
| 009-07 | `output-settings` | TID différent (D1) | `output-settings-auto-enter{,-title,-mode,-key}` | Toujours `output-settings-enter-options-auto-enter*` (`components/settings/automatic-enter-state.ts:11-24`, `automatic-enter.tsx:44-100`) | Déclaré (implementation.md:55 : « used as is ») | Julien (D1, reco A) | Aucune avec (A) |
| 009-08 | `output-settings` | Élément (visibilité) | Seule la ligne Automatic Enter | Verrou fermé : le contrôle `AutoSubmit` de `dev` (`advanced/AdvancedSettings.tsx:62-66`) | Déclaré (implementation.md:48-53) | contrat : variante selon le verrou | Aucune |

### 010 Recovery bubble

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 010-01 | `recovery-bubble` | TID absent du contrat | Aucun | `recovery-bubble-late-read` (`role="status"`) dans le corps (`overlay/recovery-bubble.tsx:130-140`, `translation.json:1325`) | Déclaré (implementation.md:68 : « to add to the Penflow contract ») | contrat | Variante dans `uj2qV` |
| 010-02 | `recovery-bubble-explanation` | TID différent (D1) | `recovery-bubble-explanation-{screen,bubble,reason,close,preview,drag-handle,copy,why}` | Préfixe de base ; seul le bloc d'explication porte `recovery-bubble-explanation-explanation(-text)` (`recovery-bubble.tsx:155-158`), quel que soit le motif. État exposé par `data-state="explained"` (`:220`) | Déclaré (implementation.md:78) | Julien (D1, reco A) | Aucune avec (A) |
| 010-03 | `recovery-bubble-no-field` | TID différent (D1) | `recovery-bubble-no-field-{screen,bubble,reason,close,preview,drag-handle,copy,why}` | Préfixe de base et `data-reason="no_text_field"` (`recovery-bubble.tsx:221`, `overlay/recovery-window.tsx:17`) | Déclaré (implementation.md:78) | Julien (D1, reco A) | Aucune avec (A) |
| 010-04 | `recovery-bubble-insertion-failed` | TID différent (D1) | `recovery-bubble-insertion-failed-{screen,bubble,reason,close,preview,explanation,explanation-text,copy,why}` | Préfixe de base et `data-reason="insertion_failed"` ; explication en `recovery-bubble-explanation-explanation*` | Déclaré (implementation.md:78) | Julien (D1, reco A) | Aucune avec (A) |
| 010-05 | `recovery-bubble-insertion-failed` | Élément ajouté | Pas de poignée de glisser | Poignée affichée sur macOS pour tous les motifs (`src-tauri/src/recovery/types.rs:78-80`, `recovery-bubble.tsx:229`), conforme à la spec 010 AC-002 | Non déclaré | contrat : nœud `drag-handle` (macOS) | Ajout dans `M3XJv` |
| 010-06 | `recovery-bubble*` | Texte (variante) | Quatre motifs dessinés | Cinquième motif `activation_failed` : « Could not bring {{app}} forward » (`translation.json:1316`, `src-tauri/src/recovery/decision.rs:55`), présent dans la spec 010 AC-002 | Non déclaré | contrat : variante de texte | Facultatif |

### 011 Dictation field return

| # | Écran | Type | Contrat | Code | Statut | Action | `ui.pen` |
|---|---|---|---|---|---|---|---|
| 011-01 | `recovery-bubble-insertion-failed` | Texte (variante) | Explication « Some text may have reached the application. The full transcript is kept in History. » | Si le retour au champ échoue : « Insertion failed » et « Nothing was sent to the application. » (`src-tauri/src/recovery/decision.rs:57-63`, `translation.json:1324`), comme le prévoit 011 AC-004 | Non déclaré (011 n'ajoute aucune UI ; FR-005) | contrat : variante d'explication | Variante facultative de `s6Bb2` |

## Écarts qui demandent une décision de Julien

1. **D1** (15 écarts) : 001-01, 001-02, 004-01, 004-02, 005-12, 006-01, 006-02, 006-08, 007-01, 008-07, 009-06, 009-07, 010-02, 010-03, 010-04. Recommandation : le code suit le préfixe de l'écran affiché.
2. **Modèles d'instance non exprimables ou absents** (3) : 001-04 (`data-name`), 004-08, 005-11. Recommandation : ajouter `data-instance-id` dans le code.
3. **001-05.** Save inactif pendant un conflit, alors que la maquette le montre actif.
4. **001-06.** Texte du bandeau de conflit : « Your edits are kept… ».
5. **001-07.** Barre Unsaved changes non collante, sous tout le groupe Sound.
6. **005-09.** Texte de la notice IA : la maquette dit « …used for insertion », alors que le test AC-005 interdit « insert ».
7. **005-10.** Toast quand le style d'overlay est `none`.
8. **006-04.** Monogramme à la place de l'icône d'application.
9. **009-03.** « Frontmost app at the end » dans le menu épinglé.
10. **009-04.** Indication « This dictation only » sur toutes les applications non épinglées.

À confirmer en passant, sans être compté comme décision : 007-04 (garder la casse Handy, comme pour 004-04).
