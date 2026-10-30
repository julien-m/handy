# Re-audit indépendant — 002-word-replacements (correctifs du premier audit)

- **Date :** 2026-10-13
- **Périmètre :** `f6d10b5` (spec AC-006 et ledger Clarify) et `3a4d9be` (code, tests, preuves), sur `feature/002-word-replacements` (HEAD `3a4d9be`). Comparaison avec `dev` pour le constat 5.
- **Rôle :** auditeur et relecteur indépendant. Je n'ai écrit ni le code ni la spec. Ni Codex ni le skill `audit` n'ont été utilisés. Aucun commit.
- **Écritures dans le repo :** ce rapport, et `.specs/features/002-word-replacements/.reviews/spec.json`, réécrit par `livespec validate --ingest-review`. Les reçus `conventions verify` vont dans `.specs/conventions/runs/`, qui est ignoré par git.
- **Reproductions et sorties brutes :** `/private/tmp/claude-501/-Users-julienm-projects-handy/6c63dd2f-66f1-4748-8d50-174f39c2881a/scratchpad/repro-002b/`
  - `crate/` : copie des modules de HEAD. `matcher.rs`, `unicode.rs` et `unicode-data.txt` sont identiques octet pour octet ; dans les autres modules, seuls `specta` et `uuid` ont été retirés.
  - `out/` : sorties des commandes.
  - `ucd/` : sources Unicode 17.
- **Sources lues :**
  - `AGENTS.md`, `.specs/constitution.md`, `.conventions/index.md`, `.specs/spec-system.md` ;
  - `spec.md`, `plan.md` (parties concernées), `progress.md`, `implementation.md` ;
  - `proofs/claude/independent-audit.md` ;
  - `.claude/skills/spec-specify/SKILL.md` (Step 6.5) et `livespec/system/review-protocol.md`.

## Verdict

**PASS** : 0 blocker, 0 major, 2 minor, 2 nit.

Les 5 constats du premier audit sont corrigés. Le major (marques combinantes) est résolu et prouvé. Les nouveaux constats sont des lacunes de la spec ou du plan ; le code respecte la lettre d'AC-006 tel que clarifié.

## Partie 1 — vérification des 5 constats

| # | Sév. d'origine | Statut | Preuve |
|---|---|---|---|
| 1 | major | **Corrigé** | `matcher.rs:228-235` utilise `is_word_character` (`unicode.rs:101`), soit Letter, Nd et Mn/Mc/Me. Reproductions sur le code de HEAD : `भारत→India` sur « भारतीय संविधान » → inchangé ; `रत→Z` sur « भारत » → inchangé ; `cafe→X` sur `cafe\u{301} ok` → inchangé ; `cafe→X` sur `café` précomposé → inchangé. Les mots entiers sont bien remplacés : `भारत का` → `India का` ; `e\u{301}` sur `cafe\u{301} e\u{301} ok` → `cafe\u{301} E ok`. 10 fixtures ajoutées (`matcher_tests.rs:257-316`). Le run rouge enregistré montre `Indiaीय`. Voir l'encadré ci-dessous pour la table Unicode 17. |
| 2 | minor | **Corrigé** | Fixtures « rule with several internal spaces » (`matcher_tests.rs:306`) et `se` contre `ße` (`matcher_tests.rs:312`), plus les voisins devanagari et NFD. 50 cas au total (SC-001 ≥ 30). |
| 3 | minor | **Corrigé à la couture de fonction, sans régression** | `rules_for_stop` (`dictation.rs:41`) reprend exactement l'ancien `take(...).unwrap_or_else(...)`, avec le même log debug et une lecture des réglages seulement en repli. `finalize_or_transcribe` (`dictation.rs:63`) reprend exactement l'ancien `match` : `Ok(Some(t))` non vide → stream ; `Ok(_)` → batch ; `Err` → propagé, donc une annulation ne lance jamais de batch. Dans `actions.rs:612-614` et `actions.rs:664-669`, un seul `compile_for_dictation` sert aux deux chemins, et `samples` est déplacé dans la closure batch comme avant. Le contrôle `was_cancelled_since` avant et après la transcription est inchangé (`actions.rs:632`, `actions.rs:695`). Le diff avec `dev` se limite à la capture, à la sélection et au passage du matcher. Le harnais `tests/replacement_pipeline.rs` passe : 7 tests. Reste hors périmètre, et documenté : les ports LLM, historique et sortie, en attente de 008. |
| 4 | nit | **Corrigé** | `decode_exchange_bytes` (`exchange.rs:113-123`) ne touche plus au BOM. `parse_exchange` (`exchange.rs:133`) retire un seul BOM initial. Reproduction : deux BOM en tête → `recognized = "\u{FEFF}A"`, un seul retiré ; texte collé `\u{FEFF}A;B` → `A`. Test : `exchange_tests.rs:171`. |
| 5 | nit | **Corrigé** | `git diff dev..HEAD` est vide sur `engine_supervisor/supervisor.rs`, `managers/gguf_meta.rs` et `portable.rs`. Dans `lib.rs`, il ne reste que `pub mod word_replacements` et le snapshot du mode headless. Dans `src/bindings.ts`, il ne reste que `word_replacements?`, `ReplacementRule` et `RuleId`. |

