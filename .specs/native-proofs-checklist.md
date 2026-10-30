# Séance de preuves natives — checklist

Séance finale de Julien sur son Mac (Apple M4 Pro). Selon la constitution (`.specs/constitution.md`, « Native platform evidence »), **seules les preuves macOS bloquent** le statut Implemented. Les preuves Windows, Linux, « ancien binaire » ou « second compte » sont des preuves de release en attente : elles ne sont jamais déclarées réussies (dernière section).

Pour chaque preuve, ranger dans `.specs/features/<feature>/proofs/native-macos-<sujet>/` :
- un `notes.md` avec la date, la version de macOS, le commit (`git rev-parse HEAD`), l'état des verrous, le `shasum -a 256` du binaire et le résultat observé ;
- les captures, les sorties CLI et l'extrait de log utile.

Noms de dossiers en kebab-case ASCII. Aucune donnée privée : texte dicté de test, pas de mot de passe réel en clair.

## 1. Préparation

### 1.1 Quitter Handy et sauvegarder les réglages

Le Handy installé (pré-008) doit être **quitté** pendant toute la séance, pour deux raisons :
- une build de test qui porte le même identifiant `com.pais.handy` le rejoindrait par l'instance unique ;
- la CLI refuse toute écriture (`unavailable`, exit 5) tant qu'un Handy pré-008 tourne.

Le store est toujours `~/Library/Application Support/com.pais.handy/settings_store.json` : le chemin est codé en dur dans `src-tauri/src/settings_service/locator.rs`, même avec un identifiant Tauri surchargé. La sauvegarde est donc obligatoire.

```bash
osascript -e 'tell application id "com.pais.handy" to quit' || true
D="$HOME/Library/Application Support/com.pais.handy"
B="$HOME/handy-tests/backup-$(date +%Y%m%d-%H%M)"
mkdir -p "$B"
rsync -a --exclude models/ "$D/" "$B/data/"          # settings_store.json, history.db, recordings/, verrous
cp -a "$HOME/Library/Logs/com.pais.handy" "$B/logs"
ditto /Applications/Handy.app "$B/handy-installed.app"
shasum -a 256 "$D/settings_store.json" "$D/history.db" > "$B/hashes.txt"
```

### 1.2 Construire les builds de test

Depuis `/Users/julienm/projects/handy`, sur `dev` (commit noté), en suivant `AGENTS.md` et `BUILD.md` :

```bash
bun install
mkdir -p src-tauri/resources/models
[ -f src-tauri/resources/models/silero_vad_v4.onnx ] || curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx
export CMAKE_POLICY_VERSION_MINIMUM=3.5
mkdir -p ~/handy-tests
# Verrous fermés (état de dev)
bun run tauri build --bundles app
ditto src-tauri/target/release/bundle/macos/Handy.app ~/handy-tests/handy-gates-closed.app
# Verrous ouverts
bun run tauri build --bundles app --features destination-return,smart-paste-dispatch
ditto src-tauri/target/release/bundle/macos/Handy.app ~/handy-tests/handy-gates-open.app
```

Les procédures 006 et 007 demandent aussi un verrou à la fois, avant la passe combinée :
- `--features destination-return` → `handy-destination-only.app` (procédure A de la 006) ;
- `--features smart-paste-dispatch` → `handy-smart-paste-only.app` (matrice AC-016 de la 007).

Pour les contrôles qui demandent le Web Inspector (clic droit › Inspecter), utiliser le mode dev :
- `bun run tauri dev -- -- --debug` ;
- `bun run tauri dev --features destination-return,smart-paste-dispatch -- -- --debug`.

En mode dev, la CLI est `src-tauri/target/debug/handy`.

### 1.3 Lancer une build et utiliser la CLI

```bash
APP=~/handy-tests/handy-gates-closed.app          # ou gates-open, destination-only, smart-paste-only
alias handy="$APP/Contents/MacOS/handy"
shasum -a 256 "$APP/Contents/MacOS/handy"
"$APP/Contents/MacOS/handy" --debug &              # ou: open "$APP"
handy settings set log_level debug --json
handy settings get --json | head
```

- Ne lancer qu'une seule build à la fois. Quitter par le menu du tray avant d'en changer.
- Les builds locales ont une signature ad hoc : macOS peut garder une autorisation Accessibilité périmée. Dans ce cas : `tccutil reset Accessibility com.pais.handy`, puis accorder de nouveau Accessibilité et Micro. Le Handy installé devra aussi être réautorisé à la fin.
- **Logs :** `~/Library/Logs/com.pais.handy/handy.log` (niveau debug avec `--debug` ou `log_level debug`). Ils contiennent des timings et des types, jamais le texte dicté.
- **Données :** `~/Library/Application Support/com.pais.handy/`, avec `settings_store.json`, `settings.lock`, `settings.instance.lock`, `history.db`, `recordings/` et `models/`.
- **Endpoint CLI :** `$(getconf DARWIN_USER_TEMP_DIR)hs-$(id -u)/`.

**Ordre conseillé :**
1. builds fermées : 001–005, 008, puis les contrôles « verrou fermé » de 006 et 007 ;
2. `destination-only` : 006 ;
3. `smart-paste-only` : 007 ;
4. `gates-open` : passe combinée et indicateur ;
5. restauration.

### 1.4 Restaurer à la fin (obligatoire)

Il faut restaurer, pour deux raisons :
- les builds de test migrent le store : `microphone_priority`, révision, et `paste_method=smart_paste` sur une build ouverte. Une build fermée ou l'ancien Handy échouerait alors à chaque dictée ;
- elles migrent aussi `history.db` : l'ancien Handy s'arrêterait sur `DatabaseTooFarAhead`.

```bash
osascript -e 'tell application id "com.pais.handy" to quit' || true; pkill -x handy || true
rsync -a --delete --exclude models/ "$B/data/" "$D/"
shasum -a 256 -c "$B/hashes.txt"
open /Applications/Handy.app
```

Réautoriser ensuite Accessibilité et Micro au Handy installé si macOS les a perdus.

## 2. Preuves macOS par feature

### 001 — Priorité des micros (`handy-gates-closed.app`, sauf mention)

- [ ] **Harnais natif.**
  - Étapes : lancer `bun scripts/microphone-native-proof.ts --platform macos --out .specs/features/001-microphone-priority/proofs/native-macos-harness --binary ~/handy-tests/handy-gates-closed.app/Contents/MacOS/handy --interactive`, depuis un terminal qui a l'accès Micro, et débrancher le micro par défaut quand on le demande.
  - Attendu : exit 0, chaque tentative ≤ 2 s, résolution ≤ 6 s, flux tardif libéré, boucle principale jamais gelée.
  - Couvre : AC-003, AC-004, FR-003. Preuve : `proofs/native-macos-harness/`.
- [x] **Repli et reconnexion.** _(2026-10-28, validé)_
  - Étapes : ordre A (USB ou AirPods) puis B (MacBook), Save, redémarrer Handy. Relire l'UI et `handy settings microphones list --json`. Débrancher A et dicter. Rebrancher A et dicter.
  - Attendu : ordre identique après redémarrage ; dictée sur B avec la raison « fallback », puis de nouveau sur A ; ordre et clamshell jamais modifiés.
  - Couvre : AC-001, AC-002, AC-004, SC-002. Preuve : `proofs/native-macos-fallback/`.
- [x] **Perte en cours de dictée.** _(2026-10-28, validé ; voir native-session-findings N1)_
  - Étapes : parler, puis débrancher A au milieu de la phrase.
  - Attendu : la dictée s'arrête une seule fois, le début est transcrit, un seul toast « input lost » apparaît, et le relâchement de la touche n'a aucun effet.
  - Couvre : AC-004, FR-004. Preuve : `proofs/native-macos-loss/`.
- [ ] **Mode always-on.** _(2026-10-28 : non utilisé par Julien, reporté)_
  - Étapes : changer la priorité hors dictée ; la changer pendant une dictée ; rebrancher A puis lancer une dictée.
  - Attendu : hors dictée, le flux se rouvre ; pendant une dictée, rien n'est interrompu et la réouverture a lieu à la fin ; avec A rebranché, le flux se rouvre sur A avant la capture. Relever les latences par démarrage.
  - Couvre : AC-003, AC-004. Preuve : `proofs/native-macos-always-on/`.
