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
