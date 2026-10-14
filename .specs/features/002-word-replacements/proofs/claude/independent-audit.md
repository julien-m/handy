# Audit indépendant — 002-word-replacements (noyau)

- **Date :** 2026-10-13
- **Périmètre :** `git diff dev..HEAD` = ace6c33 (modèle, validation), 73e257a (format, aperçus), 69c79c9 (moteur, intégration au pipeline), sur la branche `feature/002-word-replacements`.
- **Mode :** lecture seule. Aucun code, test ou commit modifié. Une reproduction jetable a été placée hors du repo, dans le scratchpad (copie des modules `unicode`, `matcher`, `validation`, `model`, `exchange`).
- **Sources lues :** `AGENTS.md`, `.specs/constitution.md`, `.conventions/index.md`, `code-conventions/general.md` et `rust.md`, `spec.md` (AC-002, AC-004 à AC-007), `plan.md`, `progress.md`, `implementation.md`.

## Exécution

| Commande (`CARGO_TARGET_DIR=…/.shared-target`) | Résultat |
|---|---|
| `cargo test --manifest-path src-tauri/Cargo.toml --lib word_replacements` | 52 réussis, 0 échec |
| `cargo test … --lib managers::transcription` | 23 réussis, 0 échec |
| `cargo test … --lib audio_toolkit::text` | 48 réussis, 0 échec |
| `cargo test … --lib` (suite complète, contrôle de régression) | 346 réussis, 0 échec, 17 ignorés |
| `cargo test … --test replacement_exchange --test replacement_performance` (debug) | 5 réussis ; le test de performance est ignoré en debug, comme prévu |
| `cargo test --release … --test replacement_performance -- --nocapture` | 1 réussi ; 1000 règles sur 2000 caractères, compilation et application comprises : max 2,02 ms sur 30 échantillons (machine de l'audit, pas un M1) |
| `cargo fmt … -- --check` | OK |

Clippy n'a pas été relancé, pour ne pas reconstruire un profil `check` sur un disque presque plein (13 Go libres).

## Conformité vérifiée (sans constat)

- **AC-002**
  - Les longueurs se comptent en caractères Unicode, après le trim des espaces Unicode : 1–100 pour le texte reconnu, 1–500 pour le remplacement. Un ensemble compte au plus 1000 règles.
  - Interdits dans les deux champs : tab, CR, LF et les sauts de ligne Unicode. Le `;` est interdit seulement dans le texte reconnu, ainsi qu'un `#` en tête.
  - La clé normalisée suit l'ordre : trim, case folding complet C+F non turc, espaces internes réduits à un seul.
  - Validation : tous les participants d'un doublon sont signalés, rien n'est écrit, et les IDs opaques doivent être uniques (`validation.rs`).
- **AC-004**
  - Si la ligne contient une tabulation, la coupe se fait sur la première tabulation, même si un `;` la précède ; sinon sur le premier `;`.
  - Lignes vides et commentaires `#` (après trim) ignorés.
  - En-tête : seule la première ligne de contenu peut en être un, comparée sans casse aux deux en-têtes `;` de la spec.
  - BOM, CRLF et les numéros de ligne d'origine sont gérés. Les guillemets CSV restent du texte littéral.
  - L'export LF avec en-tête canonique fait un aller-retour sans perte pour 1, 100 et 1000 règles, y compris quand une règle ressemble à l'en-tête (`exchange.rs`, `tests/replacement_exchange.rs`).
- **AC-005 (partie pure)**
  - Plafond de 1 000 000 octets, testé à 999 999, 1 000 000 et 1 000 001. Pour l'UTF-8 invalide, l'offset renvoyé est celui du premier octet invalide.
  - Mode merge : on garde l'orthographe, l'ID et la position, et on ajoute à la fin dans l'ordre du fichier.
  - Mode replace : l'ordre de l'entrée est conservé, les IDs sont réutilisés et les suppressions sont listées.
  - Un doublon dans l'entrée est une erreur pour tous ses participants, et toute erreur désactive Apply (`preview.rs`, `materialize.rs`).
  - L'atomicité du commit (revision et persistance) dépend de la feature 008 et n'est pas vérifiable ici ; c'est documenté dans `progress.md`.
- **AC-006**
  - Correspondance insensible à la casse ; le remplacement est inséré tel qu'écrit.
  - Une expansion de folding n'est jamais coupée (ß, ligatures, İ).
  - Une suite d'espaces correspond à toute suite d'espaces non vide.
  - Frontières de mots : les voisins Letter ou Decimal_Number bloquent. L'exemption CJK s'applique côté par côté, décidée par le premier et le dernier caractère du texte reconnu.
  - La ponctuation adjacente est conservée.
  - Les offsets d'origine sont exacts : les segments couvrent le texte source dans l'ordre (`matcher.rs`).
- **AC-007**
  - Une seule passe de gauche à droite ; la clé valide la plus longue gagne. Une égalité est impossible avec des clés uniques, comme le documente `longest_match`.
  - Pas de chaînage. Un ensemble vide laisse le texte inchangé.
  - Ordre dans le pipeline : détection de langue → conversion de script chinois → remplacements → custom words approximatifs, hors des spans protégés → fillers → normalisation. Le LLM vient ensuite, dans `actions.rs`.
  - Les remplacements s'appliquent aussi quand Whisper a reçu les custom words en prompt.
  - Couverture : le batch, la finalisation streaming, le retry d'historique et le mode headless reçoivent tous la capture.
  - Les règles sont capturées au démarrage de l'enregistrement.
- **Régressions :** sans protection, `apply_custom_words_outside_protected` équivaut exactement à `apply_custom_words` sur le texte joint. Les tests existants de custom words, fillers et streaming passent.
- **Robustesse :** aucun `unwrap`, `expect` ou `panic!` dans le code de production de `word_replacements`. Les indexations sont internes et cohérentes. Le pipeline reste fail-open via `catch_unwind`, et un ensemble stocké invalide désactive les remplacements avec un avertissement qui ne contient pas le texte des règles.

## Constats

| # | Emplacement | Sévérité | Problème | Correctif proposé |
|---|---|---|---|---|
| 1 | `src-tauri/src/word_replacements/matcher.rs:226` (`has_boundaries`), `unicode.rs:86`, `unicode_generate.py:99` | major | Les frontières de mots ne testent que Letter et Decimal_Number. Les marques combinantes (Mn, Mc, Me) ne bloquent donc pas une frontière, et une règle remplace un préfixe ou un suffixe à l'intérieur d'un mot dès que le voisin est une voyelle dépendante, un virama ou un accent décomposé. **Reproduit :** `भारत→India` sur `भारतीय संविधान` donne `Indiaीय संविधान` ; `रत→Z` sur `भारत` donne `भाZ` ; `cafe→X` sur `café ok` donne `X́ ok`. C'est une corruption silencieuse de texte pour le hindi, le bengali, le tamoul (langues proposées dans `src/lib/constants/languages.ts`) et pour l'entrée en NFD. Elle contredit le but de la Story 4 (« a rule never corrupts unrelated words ») et le risque principal du QE. Le code suit la lettre d'AC-006 (« letters or digits ») et le plan (« Letter or Decimal_Number ») : c'est une lacune de la spec. | Ajouter General_Category M (Mn, Mc, Me) à la classe qui bloque une frontière, en régénérant `unicode-data.txt` et la provenance. Préciser AC-006 via `spec-refine`, puis ajouter des fixtures : préfixe et suffixe en devanagari, NFD latin avant et après le match. |
| 2 | `src-tauri/src/word_replacements/matcher_tests.rs:16` | minor | Les 40 fixtures tiennent ce qu'elles annoncent. Il leur manque toutefois : (a) des voisins en marque combinante ou dans un script indien, ce qui a masqué le constat 1 ; (b) un texte reconnu qui contient lui-même une suite d'espaces (`cloud  code`) ; (c) le refus d'un match qui commencerait au milieu d'une expansion (`se` contre `ße`). (b) et (c) se comportent correctement dans la reproduction, mais aucun test ne les fixe. | Ajouter ces trois cas à `CASES`. |
| 3 | `src-tauri/src/actions.rs:484-486`, `actions.rs:610-615`, `actions.rs:663-671` | minor | Le câblage de la capture n'a aucun test automatisé : capture au démarrage, `take` à l'arrêt, repli sur les réglages courants, compilation unique partagée par la finalisation streaming et le batch de repli. Les tests appellent directement `post_process_transcription_text` et testent `DictationRuleCapture` isolément. Le harnais `tests/replacement_pipeline.rs` du plan, étape 4, n'a pas été construit ; c'est documenté dans le changelog et `progress.md`. | Extraire en fonction pure la sélection des règles à l'arrêt (capture sinon réglages courants) et la tester. Ajouter le harnais à ports injectés quand 008 fournira le snapshot publié. |
| 4 | `src-tauri/src/word_replacements/exchange.rs:119` et `exchange.rs:133` | nit | Le BOM est retiré deux fois : par `decode_exchange_bytes`, puis par `parse_exchange`. Un fichier qui commence par deux BOM perd les deux (reproduit), alors que le plan dit « remove only the initial BOM ». | Retirer le BOM à un seul endroit, ou documenter que `parse_exchange` le fait pour le texte collé et ne plus le retirer dans `decode_exchange_bytes`. |
| 5 | `src-tauri/src/lib.rs:630-790`, `engine_supervisor/supervisor.rs:1551`, `managers/gguf_meta.rs:484`, `portable.rs:185`, `src/bindings.ts` | nit | Changements hors du noyau 002 : macro `handy_specta_builder!`, test d'export ignoré, formatter `trim_bindings_whitespace` appliqué aussi à l'export debug de production, corrections clippy dans des tests sans rapport, et un diff d'espaces dans `bindings.ts`. Ils sont sans risque fonctionnel (le prédicat `is_none_or` équivaut à l'ancien), mais la constitution demande un « minimal scope ». | Les isoler dans un commit `chore:` séparé, ou les justifier dans le changelog de la feature. |

## Hors périmètre ou non vérifiable dans ce diff

Ces points ne sont pas des constats : ils sont documentés « Waiting for 008 » ou comme preuves de release en attente.

- Commit atomique et revision (AC-001, AC-003, AC-005).
- CLI (AC-008).
- Publication immédiate des règles pour la dictée suivante.
- Observation native de l'insertion et de l'historique.
- Budget mesuré sur un M1.
- Windows et Linux.

## Verdict

**FAIL** : 1 major, 2 minor, 2 nit, 0 blocker.

Le seul major (constat 1) est une lacune de la spec que le code applique fidèlement. Il faut une décision via `spec-refine` sur AC-006 avant de corriger le matcher. Le reste du noyau est conforme, et les tests passent.