- [ ] **Couvercle fermé.** _(2026-10-28 : reporté, pas de montage écran externe)_
  - Étapes : écran externe et capot réellement fermé, avec un override clamshell, puis sans override.
  - Attendu : avec override, le micro clamshell est utilisé (repli sur le défaut s'il est absent) ; sans override, le défaut système ; aucun réglage modifié.
  - Couvre : AC-005, SC-002. Preuve : `proofs/native-macos-clamshell/`.
- [ ] **CLI live.**
  - Étapes : lancer `handy settings microphones list --json` app lancée, puis app quittée.
  - Attendu : app lancée, la commande renvoie l'ordre, la disponibilité, le micro effectif et sa raison ; app quittée, `unavailable`, exit 5, `details` vide.
  - Couvre : AC-008. Preuve : `proofs/native-macos-cli/`.
- [x] **Éditeur en vraie fenêtre (mode dev).** _(2026-10-28 : clavier et glisser validés ; barre hors écran → N2)_
  - Étapes : comparer les 3 états aux maquettes `.specs/design/screens/001-microphone-priority/`. Glisser une ligne à la souris dans la webview. Au clavier seul : ajouter, retirer, monter, descendre, Save, avec le focus jamais perdu. Provoquer une écriture refusée.
  - Attendu : rendu conforme aux maquettes et parcours clavier complet.
  - Couvre : AC-001, SC-003. Preuve : `proofs/native-macos-ui/`.
- [x] **Conflit.** _(2026-10-28, validé)_
  - Étapes : ouvrir un brouillon, puis lancer `handy settings microphones set-priority "MacBook Pro Microphone"`.
  - Attendu : bannière « changed elsewhere » avec Save inactif (décision §3), puis Reload affiche l'ordre engagé et jette le brouillon.
  - Couvre : AC-007. Preuve : `proofs/native-macos-ui/`.
- [ ] **Pause du sondage.**
  - Étapes : fermer la fenêtre vers le tray, puis la rouvrir depuis le tray, depuis le Dock et par un second lancement. Minimiser.
  - Attendu : fenêtre cachée, `document.visibilityState` change dans le Web Inspector et les appels `get_microphone_status` cessent ; à la réouverture, une lecture immédiate puis une toutes les 5 s ; minimisée, le sondage est en pause.
  - Couvre : AC-002. Preuve : `proofs/native-macos-polling/`.
- [x] **Barre « Unsaved changes ».** _(2026-10-28 : hors écran → N2, décision : barre compacte collante)_
  - Étapes : dans une fenêtre de 680×570, modifier l'ordre.
  - Attendu : la barre est visible sans défiler. Sinon, décision : la placer après la ligne closed-lid, ou la rendre sticky.
  - Couvre : AC-001. Preuve : `proofs/native-macos-ui/`.

### 002 — Remplacements de mots (`handy-gates-closed.app` ou mode dev)

- [x] **Éditeur comparé aux 4 maquettes** _(2026-10-28, Computer Use : liste, édition, suppression, garde, Texte ; focus → N5)_ (`.specs/design/screens/002-word-replacements/`, écrans `replacement-settings`, `-empty`, `-bulk`, `-import`).
  - Étapes : passer par la liste, l'ajout et la modification sur la ligne, la suppression avec confirmation, la barre Discard/Save, l'avertissement en quittant la section, le bouton List/Text et le bouton Import… (qui exigent Save ou Discard).
  - Attendu : chaque écran et chaque état sont conformes aux maquettes.
  - Couvre : AC-001, `has_dirty_state`. Preuve : `proofs/native-macos-ui/`.
- [x] **Dialogues natifs.** _(2026-10-28, Computer Use ; alerte mal placée → N6)_
  - Étapes : Import… d'un fichier, Export des règles, puis Import d'un fichier illisible.
  - Attendu : le dialogue système s'ouvre et le fichier est lu ou écrit ; le fichier illisible affiche l'alerte « Couldn't read ».
  - Couvre : FR-003, AC-005. Preuve : `proofs/native-macos-files/`.
- [ ] **Aller-retour par l'UI.**
  - Étapes : export, puis import en mode replace, pour 1, 100 et 1000 règles.
  - Attendu : identifiants, contenu et ordre identiques.
  - Couvre : SC-002, AC-004, AC-005. Preuve : `proofs/native-macos-files/`.
- [ ] **Redémarrage.**
  - Étapes : Save depuis l'UI, quitter, relancer.
  - Attendu : mêmes règles, même ordre.
  - Couvre : AC-001. Preuve : `proofs/native-macos-ui/`.
- [x] **Dictée réelle.** _(2026-10-28 : remplacement appliqué ; modèle batch Parakeet seulement)_
  - Étapes : `handy settings replacements add "cloud code" "Claude Code"`, puis dicter « open cloud code now » avec un modèle batch, puis avec un modèle streaming. Modifier ensuite la règle pendant un enregistrement.
  - Attendu : « open Claude Code now » inséré et dans l'historique ; la dictée en cours garde l'ancienne règle, la suivante prend la nouvelle.
  - Couvre : AC-007, FR-005. Preuve : `proofs/native-macos-dictation/`.
- [ ] **Clavier.**
  - Étapes : parcourir au Tab ; Entrée puis confirmation sur Supprimer.
  - Attendu : anneau de focus visible, y compris sur `Button secondary` ; ordre Tab logique ; après suppression, le focus passe à la règle suivante, ou au formulaire d'ajout.
  - Couvre : `is_keyboard_navigable`. Preuve : `proofs/native-macos-ui/`.

### 003 — Vocabulaire de reconnaissance (`handy-gates-closed.app` ou mode dev)

- [ ] **4 écrans comparés aux maquettes** (`.specs/design/screens/003-recognition-vocabulary/`, écrans `vocabulary-settings`, `-paste`, `-unsupported`, `-empty`).
  - Attendu : écrans conformes, avec les écarts déclarés notés.
  - Couvre : AC-001, AC-002, AC-003. Preuve : `proofs/native-macos-ui/`.
- [ ] **Clavier.**
  - Étapes : Tab à travers le champ, la liste, la boîte de collage puis le statut ; Entrée sur Supprimer ; Apply en échec, puis Retry.
  - Attendu : focus visible partout, y compris un anneau d'Apply bien lisible ; après suppression, le focus passe au terme suivant ou au champ ; il est cohérent après l'Apply en échec et après le Retry.
  - Couvre : `is_keyboard_navigable`. Preuve : `proofs/native-macos-ui/`.
- [ ] **Statut.**
  - Étapes : `handy settings vocabulary status --json` et le statut affiché dans l'UI, avec un modèle chargé, puis sans modèle chargé, puis avec un modèle sans support.
  - Attendu : les termes envoyés et non envoyés sont corrects, avec le bon statut.
  - Couvre : AC-003, AC-004, AC-007. Preuve : `proofs/native-macos-status/`.
  - 2026-10-28 : cas « modèle sans support » (Parakeet) conforme, UI et CLI concordantes ; collage, doublon, erreurs et suppression conformes (`proofs/native-macos-status/observations.md`). Restent : Whisper chargé, aucun modèle chargé.
- [ ] **Effet réel.**
  - Étapes : enregistrer un échantillon contenant « Claude Code », sans vocabulaire, puis avec le terme ajouté. Noter l'id du modèle. Le script prévu `scripts/vocabulary-native-proof.ts` n'existe pas, donc la preuve est manuelle.
  - Attendu : l'orthographe est correcte avec le vocabulaire.
  - Couvre : SC-003, SC-001, AC-004. Preuve : `proofs/native-macos-recognition/`.
- [ ] **Pas de rechargement.**
  - Étapes : modifier le vocabulaire entre deux dictées.
  - Attendu : aucun rechargement du modèle dans le log, et le terme est pris à la dictée suivante.
  - Couvre : AC-006. Preuve : `proofs/native-macos-recognition/`.

### 004 — Raccourcis de dictée multiples (`handy-gates-closed.app`)

Pour simuler un « raccourci pris par une autre app », l'assigner d'abord dans une autre application, par exemple Raycast ou Réglages › Clavier › Raccourcis.

- [ ] **Matrice avec le backend Tauri.**
  - Étapes : ajouter `ctrl+option+d` à Transcribe, puis redémarrer. En toggle : démarrer avec `option+space`, arrêter avec `ctrl+option+d`. En push-to-talk : maintenir `option+space`, presser puis relâcher `ctrl+option+d`. En hold-or-toggle : la même chose.
  - Attendu : en push-to-talk, l'enregistrement dure jusqu'au relâchement de `option+space`.
  - Couvre : AC-001, AC-003, SC-001. Preuve : `proofs/native-macos-matrix/`.
- [ ] **Même matrice avec le backend HandyKeys** (Accessibilité accordée).
  - En plus : un raccourci modificateur seul (`option`), et `option+space` refusé sur une autre action.
  - Couvre : AC-003, AC-004. Preuve : `proofs/native-macos-matrix/`.
- [ ] **Refus de l'OS**, avec le backend Tauri.
  - Étapes : ajouter des raccourcis pris par une autre app. Transcribe `ctrl+option+d`. « Transcribe with Post-Processing » `ctrl+option+p`, post-processing désactivé. Cancel `ctrl+option+x`, hors dictée. Puis lancer `handy settings shortcuts add transcribe ctrl+option+x --json`.
  - Attendu : l'UI affiche la raison système et la liste stockée ne change pas ; la CLI renvoie `validation_failed` / `registration_refused` et la raison est dans le log.
  - Couvre : AC-004, AC-005, AC-006. Preuve : `proofs/native-macos-os-refusal/`.
- [ ] **Secure Input.**
  - Étapes : backend HandyKeys avec 2 raccourcis, focus dans un champ mot de passe ; presser chaque raccourci, puis Add.
  - Attendu : les deux raccourcis déclenchent la dictée ; relâcher le raccourci de départ termine une session maintenue.
  - Couvre : FR-002. Preuve : `proofs/native-macos-secure-input/`.
- [ ] **Conflit hérité.**
  - Étapes : Handy quitté, régler dans `settings_store.json` `transcribe` = `alt+space` et `transcribe_with_post_process` = `option+space`, post-processing activé, puis redémarrer. Changer Transcribe en `f5`. Désactiver ensuite le post-processing.
  - Attendu : `option+space` lance toujours une dictée post-traitée, sans redémarrage ; après la désactivation, `alt+space` lance Transcribe.
  - Couvre : AC-002, SC-003. Preuve : `proofs/native-macos-legacy-conflict/`.
- [ ] **Activation tardive refusée.**
  - Étapes : raccourci post-processing pris par une autre app, puis activer le post-processing. Remplacer ensuite le raccourci par une combinaison libre.
  - Attendu : le toggle reste activé, y compris après redémarrage, avec un toast qui nomme le raccourci et un warning sur sa ligne ; la combinaison libre efface le warning.
  - Couvre : AC-002, AC-004. Preuve : `proofs/native-macos-os-refusal/`.
- [ ] **CLI live, backend Tauri puis HandyKeys.**
  - Étapes : `handy settings shortcuts add transcribe ctrl+option+d --json`, puis presser ce raccourci ; `remove` ; `handy settings set keyboard_implementation handy_keys --json`.
  - Attendu : `ok` avec la nouvelle révision, et la dictée démarre sans redémarrage ; après `remove`, le raccourci est inerte ; après le changement de backend, les deux raccourcis marchent encore.
  - Couvre : AC-006, FR-005. Preuve : `proofs/native-macos-cli/`.
  - 2026-10-28 : ajout à chaud avec HandyKeys conforme (révision 22, enregistré sans redémarrage). Restent : presser le raccourci, `remove`, backend Tauri.
- [ ] **Session occupée.**
  - Étapes : maintenir le raccourci push-to-talk, puis lancer `handy settings shortcuts add transcribe f13 --json` et basculer le post-processing dans les réglages.
  - Attendu : `unavailable` / `shortcut_session_busy`, store inchangé, et le relâchement arrête la dictée.
  - Couvre : FR-002, AC-006. Preuve : `proofs/native-macos-cli/`.
  - 2026-10-28 : refus CLI `shortcut_session_busy` observé pendant une dictée toggle, store inchangé. Restent : le cas push-to-talk maintenu et la bascule du post-processing dans l'UI.
- [ ] **Fenêtre de réglages.**
  - Étapes : comparer les états `shortcut-settings`, `-conflict` et `-limit` aux maquettes (`.specs/design/screens/004-multiple-dictation-shortcuts/`). Faire Tab pendant une capture HandyKeys. Provoquer un refus de l'OS au démarrage du recorder HandyKeys, puis cliquer Change.
  - Attendu : rendu conforme ; Tab pendant la capture rend le focus de la puce ; le brouillon n'est pas bloqué en enregistrement, Discard et Retry restent disponibles, et la demande Change ne se relance pas toute seule.
  - Couvre : AC-001, Behavioral AC. Preuve : `proofs/native-macos-ui/`.

### 005 — Prompts IA personnalisés (`handy-gates-closed.app` ; indicateur aussi avec `handy-gates-open.app`)

- [ ] **Pastille de prompt dans le panneau non activant.**
  - Étapes : IA activée, curseur dans TextEdit. Dicter, survoler la pastille au-dessus de l'indicateur, puis choisir ⌘2. Refaire avec 51 prompts. Tester en bord d'écran, sur un second écran et en Retina. Le script prévu `scripts/prompts-native-proof.ts` n'existe pas, donc la preuve est manuelle.
  - Attendu :
    - le menu ⌘1/⌘2… avec la coche s'ouvre en 200 ms ou moins ;
    - TextEdit garde le focus et le curseur ;
    - avec 51 prompts, on défile jusqu'au dernier (8 lignes visibles) ;
    - un clic extérieur ferme le menu, et la sortie du pointeur le ferme après 250 ms ;
    - le choix vaut pour cette dictée seulement, le défaut ne change pas, et un prompt invalide est désactivé.
  - Couvre : AC-003, FR-002. Preuve : `proofs/native-macos-prompt-pill/`.
- [ ] **Échec IA.**
  - Étapes : provoquer une clé invalide, un endpoint coupé, un résultat vide et un dépassement de 30 s. Refaire avec le style d'overlay `none`.
  - Attendu : l'avis reste au moins 4 s, même après le masquage de l'indicateur ; le texte brut est inséré ; l'historique garde le texte brut, le prompt et la raison. Avec `none`, un toast d'au moins 4 s apparaît. Texte de l'avis : décision §3.
  - Couvre : AC-005, FR-004, SC-001. Preuve : `proofs/native-macos-fallback/`.
- [ ] **Raccourcis de prompt.**
  - Étapes : assigner un raccourci, l'utiliser, puis couper l'IA. Supprimer ensuite un prompt qui a des raccourcis pendant une dictée.
  - Attendu : le raccourci s'enregistre, puis se libère quand l'IA est coupée ; la suppression pendant la dictée est refusée avec `shortcut_session_busy`.
  - Couvre : AC-006. Preuve : `proofs/native-macos-shortcuts/`.
- [ ] **Fournisseurs réels.**
  - Étapes : un fournisseur local (Ollama `http://localhost:11434/v1`), un fournisseur distant, et une surcharge fournisseur/modèle sur un prompt. Avec Apple Intelligence : une dictée réelle, deux vérifications simultanées, puis le délai de 30 s.
  - Attendu : le bon endpoint et le bon modèle sont appelés ; aucun faux `provider_unavailable` ; `provider_busy` quand c'est le cas.
  - Couvre : AC-004, FR-003. Preuve : `proofs/native-macos-providers/`.
- [ ] **Réglages en vraie fenêtre.**
  - Étapes : la liste, l'éditeur, la confirmation de suppression, le badge invalide, l'état « non prêt » avec l'élément manquant, et le panneau de test (busy, retry, durée). Saisir une clé API inline, puis changer le fournisseur du prompt.
  - Attendu :
    - le focus va au champ nom à l'ouverture, puis revient au bouton d'édition ;
    - la clé inline garde le brouillon et ne change pas le fournisseur par défaut ;
    - le champ de clé ne garde pas l'ancien texte après le changement de fournisseur.
  - Couvre : AC-001, AC-002, AC-004, AC-007. Preuve : `proofs/native-macos-ui/`.

### 006 — Application de destination (`handy-destination-only.app`, puis `handy-gates-open.app`)

- [ ] **Verrou fermé** (`handy-gates-closed.app`).
  - Étapes : ouvrir les réglages de sortie, puis lancer `handy settings output set return_to_dictation_app true --json`.
  - Attendu : la ligne « Return to the dictation's application » est absente ; la CLI renvoie `validation_failed`, exit 2, `details.reason = "unsupported_platform"` ; l'insertion se fait comme sur dev.
  - Couvre : AC-007. Preuve : `proofs/native-macos-gate-closed/`.
- [ ] **Retour à l'application.**
  - Étapes : `handy settings output set return_to_dictation_app true --json` avec `paste_method` `ctrl_v`. Dicter dans TextEdit, faire Cmd+Tab vers Notes, puis arrêter.
  - Attendu : TextEdit revient en moins d'1 s et reçoit le texte ; Notes ne reçoit rien ; le log contient `Insertion finished as InsertionAttempted`.
  - Couvre : AC-002, FR-002. Preuve : `proofs/native-macos/`.
- [ ] **Quatre applications et plus.**
  - Étapes : TextEdit, Notes, ChatGPT et Comet comme destinations, en changeant d'app pendant la dictée. Puis le scénario ChatGPT → Comet par le menu du badge.
  - Attendu : un résultat par application.
  - Couvre : SC-001, SC-003. Preuve : `proofs/native-macos/`.
- [ ] **Destination fermée.**
  - Étapes : quitter TextEdit avant l'arrêt.
  - Attendu : rien n'est inséré, un toast d'erreur apparaît, et le log contient `Transcription not inserted (Some(AppClosed))`.
  - Couvre : AC-005, FR-004. Preuve : `proofs/native-macos/`.
- [ ] **Garde pendant le délai de collage.**
  - Étapes : `handy settings output set paste_delay_ms 5000 --json`, puis Cmd+Tab vers Notes environ 1,5 s après l'arrêt. Refaire sans changer d'app. Remettre ensuite `paste_delay_ms 60`.
  - Attendu : avec le changement d'app, aucun texte, presse-papiers restauré, et `Some(ActivationFailed)` dans le log ; sans changement d'app, le texte est inséré. Noter si l'UI de Handy gèle pendant le délai, et si l'activation demande le thread principal (macOS 14+) : décision §3.
  - Couvre : AC-002. Preuve : `proofs/native-macos/`.
- [ ] **Indicateur (badge).**
  - Étapes : survoler le badge, ouvrir le menu, utiliser la croix ; écouter avec VoiceOver ; provoquer une raison d'échec.
  - Attendu :
    - le survol est reçu par le panneau non-key sans voler le focus ;
    - le menu se met à jour en 200 ms ou moins ;
    - VoiceOver lit « Destination: <name> » et « Remove destination » ;
    - la raison d'échec reste au moins 4 s ;
    - le badge affiche un monogramme, faute de cache d'icônes.
  - Couvre : AC-003, AC-004, FR-003. Preuve : `proofs/native-macos-indicator/`.
- [ ] **Toggle des réglages, verrou ouvert.**
  - Étapes : capture de la fenêtre, toggle au clavier, VoiceOver.
  - Attendu : la ligne est visible ; la valeur est annoncée ; la raison est affichée si l'app au premier plan est illisible.
  - Couvre : AC-007, `is_keyboard_navigable`. Preuve : `proofs/native-macos-ui/`.

### 007 — Collage intelligent (`handy-smart-paste-only.app`, puis passe combinée)

Suivre la procédure détaillée de `.specs/features/007-smart-paste/implementation.md` (« Native acceptance procedure (AC-016) with Julien »), avec `clipboard_handling` `dont_modify`. Résultats dans `evidence/native-compatibility.md` et `evidence/native-compatibility.json`, logs dans `proofs/native-macos-<sujet>/`.

- [ ] **Verrou fermé** (`handy-gates-closed.app`).
  - Étapes : ouvrir le sélecteur de méthode de collage, puis lancer `handy settings output set paste_method smart_paste --json`.
  - Attendu : smart paste est absent du sélecteur et la CLI le refuse ; la dictée se fait comme sur dev.
  - Couvre : verrou fermé (`implementation.md`, « Rollout gates »). Preuve : `proofs/native-macos-gate-closed/`.
- [ ] **Préparation, verrou ouvert.**
  - Étapes : `handy settings output get paste_method --json`, puis activer les dispositions French AZERTY, ABC (ou U.S.) et Dvorak, et préparer la fixture `fixture-rich.rtfd` (texte stylé et PNG) ainsi qu'une copie de deux fichiers.
  - Attendu : `paste_method` vaut `smart_paste` (migration).
  - Couvre : AC-016.
- [ ] **Matrice AC-016.**
  - Étapes : TextEdit, Notes, ChatGPT, Comet et une app Chromium, croisés avec AZERTY, U.S. et Dvorak, 10 runs par paire. Relever `osascript -e 'clipboard info'` avant, puis 1 s après.
  - Attendu :
    - le texte apparaît une seule fois ;
    - `inserted_verified` au moins 9/10 ;
    - le presse-papiers est restauré au moins 9/10 ;
    - la disposition est restaurée ;
    - **une seule livraison d'un ancien contenu ou d'une ancienne transcription bloque la release.**
  - Couvre : AC-016, FR-010, SC-005, SC-006. Preuve : `evidence/native-compatibility.*`.
- [ ] **Dispositions clavier.**
  - Étapes : French actif avec ABC activé (puis U.S. seul). Refaire avec Japanese Kana.
  - Attendu : ABC (ou U.S.) est actif pendant Cmd+V, et French revient 100 ms après le relâchement, même si le raccourci échoue ; aucun changement avec Kana.
  - Couvre : AC-011, FR-007. Preuve : `proofs/native-macos-layouts/`.
- [ ] **Insertion directe.**
  - Attendu : le texte apparaît exactement une fois dans 100 % des runs, et le presse-papiers n'est pas touché. Une écriture AX appliquée en retard bloque la release.
  - Couvre : SC-007, AC-017, AC-018, FR-011. Preuve : `proofs/native-macos-direct/`.
- [ ] **Applications distantes.**
  - Étapes : iPhone Mirroring et Windows App ; Screen Sharing ; Jump Desktop.
  - Attendu : iPhone Mirroring et Windows App donnent `typed`, presse-papiers intact ; Screen Sharing donne `clipboard_handed_over`, avec le message au moins 4 s sur l'indicateur et aucune frappe ; Jump Desktop : 1,5 s de calme, 3 s de décision. Une app absente est notée « not verified ».
  - Couvre : AC-010, AC-012. Preuve : `proofs/native-macos-remote/`.
- [ ] **Secrets.**
  - Étapes : copier depuis un gestionnaire de mots de passe qui marque Concealed ou Transient, puis dicter.
  - Attendu : le secret n'est pas restauré et la transcription reste dans le presse-papiers.
  - Couvre : décision Julien du 2026-10-13. Preuve : `proofs/native-macos-secrets/`.
- [ ] **Gestionnaire de presse-papiers (R-1).**
  - Étapes : dicter avec un gestionnaire qui respecte `org.nspasteboard`.
  - Attendu : la transcription n'est pas enregistrée par le gestionnaire ; une lecture avant la frappe donne `not_read`. Vérifier aussi si AppKit rappelle le fournisseur.
  - Couvre : AC-009. Preuve : `proofs/native-macos-clipboard-manager/`.
- [ ] **Lectures tardives et chevauchements (R-2).**
  - Étapes : une sonde Chromium puis une lecture, une lecture après le calme, deux dictées rapprochées.
  - Attendu : aucun ancien contenu livré ; après un `not_read`, une lecture tardive reçoit la transcription et émet `smart-paste-late-read`.
  - Couvre : SC-008, SC-004, AC-012. Preuve : `proofs/native-macos-late-reads/`.
- [ ] **Cas limites.**
  - Étapes : quitter Handy moins de 300 ms après une dictée, puis après 1 s ; copier plus de 10 000 fichiers dans le Finder et dicter ; mesurer la première capture du helper sur une app installée ; avec un second appareil Apple, vérifier Universal Clipboard.
  - Attendu :
    - quitter avant 300 ms : la sortie attend et la transcription reste dans le presse-papiers ;
    - quitter après 1 s : le contenu précédent est restauré ;
    - copie Finder : 10 000 éléments restaurés, le reste compté dans le log ;
    - première capture du helper : moins de 200 ms ;
    - Universal Clipboard : rien n'est reçu sur l'autre appareil (`currentHostOnly`).
  - Couvre : lignes « Exit », « Large Finder copy », « Helper start » et « Universal Clipboard » de `implementation.md`. Preuve : `proofs/native-macos-edge-cases/`.
- [ ] **Vérification du collage (N11 et N28, expérimental, AC-019).**
  - Étapes : Expérimental activé, `handy settings output set paste_verification true --json`, collage intelligent actif, `log_level` `debug`. Dans Chrome, champ « Posez une question » de Google en mode IA : 10 dictées avec Entrée automatique, puis 10 sans. Refaire 5 dictées dans TextEdit (champ lisible), 10 dans cmux (terminal, 5 avec Entrée automatique, 5 sans), 5 dans Claude Desktop (Electron), 5 dans un champ mot de passe, puis copier un texte entre la lecture et la relance (si reproductible). Enfin `paste_verification false` et 5 dictées dans la page Google.
  - Attendu :
    - chaque dictée logue une ligne `[smart-paste] verify dictation N: Verified|RetriedVerified|Failed|Unreadable in Xms`, sans contenu ;
    - page Google : le texte apparaît une seule fois dans le champ, ou bien `Failed` suivi de la bulle 010 avec la transcription et **aucune Entrée** sur un champ vide ; noter le nombre de `RetriedVerified` et tout double collage (un collage tardif vu par la dernière lecture avant la relance ne doit plus produire de doublon, N28) ;
    - TextEdit : un seul Cmd+V ; `Unreadable` avec `unreadable: NotWebText` si le collage passe par Cmd+V (champ AppKit hors zone web, N28), ou aucune ligne si l'insertion directe AX a réussi ;
    - cmux (terminal, N28) : le texte apparaît **une seule fois**, un seul Cmd+V, `verify dictation N: Unreadable` puis `verify dictation N unreadable: NotWebText`, **aucune bulle** 010, l'Entrée automatique suit ses règles habituelles ;
    - Claude Desktop (Electron) : `Verified`, un seul Cmd+V ;
    - champ mot de passe : `Unreadable`, comportement inchangé ;
    - presse-papiers changé avant la relance : aucun second Cmd+V, `Failed`, la copie de l'utilisateur reste dans le presse-papiers ;
    - réglage désactivé : aucune ligne `verify dictation`, comportement de N11 inchangé.
  - Couvre : AC-019, FR-012, décisions N11 (« vérification après collage, en test ») et N28 (terminal : aucun doublon, verdict Unreadable). Preuve : `proofs/native-macos-paste-verification/`.
- [ ] **Entrée automatique.**
  - Étapes : `handy settings output set auto_submit true --json`, puis dicter dans les différents cas.
  - Attendu : Entrée est envoyée seulement après `inserted_verified`, `read_observed` ou `typed`, jamais après Screen Sharing ni après un échec.
  - Couvre : AC-012. Preuve : `proofs/native-macos-enter/`.
- [ ] **Sélecteur en vraie fenêtre.**
  - Étapes : Tab jusqu'au sélecteur, puis flèches et Entrée.
  - Attendu : smart paste est listé avec un anneau de focus visible, et VoiceOver annonce la nouvelle valeur.
  - Couvre : `is_keyboard_navigable`. Preuve : `proofs/native-macos-ui/`.
- [ ] **Passe combinée** (`handy-gates-open.app`, après la preuve 006).
  - Étapes : refaire la matrice réduite : TextEdit, Notes, ChatGPT et Comet en AZERTY, avec un changement d'app pendant la dictée.
  - Attendu : le texte est inséré dans l'application de destination ; presse-papiers et disposition sont restaurés.
  - Couvre : AC-016 avec la 006. Preuve : `proofs/native-macos-combined/`.

### 008 — CLI des réglages (`handy-gates-closed.app`)

- [ ] **Changements visibles dans la fenêtre ouverte.**
  - Étapes : `handy settings set theme dark`, puis `handy settings output set paste_method direct`.
  - Attendu : la fenêtre affiche les nouvelles valeurs sans redémarrage.
  - Couvre : AC-005, FR-004. Preuve : `proofs/native-macos-live/`.
- [ ] **Backend des raccourcis.**
  - Étapes : `handy settings set keyboard_implementation handy_keys`, puis presser le raccourci Transcribe.
  - Attendu : l'enregistrement démarre.
  - Couvre : AC-005. Preuve : `proofs/native-macos-live/`.
- [ ] **Sortie par le tray.**
  - Étapes : quitter par le menu du tray, puis `ls "$(getconf DARWIN_USER_TEMP_DIR)hs-$(id -u)/"`.
  - Attendu : le socket a disparu. Jusqu'ici, seul SIGTERM avait été observé.
  - Couvre : AC-005. Preuve : `proofs/native-macos-live/`.
- [ ] **Conflit et Reload.**
  - Étapes : fenêtre ouverte, changer un réglage par la CLI, puis modifier un réglage dans la fenêtre.
  - Attendu : l'alerte « Settings changed elsewhere » apparaît ; Reload ne la ferme qu'une fois les valeurs affichées ; si la relecture échoue, il n'y a pas de faux succès.
  - Couvre : AC-003. Preuve : `proofs/native-macos-ui/`.
- [ ] **History limit.**
  - Étapes : saisir 1001, puis 12 suivi d'Entrée, puis 12 suivi d'un blur, enfin une perte de focus de la fenêtre.
  - Attendu : 1001 affiche « Enter a whole number from 0 to 1000. » et rien n'est envoyé ; 12 est envoyé une seule fois ; une valeur inchangée n'est pas renvoyée ; une perte de focus de la fenêtre n'enregistre rien. Vérifier aussi `hasFocus()` dans la webview.
  - Couvre : AC-004. Preuve : `proofs/native-macos-ui/`.
  - 2026-10-28 : 1001, 12 + Entrée et blur conformes (`proofs/native-macos-ui/history-limit-observations.md`). Reste : perte de focus avec la fenêtre au premier plan et `hasFocus()`.
- [ ] **Store illisible.**
  - Étapes : avec un HOME isolé (`HOME=$(mktemp -d)`, jamais le vrai store), suivre `.specs/features/008-settings-cli/proofs/claude-step7/native-read-only/procedure.txt`, en ouvrant cette fois la fenêtre.
  - Attendu : le toast traduit `errors.settings.readOnly` est visible ; la CLI renvoie exit 5 ; le store est intact.
  - Couvre : correctif A-M2 de l'étape 4. Preuve : `proofs/native-macos-read-only/`.
- [ ] **Démarrage masqué et harnais en release.**
  - Étapes : lancer `bun scripts/settings-native-proof.ts --binary ~/handy-tests/handy-gates-closed.app/Contents/MacOS/handy --app ~/handy-tests/handy-gates-closed.app/Contents/MacOS/handy --home "$(mktemp -d)" --out .specs/features/008-settings-cli/proofs/native-macos-release --samples 30 --window-list <binaire de swiftc -O scripts/macos-window-list.swift>`.
  - Attendu : aucune icône Dock en `--start-hidden` ; chaque lecture ≤ 2000 ms dans `latency.json`. Le Mac est un M4 Pro : le seuil M1 reste une preuve de release.
  - Couvre : AC-001, régression #1787. Preuve : `proofs/native-macos-release/`.

### 009 — Mode PIN et règle Entrée (`handy-gates-open.app`, et `handy-gates-closed.app` pour le verrou fermé)

Preuves natives en attente avec Julien : aucune n'est déclarée réussie. Harnais : `bun scripts/pin-enter-native-proof.ts --binary <absolu>/handy-gates-open.app/Contents/MacOS/handy --platform macos --out .specs/features/009-pin-mode-and-enter-rule/proofs/native-macos/ --interactive` (il lance les tests natifs ignorés en lecture seule, puis enregistre les observations tapées ; il n'envoie aucune touche et n'active aucune app). Utiliser un champ de test et une phrase de test sûre ; aucun message réel ne doit partir (ChatGPT : nouvelle conversation vide ; Comet : champ de recherche).

- [ ] **Verrou fermé** (`handy-gates-closed.app`).
  - Étapes : `handy settings destination status --json` ; `handy settings output set automatic_enter always --json` ; ouvrir les réglages de sortie ; avec `auto_submit` vrai, dicter dans TextEdit.
  - Attendu : `usage_error`/2 ; `not_found`/4 ; seul « Auto Submit » est affiché ; une Entrée 50 ms après le collage, comme sur dev ; aucun badge d'épingle.
  - Couvre : verrou fermé identique à dev (Clarifications), AC-004. Preuve : `proofs/native-macos-gate-closed/`.
- [ ] **Migration réelle.**
  - Étapes : avec `auto_submit` vrai et `auto_submit_key` `ctrl_enter` réglés par la build fermée, lancer une fois `handy-gates-open.app`, puis `handy settings output get automatic_enter --json` et `handy settings output get auto_submit_key --json` ; relancer ensuite la build fermée et dicter.
  - Attendu : `always` et `ctrl_enter` ; la build fermée garde `auto_submit` vrai, Ctrl+Entrée et son comportement.
  - Couvre : AC-004, SC-002. Preuve : `proofs/native-macos/migration/`.
- [ ] **Épingler depuis le badge, puis cinq dictées (SC-003).**
  - Étapes : dicter dans ChatGPT, ouvrir le menu du badge, choisir « Pin ChatGPT » ; puis dicter cinq fois depuis Safari, Notes, TextEdit, Safari, Notes.
  - Attendu : le badge montre ChatGPT avec l'épingle et VoiceOver lit « Destination: ChatGPT, pinned » ; chaque texte arrive dans ChatGPT ; le menu s'ouvre sans voler le focus.
  - Couvre : AC-001, SC-003, FR-001. Preuve : `proofs/native-macos/sc-003/`.
- [ ] **Autre app pour cette dictée, croix, Unpin.**
  - Étapes : épinglé sur ChatGPT, choisir Comet dans le menu ; dictée suivante ; puis la croix ; puis « Unpin ChatGPT » avant la fin d'une dictée commencée dans Safari.
  - Attendu : Comet pour cette dictée seulement, ChatGPT à la suivante ; la croix envoie à l'app au premier plan à la fin et garde l'épingle ; Unpin renvoie à Safari (app capturée au départ) et la dictée suivante capture l'app au premier plan.
  - Couvre : AC-002. Preuve : `proofs/native-macos/override/`.
- [ ] **Retour à l'app désactivé.**
  - Étapes : `handy settings output set return_to_dictation_app false --json` ; sans épingle, dicter ; puis épingler ChatGPT par `handy settings destination pin com.openai.chat --json` et dicter depuis Safari.
  - Attendu : sans épingle, pas de badge et insertion dans l'app au premier plan ; avec épingle, le badge s'affiche et ChatGPT est activé et reçoit le texte.
  - Couvre : AC-001, AC-002 (`return_off_and_pin`). Preuve : `proofs/native-macos/return-off/`.
- [ ] **App épinglée fermée, non ramenée, rouverte.**
  - Étapes : épingler Notes, quitter Notes, dicter ; rouvrir Notes, dicter ; puis rendre Notes impossible à activer (fenêtre sur un autre Space en plein écran, ou l'activation refusée) et dicter.
  - Attendu : rien n'est inséré nulle part, l'avis « Notes is pinned but not open » reste au moins 4 s, le texte est dans l'Historique, l'épingle reste ; Notes rouverte reçoit le texte ; non ramenée : `activation_failed`, rien d'inséré, épingle gardée.
  - Couvre : AC-003, FR-002. Preuve : `proofs/native-macos/unavailable/`.
- [ ] **Entrée automatique : options et compte.**
  - Étapes : pour Never, Always, Only in PIN mode, dicter une fois épinglé et une fois non épinglé dans un champ de test qui compte les Entrées ; noter le t0 et l'instant de la touche dans le log debug (`Automatic Enter of dictation …`).
  - Attendu : 0,0 ; 1,1 ; 0 non épinglé et 1 épinglé ; chaque Entrée éligible entre 50 et 100 ms après t0 (cible QE, mesurée, pas un critère d'acceptation).
  - Couvre : AC-005, SC-001 (natif), FR-004. Preuve : `proofs/native-macos/enter-matrix/`.
- [ ] **Pas d'Entrée sans insertion réussie.**
  - Étapes : méthode `none` ; smart paste sans lecture (« not read ») ; app quittée avant l'insertion ; Cmd+Tab vers une autre app juste après l'insertion ; démarrer une deuxième dictée avant l'Entrée de la première (smart paste qui attend la lecture).
  - Attendu : aucune Entrée dans chaque cas ; une seule Entrée au plus par dictée.
  - Couvre : AC-005. Preuve : `proofs/native-macos/no-enter/`.
- [ ] **Chromium (ChatGPT, Comet) qui lisent deux fois à 150 ms.**
  - Étapes : Always, épinglé sur ChatGPT puis Comet, smart paste ; dicter une phrase longue.
  - Attendu : tout le texte est dans le champ quand l'Entrée arrive. Si l'Entrée part avant la vraie lecture (007 publie « read observed » à la première lecture), c'est un échec à remonter (risque de la revue r8), sans correctif dans cette implémentation.
  - Couvre : AC-005. Preuve : `proofs/native-macos/chromium/`.
- [ ] **Paste fiable (bêta) macOS sur la session.**
  - Étapes : `smart-paste-dispatch` fermé (`handy-destination-only.app`), `reliable_paste` vrai, Always.
  - Attendu : une Entrée après la lecture du presse-papiers par la cible, aucune sans lecture.
  - Couvre : AC-005. Preuve : `proofs/native-macos/reliable/`.
- [ ] **Réglages : sélecteur et clavier.**
  - Étapes : ouvrir les réglages de sortie ; Tab, flèches, Entrée ; VoiceOver.
  - Attendu : seul « Automatic Enter » (Never, Always, Only in PIN mode) et la touche ; le choix est enregistré et annoncé ; focus visible dans l'ordre de lecture.
  - Couvre : AC-004, Behavioral AC. Preuve : `proofs/native-macos-ui/`.
- [ ] **CLI avec Handy lancé.**
  - Étapes : `handy settings destination pin com.openai.chat --json`, `status --json`, `unpin --json` pendant et hors dictée ; `pin com.example.absent --json`.
  - Attendu : réponses `{pinned: …}` ; même effet que le badge sur la dictée en cours ; `not_found`/4 sans changement.
  - Couvre : AC-006. Preuve : `proofs/native-macos/cli/`.
  - 2026-10-28 : hors dictée conforme (`proofs/native-macos/cli/observations-hors-dictee.md`). Reste : pendant une dictée.

### 010 — Bulle de récupération (`handy-gates-open.app`, et `handy-gates-closed.app` pour le verrou fermé)

Preuves natives en attente avec Julien : aucune n'est déclarée réussie. Harnais : `bun scripts/recovery-native-proof.ts --binary <absolu>/handy-gates-open.app/Contents/MacOS/handy --platform macos --out .specs/features/010-recovery-bubble/proofs/native-macos/ --interactive` (il lance `tests/recovery_native.rs` en lecture seule — classification de l'app au premier plan sur 30 échantillons et lecture du premier plan de la 006 inchangée —, puis enregistre les observations tapées ; il n'affiche aucune bulle, n'active aucune app, n'envoie aucune touche et ne touche pas au presse-papiers). Activer le log debug pour les temps (`Recovery bubble N presented … after the recovery decision`, `Focused element classified …`). Phrase de test sûre, champs de test vides ; aucun message réel ne doit partir.

- [ ] **Verrou fermé** (`handy-gates-closed.app`).
  - Étapes : quitter TextEdit pendant une dictée commencée dedans ; `handy settings output get show_recovery_bubble_without_text_field --json` ; ouvrir les réglages de sortie.
  - Attendu : le toast `paste-error` comme sur dev, aucune bulle ; `not_found`/4 ; aucun réglage « Show the bubble… ».
  - Couvre : verrou fermé identique à dev (Clarifications). Preuve : `proofs/native-macos-gate-closed/`.
- [ ] **Spike de surface : premier clic sans activation (étape 1).**
  - Étapes : TextEdit au premier plan avec un document actif ; provoquer une bulle (Notes quittée pendant la dictée) ; cliquer une fois Copy, Close et Why? ; noter le PID au premier plan et la fenêtre clé avant/après (`lsappinfo front`, Accessibility Inspector).
  - Attendu : chaque contrôle agit au premier clic ; TextEdit reste au premier plan avec sa fenêtre clé ; la bulle apparaît à la place de l'indicateur, y compris avec le style `none`.
  - Couvre : AC-003, FR-002. Preuve : `proofs/native-macos/surface/`.
- [ ] **Classification du champ (étape 1, matrice de calibration).**
  - Étapes : réglage activé, permission Accessibilité accordée ; dicter vers : liste d'une fenêtre Finder, bureau Finder, page Safari sans champ focalisé, document TextEdit, note Notes, champ de recherche, contrôle personnalisé inconnu ; puis, avec un champ focalisé, Comet, Chrome, ChatGPT (app), Slack et VS Code (éditeur et barre de recherche), chacun aussi juste après son lancement (arbre d'accessibilité Chromium/Electron pas encore activé) ; puis Spotlight ouvert au-dessus de Finder, puis au-dessus de TextEdit ; lancer `recovery_native` (30 échantillons).
  - Attendu : bulle « No text field is focused » seulement sur preuve positive (rôle non textuel connu — liste, bouton, image… — dont les attributs d'édition sont lus et non réglables) ; insertion pour un champ éditable, un rôle inconnu ou conteneur (`AXWebArea`, `AXWindow`, `AXGroup`, `AXApplication`, `AXScrollArea`), un attribut illisible, un champ sécurisé ou un focus d'un autre processus ; **jamais de bulle à tort dans Comet, Chrome, ChatGPT, Slack et VS Code** ; Spotlight (limite acceptée le 2026-10-15) : résultat noté ; aucun échantillon au-dessus de 60 ms, pas de gel d'UI ; lecture du premier plan de la 006 inchangée.
  - Couvre : AC-002, FR-001. Si le critère des 60 ms échoue, la détection de champ n'est pas livrée (verdict `Insert`) et la feature ne passe pas Implemented sans décision de Julien. Preuve : `proofs/native-macos/classification/`.
- [ ] **Raisons avant insertion et Entrée.**
  - Étapes : Entrée automatique `always` ; Notes quittée pendant la dictée ; Notes épinglée puis quittée ; Notes non ramenée au premier plan (plein écran sur un autre Space) ; Finder sans champ.
  - Attendu : « Notes is no longer open », « Notes is pinned but not open », « Could not bring Notes forward », « No text field is focused » ; rien n'est inséré ; aucune Entrée ; aucun toast `paste-error` ; « Why? » dit « Handy didn't insert anything and sent no Enter. », l'Historique selon la sauvegarde, et le conseil.
  - Couvre : AC-001, AC-003, SC-003. Preuve : `proofs/native-macos/reasons/`.
- [ ] **Échecs de méthode et smart paste.**
  - Étapes : smart paste sans lecture (« not read », ex. un champ qui ignore Cmd+V) ; puis une lecture tardive ; permission Accessibilité retirée (`permission_missing`) ; méthode classique en échec.
  - Attendu : « Insertion failed » ; détail « Some text may have reached the application » (not read, uncertain, échec de méthode) ; « Nothing was sent to the application » (permission absente) ; après la lecture tardive, la même bulle affiche « may have been pasted after all » dans son texte principal, sans ouvrir « Why? », et VoiceOver l'annonce ; aucune Entrée.
  - Couvre : AC-001. Preuve : `proofs/native-macos/method-failures/`.
- [ ] **Avis de l'indicateur conservés.**
  - Étapes : un prompt IA qui échoue (clé invalide) puis une insertion qui échoue (app quittée) ; refaire en quittant l'app juste après l'échec IA, pour que le refus soit décidé en quelques millisecondes, avant que l'avis ait sa taille ; Screen Sharing en remise de presse-papiers puis une bulle ; refaire avec l'indicateur en haut puis en bas de l'écran.
  - Attendu : l'avis IA (005) reste au moins 4 s et l'avis de remise (007) toute sa durée, même quand la bulle apparaît dans sa propre fenêtre ; l'avis et la bulle sont lisibles tous les deux : la bulle est au-dessus de l'avis (indicateur en bas) ou au-dessous (indicateur en haut), 8 pt plus loin, sans chevauchement et sans sortir de l'écran, y compris quand l'avis grandit après l'apparition de la bulle (elle se replace, sans clignoter ni osciller) ; elle ne bouge plus à la fin de l'avis.
  - Couvre : 005 AC-005, 007 AC-012, 010 FR-002. Preuve : `proofs/native-macos/notices/`.
- [ ] **Glisser-déposer natif (étape 6, SC-002).**
  - Étapes : glisser la bulle vers TextEdit, Notes, ChatGPT (champ vide) et Comet (champ de recherche) ; puis relâcher sur le bureau (Échap pendant un glisser : item N25 ci-dessous) ; essayer avec Option et Cmd enfoncés ; démarrer une dictée pendant un glisser.
  - Attendu : la transcription complète dans TextEdit et Notes (obligatoire), résultat noté pour ChatGPT et Comet ; la bulle est cachée pendant le glisser, revient inchangée après un refus (à côté de l'avis de l'indicateur si un avis le tient alors), se ferme après un dépôt accepté (log « drop accepted » sans texte) ; toujours une copie, quels que soient les modificateurs ; le presse-papiers général est inchangé ; une nouvelle dictée ferme la bulle pour de bon et le dépôt tardif ne la rouvre pas ; avec « espace final » actif, le texte déposé et celui de Copy finissent par l'espace comme une insertion, l'aperçu non.
  - Couvre : AC-004, FR-003, SC-002. Preuve : `proofs/native-macos/drag/`.
- [ ] **Toute la bulle est la source du glisser (N24).**
  - Étapes : saisir la bulle par le texte de l'aperçu, par le fond près de la raison et par la bordure, et la déposer dans TextEdit ; survoler le texte puis Copier ; cliquer sur l'aperçu sans bouger ; appuyer sur Copier, Pourquoi ? et × en bougeant légèrement.
  - Attendu : chaque prise démarre le même glisser que le bouton « Glisser dans un champ » (transcription complète, log `Recovery bubble N: drop accepted`) ; le curseur est une main ouverte au survol de la surface et une main de lien sur les boutons ; un clic sans mouvement ne démarre rien ; Copier, Pourquoi ? et × gardent leur action et ne glissent jamais.
  - Couvre : AC-004, FR-003. Preuve : `proofs/native-macos/drag/`.
- [ ] **Échap pendant un glisser ferme la bulle (N25).**
  - Étapes : `RUST_LOG=info`, TextEdit au premier plan avec un document vide ; glisser la bulle au-dessus du document puis appuyer sur Échap sans relâcher, relâcher ensuite ; nouvelle bulle : glisser au-dessus du bureau puis appuyer sur Échap ; nouvelle bulle : glisser et relâcher sur le bureau sans Échap ; refaire le premier cas avec Handy (fenêtre des réglages) au premier plan ; nouvelle bulle : déposer dans TextEdit puis appuyer sur Échap juste après.
  - Attendu : après Échap, rien n'est inséré dans TextEdit (document toujours vide), la bulle ne réapparaît pas et le log dit `Recovery bubble N: drag cancelled by Escape, dismissed` sans texte ; la transcription reste dans l'historique ; un relâchement sur le bureau sans Échap fait revenir la bulle inchangée ; un dépôt accepté puis Échap reste `drop accepted`. Noter dans la preuve quel premier plan a été testé ; si un cas d'Échap fait revenir la bulle, le moniteur n'a pas vu la touche : noter le premier plan concerné (régression vers le comportement d'avant N25, pas d'insertion).
  - Couvre : AC-004, FR-003. Preuve : `proofs/native-macos/drag/`.
- [ ] **Échap sans retour animé ni flash de la bulle (N26).**
  - Étapes : `RUST_LOG=info`, filmer l'écran (Cmd+Maj+5) ; glisser la bulle au-dessus d'un document TextEdit vide, appuyer sur Échap sans relâcher ; nouvelle bulle : même chose au-dessus du bureau ; nouvelle bulle : glisser et relâcher sur le bureau sans Échap ; refaire le premier cas avec Handy (fenêtre des réglages) au premier plan. Revoir la vidéo image par image autour d'Échap.
  - Attendu : après Échap, l'aperçu glissé disparaît sur place sans revenir vers la bulle, la bulle ne réapparaît à aucune image avant de partir, rien n'est inséré, log `Recovery bubble N: drag cancelled by Escape, dismissed` ; sans Échap, l'aperçu revient vers la bulle et la bulle réapparaît inchangée. Si l'aperçu revient encore après Échap mais sans flash de la bulle, Échap n'a été vu qu'à la fin de la session (limite connue N26) : noter le premier plan concerné.
  - Couvre : AC-004, FR-003. Preuve : `proofs/native-macos/drag/`.
- [ ] **Copy contre la restauration du smart paste.**
  - Étapes : dictée A insérée par smart paste avec une restauration en attente ; dictée B qui ouvre une bulle ; Copy pendant la période calme ; attendre 9 s.
  - Attendu : la transcription de B reste dans le presse-papiers ; la bulle se ferme ; aucune restauration ne l'écrase.
  - Couvre : AC-005. Preuve : `proofs/native-macos/copy/`.
- [ ] **Durée de vie.**
  - Étapes : laisser une bulle 60 s, verrouiller puis déverrouiller l'écran ; Close ; nouvelle dictée sur une bulle affichée depuis 1 s ; nouvelle dictée dont l'enregistrement échoue (micro débranché ou refusé).
  - Attendu : la bulle reste jusqu'à une action ; Close la ferme sans toucher l'Historique ; la nouvelle dictée la ferme dès que l'enregistrement démarre et l'indicateur apparaît ; si l'enregistrement ne démarre pas, la bulle reste avec son texte.
  - Couvre : AC-005. Preuve : `proofs/native-macos/lifetime/`.
- [ ] **VoiceOver et clavier.**
  - Étapes : VoiceOver sur la bulle (sans lui donner le focus clavier) ; réglages de sortie : Tab jusqu'à « Show the bubble when no text field is focused », Espace.
  - Attendu : noms « Close », « Drag into a field », « Copy », « Why? » et la raison ; chaque contrôle s'active au curseur VoiceOver ; le réglage bascule et est annoncé.
  - Couvre : AC-003, Behavioral AC, AC-007. Preuve : `proofs/native-macos-ui/`.
- [ ] **Temps de présentation (cible QE de 300 ms).**
  - Étapes : relever dans le log debug chaque échantillon décision → bulle visible (froid : première bulle après lancement ; chaud : les suivantes).
  - Attendu : tous les échantillons notés ; ceux au-dessus de 300 ms sont un constat QE à remonter, jamais un échec de la bulle.
  - Couvre : QE (spec). Preuve : `proofs/native-macos/latency/`.
- [ ] **CLI avec Handy lancé.**
  - Étapes : `handy settings output set show_recovery_bubble_without_text_field false --json`, puis Finder sans champ ; `get --json`.
  - Attendu : enveloppe JSON seule ; la dictée suivante s'insère (pas de bulle) ; parité avec le réglage affiché.
  - Couvre : AC-007. Preuve : `proofs/native-macos/cli/`.

### 011 — Retour au champ de la dictée (`handy-gates-open.app`, et `handy-gates-closed.app` pour le verrou fermé)

Preuves natives en attente avec Julien : aucune n'est déclarée réussie. Harnais : `bun scripts/field-return-native-proof.ts --binary <absolu>/handy-gates-open.app/Contents/MacOS/handy --platform macos --out .specs/features/011-dictation-field-return/proofs/native-macos/ --interactive --log <fichier de log Handy> --sentinel <phrase sentinelle>`. Il lance `tests/destination_field_native.rs` en lecture seule (lecture du champ focalisé de l'app au premier plan, ou du PID donné par `HANDY_FIELD_PID` pour une app en arrière-plan ; lectures de vérification ; lecture du premier plan de la 006 inchangée), puis enregistre les observations tapées et vérifie que la phrase sentinelle tapée dans le brouillon n'apparaît pas dans le log. Le harnais ne donne jamais le focus, ne restaure aucune plage, ne lève aucune fenêtre, n'active aucune app, n'envoie aucune touche et ne touche pas au presse-papiers : c'est Julien qui dicte. Activer le log debug (`field_return …`). Brouillons de test sûrs, aucun message réel ne doit partir (Entrée automatique « Never » sauf pour l'item Entrée). Résultats de l'étape 1 à ranger dans `proofs/macos/native-spike.md` (OS, hash du binaire, matrice par app, temps).

- [ ] **Verrou fermé** (`handy-gates-closed.app`).
  - Étapes : dicter depuis le champ ChatGPT de Comet, passer à Notes, arrêter.
  - Attendu : comportement de dev (texte collé dans l'app au premier plan à la fin) ; aucune ligne `field_return` dans le log.
  - Couvre : AC-006, SC-003. Preuve : `proofs/native-macos-gate-closed/`.
- [ ] **Exposition de Comet (étape 1, précondition de SC-001).**
  - Étapes : Comet au premier plan, page ChatGPT, curseur dans un brouillon ; lancer le harnais (sans `--interactive` d'abord).
  - Attendu : `remembered(range_read: true)` ou un `not_remembered(...)` noté ; aucun changement du mode d'accessibilité de Comet (`AXManualAccessibility`, `AXEnhancedUserInterface` jamais posés). Si l'input n'est pas exposé, les 5 essais Comet de SC-001 doivent suivre le repli sûr (rien de mémorisé, 006) et le résultat est remonté à Julien.
  - Couvre : AC-001, SC-001. Preuve : `proofs/macos/native-spike.md`.
- [ ] **Retour au curseur avec brouillon (SC-001, 5 essais par app).**
  - Étapes : Comet (page ChatGPT, même onglet), Notes et ChatGPT desktop : un brouillon avec la sentinelle, curseur au milieu ; dicter ; passer à une autre app ; arrêter. Refaire avec une sélection non vide avant le départ.
  - Attendu : le texte complet au curseur, brouillon intact (ou remplacement de la sélection faite avant le départ) ; 5/5 par app ; noter si donner le focus sélectionne tout ou déplace le curseur avant la restauration de la plage.
  - Couvre : AC-001, AC-002, FR-001, FR-002, SC-001. Preuve : `proofs/native-macos/caret/`.
- [ ] **N18 — App lente après activation (Codex, champ Google dans Chrome, Comet).**
  - Étapes : build signée avec `destination-return,smart-paste-dispatch`, log debug ; dicter depuis le champ de Codex, puis depuis le champ de recherche Google dans Chrome, curseur seul puis sélection non vide ; passer à une autre app ; arrêter. 5 essais par app.
  - Attendu : le texte au curseur du champ mémorisé (ou à un curseur déplacé dans ce même champ), jamais sur une sélection non vérifiée ni dans un autre champ ; ligne `field_return Verified (stage …, ax_error …, settled … in N reads, guard_range …, note …)`. Relever pour chaque échec le stage, l'erreur AX et `guard_range`.
  - Couvre : AC-002, AC-004, FR-004 (décision `verified-at-guard`). Preuve : `proofs/native-macos/n18-late-app/`.
- [ ] **Deuxième fenêtre.**
  - Étapes : Notes avec deux fenêtres ; dicter depuis l'éditeur de la fenêtre A ; amener la fenêtre B ; arrêter.
  - Attendu : la fenêtre A est levée et le texte arrive dans son éditeur ; sinon la bulle, jamais un collage dans la fenêtre B ; noter si le focus survit à `AXRaise` (un ordre inversé est remonté à Julien, jamais changé en silence).
  - Couvre : AC-002, AC-004. Preuve : `proofs/native-macos/second-window/`.
- [ ] **Onglet changé en arrière-plan.**
  - Étapes : Comet, dicter depuis le champ ChatGPT ; sélectionner un autre onglet de Comet pendant l'enregistrement ; arrêter.
  - Attendu : rien n'est collé ; bulle 010 « Insertion failed » avec l'explication que rien n'a été envoyé ; aucune Entrée. Un collage dans un onglet caché est un constat à remonter à Julien. Noter aussi le cas où les écritures de focus et de plage sont acquittées mais où le focus n'a pas encore bougé à la garde (Chromium peut appliquer après coup) : la branche (a) colle alors dans l'élément noté ; remonter tout collage inattendu (audit de livraison, tour 1).
  - Couvre : AC-004 (b), SC-002. Preuve : `proofs/native-macos/changed-tab/`.
- [ ] **Page rechargée.**
  - Étapes : Comet, dicter depuis le champ ChatGPT ; recharger la page pendant l'enregistrement ; arrêter.
  - Attendu : insertion comme aujourd'hui (006/010) si la tentative n'a rien changé, sinon la bulle ; jamais un collage ailleurs. Noter lequel, et si Comet expose alors un élément focalisé (le cas « aucun élément focalisé » est traité comme non noté : bulle).
  - Couvre : AC-004 (a) et (b). Preuve : `proofs/native-macos/reload/`.
- [ ] **Frappe dans le même champ pendant l'enregistrement.**
  - Étapes : dicter depuis un champ avec brouillon et continuer à taper dans ce même champ pendant l'enregistrement ; arrêter. Refaire avec une sélection non vide au départ.
  - Attendu : noter où arrive le texte et si du texte tapé est remplacé (la plage du départ est restaurée telle quelle) ; constat à trancher par Julien (ambiguïté de la revue de spec r3, AC-002).
  - Couvre : AC-002. Preuve : `proofs/native-macos/same-field-typing/`.
- [ ] **Champ mot de passe.**
  - Étapes : dicter depuis un champ mot de passe (page de connexion Safari de test) ; passer à une autre app ; arrêter.
  - Attendu : rien de mémorisé, comportement 006 ; rien n'est tapé dans le champ mot de passe.
  - Couvre : AC-001. Preuve : `proofs/native-macos/password/`.
- [ ] **App épinglée en arrière-plan.**
  - Étapes : épingler Comet avec son champ ChatGPT focalisé ; amener Notes ; dicter deux fois (harnais avec `HANDY_FIELD_PID=<pid de Comet>` pour la lecture seule).
  - Attendu : chaque texte dans le champ ChatGPT après le retour de Comet.
  - Couvre : AC-003, FR-003. Preuve : `proofs/native-macos/pin/`.
- [ ] **Changement de destination par le badge, méthode « none ».**
  - Étapes : dicter depuis le champ de Comet puis choisir TextEdit dans le badge ; puis méthode de collage « none ».
  - Attendu : texte dans TextEdit comme en 006, rien dans Comet ; avec « none », aucune activation ni changement de focus, transcription dans l'historique seulement.
  - Couvre : AC-003, AC-006. Preuve : `proofs/native-macos/destination-change/`.
- [ ] **Bulle forcée, Entrée et `paste-error`.**
  - Étapes : provoquer un cas dévié (onglet changé) avec l'Entrée automatique « Always » ; puis le même cas écran verrouillé pendant la présentation.
  - Attendu : bulle avec Copy fonctionnel, aucune Entrée, aucun toast `paste-error` ; si la bulle ne peut pas être présentée, un seul `paste-error` et la transcription dans l'historique.
  - Couvre : AC-004, AC-005, SC-002. Preuve : `proofs/native-macos/bubble/`.
- [ ] **Temps et thread principal.**
  - Étapes : relever dans le log debug `field_return … (attempt …, verify …)` et `not_remembered(...) in …` ; 30 échantillons de `check()` sur le thread principal avec la méthode classique.
  - Attendu : lecture de départ sous 50 ms, tentative sous 50 ms, vérification sous 10 ms ; aucun échantillon de `check()` au-dessus de 70 ms ; pas de gel d'UI. Un dépassement est remonté à Julien, jamais élargi en silence.
  - Couvre : QE (performance), AC-001, AC-002, SC-005. Preuve : `proofs/native-macos/timing/`.
- [ ] **Sentinelle absente des logs et des événements.**
  - Étapes : `--log` et `--sentinel` du harnais sur toute la séance ; `handy settings output get return_to_dictation_app --json`.
  - Attendu : la sentinelle du brouillon n'apparaît ni dans le log ni dans la sortie CLI ; aucune clé de réglage nouvelle.
  - Couvre : AC-008, SC-004. Preuve : `proofs/native-macos/privacy/`.

### 017 — Espacement intelligent (`handy-gates-open.app` ou mode dev avec les deux verrous)

Protocole complet (lignes 1 à 17) dans `.specs/features/017-smart-spacing/implementation.md`, « Native proof protocol ». Accessibilité accordée, « ␣ Espacement intelligent » et « Aa Majuscules intelligentes » actifs, log debug. Aucune preuve n'est déclarée réussie.

- [ ] **N30 — Point final de la dictée au milieu d'une phrase (TextEdit, modèle Canary).**
  - Étapes : dans TextEdit, curseur dans `Bonjour|demain`, dicter « à tous » ; puis curseur dans `Bonjour|, ça va`, dicter « tout le monde » ; puis curseur dans `Bonjour|Demain`, dicter « à tous » ; puis curseur en fin de champ après `Bonjour`, dicter « à tous » ; refaire le premier essai avec l'espacement intelligent désactivé, puis dans cmux (route 013 adressée).
  - Attendu : `Bonjour à tous demain` et `Bonjour tout le monde, ça va` (point final retiré) ; `Bonjour à tous. Demain` et `Bonjour à tous.` (point gardé devant une majuscule et en fin de champ) ; option désactivée : sortie d'avant la 017, point gardé et aucune espace ajoutée ; cmux : le point de Canary est gardé. Log : seulement le genre de contexte et les drapeaux, jamais le texte.
  - Couvre : AC-017 (décision `n30_trailing_period`), protocole lignes 18 à 21. Preuve : `.specs/features/017-smart-spacing/proofs/native-macos/n30/`.

### UI en vraie fenêtre (transverse)

- [ ] **Indicateur** (`handy-gates-open.app`).
  - Étapes : afficher l'indicateur sur un second écran et en Retina. Cliquer juste à côté de la zone visible. Observer les avis. Tester le style `none`.
  - Attendu :
    - l'indicateur reste borné à son moniteur ;
    - un clic dans la zone transparente agrandie atteint l'app dessous et n'est pas capté ;
    - les avis durent à partir de l'événement, sans être raccourcis ;
    - le toast affiché avec le style `none` est assez visible.
  - Couvre : 005, 006, 007. Preuve : `proofs/native-macos-indicator/` de la 006.
- [ ] **Navigation et taille de fenêtre minimale** (680×570).
  - Étapes : traverser au clavier chaque page modifiée (Sound, Advanced › Replacements, Vocabulary, Shortcuts, Post Process, Output, History).
  - Attendu : focus visible ; rien n'est coupé ; le focus programmatique ne fait pas sauter le défilement (001, `focus()` sans `preventScroll`).
  - Preuve : `proofs/native-macos-ui/` de chaque feature.
- [ ] **Dialogues de garde de la 002 pendant Save/Apply.**
  - Étapes : Save ou Apply en cours, puis changer de vue ou cliquer dans la sidebar.
  - Attendu : aucun Discard ne lève le verrou en cours d'écriture ; le clic dans la sidebar donne un retour.
  - Preuve : `proofs/native-macos-ui/` de la 002.

## 3. Décisions à confirmer avec Julien

- [ ] **009 — durée de l'épingle.** L'orchestrateur a proposé : l'épingle dure jusqu'à Unpin ou la fermeture de Handy, et n'est pas restaurée au redémarrage (`ship.md`, r5).
- [ ] **009 — propositions de la revue de plan** (tâche #1, si elle n'est pas déjà tranchée) :
  - AC-002 : retour désactivé et rien d'épinglé → `FrontmostAtEnd` ; épinglé → activation ;
  - retirer `destination pin current` de la spec, et le rapport d'effet current/next du plan ;
  - garder `--dry-run` ;
  - épingle active mais support illisible → échec fermé.
- [ ] **001 — Save inactif pendant un conflit.** Conforme à AC-007, mais la maquette error le montre actif. Le bandeau « Your edits are kept. Reload shows the saved order and discards your edits. » est trompeur. Il faut aligner la maquette et le contrat Penflow, ou réactiver Save en indiquant qu'il remplace l'ordre externe.
- [ ] **001 — autres écarts :**
  - pas de contrôle Reset (Reset = retirer toutes les lignes ; `spec-refine` à faire) ;
  - « None » dans Sound contre « Default » dans le sélecteur debug ;
  - position de la barre « Unsaved changes » (§2).
- [ ] **005 — texte de l'avis IA.** AC-005 et la décision du 2026-10-14 disent « …the original text was used for insertion ». Le texte livré est « AI processing failed. The original text was used. », parce qu'un test interdit « insert ». Choisir : modifier le test ou modifier la spec.
- [ ] **005 — clé API inline** dans l'éditeur de prompt. C'est un écart avec le contrat Penflow, qui prévoyait d'aller à la page du fournisseur. Valider, puis aligner le contrat.
- [ ] **005 — autres choix :**
  - avec le style `none`, toast d'au moins 4 s ;
  - règle « prompt fait uniquement d'espaces refusé » : à approuver ou à retirer (`spec-refine`) ;
  - commandes de prompts sans `expected_revision` : le dernier qui écrit gagne face à la CLI ;
  - préfixes des test IDs `prompt-settings-invalid-*`.
- [ ] **004 :**
  - basculer le post-processing pendant une dictée est maintenant refusé (`shortcut_session_busy`) ;
  - les échecs de compensation sont seulement journalisés ;
  - écarts à la maquette : Add inactif tant que le recorder est vide, Discard ajouté, « Try again » avec le message de l'OS, l'action IA reste sur la page Post Process.
- [ ] **006 :**
  - activation depuis le thread principal (macOS 14+), à trancher pendant la preuve ;
  - `[ASSUMED]` : attente de 1 s, raison affichée au moins 4 s, réglage activé par défaut à l'ouverture du verrou ;
  - écarts UI : monogramme, pas d'entrée « Pin <app> », fermeture du menu 400 ms après la sortie du pointeur.
- [ ] **007 :**
  - en mode « copy to clipboard », une lecture tardive non signalable est une limite acceptée (AC-012 à préciser) ;
  - un collage interrompu laisse la transcription dans le presse-papiers ;
  - B NIT-4 : matérialiser ou restaurer après une perte de destination ou une écriture AX tardive ;
  - `[ASSUMED]` : les lectures avant la frappe ne comptent pas, et Screen Sharing garde la transcription.
- [ ] **009 :**
  - Windows : `bun scripts/pin-enter-native-proof.ts --binary <absolute-windows-release-binary> --platform windows --out <proof-directory> --interactive`, avec l'activation de l'app épinglée, la confirmation de lecture du paste fiable, le compte d'Entrées et le menu sans activation ;
  - Linux (build verrou ouvert) : `handy settings destination status --json` renvoie `validation_failed`/2 `unsupported_platform` ; Always et Never insèrent comme aujourd'hui ; `pin_only` n'envoie aucune Entrée ;
  - ancien binaire : un Handy antérieur avec `auto_submit` vrai, puis un premier lancement de la build ouverte : `handy settings output get automatic_enter --json` vaut `always` et `auto_submit_key` est gardé ; l'ancien binaire relancé garde son comportement ;
  - `cargo check` Windows et Linux en CI (le code `paste_tx/windows.rs` de la 009 n'est compilé que sur Windows).
- [ ] **010 :**
  - la bulle ne coupe plus les avis 005 et 007 et se place à côté de la fenêtre de l'indicateur tant qu'un avis la tient (revue de livraison, tour 2) : position calculée et testée hors natif (`overlay_surface::stack_beside`), replacée à chaque changement de taille de l'indicateur tenu et après un glisser refusé (revue de livraison, tour 3), lisibilité des deux fenêtres à prouver en natif (item « Avis de l'indicateur conservés ») ;
  - en mode « copy to clipboard », 007 n'émet pas de lecture tardive : la bulle garde le texte « not read » simple (limite 007 à décider par son propriétaire) ;
  - classification (revue de livraison, 2026-10-15) : « non éditable » seulement sur preuve positive ; tout attribut illisible et les conteneurs `AXWebArea`, `AXWindow`, `AXGroup`, `AXApplication`, `AXScrollArea` sont indéterminés ; à revoir après la matrice de calibration ;
  - la poignée de glisser est un bouton sans action au clavier ni pour VoiceOver (Copy reste l'alternative) ; le glisser part de l'événement souris courant d'AppKit, à prouver avec un geste court (revue de livraison) ;
  - les commandes de la bulle sont enregistrées dans les deux états du verrou (un seul `bindings.ts`) et inertes sans le service.
- [ ] **011 :**
  - un élément focalisé absent (`AXFocusedUIElement` sans valeur) à la note de l'étape (1) est traité comme « non noté » : bulle, jamais branche (a) (ambiguïté AC-004 de la revue r3, choix sûr du plan) ;
  - la plage du départ est restaurée même si l'utilisateur a tapé dans le même champ pendant l'enregistrement (ambiguïté AC-002 de la revue r3 ; avertissement de la revue de livraison, tour 1) : à trancher après l'item « Frappe dans le même champ ». Proposition de la revue : retenir aussi le nombre de caractères du champ au départ (une longueur seule, jamais le contenu, par exemple `AXNumberOfCharacters`) et passer en AC-004 (b) à la garde s'il a changé, au moins quand la sélection du départ n'était pas vide ; cela demande d'amender AC-008 (champs retenus) par `/spec-refine`. Comportement actuel inchangé en attendant ;
  - Windows : une app épinglée qui n'est pas au premier plan au départ ne mémorise rien (UI Automation n'expose que le focus système ; avertissement de la revue de plan sur AC-003, repris par la revue de livraison, tour 1). Proposition de la revue : acter cette exception Windows d'AC-003 dans AC-007 (ou AC-003) par `/spec-refine`, ou lire le focus du processus épinglé autrement ; l'item natif Windows reste. Comportement actuel inchangé en attendant ;
  - N18 (décision `verified-at-guard`) : un curseur déplacé ailleurs dans le champ mémorisé reçoit le collage (aucune sélection, rien d'écrasé) ; note de l'étape (1) bornée à 300 ms ; à valider par Julien ;
  - les documents 006 et 010 sont à réaligner sur les deltas listés dans `implementation.md` de la 011 (`/spec-refine 006-destination-application plan`, `/spec-refine 010-recovery-bubble plan`).
- [ ] **008 :**
  - store illisible : lecture seule à chaque lancement, onboarding répété, aucune voie de sortie documentée ;
  - plancher Windows supporté (Windows 10 ?) ;
  - Linux : l'instance garde-t-elle le verrou toute sa vie ? (décision de sonde d'instance, `ship.md`).
- [ ] **002 :**
  - le hangul est exempté, mais pas le thaï, le lao, le khmer, le birman ni le tibétain ;
  - faut-il refuser les caractères de format invisibles (U+200B, U+FEFF…) et échapper la sortie `list` ?
  - une écriture concurrente sans rapport est résolue au lieu d'être un conflit ;
  - la note « Reload drops edits » remplace une confirmation ;
  - décisions C0/C1/bidi et ZWJ/ZWNJ CJK à reporter dans la spec.
- [ ] **003 :**
  - `[ASSUMED]` : 500 termes de 100 caractères, `<`, `>` et `"` interdits ;
  - familles génériques (Qwen3-ASR, Fun-ASR-Nano, Granite, MOSS) désactivées (`context_budget_unverified`), ce qui bloque AC-004 pour elles ;
  - écarts au contrat Penflow (`term-<index>`, fermeture après Apply, `-loading` et `-list-error`).

## 4. Ouverture des verrous

Les deux verrous sont des features Cargo fermées par défaut (`src-tauri/Cargo.toml`, `[features]`).

### `destination-return` (006 ; 009, 010 et 011 sont aussi derrière ce verrou)

Fermé, une dictée s'insère comme sur dev : pas de capture, pas d'activation, pas de garde. La ligne de réglage est masquée, et `return_to_dictation_app` est refusé avec `unsupported_platform` (`src-tauri/src/destination/rollout.rs`).

- [ ] Preuve macOS A de la 006 réussie (§2) : retour, quatre apps et plus, destination fermée, garde pendant le délai.
- [ ] UI certifiée par Penflow : badge, croix, menu non activant, toggle avec sa raison, raison affichée au moins 4 s.
- [ ] Cache d'icônes : aujourd'hui `icon_ref` vaut toujours `None`, et AC-001, AC-003 et AC-004 sont partiels. Le construire, ou faire accepter le monogramme.
- [ ] Décision sur l'activation depuis le thread principal. Déplacer le verdict 010 avant le branchement de la 010.
- [ ] `useDestinationSupport` : écouter `destination-support-changed`, corriger la course au démarrage et ajouter un `.catch` sur `listen()`.
- [ ] Suivis de l'indicateur :
  - preuve native que la zone transparente ne capte pas les clics ;
  - durée des avis comptée depuis l'événement ;
  - toast `none` assez visible ;
  - ligne FR-003 « UI not implemented » périmée dans `implementation.md` de la 006 ;
  - test « verrous fermés » comparé à dev ;
  - `cargo check` Windows et Linux en CI.
- [ ] 009 et 010 implémentées et prouvées (elles dépendent de ce verrou) ; pour la 010 : preuves macOS du §2, contrat Penflow certifié pour `recovery-bubble` et `output-settings`, et réalignement des documents 006 et 007 (`/spec-refine 006-destination-application plan`, `/spec-refine 007-smart-paste plan`) sur les deltas listés dans `implementation.md` de la 010.
- [ ] 011 implémentée et prouvée : preuves macOS du §2 (section 011) et réalignement des documents 006 et 010 sur les deltas listés dans `implementation.md` de la 011.
- [ ] Changement séparé : activer la feature par défaut (ou retirer le verrou et `rollout.rs`), relancer toute la suite, puis noter dans le changelog que `return_to_dictation_app` passe à `true` sur macOS et Windows.

### `smart-paste-dispatch` (007)

Fermé : smart paste n'est ni proposé ni migré, et la dictée est inchangée. Ouvert : la dictée passe par smart paste, dans la session 006 si `destination-return` est ouvert (`src-tauri/src/smart_paste/setting_policy.rs`).

- [ ] Matrice AC-016 réussie avec Julien (§2), plus R-1 (cache AppKit), R-2 (SC-008) et SC-007 (écriture AX tardive), qui bloquent la release.
- [ ] Normaliser `paste_method=smart_paste` quand le verrou est fermé : aujourd'hui, revenir à une build fermée fait échouer chaque dictée.
- [ ] Redémarrage après mise à jour (`RESTART_EXIT_CODE`) : régler l'acteur avant de quitter (`exit_gate.rs:29`).
- [ ] Refus de la garde 006 qui masque `uncertain` (`destination_bridge.rs:64`).
- [ ] Revalider la destination avant l'Entrée automatique (`app.rs:200`).
- [ ] Vérifier l'identité du binaire helper (`macos_ports.rs:27`).
- [ ] Message Screen Sharing sur l'indicateur (aujourd'hui un toast dans la fenêtre des réglages).
- [ ] Test d'« uncertain » artificiel pour le timeout de l'acteur (environ 4,6 s).
- [ ] Conventions :
  - `run_strategy`, `await_first_read` et `select_source_on_main` dépassent 30 lignes ;
  - `scenario.rs` fait 407 lignes.
- [ ] Contrat Penflow, maquette et preuve visuelle du sélecteur.
- [ ] Changement séparé : activer la feature par défaut, et mettre à jour le schéma 008 enregistré (aujourd'hui en état fermé).

## 5. Release Windows/Linux (preuves en attente, jamais déclarées réussies)

- [ ] **Partout :**
  - `cargo build --manifest-path src-tauri/Cargo.toml` ;
  - `cargo test --manifest-path src-tauri/Cargo.toml` ;
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --bins -- -D warnings` ;
  - `bun run lint` ;
  - `bunx --no-install tsc --noEmit` ;
  - builds d'`overlay_indicator.rs`, et `cargo check` Windows et Linux en CI.
- [ ] **001 :**
  - `bun scripts/microphone-native-proof.ts --platform windows --out <dir> --binary <abs Handy.exe> --interactive` (WASAPI) ;
  - `bun scripts/microphone-native-proof.ts --platform linux --out <dir> --binary <abs handy> --interactive` (ALSA/PulseAudio) ;
  - pause du sondage sous WebView2 (fenêtre cachée et minimisée) et sous Linux ;
  - erreurs ALSA transitoires (POLLERR) qui coupent une dictée.
- [ ] **002 :**
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib word_replacements` ;
  - `cargo test --manifest-path src-tauri/Cargo.toml --test replacement_exchange` ;
  - CLI manuelle sur Windows, avec un import par étapes et une liste de plus de 1 MiB ;
  - sur un Mac M1 : `cargo test --release --manifest-path src-tauri/Cargo.toml --test replacement_performance -- --nocapture`, attendu sous 20 ms ;
  - sur un Mac M1 : `proofs/claude-integration/native/native-check.sh.txt` adapté, en release.
- [ ] **003 :** échantillon natif et contrôles clavier et statut de l'UI sur Windows et Linux.
- [ ] **004 :**
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib shortcut transcription_coordinator` ;
  - A/B physique toggle et push-to-talk avec le backend Tauri ;
  - Linux : raccourci Cancel caché et non enregistré ;
  - ancien binaire : avec 2 raccourcis, le premier marche et le second est ignoré.
- [ ] **005 :**
  - spike du menu et délais d'ouverture sous Windows et Linux (X11, Wayland, layer-shell désactivé), avec `bun scripts/prompts-native-proof.ts --binary <release-binary> --platform windows|linux --out <proof-dir>` (script à écrire) ;
  - ancien binaire après la migration 6.
- [ ] **006 :**
  - Windows : `bun scripts/destination-native-proof.ts --binary <absolute-windows-release-binary> --platform windows --out <proof-directory>` (script à écrire), avec les cas UWP Calculator → Settings refusé et `ApplicationFrameHost.exe` jamais affiché ;
  - Linux : la ligne indisponible avec sa raison, et `handy settings output set return_to_dictation_app true --json` qui renvoie `validation_failed`/2 `unsupported_platform` ;
  - ancien binaire : `DatabaseTooFarAhead`.
- [ ] **007 :** reliable paste de Windows et méthodes Linux inchangés ; `handy settings output set paste_method smart_paste --json` refusé (exit 2) et smart paste absent du sélecteur, dans les deux états du verrou.
- [ ] **010 :**
  - Windows : `bun scripts/recovery-native-proof.ts --binary <absolute-windows-release-binary> --platform windows --out <proof-directory> --interactive` : bulle réduite (Copy, Close, Why?, sans glisser) pour app fermée, activation non confirmée et échec de méthode ; premier clic sans voler le focus ; Narrator ; `cargo test --manifest-path src-tauri/Cargo.toml --features destination-return` ;
  - Linux : `bun scripts/recovery-native-proof.ts --binary <absolute-linux-release-binary> --platform linux --out <proof-directory> --interactive` : bulle seulement pour un échec de méthode, sous X11, Wayland et avec `HANDY_NO_GTK_LAYER_SHELL=1` ; Orca ; style `none` ; `handy settings output set show_recovery_bubble_without_text_field true --json` refusé `validation_failed`/2 `unsupported_platform` et la lecture vaut `false` ;
  - CI : le job `windows-check` (`cargo check --all-targets --features destination-return`) et la matrice Linux des quatre combinaisons de `test.yml` doivent passer (le code Linux de la fenêtre n'est compilé qu'en CI).
- [ ] **011 :**
  - Windows : `bun scripts/field-return-native-proof.ts --binary <absolute-windows-release-binary> --platform windows --out <proof-directory> --interactive` : Notepad et un champ de navigateur avec brouillon, curseur au milieu (UI Automation `SetFocus` et `Select`, texte au curseur) ; un champ sans `TextPattern` donne la bulle, jamais un collage ; une app épinglée en arrière-plan ne mémorise rien ; `cargo test --manifest-path src-tauri/Cargo.toml --features destination-return` ;
  - CI : `cargo clippy --target x86_64-pc-windows-msvc -- -D warnings` du harnais `proofs/impl-20261015/windows-check/` (adaptateur compilé seul) et le job `windows-check` de la CI ;
  - Linux : rien n'est mémorisé (aucune destination 006) ; comportement 006 inchangé.
- [ ] **008 :**
  - Windows 10 et 11 24H2 : pas de console au lancement ; dans cmd et PowerShell, sorties, redirections et codes 0, 2 ou 5 (`start /wait`, `Start-Process -Wait -PassThru`) ; pipe bloqué fermé après 5 s ; écriture hors ligne ; `FreeConsole` non fatal en release ; logs debug ; squat du pipe par un autre compte ; `SECURITY_SQOS_PRESENT` ;
  - Linux : socket privé et D-Bus sous X11 et Wayland ; `handy settings set paste_method direct --json` hors ligne, exit 0 ; refus `unavailable`/5 avec un Handy 008 dans une autre session ;
  - ancien binaire : `unavailable`/5, store inchangé ;
  - second compte : UID ou SID étranger refusé ;
  - latence sur un Mac M1 en release (≤ 2000 ms).