**Table Unicode 17 et générateur (constat 1)**

- J'ai téléchargé à nouveau les 5 sources UCD 17.0.0 : leurs SHA-256 sont identiques à `PINNED`, et celui de la licence aussi.
- `build_records` du générateur, rejoué hors du repo, produit une sortie identique octet pour octet à `unicode-data.txt`, avec le SHA-256 `07a27a36…` attendu par `unicode-sources.json`.
- Un calcul indépendant de Mn/Mc/Me donne 2543 points (2059 Mn, 471 Mc, 13 Me), égaux à la table `M`.
- Seuls l'en-tête et les lignes `M;` changent ; `L`, `W`, `F` et `K` sont intacts.
- Le générateur est donc reproductible.

**Autres cas explorés (reproduits sur le code de HEAD)**

| Cas | Entrée → sortie | Lecture |
|---|---|---|
| Arabe avec harakat | `كتب→X` sur `كتبَ` → inchangé ; sur `هذا كتب جميل` → `هذا X جميل` | OK |
| Hébreu avec niqqud | `שלום→X` sur `שָׁלוֹם שלום` → `שָׁלוֹם X` | OK (littéral) |
| Tamoul, bengali | `தமிழ்` dans `தமிழ்நாடு`, `বাংলা` dans `বাংলাদেশ` → inchangés | OK |
| Thaï | `ก→X` sur `กิน` → inchangé ; `ไทย` dans `ประเทศไทย` → inchangé ; `ประเทศ ไทย ดี` → remplacé | Pas de fragment. Règles inertes dans un texte thaï sans espaces : c'est un point de spec, voir la partie 2. |
| Coréen | `한국→Korea` sur `한국은` → `Korea은` | Exemption CJK voulue par la spec |
| Règle qui commence par une marque | `\u{301}x→Y` sur `e\u{301}x z` → inchangé ; sur `a \u{301}x z` → `a Y z` | OK |
| Keycap | `1→one` sur `1\u{FE0F}\u{20E3} go` → inchangé | OK (Me bloque) |
| ZWNJ / ZWJ | voir le constat N1 | **minor** |
| Kana + dakuten NFD | voir le constat N3 | nit |
| Emoji + VS16 | voir le constat N4 | nit |

## Nouveaux constats

