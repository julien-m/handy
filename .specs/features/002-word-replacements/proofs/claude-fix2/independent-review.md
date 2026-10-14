# Revue indépendante — 002-word-replacements (correctifs du re-audit)

- **Date :** 2026-10-13
- **Périmètre :** `3607812` (spec, plan, ledger Clarify) et `49c0281` (code, tests, données Unicode, preuves), sur `feature/002-word-replacements` (HEAD `49c0281`). Base de comparaison : `96f0600`, ou `3a4d9be` pour le code.
- **Rôle :** relecteur indépendant. Je n'ai écrit ni la spec, ni le plan, ni le code. Ni Codex ni le skill `audit` n'ont été utilisés.
- **Agent :** Claude Code, modèle `claude-opus-5-5`, revue native en lecture seule.
- **Écritures dans le repo :**
  - ce rapport ;
  - `.reviews/spec.json` et `.reviews/plan.json`, écrits par `livespec validate --ingest-review`.
  - Le reçu `conventions verify` est dans `.specs/conventions/runs/20261013T161951Z/`, un dossier ignoré par git.
- **Sorties brutes et reproductions :** `/private/tmp/claude-501/-Users-julienm-projects-handy/6c63dd2f-66f1-4748-8d50-174f39c2881a/scratchpad/rev3/`
  - `out/` : sorties des commandes ;
  - `crate/` : modules de HEAD, avec `specta` et `uuid` retirés ; `matcher.rs`, `unicode.rs` et `unicode-data.txt` sont identiques octet pour octet ;
  - `*-response.json` et `*-bundle*.json` : réponses brutes de revue.
- **Sources lues :**
  - `AGENTS.md`, `.specs/spec-system.md`, `.specs/constitution.md`, `.conventions/index.md` et `code-conventions/general.md` ;
  - les sections revue et Analyze des `SKILL.md` de spec-feature, spec-specify (Step 6.5) et spec-plan (Step 9.7), ainsi que `livespec/system/review-protocol.md` ;
  - dans le dossier de la feature : `spec.md`, `plan.md`, `progress.md`, `implementation.md` et `proofs/claude-fix1/independent-reaudit.md` ;
  - `008-settings-cli/plan.md`, inclus comme contexte dans la revue du plan.

## Verdict

**PASS** : 0 blocker, 0 major, 2 minor, 2 nit.

- Les décisions sur AC-004 à AC-007 sont appliquées fidèlement.
- La table `Join_Control` est reproductible.
- L'exemption CJK pour un signe combinant adjacent fonctionne des deux côtés.
- Aucune régression n'a été trouvée.
- Les constats restants portent sur des tests manquants exigés par le plan, sur la documentation du budget de revue et sur des commentaires.

## État des reçus et de la progression

**Budget de revue.** Avec le budget par défaut (60 000 caractères), la revue du plan se découpe en 51 lots qui se recouvrent, parce que le plan 008 est inclus comme contexte (145 kB uniques au total). J'ai donc repris le budget explicite déjà utilisé par ce pipeline, `--review-max-chars 180000` (voir `logs/2026-10-13.md:5`). Avec ce budget, chaque revue tient en un seul lot.

Le budget fait partie de l'identité des deux reçus. Les gates doivent donc être lancées avec `--review-max-chars 180000` ; sans ce paramètre, elles répondent `BLOCKED` (constat M2).

Le prompt de la revue de spec est identique octet pour octet avec les budgets 60 000 et 180 000. La même réponse brute a donc été ingérée avec les deux budgets ; le reçu actuel est celui du budget 180 000.

| Reçu | `reviewer_model` | complete | ready | errors | `context_hash` | SHA-256 du fichier |
|---|---|---|---|---|---|---|
| `.reviews/spec.json` | `claude-opus-5-5` | true | true | `[]` | `0332c511…1e22` | `b294237f…211b` |
| `.reviews/plan.json` | `claude-opus-5-5` | true | true | `[]` | `b7bb715a…ef9` | `9f11963e…3492` |

**Gates** (`livespec` = `/opt/homebrew/bin/livespec`, en mode dev) :

