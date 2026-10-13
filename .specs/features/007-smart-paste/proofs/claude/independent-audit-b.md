# Audit indépendant B — 007 Smart Paste, étapes 4 à 6 et réglage

- Auditeur : Claude (Opus 5.5), lecture seule, n'a pas écrit ce code.
- Date : 2026-10-13. Branche `feature/007-smart-paste`, HEAD `a65181b`, base `git merge-base HEAD dev` = `967eb38`.
- Commits audités : `de3d766` (étape 4), `005b61c` (étape 6), `4c5b365` (étape 5), `f9dd082` (réglage `smart_paste`). Les étapes 1 à 3 ne sont regardées qu'à leurs interfaces.
- Reproductions et sorties brutes : `/private/tmp/claude-501/-Users-julienm-projects-handy/6c63dd2f-66f1-4748-8d50-174f39c2881a/scratchpad/repro-007b/`.

## Verdict : FAIL

Il y a 1 constat majeur. Il est facile à corriger. Le noyau est solide sur les points demandés :

- un seul fil d'exécution et un seul résultat final par dictée ;
- pas de restauration intermédiaire ;
- un instantané abandonné après « not read » ne revient jamais ;
- toutes les écritures sont gardées par `changeCount` ;
- aucun contenu n'apparaît dans les logs ;
- l'instantané est borné à 200 ms et 50 Mo ;
- la restauration est identique octet pour octet, ce que j'ai reproduit nativement.

## Constats

### MAJOR-1 — La restauration republie le contenu sauvegardé vers les autres appareils (Universal Clipboard) et neutralise le nettoyage automatique des gestionnaires de mots de passe

- **Où** : `src-tauri/src/smart_paste/macos_pasteboard.rs:192-217` (`write_saved_items` : `clearContents` puis `writeObjects`) et `:272-283` (`restore_snapshot`).
- **Scénario** :
  1. L'utilisateur copie un mot de passe depuis un gestionnaire. Celui-ci publie en `currentHostOnly` avec `org.nspasteboard.ConcealedType` et programme un effacement guardé par `changeCount`.
  2. Il dicte dans un champ où la voie AX échoue, ce qui fait passer par le presse-papiers.
  3. Handy sauvegarde le secret, publie la transcription, puis le restaure avec `clearContents`, sans `currentHostOnly`.
- **Effets** :
  - la restauration est une nouvelle copie locale, donc Universal Clipboard la diffuse aux appareils voisins, alors que l'émetteur d'origine l'avait interdit ;
  - `changeCount` a changé, donc le gestionnaire considère que l'utilisateur a recopié et n'efface plus le secret.
- **Limite de la preuve** : AppKit n'expose pas l'option d'origine. Le constat repose sur la sémantique documentée de `NSPasteboardContentsOptions` et sur le code. La diffusion réelle vers un second appareil n'a pas été observée (aucun appareil disponible).
- **Correctifs proposés** :
  - restaurer avec `prepareForNewContentsWithOptions(.currentHostOnly)` au lieu de `clearContents`. Les types et les octets restent inchangés, AC-009 est donc respecté. Le contenu d'origine, s'il était éligible, a déjà été synchronisé lors de sa copie. Ajouter un test natif qui vérifie l'option sur un presse-papiers nommé ;
  - décision produit à demander à Julien : quand l'instantané contient `ConcealedType` ou `TransientType`, faut-il restaurer, ou effacer le presse-papiers à la place ?

### MINOR-1 — Un type dérivé lent fait perdre les vrais types des items suivants (UTF-16, promesses)

- **Où** : `macos_snapshot_stream.rs:352-367`. Les types sont lus dans l'ordre de l'inventaire, item par item.
- **Scénario reproduit** (`repro-007b/derived/owner2.swift`, `decode.py`, avec le vrai binaire helper de 007) :
  - **Contenu** : item 1 = texte + RTF ; item 2 = PNG en données immédiates.
  - **Si le propriétaire est occupé** (fil principal bloqué) : la conversion dérivée `public.utf16-external-plain-text` de l'item 1 bloque le helper jusqu'à l'échéance. Le PNG de l'item 2, qui ne dépend pas du propriétaire, est perdu.
  - **Si le propriétaire est au repos** : les 4 types passent, en 149 ms.
  - **Test natif de l'implémenteur** : 167 ms puis 187 ms pour quelques centaines d'octets, donc près de la borne de 200 ms.
- **Correctif** : lire d'abord, sur tous les items, les types déclarés ou peu coûteux. Lire les types dérivés (`public.utf16-external-plain-text`, `NSStringPboardType`, `dyn.*`) en dernier, ou ne pas les sauvegarder du tout, puisque AppKit les re-dérive à partir du type de base restauré.