| # | Emplacement | Sév. | Scénario concret | Correctif proposé |
|---|---|---|---|---|
| N1 | `src-tauri/src/word_replacements/unicode.rs:101`, `matcher.rs:228-235`, `.specs/features/002-word-replacements/spec.md:244` (AC-006) | minor | U+200C ZWNJ et U+200D ZWJ (Cf) ne sont pas des caractères de mot. Une règle remplace donc un morphème joint par ZWNJ à l'intérieur d'un mot persan ou ourdou, deux langues proposées par Handy (`src/lib/constants/languages.ts:45,55`). `می→X` sur `می‌خواهم` (« je veux ») → `X‌خواهم` ; `خواهم→X` sur le même mot → `می‌X` ; `ok→X` sur `ok‍x` → `X‍x`. UAX #29 (WB4) traite ces joiners comme internes au mot. Le code suit la lettre d'AC-006 (L, Nd, M) ; le test `unicode.rs:301` fige d'ailleurs ZWJ comme non-marque. En revanche, la phrase « a rule never replaces a word fragment » et la Story 4 ne sont pas tenues. Le texte produit reste bien formé, ce qui distingue ce cas du major d'origine. | Prendre une décision via `/spec-refine 002-word-replacements` sur AC-006 : traiter Join_Control (U+200C, U+200D) comme caractère de mot lorsqu'il touche le match, ou restreindre explicitement la promesse. Ajouter ensuite au générateur une classe tirée de `PropList.txt` (`Join_Control`, déjà épinglé), et des fixtures persanes ainsi que `क्‍ष`. |
| N2 | `.specs/features/002-word-replacements/plan.md:87` | minor | Le plan dit encore « Unicode Letter or Decimal_Number blocks a boundary », ce qui contredit AC-006 clarifié et le code. Le reçu de revue du plan (`.reviews/plan.json`, `gpt-6.1-sol`) est antérieur à la modification de la spec. Résultat : `livespec validate … --progression implement --model claude-opus-5-5` renvoie `BLOCKED: analyze_not_ready:incomplete`. | Lancer `/spec-refine 002-word-replacements plan` pour aligner la ligne 87 (marques Mn/Mc/Me et exemption CJK), puis `--prepare-review plan` et `--ingest-review --review-kind plan`, puis Analyze. |
| N3 | `matcher.rs:161-162` et `matcher.rs:234` | nit | Un côté exempté CJK ne teste plus rien, même une marque combinante. `か→X` sur `か\u{3099}き` (が en NFD) → `X\u{3099}き` : le dakuten reste orphelin. C'est conforme à l'exemption d'AC-006, mais contraire à « … next to a decomposed (NFD) accent ». Rare en pratique, car l'ASR produit du NFC. | Décision de spec : une marque M juste après le match bloque, même sur un côté exempté. Dans le code, cela revient à ajouter `&& !is_combining_mark(after)` côté fin, plus une fixture. |
| N4 | `unicode-data.txt` (classe `M`, qui inclut les variation selectors) et `matcher.rs:229` | nit | U+FE0F (Mn) après un emoji compte comme caractère de mot. `ok→X` sur `❤️ok` → inchangé, et `❤→love` sur `I ❤️ you` → inchangé. Ce sont des faux négatifs, sans corruption ; une règle écrite avec le VS (`❤️→love`) fonctionne. | Documenter ce comportement dans les Edge Cases, ou exclure `Variation_Selector` de la classe M après décision de spec. Aucune urgence. |

Hors constat : une capture faite au démarrage reste dans la map si l'enregistrement est annulé sans arrêt. Elle est écrasée au démarrage suivant et n'est relue que si un démarrage échoue, et dans ce cas il n'y a pas d'échantillons. Ce comportement existait avant ce correctif et n'a pas d'effet observable.

## Partie 2 — revue indépendante de la spec (procédure LiveSpec)

Procédure suivie : `.specs/spec-system.md`, Step 6.5 du `SKILL.md` de spec-specify et `system/review-protocol.md`, avec `/opt/homebrew/bin/livespec` (validator du checkout dev `~/projects/livespec`).

1. **Gate avant revue :** `--progression implement --model claude-opus-5-5` → `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review;spec_review_model_mismatch:gpt-6.1-sol!=claude-opus-5-5`.
2. **Préparation :** `livespec validate .specs/features/002-word-replacements --prepare-review spec --model claude-opus-5-5` → exit 0, sans erreur. 1 lot (`batch-001`), 61 sections (spec, constitution, project, stack, testing) et 17 exigences, budget de 60 000 caractères.
3. **Analyse :** j'ai lu le prompt complet, puis produit une réponse JSON stricte : 61 sections déclarées revues, 17 conclusions (toutes `covered`), 3 ambiguïtés non critiques, 6 constats (3 warnings, 3 infos), aucun extra-scope, confiance 4/5. Chaque citation a été vérifiée comme sous-chaîne exacte de sa section avant l'ingestion.
4. **Ingestion :** `livespec validate … --ingest-review <bundle> --review-kind spec --model claude-opus-5-5` → exit 0.

**Reçu :** `.specs/features/002-word-replacements/.reviews/spec.json`

| Champ | Valeur |
|---|---|
| `reviewer_model` | `claude-opus-5-5` |
| `complete` | `true` |
| `ready` | `true` |
| `errors` | `[]` |
| `context_hash` | `838c04cea916c7f75253f88e371ac840bd70d67e2f5cb5d9a9e54e0cc38ee329` |
| `spec.md` (hash source) | `679c8db3…af190` |
| SHA-256 du fichier | `a6b05dfe3f1f5be0e09a37be69b0a9a2d8cc5b2c61182c9b31fe9809007fd22d` |

Le fichier est suivi par git et apparaît modifié, non commité.

**Gates après revue :**

- `--progression plan --model claude-opus-5-5` → `Progression: READY`. Le blocage `clarify_not_ready` est levé.
- `--progression implement --model claude-opus-5-5` → `BLOCKED: analyze_not_ready:incomplete`. Ce blocage relève de la revue de plan et d'Analyze, pas de la spec ; voir le constat N2.