| Commande | Résultat |
|---|---|
| `validate … --progression plan --model claude-opus-5-5 --review-max-chars 180000` | `Progression: READY` (exit 0) |
| `validate … --progression implement --model claude-opus-5-5 --review-max-chars 180000` | **`Progression: READY`** (exit 0) |
| `validate … --pre-impl --format json --model claude-opus-5-5 --review-max-chars 180000` (Analyze) | exit 0 ; `findings: []`, `semantic_status: covered`, 17/17, 0 CRITICAL et 0 HIGH. Aucun fichier du repo modifié (vérifié par `git status`). |
| `validate .specs/features/002-word-replacements` | exit 0 ; tous les fichiers obtiennent 100/100 |
| `validate … --progression plan` et `--progression implement`, sans budget | `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review` (reçus liés au budget 180 000) |
| `livespec doctor --format json` (lancé par le hook pre-commit) | `OK`, 0 erreur |

**Revue de spec** (lot unique, 61 sections, 17 exigences) : les 17 exigences sont `covered`, avec 1 ambiguïté non critique, 2 warnings et 4 infos, et une confiance de 4/5.

- **Ambiguïté :** un ZWJ ou un ZWNJ voisin d'un côté CJK bloque-t-il le match ?
- **Warnings :**
  - les extenseurs qui ne sont pas des marques (U+FF9E/U+FF9F, jamo NFD) restent séparables d'un côté CJK ;
  - la justification de la liste des écritures exemptées est fausse pour le coréen, et la spec ne dit rien du thaï, du lao, du khmer et du birman.
- **Infos :**
  - une marque placée avant un côté CJK bloque le match, ce qui fait différer NFC et NFD ;
  - le nouveau critère d'en-tête ;
  - l'unité de longueur et l'absence de NFC dans AC-002 ;
  - des métadonnées périmées.

**Revue de plan** (lot unique, 121 sections, 17 exigences) : les 17 exigences sont `covered`, avec 1 ambiguïté non critique (la même), 2 warnings et 2 infos, et une confiance de 4/5.

- **Warnings :**
  - `plan.md:87` exige des tests sur les jamo hangul décomposés, sans oracle ;
  - la promesse de round-trip d'AC-005 n'est pas tenue au-delà de 1 000 000 d'octets, risque déjà documenté dans le plan.
- **Infos :**
  - Summary et Next Action sont périmés ;
  - le staging IPC est une nécessité technique justifiée.

Chaque citation a été vérifiée comme sous-chaîne exacte de sa section avant l'ingestion.

## Revue du diff (`3607812`, `49c0281`)

Identités SHA-256 des fichiers source revus (HEAD `49c0281`, sans modification non commitée) :

| Fichier | SHA-256 |
|---|---|
| `src-tauri/src/word_replacements/exchange.rs` | `ab4915b3…6526` |
| `src-tauri/src/word_replacements/exchange_tests.rs` | `ff3d2eb1…2f2b` |
| `src-tauri/src/word_replacements/matcher.rs` | `78f44316…0e45` |
| `src-tauri/src/word_replacements/matcher_script_tests.rs` | `f21335a2…643d` |
| `src-tauri/src/word_replacements/matcher_tests.rs` | `1b18e5de…f36a` |
| `src-tauri/src/word_replacements/mod.rs` | `b9c467df…d6f` |
| `src-tauri/src/word_replacements/unicode.rs` | `80e927d2…790` |
| `src-tauri/src/word_replacements/unicode-data.txt` | `f44ce766…41a9` |
| `src-tauri/src/word_replacements/unicode-sources.json` | `9beda915…d47b` |
| `src-tauri/src/word_replacements/unicode_generate.py` | `ec38d162…4be9` |

### Points demandés