### MINOR-2 — Les types ignorés ne sont jamais journalisés (cas limite de la spec)

- **Où** : `snapshot.rs:159-162`. `skipped()` n'a aucun appelant en production ; voir `coordinator.rs:314-319`.
- **Exigence non tenue** : la spec demande « listed in the debug log by type name only ». Une restauration partielle passe donc inaperçue.
- **Correctif** : à la capture, émettre un `log::debug!` par type ignoré (UTI et cause) et le nombre de types sauvegardés.

### MINOR-3 — Un presse-papiers de plus de 10 000 items fait échouer tout le smart paste

- **Où** : `snapshot_protocol.rs:28` (`MAX_ITEMS`), `:220` ; `macos_snapshot_stream.rs:331-339` (aucune troncature côté helper).
- **Scénario** : une copie Finder de plus de 10 000 fichiers. `Begin` est rejeté comme `Malformed`, donc `insertion_failed` et aucun collage.
- **Correctif** : côté helper, tronquer l'inventaire à la limite et marquer le surplus comme ignoré (`Oversize`) au lieu de produire un flux invalide. Ajouter un test.

### MINOR-4 — Disposition non restaurée en cas de panique

- **Où** : `coordinator_chord.rs:16-32` et `:87-94` ; `keyboard.rs:106-142`.
- **Constat** : la restauration est un appel explicite, sans garde RAII. Une panique entre `switch_for_paste` et `restore_layout` (dans un port ou une évolution future) laisse ABC active et tue le fil de l'acteur.
- **Correctif** : un garde `LayoutRestoreGuard` dont le `Drop` resélectionne `previous`, désarmé après la restauration normale. Ajouter un test avec un port qui panique.

### MINOR-5 — Fil de l'acteur : fuite et arrêt non préemptif

- **Où** : `actor.rs:44-60`, `:92` (pas de `Drop`), `:136-137` et `:168-171`.
- **Pas de fermeture du canal** : le `ReadSink` contient un clone du `Sender` et appartient aux ports, donc au fil lui-même. Le canal ne se ferme jamais : `Next::Disconnected` est inatteignable, contrairement au commentaire. Abandonner `SmartPasteActor` sans `shutdown` laisse fuir le fil sans règlement.
- **Arrêt différé** : `Shutdown` est mis en attente pendant une insertion. Une sortie qui attend moins que la durée maximale d'une insertion se fait sans nettoyage. Cette durée peut atteindre environ 3,6 s pour Jump : AX + instantané + `paste_delay` + 50 + 100 ms + 3 s.
- **Correctifs** :
  - `impl Drop` qui envoie `Shutdown` ;
  - documenter le délai minimal d'arrêt pour l'étape 7, ou permettre à `Shutdown` d'interrompre l'attente du premier read ;
  - corriger le commentaire.

### MINOR-6 — `bindings.ts` ne correspond pas à la sortie de specta

- **Où** : `src/bindings.ts:1099` et `settings.rs:157-162`.
- **Constat** : `progress.md` affirme que la ligne « matches specta output ». Or specta émet un JSDoc par variante documentée ; c'est observable sur `ShortcutActivation`, `ChineseScript` et `EngineType` dans le même fichier. La doc `///` de `SmartPaste` sera donc insérée au prochain `tauri dev`, qui produira un diff. L'équivalence est de type, pas de texte.
- **Correctif** : passer ce commentaire en `//`, ou régénérer via l'export specta.

### MINOR-7 — Tests : plusieurs AC sont prouvés sur des faux plus favorables que le natif

- **Lectures répétées** : `tests/smart_paste/coordinator.rs:56-66` (« probe then read ») suppose un rappel par lecture. Le cache AppKit n'en donne qu'un (voir `evidence/native-pasteboard-adapter.json`, `second_external_read_events: 0`). Le test prouve la logique, pas l'AC. Il faut le dire dans le test et le relier au risque connu R-1.
- **Chemins non testés** :
  - l'arrêt du helper au milieu du flux (`snapshot_helper.rs:313-317`) ;
  - `Collector` avec un double `Begin` ou un `End` sans `Begin` ;
  - la lecture de la touche V après la sélection de la disposition (ordre).
- **Test sensible au temps** : `actor.rs:116-131` (`shutdown_before_the_quiet_period…`) échoue si plus de 300 ms séparent le résultat de `shutdown` sur une machine chargée.
- **Reste inutile** : `chain.rs:205` (`let _ = rich_items();`).