**Défauts de spec rapportés dans le reçu** (aucun bloquant) :

| Sév. | Exigence | Défaut | Correctif proposé |
|---|---|---|---|
| warning | AC-006 (`spec.md:244`) | ZWNJ et ZWJ ne sont pas des caractères de mot, alors que la spec promet « never replaces a word fragment ». C'est la contrepartie spec du constat N1. | Ajouter Join_Control, ou restreindre la promesse. |
| warning | AC-006 et Clarifications (`spec.md:412`) | L'exemption CJK annule aussi la règle des marques (kana + U+3099 en NFD). C'est la contrepartie spec du constat N3. | Une marque après le match bloque sur tous les côtés. |
| warning | AC-006 | La justification « written without spaces » ne colle pas à la liste : le hangul est exempté alors que le coréen s'écrit avec des espaces, et le thaï, le lao, le khmer, le birman et le tibétain, écrits sans espaces, ne le sont pas. Les règles sont donc presque inertes dans ces écritures. | Nommer les écritures exemptées et justifier chaque choix ; ajouter une fixture par décision. |
| info | AC-002 | L'unité des longueurs (scalaire ou graphème) n'est pas dite, et la clé ne précise pas l'équivalence canonique : « café » en NFC et en NFD donnent deux clés distinctes qui s'affichent pareil. | Préciser « scalaires Unicode, sans normalisation », ou normaliser en NFC. |
| info | En-tête, Clarifications | Métadonnées périmées (Branch « specification only », Status Planned). La première décision du ledger a un `requirement_id` vide et une clé qui fait un paragraphe entier. | Rafraîchir ces champs via les commandes de cycle de vie. |
| info | Penflow | Le contrat UI reste `[NEEDS CLARIFICATION]`. Cette revue ne certifie pas le design. | Garder la gate Penflow séparée. |
| ambiguïté non critique | AC-004 | Un en-tête séparé par tabulation (`recognized<TAB>replacement`, issu d'un copier-coller de tableur) est importé comme règle. Le plan (`plan.md:73`) l'assume, mais la spec ne le dit pas. | Le dire explicitement, ou accepter aussi l'en-tête tabulé. |
| ambiguïté non critique | AC-005 | « 1 MB » : 1 000 000 ou 1 048 576 octets ? Le plan et le code retiennent 1 000 000. | Écrire « 1 000 000 octets » dans l'AC. |
| ambiguïté non critique | AC-007 | Avec le post-traitement IA, le texte inséré est la sortie du LLM. Or « inserted text … include the replacements » est inconditionnel. Le plan (`plan.md:95`) précise déjà la sémantique par étape. | Limiter la garantie au texte transmis au LLM et au champ `transcription_text` de l'historique. |

Aucune contradiction spec ↔ plan n'est relevée par la revue de spec, qui ne lit pas le plan. La contradiction plan ↔ spec sur AC-006 est le constat N2.

## Tests et commandes

Toutes les commandes ont été lancées avec `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`, en debug, sans build release ni `cargo clean`.

| Commande | Résultat |
|---|---|
| `cargo test --manifest-path src-tauri/Cargo.toml --lib` | **348 réussis, 0 échec, 16 ignorés** (exit 0) |
| `cargo test --manifest-path src-tauri/Cargo.toml --test replacement_exchange --test replacement_pipeline` | `replacement_exchange` : **5 réussis** ; `replacement_pipeline` : **7 réussis** (exit 0) |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | **OK** (exit 0, aucune sortie) |
| `livespec conventions verify --feature repo --json` | **verdict PASS**, run `20261013T153852Z`, 0 blocker, 0 violation sur les fichiers 002 (410 violations hors périmètre et 674 baselinées, toutes antérieures) ; reçu `.specs/conventions/runs/20261013T153852Z/receipt.json` |
| Générateur Unicode rejoué hors du repo (`scripts/regen_check.py`) | Sortie identique octet pour octet, SHA-256 `07a27a36…` |
| Reproduction du matcher (`crate/`, `out/repro-matcher.txt`) | 31 cas : ceux du premier audit et ceux de ce re-audit, détaillés plus haut |
| `livespec validate … --prepare-review spec` puis `--ingest-review` | complete=true, ready=true |
| `livespec validate … --progression plan` | READY |
| `livespec validate … --progression implement` | BLOCKED `analyze_not_ready:incomplete` |

Clippy n'a pas été relancé : la consigne ne le demandait pas, et la preuve stricte existante est dans `proofs/claude-fix1/clippy-*`.