| Point | Statut | Preuve |
|---|---|---|
| AC-004 : en-tête reconnu par champs, quel que soit le séparateur | **Fidèle** | `exchange.rs:183` (`is_header`) découpe la ligne comme une règle (`split_fields`, `exchange.rs:196` : tabulation d'abord, sinon `;`), puis compare la clé normalisée de chaque champ aux deux paires de `ACCEPTED_HEADERS` (`exchange.rs:20`). Seule la première ligne de contenu est candidate. Reproductions : `recognized ; replacement`, ` recognized \t replacement `, `RECOGNIZED;REPLACEMENT`, `texte_reconnu ; remplacement` et des champs entourés d'espaces insécables (NBSP) sont des en-têtes. `recognized;remplacement`, une paire mélangée, reste une règle, ce qui est conforme aux deux paires de la spec. Une ligne `recognized;replacement` après une règle reste une règle. Tests : `exchange_tests.rs:66`, `:82`, `:90`. |
| AC-005 : 1 MB = 1 000 000 octets, borne incluse | **Fidèle, inchangé** | `exchange.rs:15` et `exchange.rs:118` (`>` strict) ; tests `exchange_tests.rs:194-199` et `replacement_exchange` (999 999 / 1 000 000 / 1 000 001 octets). |
| AC-006 : ZWNJ et ZWJ sont des caractères de mot | **Fidèle** | `unicode.rs:108` : `is_word_character` = L/Nd, ou M, ou J. Fixtures persanes et ZWJ dans `matcher_script_tests.rs`. Reproduction : `می→X` et `خواهم→X` sur `می‌خواهم` → inchangé ; `क्→X` sur `क्‍ष` → inchangé. |
| AC-006 : côté CJK, seule une marque combinante adjacente bloque | **Fidèle, des deux côtés** | `matcher.rs:241` (`is_side_open`) s'applique au voisin avant comme au voisin après. Reproduction : `か→X` sur `がき` → inchangé (côté fin) ; `かい→X` sur `ぎかい` → inchangé (côté début) ; `か→X` sur `あかい` → `あXい`. Contre-épreuve : en remettant l'ancienne exemption (`Some(_) if is_cjk_exempt => true`) dans la copie de reproduction, on obtient `X゙き` et `ぎX`. Le correctif agit donc bien des deux côtés. La fixture « rule ending with an NFD dakuten keeps the CJK exemption » le confirme. |
| AC-007 : la garantie s'arrête au texte transmis à l'IA | **Fidèle** | Changement de spec et de plan seulement (`plan.md:95`). Aucun code n'était à modifier : la sortie du matcher alimente `transcription_text` et le texte transmis au LLM, comme le décrivaient déjà les seams relus au re-audit. |
| Table `Join_Control` reproductible | **Oui** | J'ai rejoué `build_records` hors du repo sur les 5 sources UCD 17.0.0, dont les SHA-256 sont égaux à `PINNED`. La sortie est identique octet pour octet à `unicode-data.txt` (`f44ce766…41a9`, égal à `generated_sha256` du manifeste), et deux exécutions donnent la même sortie. Un calcul indépendant de `Join_Control` dans `PropList.txt` donne {U+200C, U+200D}, égal à la table `J`. Le diff de la table se limite à l'en-tête et à `J;200C;200D`. |
| Effet de bord du nouveau critère d'en-tête | **Acceptable** | Voir ci-dessous. |
| Découpage des tests | **Cohérent** | `matcher_tests.rs` (352 lignes) et `matcher_script_tests.rs` (104 lignes) contiennent 42 + 16 = 58 cas. `all_cases()` les parcourt tous, dans `match_every_exact_fixture` comme dans l'invariant de segments. Le nombre affiché dans `implementation.md` est exact. Voir le nit N2. |
| Non-régression | **OK** | Voir les tests. Le test `treat_tab_or_spaced_header_as_ordinary_rule` a été remplacé intentionnellement, conformément à la décision AC-004. |

**Effet de bord du critère d'en-tête.** Le cas `recognized ; replacement` en première vraie règle est acceptable :

1. Avant ce commit, la forme exacte `recognized;replacement` était déjà un en-tête. La règle « recognized → replacement » en première ligne était donc déjà absorbée ; seules les variantes avec espaces ou tabulation sont nouvelles.
2. La ligne n'est pas perdue en silence : la preview l'affiche comme `ignored` (header). En mode replace, une règle existante de clé `recognized` apparaîtrait comme `delete` dans la preview, et Apply reste un geste explicite.
3. L'export et l'éditeur en masse écrivent toujours l'en-tête canonique en ligne 1, et une ligne identique plus loin reste une règle. Les round-trips SC-002 ne sont pas touchés.
4. Une telle règle (remplacer le mot « recognized » par « replacement ») n'a pratiquement aucune valeur.

Correctif facultatif : une phrase dans les Edge Cases (« a first content line equal to a header pair is always a header »), à ajouter lors de la prochaine révision de spec.

### Constats

| # | Sév. | Emplacement | Scénario | Correctif |
|---|---|---|---|---|
| M1 | minor | `.specs/features/002-word-replacements/plan.md:87` ; absence de fixture dans `src-tauri/src/word_replacements/matcher_script_tests.rs` | Le plan exige de « test … halfwidth kana and decomposed hangul jamo ». Aucune fixture ne couvre les jamo NFD, ni les marques sonores demi-chasse U+FF9E/U+FF9F (`grep 1112\|1161\|11AB\|FF9E` ne trouve rien). Comportement réel, reproduit : `하`(NFD `하`)`→X` sur `한` en NFD → `Xᆫ` ; `ｶ→X` sur `ｶﾞｷ` → `Xﾞｷ`. Le comportement est conforme à la lettre d'AC-006 (Lo et Lm sont des caractères de base), mais il n'est ni figé ni décidé. Ce sont les questions ouvertes 2 et 3. | Après décision sur ces questions, ajouter deux fixtures qui figent le comportement choisi, dans `matcher_script_tests.rs`. |
| M2 | minor | `.specs/features/002-word-replacements/progress.md:51` | Le progress donne la commande de gate `--progression implement --model claude-opus-5-5` sans budget. Les reçus valides sont liés à `--review-max-chars 180000` ; la commande telle qu'elle est écrite répond `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review`. Un orchestrateur qui la relance conclurait à tort que la revue manque. | Noter dans `progress.md` (et dans la prochaine entrée du `pipeline.md`) que les reçus et les gates utilisent `--model claude-opus-5-5 --review-max-chars 180000`, et propager ce budget à `pipeline update` et au goal d'implémentation. |
| N1 | nit | `src-tauri/src/word_replacements/matcher.rs:238-245` | Le commentaire dit « A CJK side … ignores neighbouring base characters only ». En réalité, le code ignore tout sauf les marques combinantes, y compris ZWJ et ZWNJ (Cf, qui ne sont pas des caractères de base au sens Unicode D51). Reproduction : `か→X` sur `か‍き` → `X‍き`. La spec ne tranche pas ce cas (ambiguïté non critique, dans les deux reçus). | Trancher dans la spec. Si les joiners doivent bloquer, ajouter `\|\| data.is_join_control(character)` dans la branche CJK. Sinon, écrire dans le commentaire « ignores every neighbour except a combining mark ». |
| N2 | nit | `src-tauri/src/word_replacements/matcher_script_tests.rs:1-3` et `matcher_tests.rs:168-210` | L'en-tête du module annonce « Script-boundary AC-006 fixtures », alors que les cas de frontière Han, kana, hangul, demi-chasse et CJK d'un seul côté restent dans `matcher_tests.rs`. Le découpage suit en fait le critère « marques et joiners », pas le critère « écriture ». | Reformuler le doc-comment (« combining-mark and join-control boundary fixtures »), ou déplacer les cas CJK. |

Observation sans constat : côté CJK, une marque placée *avant* le match bloque, alors qu'elle appartient au caractère précédent. `かい→X` remplace dans `ぎかい` en NFC, mais pas en NFD. C'est conforme à la décision `cjk-exemption-applies-to-base-characters`, et c'est un faux négatif, jamais une corruption. Je l'ai signalé en info dans le reçu de spec.

Hors du diff, mais exigé par le hook `after-implement-step` : ce hook demande d'enregistrer le rapport de revue native sous `checks/`. La consigne de cette mission limite mes écritures à ce chemin `proofs/claude-fix2/`. Le rapport reste donc ici ; l'orchestrateur peut le référencer depuis `checks/` s'il le souhaite.

## Questions ouvertes — recommandations

| Question | Recommandation | Argument |
|---|---|---|
| Le coréen est exempté, contrairement au thaï, au lao, au khmer et au birman | **Accepter et documenter** (Edge Cases, lors de la prochaine révision de spec). Aucun changement de code maintenant. | Pour le thaï, le lao, le khmer et le birman, l'absence d'exemption produit seulement des faux négatifs, sans corruption. Les exempter ferait matcher des fragments à l'intérieur des mots sans segmentation par dictionnaire (ICU), ce qui introduirait le risque principal de la feature. Pour le coréen, l'exemption est utile, car `한국은` → `Korea은` traite la particule collée. Elle permet aussi des fragments (`국→X` sur `한국은` → `한X은`). La justification « written without spaces » est fausse pour le coréen et doit être remplacée par « particles attach to the noun ». Une évolution possible relève d'une décision produit : n'exempter le hangul que du côté fin, où se collent les particules. |
| U+FF9E/U+FF9F : `ｶ` matche la base de `ｶﾞ` | **Accepter et documenter maintenant ; corriger lors de la prochaine révision de spec, avec la question des jamo.** | C'est une vraie séparation de caractère (`Xﾞ`), de la même classe que N3, mais U+FF9E est Lm. Corriger exige donc d'abord une décision de spec, qui invaliderait encore les deux reçus, pour une entrée que les moteurs ASR de Handy ne produisent pas (sortie pleine chasse NFC). Le correctif est peu coûteux : U+FF9E/U+FF9F ont `Other_Grapheme_Extend`, présent dans `PropList.txt`, déjà épinglé. La règle la plus propre : d'un côté CJK, un match ne commence ni ne finit à l'intérieur d'un cluster de graphèmes étendu (UAX #29). Elle couvrirait N3, FF9E/FF9F et les jamo. |
| Jamo hangul en NFD | **Accepter et documenter.** Corriger éventuellement avec la règle de cluster ci-dessus. | Le cas exige à la fois un texte et une règle en jamo conjoints. Une règle saisie en NFC (`한`) ne matche jamais un texte NFD, puisque la spec exclut la normalisation : c'est un faux négatif. Une règle saisie en NFD est improbable avec les méthodes de saisie coréennes. Le risque est faible ; une fixture doit tout de même figer le comportement retenu (constat M1). |

## Tests et commandes

Toutes les commandes cargo ont été lancées avec `CARGO_TARGET_DIR=/Users/julienm/projects/handy-wt/.shared-target`, en debug, sans build release ni `cargo clean`.

| Commande | Résultat |
|---|---|
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | **OK** (exit 0, aucune sortie) |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib` | **351 réussis, 0 échec, 16 ignorés** (exit 0) |
| `cargo test --manifest-path src-tauri/Cargo.toml --test replacement_exchange` | **5 réussis** (exit 0) |
| `cargo test --manifest-path src-tauri/Cargo.toml --test replacement_pipeline` | **7 réussis** (exit 0) |
| `cargo clippy … --lib --bins --test replacement_exchange --test replacement_pipeline --test replacement_performance -- -D warnings` | exit 0 |
| `cargo clippy … --lib --tests -- -D warnings`, avec les 4 lints de base de `dev` autorisés | exit 0 |
| `livespec conventions verify --feature repo --json` | **PASS**, run `20261013T161951Z`, 0 blocker, 0 violation sur les fichiers 002 (410 violations hors périmètre et 674 dans la baseline) |
| Générateur Unicode rejoué hors du repo (`scripts/regen_j.py`) | Identique octet pour octet et déterministe ; table `J` égale au calcul indépendant |
| Matcher et parser de HEAD dans `crate/` (`out/` : 20 cas matcher, 10 cas d'en-tête) | Résultats cités plus haut ; contre-épreuve de l'exemption CJK rouge avec l'ancienne logique |
| `livespec validate … --prepare-review spec\|plan`, puis `--ingest-review` (`--model claude-opus-5-5 --review-max-chars 180000`) | Deux reçus avec complete=true et ready=true |
| `livespec validate … --progression implement --model claude-opus-5-5 --review-max-chars 180000` | **READY** |
| `livespec validate … --pre-impl --format json` (Analyze) | exit 0, aucun constat |
| `livespec doctor --format json` | OK |