### MINOR-8 — La preuve native peut viser le binaire d'un autre worktree

- **Où** : `tests/smart_paste_native_pasteboard/support.rs:163-166` (`CARGO_BIN_EXE_handy` dans un `CARGO_TARGET_DIR` partagé).
- **Observé pendant l'audit** : `.shared-target/debug/handy` a été reconstruit à 18:13 par un autre worktree, sans `--smart-paste-snapshot`. Le binaire a alors répondu `error: unexpected argument '--smart-paste-snapshot'`.
- **Risque** : les reçus ne hachent que les sources, pas le binaire helper utilisé.
- **Correctif** : ajouter au reçu le SHA-256 du binaire helper, et vérifier son en-tête `HSPS` avant de lancer le scénario.

### MINOR-9 — Démarrage du helper inclus dans les 200 ms

- **Où** : `snapshot_helper.rs:200-215`.
- **Mesures** (`repro-007b/startup.py`, 30 lancements) :

  | Condition | Médiane | Max |
  |---|---|---|
  | Charge moyenne 9,6 | 7,8 ms | 138 ms |
  | Charge forte (builds parallèles) | — | 180 ms |

- **Effet sous charge forte** : en-tête reçu à 180 ms, aucun type reçu avant l'échéance. Le résultat est `NothingSaved`, donc `insertion_failed` sans collage.
- **Statut** : déjà listé en « pending release evidence ». Je l'ajoute avec ces mesures, à confirmer en release.

### NIT

1. `snapshot_helper.rs:313-317` : un helper mort après des types complets rejette tout (`UnexpectedEof`), alors que l'échéance accepte un instantané partiel. C'est conservateur mais incohérent.
2. `snapshot_helper.rs:207-208` : si `stdout.take()` échoue, l'enfant n'est ni tué ni récupéré. C'est théorique.
3. `macos_pasteboard.rs:196,212-216` : si `writeObjects` échoue après `clearContents`, le presse-papiers reste vide (ni ancien contenu, ni transcription).
4. `coordinator_chord.rs:96-105` et `settlement.rs:310-326` : si la destination est perdue, ou si une écriture AX tardive arrive avant la frappe, la transcription est matérialisée et l'ancien contenu perdu, alors qu'aucune frappe n'a eu lieu et qu'une restauration serait sûre. Le plan le prévoit ; question de conception pour Julien.
5. `timing.rs:10` : vérifier la variante hors App Store de Jump Desktop (je crois `com.p5sys.jump.mac.viewer.web`), ainsi qu'une éventuelle variante bêta de Windows App.
6. `shortcut/mod.rs:941-951` : `change_paste_method_setting("smart_paste")` enregistre `ctrl_v` sans erreur visible. Le comportement existe déjà ; 008 le remplace par une liste explicite, qui exclut encore `smart_paste`.
7. `clipboard.rs:839-845` : le refus de `SmartPaste` sort avant « copy to clipboard ». La transcription ne reste alors que dans l'historique. Ce chemin est inatteignable depuis l'UI.
8. `macos_pasteboard.rs:285-297` : la matérialisation à la sortie, après « not read » ou après une annulation, publie la transcription sans `currentHostOnly`. Le plan le prévoit pour le mode copie, pas explicitement pour ces cas.
9. `Cargo.toml:195-204` : les cibles sans harness ignorent les filtres et `--list`, ce qui les rend incompatibles avec nextest et la découverte par l'IDE. `cargo test -- --ignored` lance aussi les vérifications natives.
10. Store : une version plus ancienne qui lit `smart_paste` récupère les autres champs (`salvage_settings`) et réécrit `paste_method` à sa valeur par défaut. C'est acceptable ; il faut le documenter.

## Risques connus, non comptés (déjà suivis comme bloquants de release)

- **R-1, cache AppKit.** Natif : 0 rappel au deuxième lecteur. Les lectures après la première ne sont pas observables, donc :
  - le délai calme se mesure depuis la première lecture ;
  - une vraie lecture plus de 300 ms après la sonde recevrait l'ancien contenu ;
  - une lecture avant la frappe par un gestionnaire de presse-papiers produit un `not_read` à tort : l'instantané est jeté et l'ancien contenu perdu.

  La décision (1) de Julien n'est donc pas garantie nativement. L'implémentation le documente. Le plan en fait un bloquant de release ; il ne bloque pas l'implémentation.
- **R-2.** Une frappe retardée d'une dictée précédente peut lire la publication suivante (plan, « overlapping »).

## Vérifications par point demandé

