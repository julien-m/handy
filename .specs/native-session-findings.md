# Séance de preuves natives — constats (2026-10-28)

Builds : `~/handy-tests/handy-*.app`, commit `fc362e37`. Sauvegarde : `~/handy-tests/backup-20261028-1504`.

| # | Feature | Constat | Gravité | Suite |
|---|---|---|---|---|
| N1 | 001 | Le toast « input lost » (perte du micro pendant une dictée) ne s'affiche que dans la fenêtre des réglages. Fenêtre cachée (cas normal, Handy dans la barre de menus) : l'utilisateur ne le voit pas. Il devrait apparaître sur l'indicateur, comme les avis 005/007. | Moyenne (UX) | Correctif après la séance |
| N2 | 001 | La barre « Modifications non enregistrées » est en bas de la page Son, hors écran : sans défiler, on ne sait pas qu'il faut enregistrer. Clavier et glisser OK. | Moyenne (UX) | Décision Julien : barre compacte et collante en bas de la zone visible, affichée seulement s'il y a une modification, avec uniquement les boutons « Abandonner » et « Enregistrer » (pas de texte explicatif). |
| N3 | 001 | Conflit : bannière « Réglages modifiés ailleurs » correcte et Enregistrer inactif, mais la bannière est en haut de la page alors que la barre d'enregistrement est en bas : on ne la voit pas sans défiler. | Faible (UX) | Avec N2 : afficher l'état de conflit dans la barre collante (Recharger à la place d'Enregistrer) |
| N4 | transverse | Fenêtre des réglages : deux barres de défilement. La barre extérieure fait défiler toute la fenêtre (barre latérale, contenu, pied) et peut la vider entièrement (zone blanche). Seule la zone de contenu doit défiler. | Haute (régression visible) | À corriger ; vérifier si présent sur `dev` avant le programme |
| N5 | 002 (et transverse ?) | Focus clavier perdu (retombe sur la page) après « Terminer la modification » d'une règle, après Enregistrer et après la fermeture des dialogues Enregistrer/Abandonner et import. | Faible (clavier) | Rendre le focus au déclencheur ou à un point stable |
| N6 | 002 | L'alerte d'import refusé (fichier non UTF-8) s'affiche au-dessus du titre « REMPLACEMENTS », donc visuellement dans la section Vocabulaire. | Faible (UX) | La placer sous le titre de la section Remplacements |
| N7 | 003 | Limite du vocabulaire à 500 termes (hypothèse de spec non tranchée). | Décision | Décision Julien : 5 000 termes, avec test de performance de la correction approximative (< 50 ms par dictée) ; `spec-refine` 003 |
| N8 | 004 | Ajout d'un raccourci : après la capture (« Right Command »), la puce brouillon a exactement l'aspect d'un raccourci enregistré, et la validation se fait en recliquant le même bouton « Ajouter un raccourci », dont le libellé ne change pas. Julien a cru l'ajout fait, puis Reset l'a effacé. Une fois validé (2ᵉ clic), le raccourci est bien enregistré (CLI : `command_right`). | Haute (UX bloquante) | Brouillon visuellement distinct (pointillés/« non enregistré ») + boutons explicites « Enregistrer » / « Abandonner » à la place de « Ajouter un raccourci » pendant le brouillon |
| N9 | 004 | Faux conflit : avec « Right Command » (modificateur seul, côté droit), ajouter « Left Command + J » est refusé (« déclencherait aussi Raccourci de Transcription »). `shortcut/conflicts.rs` fusionne gauche/droite pour la comparaison, alors que HandyKeys n'enregistre que le côté droit. | Moyenne | Comparer les côtés réellement enregistrés : sans côté = les deux ; gauche et droite distincts ne se chevauchent pas |

Note N9 (correctif `fix/004-side-overlap`) : spec-refine 004 à faire : AC-004 côté des modificateurs seuls (sans côté = les deux côtés ; gauche et droite distincts ne se chevauchent pas ; l'égalité ignore toujours les côtés).
- N9 corrigé et mergé (fix/004-side-overlap). Suivis : conflicts.rs à 500 lignes (sortir les tests) ; spec-refine 004 AC-004 (côtés, asymétrie Duplicate/overlap, cas des deux Cmd tenues).
| N10 | transverse (onboarding) | Écran « Autorisations requises » : après un clic sur « Accorder l'autorisation », si macOS n'affiche pas sa demande (déjà montrée une fois), le bouton reste en attente indéfiniment, sans réessayer ni ouvrir Réglages Système. Cas typique : binaire re-signé (build locale) dont la case Accessibilité reste cochée pour l'ancienne signature. Variante observée : autorisation accordée pendant que Handy tourne → « En attente… » en boucle ; un redémarrage de Handy la prend en compte. | Faible | Ajouter « Ouvrir les Réglages Système » et une re-vérification périodique / au retour de focus |

## N11 — Collage intermittent dans une page Google (mode IA) sous Chrome (transverse, moyenne, cause à établir)

- Observé par Julien le 2026-10-28 : dans le champ « Posez une question » de Google (mode IA, onglet de ~700 Mo), une dictée sur deux à trois sur quatre n'insère rien ; Codex, Claude et une page de test locale dans le même Chrome marchent.
- Côté Handy, chaque collage est déclaré réussi (`Text pasted successfully`, Cmd+V résolu en keycode 9, ~225 ms). Le raccourci n'est pas en cause (même échec avec Commande droite et Commande gauche + J ; Cmd+J est bien bloqué par HandyKeys).
- Page de test locale (`paste-test.html`) : Chrome reçoit Meta, V, puis l'événement `paste`, et le texte est inséré.
- Méthode Direct essayée (révision 25) : « marche à peu près », toujours intermittent.
- Pistes : focus du champ perdu par la page (re-rendu React/streaming), page occupée qui traite Cmd+V en retard, collage intercepté par la page. La relecture du code montre aussi une fenêtre de 60 ms (`paste_delay_after_ms`) avant la restauration du presse-papiers, défaut d'origine que la 007 corrige ; elle n'explique pas un champ vide (on verrait l'ancien contenu).
- Suite : trace console (`keydown`/`paste`/`beforeinput`/`focusin|out` + `active=`) sur un échec réel, puis correctif ; remettre `paste_method` à `ctrl_v` après l'essai.

## N12 — Plantage du collage intelligent : `CFRelease(NULL)` sur le thread principal (007, haute)

- Build `handy-all-gates.app` (verrous 006 + 007 ouverts, commit `8761d748`), 2026-10-28 18:39:41, dictation 3 vers Chrome (page Google).
- Log : `Focused element classified Editable`, deux lignes `no actor reply` (journalisées à la construction des issues synthétiques, avant tout envoi : log trompeur), `snapshot saved 1 types (46 bytes)`, puis arrêt.
- Rapport : `EXC_BREAKPOINT`, `*** CFRelease() called with NULL ***`, thread principal, depuis un job de la file principale (`_dispatch_main_queue_drain`) ; binaire sans symboles (`strip = true`). Copie : `.specs/features/007-smart-paste/proofs/native-macos-migration/crash-cfrelease-null.ips`.
- Contournement : `paste_method` remis à `ctrl_v` (révision suivante).
- Cause : `CfOwned::from_create_rule` construisait `Self(NULL)` via `then_some` (évaluation immédiate) ; `TISCreateInputSourceList` renvoie NULL quand la disposition ABC n'est pas activée (seule French l'est) pendant la bascule AC-011, d'où `CFRelease(NULL)`. Prouvé par désassemblage du binaire planté.
- Corrigé : `2da24d68` (`CfOwned` en `NonNull`), log `no actor reply` émis seulement sans réponse ; mergé dans dev. Reste : validation native (French seule, puis avec ABC).

## N13 — En mode Auto, un appui court arrête la dictée aussitôt démarrée (transverse 001/004, haute)

- Log `handy-all-gates` 17:49:13 et 17:51:21 UTC : `Pressed`, `Released`, `TranscribeAction::start completed in 355 ms` (resp. 307 ms), puis `TranscribeAction::stop` immédiat et `Recording produced no audio samples`. Quand le démarrage prend 50 à 143 ms, le même appui court verrouille bien l'enregistrement.
- Cause : `transcription_coordinator::on_release` mesure la durée d'appui avec l'instant de *traitement* du relâchement (`now`), pas celui de l'événement clavier. Le relâchement est traité après la fin de `start` ; si `start` dure plus que `hold_threshold_ms` (300 ms), un tap est lu comme un maintien et la dictée s'arrête (comportement « comme Annuler » rapporté par Julien).
- Facteurs aggravants : ouverture du flux par la priorité des micros (001, micro Bluetooth OpenDots), capture de destination (006), charge CPU des compilations en parallèle.
- Correctif attendu : horodater appui et relâchement à la réception de l'événement (thread HandyKeys / Tauri) et mesurer avec ces instants ; test de non-régression avec un `start` lent.

## N14 — Transcription et collage lents pendant la séance (constat d'environnement, faible)

- Transcriptions à 3,3–4,5× temps réel (1 à 2 s) au lieu de 20–40× habituellement, pendant que trois agents compilent Rust en parallèle (charge système ~32). Le collage lui-même reste ~230 ms.
- Pas de défaut Handy identifié ; à revérifier une fois les compilations terminées. Si la lenteur persiste : mesurer `start` (ouverture du micro) et la capture de destination 006.

## Validation native de la build intégrée (2026-10-28, `handy-all-gates.app`, commit `21bc07aa`, signée)

- **N11, N12, N13 validés par Julien** en usage réel : collage intelligent réactivé (révision 31), disposition French seule, dictées dans Chrome (page Google comprise) sans plantage ni échec ; log `[smart-paste] dictation N: ReadObserved via ClipboardPaste`, `Insertion finished as InsertionAttempted` en 320–375 ms ; appuis courts `Tap (…): recording locked on until the next press`.
- Tests intégrés sur dev : Rust `cargo test --lib` 1 067 (verrous ouverts) et 1 061 (fermés) réussis ; 16 suites bun, 63 Playwright, tsc, lint, format, traductions à 0.

## N15 — Horodatage HandyKeys pris au drain de la file, pas à l'événement clavier (004, faible)

- Log : `Tap (107 µs)`, `Tap (665 µs)` pour des appuis humains (~100 ms) : appui et relâchement reçoivent un `received_at` presque identique parce que le thread HandyKeys les vide de sa file ensemble.
- Effet : la durée d'appui est quantifiée par l'intervalle de drain ; sans conséquence avec un seuil de 300 ms tant que le drain est fréquent, mais le commentaire de N13 (« as the backend received it ») est inexact.
- Correctif attendu : utiliser l'horodatage de l'événement fourni par handy-keys / CGEvent s'il existe, sinon horodater dans le callback de l'event tap ; test.

## N16 — Retour à l'application refusé : `ActivationFailed` (006, haute)

- Build intégrée `21bc07aa`, `return_to_dictation_app: true`, smart paste. Scénario de Julien : dictée commencée dans Claude (desktop), passage à Codex, arrêt → rien n'est inséré. Log 18:21:02, 18:22:06, 18:22:17, 18:22:50 UTC : `Transcription not inserted (Some(ActivationFailed)); retained for recovery: true; history: Saved; reason: Unpresentable`. Les dictées terminées dans l'app de départ réussissent (`field_return Verified`, `ReadObserved via ClipboardPaste`).
- Hypothèse : activation coopérative de macOS 14+ (ici macOS 26) — une app accessoire en arrière-plan ne peut plus activer une autre app par `activate`. Décision §3 de la checklist (« si l'activation demande le thread principal (macOS 14+) »).

## N17 — Bulle de récupération jamais présentée : `PanelNotFound` (010, haute) ; erreurs affichées dans la fenêtre des réglages (transverse, moyenne)

- Même log : `recovery::surface Recovery window could not be shown: PanelNotFound` puis `Recovery bubble N could not be presented`, à chaque échec d'insertion.
- Conséquence : le texte n'est récupérable que par l'historique, et l'erreur « Le texte n'a pas pu être collé dans l'application active » part en toast dans la fenêtre des réglages, invisible si elle est fermée (remarque de Julien, même famille que N1).
- Attendu : la bulle 010 près de l'indicateur, avec Copier ; aucune erreur d'insertion réservée à la fenêtre des réglages.

## Validation N16 / N17 (2026-10-29, build `42b9e3aa`)

- **N16 validé** : Claude → Codex fonctionne ; toutes les activations du log sont `Destination activation InFront after 18–92 ms: [Workspace Sent]`, sans refus.
- **N17 validé** : la bulle « Échec de l'insertion » apparaît près de l'indicateur avec Glisser / Copier / Pourquoi ? (`Recovery bubble N presented` en 31–741 ms), sans toast dans les réglages ; dépôt par glisser accepté (`drop accepted`).

## N18 — Retour au champ abandonné dans Codex et les pages web Chrome : budget 50 ms saturé (011, haute)

- Scénarios de Julien : dictée commencée dans Codex ou dans un champ de page web Chrome, puis changement d'app → bulle au lieu du collage. Les champs natifs (ProjectAtlas, BOA, barre d'adresse Chrome) réussissent.
- Log 03:54:14, 03:56:00 : `field_return Diverted(Attempt(NotNoted)) (attempt 51.9 / 55.0 ms)` ; 03:57:14, 03:57:30 : `Diverted(RangeNotVerified) (attempt 50.4 / 48.5 ms)`. Les réussites `Verified` prennent 0,4–21 ms. Chaque échec sature `FIELD_RESTORE_BUDGET` (50 ms) : une app Chromium/Electron fraîchement activée répond aux requêtes AX trop lentement.
- Correctif en cours (`spec-fix` 011).

## N19 — L'indicateur change d'écran quand on change d'app pendant une dictée à destination mémorisée (006/009, moyenne)

- Julien : dictée dans Codex sur l'écran central, passage au navigateur sur l'écran du haut → le live monte sur l'écran du haut. Attendu : il reste sur l'écran de l'app destination jusqu'à la fin.
- Correctif en cours (`spec-fix`).

## N14 bis — Transcription lente pendant les correctifs (2026-10-29, environnement)

- 04:05–04:06 UTC : transcriptions à 0,68–5,6× temps réel (jusqu'à 8 s pour 18 s d'audio) au lieu de 15–20× une heure plus tôt (`Transcription completed in 0.55s for 8.13s`). Charge système 54 : deux compilations Rust de mes agents. Le post-traitement est désactivé ; aucun traitement supplémentaire côté Handy.
- Mesure prise : compilations passées en `nice -n 19` et limitées à 3 jobs.

## N20 — Collage refusé quand l'instantané du presse-papiers arrive après 200 ms : `SnapshotFailed(NoInventory)` (007, moyenne)

- Log 04:05:25 : `[smart-paste] clipboard transaction could not start: SnapshotFailed(NoInventory)` → `InsertionFailed via ClipboardPaste` → bulle. Le helper qui lit le presse-papiers n'a pas livré son inventaire avant `SNAPSHOT_DEADLINE` (200 ms, `smart_paste/snapshot.rs`), sous une charge de 54.
- La règle « ne pas coller si le presse-papiers ne peut pas être sauvegardé » protège le contenu de l'utilisateur, mais un démarrage lent du helper suffit à faire échouer la dictée. Piste : démarrer le helper dès le début de la dictée, ou le garder prêt, pour que l'instantané ne paie pas le lancement du processus ; à traiter par `spec-fix` 007.

## N21 — Parakeet v3 transcrit parfois en anglais alors que la langue choisie est le français (transverse, moyenne)

- Réglages : `selected_model` Parakeet TDT 0.6B v3 (GGUF Q8, transcribe-cpp), `selected_language: "fr"`. Le log affiche `language=Some("fr")` et `Output language evidence: UserSelected("fr")`.
- Cause : dans transcribe-cpp-sys 0.3.1 (`src/arch/parakeet/model.cpp`, `resolve_prompt_id`), la langue demandée n'est utilisée que par les variantes Parakeet conditionnées par prompt (`has_prompt`). Parakeet v3 n'en fait pas partie : il détecte la langue à chaque énoncé et ignore l'indication `fr`. Une phrase courte ou riche en termes anglais (Claude Code, LiveSpec…) peut être décodée en anglais.
- Défaut Handy : l'interface laisse choisir « Français » et le log présente ce choix comme appliqué, alors que le modèle l'ignore. Attendu : signaler que ce modèle détecte la langue lui-même, et proposer un modèle qui respecte la langue choisie (Canary 1B v2, conditionné par la langue, 25 langues européennes ; ou Whisper avec langue forcée).

## N22 — « je ne le vois pas » corrigé en « je LiveSpec » par la correction approximative des mots personnalisés (moyenne)

- Mots personnalisés : `LiveSpec` (entre autres) ; `word_correction_threshold` 0,18.
- Cause (`audio_toolkit/text.rs` `best_ngram_at`, `custom_word_index.rs` `score_key`) : le trigramme « le vois pas » donne la clé `levoispas`, de même Soundex que `livespec` (L121). Levenshtein 5 sur 9 donne 0,556, multiplié par `PHONETIC_WEIGHT` 0,3, soit 0,167 < 0,18 : remplacement. La pondération phonétique laisse passer un écart de plus de la moitié des lettres, et les n-grammes de plusieurs mots courants y sont exposés.
- Correctif attendu : borner la distance brute acceptée malgré la phonétique, et durcir les n-grammes multi-mots, avec ce cas en test de non-régression ; `spec-fix` de la feature qui porte la correction approximative.

## N21 — suite

- Modèle actif passé à Canary 1B v2 (`canary-1b-v2-Q5_K_M.gguf`, chargé en 528 ms sur Metal) le 2026-10-29 à 12:22 UTC ; langue `fr` conservée. Dans transcribe-cpp, Canary conditionne le décodage sur la langue demandée (`canary_target_language`).

## N23 — Le prompt choisi dans l'indicateur n'est pas gardé : chaque dictée simple repart sur « Aucun » (005, moyenne)

- Raccourci simple (`binding: transcribe`) : l'indicateur affiche « Aucun (sans traitement IA) » à chaque dictée ; Julien doit choisir Improve Transcriptions à la main à chaque fois. Conforme au texte actuel (« La dictée simple démarre avec Aucun »), mais pas à l'usage voulu.
- Apple Intelligence fonctionne (dictées 33 et 34, 12:34 UTC) : environ 1 à 2 s de l'arrêt au collage, transcription Canary comprise (0,15–0,26 s). La durée de l'étape IA seule n'est pas loggée.
- Correctif en cours (`spec-fix` 005) : mémoriser le prompt choisi pour les dictées simples suivantes, et logger la durée de l'étape IA.

## N11 — reproduction (2026-10-29, build `be40524b`, champ « Posez une question » de Google en mode IA, Chrome au premier plan)

- 13 dictées pilotées par `say` + `--toggle-transcription`, Entrée automatique coupée le temps des essais :
  - toutes donnent `field_return Verified` et `[smart-paste] ReadObserved` ;
  - seule la **première dictée dans un champ vide** apparaît (essai 2, C, G) ; toutes les suivantes, champ non vide, ne s'affichent pas ;
  - même résultat avec le retour au champ désactivé (C–F), et avec 15 s d'écart entre dictées (G–I) : ni 011 ni le rythme ne sont en cause.
- **Un Cmd+V clavier envoyé par Computer Use reproduit le refus** : il colle dans le champ vide, et ne colle rien quand le champ contient déjà « Mouger. ». La page Google ignore donc les collages suivants ; le presse-papiers contient bien la transcription (`pbpaste`).
- Conclusion provisoire : comportement de la page, pas de Handy. `ReadObserved` ne prouve pas l'insertion : Chrome lit le presse-papiers même quand la page rejette le collage. Reste à confirmer par un Cmd+V manuel de Julien dans le champ non vide.
- Réglages rétablis après les essais : `return_to_dictation_app` true (rév. 47), `automatic_enter` always (rév. 48).

### N11 — mesures instrumentées (2026-10-29, onglet Google mode IA au premier plan, écouteurs `keydown`/`paste`/`beforeinput`/`input` dans la page)

- La conclusion « champ non vide » est **fausse** : Julien a aussi des échecs dans un champ vide.
- À chaque dictée, la page reçoit Cmd, V, puis `paste` avec **le bon texte** et `beforeinput insertFromPaste`, non annulé (`defaultPrevented` false en capture et en remontée), sans aucun changement de focus. Pour les échecs, **aucun `input`** ne suit : Chrome abandonne l'insertion après le `beforeinput`.
- Taux observés :

  | Méthode | Collages passés |
  |---|---|
  | collage intelligent, 60 ms | 1/4 puis 3/6 |
  | collage intelligent, 250 ms | 4/6 |
  | collage intelligent, 1 s | 5/6 |
  | collage classique Cmd+V | 5/6 |
  | Cmd+V par Computer Use sur un presse-papiers inchangé | **6/6** |

- Lecture : l'abandon touche les collages qui suivent une écriture récente du presse-papiers. Le délai réduit le taux d'échec sans l'annuler. La cause exacte côté Chrome reste inconnue.
- Défaut Handy certain : `ReadObserved` est compté comme une réussite alors que le texte n'a pas été inséré. L'échec est donc silencieux : pas de bulle, et l'Entrée automatique part sur un champ vide.
- Correctif proposé (007) : après le collage dans un champ dont la valeur est lisible par l'accessibilité, vérifier que la valeur a changé ; sinon renvoyer le Cmd+V une fois, puis afficher la bulle de récupération, sans jamais envoyer l'Entrée automatique.
- Réglages rétablis après les essais : `automatic_enter` always (rév. 55), `paste_delay_ms` 60, `paste_method` smart_paste.

## N27 — « Épingler » invisible dans le menu du badge (009, haute)

- Julien ne voit aucune entrée « Épingler » dans le menu du badge (capture : une vingtaine d'apps puis « App au premier plan à la fin »).
- Cause probable (`src/overlay/pin-state.ts`, `pinEntries`) : « Pin <destination> » est placé **au-dessus** de la liste des apps. Le menu s'ouvre vers le haut depuis le badge en bas d'écran, donc avec une longue liste l'entrée sort de la vue. Le mode PIN est inutilisable en pratique, alors que la fonction existe.
- Correctif en cours avec la feature des favoris : un pied de menu fixe près du badge (Épingler / Désépingler / App au premier plan à la fin), la liste des apps défilante au-dessus.

## N28 — Vérification après collage : texte collé deux fois dans cmux (007, haute)

- Build `9487b45d`, `paste_verification` actif. Dans cmux (terminal) : `verify dictation 4/5/9: Failed in ~850ms (UnchangedAfterRetry)` → second Cmd+V (doublon) puis bulle. Dans Claude Desktop et la page Google : `Verified in 105–151 ms`.
- Cause : la valeur AX de l'élément du terminal ne reflète pas le collage. Handy conclut à tort à l'échec et renvoie Cmd+V.
- Mesure immédiate : `paste_verification` coupé (rév. 58). Correctif en cours (`spec-fix` 007) : relance seulement pour un champ de texte web (ancêtre `AXWebArea`), relecture juste avant la relance, `Unreadable` sinon.

## Validation 015 et 016 (2026-10-29, build `07b298ad`)

- **015, majuscules intelligentes** : validées par Julien en usage réel (« la majuscule c'est vraiment pas mal, ça marche bien »). Log : `Smart capitalization: Keep` ×5 et `Capitalize` ×1, sans contenu journalisé.
- **016, paragraphes intelligents** : activés. Six dictées courtes donnent `Smart paragraphs left the text unchanged: NoBreak`, comme attendu sous le seuil de 45 mots / 4 phrases. Une dictée longue reste à tester.
- **016 validé en usage réel** : dictée d'environ 130 mots, découpée en 3 paragraphes de 4, 4 et 3 phrases, séparés par une ligne vide, sans phrase coupée (capture de Julien).
- **014, Ctrl gauche « sans Entrée », validé** (2026-10-29, 19:27 heure locale) : dictée 19, `In-dictation key applied … Enter skipped true` puis `Automatic Enter of dictation 19: Suppressed(UserSkipped)`. Les dictées 16–18 et 20, sans Ctrl : `Sent`. La politique `always` est conservée.