| Point | Résultat |
|---|---|
| 1. Instantané | **Borne et contenu** : borne 200 ms / 50 Mo correcte. Instantané multi-items exact octet pour octet (natif, rejoué). Vide distingué de « rien sauvé » : `NothingSaved` refuse le démarrage.<br>**Sécurité du helper** : lancé sans shell ; argv = nonce, budget, nom optionnel ; aucune injection possible. Le lancer à la main équivaut à `pbpaste`, donc pas d'élévation. Tué et récupéré à l'échéance ; lecteur joint. Mort ou flux invalide donnent `insertion_failed`, presse-papiers intact.<br>**Restauration** : constats MAJOR-1 et MINOR-1/2/3. |
| 2. Fournisseur | Marqueurs immédiats, `currentHostOnly` et `Transient`/`Concealed`/`AutoGenerated` présents (natif). Seule une demande `public.utf8-plain-text` compte comme lecture. Toute la transcription est servie à chaque demande. Cache AppKit : R-1. |
| 3. Coordinateur | **Exécution** : un seul fil ; un seul `PasteOutcome` par `insert`. Les lectures d'items remplacés sont ignorées.<br>**Succession** : `CarryPendingSnapshot` / `PendingTranscript` / `CaptureCurrent` conformes. Instantané rejeté après « not read ». Diagnostic de lecture tardive émis une fois.<br>**Logs** : contenu-free (`Transcript` et `SavedType` masqués, erreurs en enums), plus un test de sentinelles.<br>**Restes** : fuite et arrêt (MINOR-5). |
| 4. Stratégie | Identifiants de bundle exacts, jamais le nom de l'app : iPhone Mirroring et Windows App → frappe Unicode (permission vérifiée) ; Partage d'écran → remise sans frappe ni permission, sans restauration ; Jump → 1,5 s / 3 s, testé en 1600 = 100 + 1500 et 3000. |
| 5. Clavier | ABC puis US, attente 50 ms, restauration 100 ms après le relâchement de Cmd, même si l'accord échoue (testé). Les sélections main-thread abandonnées ne s'exécutent jamais tard. Cmd+V : keycode 9 en AZERTY (natif, lecture seule). Ni ABC ni US : source inchangée, accord avec la touche V de la disposition courante (testé). Panique : MINOR-4. |
| 6. Réglage | `SmartPaste` n'est le défaut nulle part (testé). Le pipeline le refuse explicitement, sans le repli à délai fixe. Le store est rétrocompatible par récupération (salvage). `bindings.ts` : MINOR-6. |
| 7. Robustesse | Aucun `unwrap`/`expect`/`panic` en production dans `smart_paste/`. FFI relue (CF Create/Get, TIS, CGEvent, GCD) : correcte. Cibles sans harness : sans effet sous `cargo test` (opt-in `--ignored`, `fn main(){}` hors macOS ; la CI Linux lance `cargo test`). |
| 8. Tests | Couverture AC-008 à AC-013 correcte en logique ; limites décrites en MINOR-7. |

## Tests exécutés

`CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`

| Commande | Résultat |
|---|---|
| `cargo test --lib --test smart_paste` | lib 319 réussis, 0 échec, 16 ignorés ; smart_paste 86 réussis, 0 échec |
| `cargo test --test smart_paste_native_pasteboard --test smart_paste_native_keyboard` (sans opt-in) | « 1 ignored » chacun, exit 0, aucun effet |
| `cargo test --test smart_paste_native_keyboard -- --ignored` | PASS : French, `TryAbcThenUs`, keycode 9 |
| `cargo test --test smart_paste_native_pasteboard -- --ignored` (presse-papiers nommé) | PASS : aller-retour riche exact, copie plus récente intacte, vide, budget, promesse lente coupée à 205 ms |
| `cargo fmt --check` | exit 0 |
| `bun run lint` | exit 0 |
| `tsc --noEmit` | exit 0 |
| `livespec conventions verify --feature repo --json` | PASS, 0 bloquant, 0 nouveau ; 11 warnings sur smart paste (longueur de fonctions, documentation) |

- **Non exécuté**, comme demandé : la cible native AX et le spike AppKit (étapes 1 à 3), Codex, `audit`, et tout build release.
- **Effets de bord** :
  - la commande conventions a créé `.specs/conventions/runs/20261013T160900Z/` ;
  - 2 presse-papiers nommés de test (`com.handy.audit007b.*`), créés par la première sonde `owner.swift`, n'ont pas été libérés ;
  - aucun presse-papiers général, aucune touche, aucune disposition système n'ont été touchés.
