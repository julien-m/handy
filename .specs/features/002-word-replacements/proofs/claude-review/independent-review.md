# Revue indépendante des specs — 002-word-replacements

- **Date :** 2026-10-13
- **Périmètre :** `spec.md` et `plan.md` à HEAD `369c1083254be8e421691e0c5ab30692105fb27d` (`feature/002-word-replacements`), après le correctif `369c108` qui a modifié AC-006, les Edge Cases et le ledger Clarify.
- **Rôle :** relecteur indépendant. Je n'ai écrit ni la spec, ni le plan, ni le code. Ni Codex ni le skill `audit` n'ont été utilisés.
- **Agent :** Claude Code, modèle `claude-opus-5-5` (celui de `.specs/semantic/config.yaml`), revue native en lecture seule.
- **Outil :** `/opt/homebrew/bin/livespec` (mode dev sur `/Users/julienm/projects/livespec`), avec `--model claude-opus-5-5 --review-max-chars 180000` sur toutes les commandes de revue et de gate.
- **Écritures dans le repo :**
  - `.reviews/spec.json`, `.reviews/plan.json`, `.reviews/spec.rounds.json` et `.reviews/plan.rounds.json`, écrits par `livespec validate --ingest-review` ;
  - ce rapport.
- **Sources lues :**
  - `livespec/system/spec-system.md` (Delivery lanes), `review-protocol.md` et `delivery-review.md` ;
  - les sections revue et Analyze des `SKILL.md` de spec-feature, spec-specify et spec-plan ;
  - `spec.md`, `plan.md` et le rapport précédent `proofs/claude-fix2/independent-review.md` ;
  - `matcher.rs`, pour vérifier la question ZWJ/ZWNJ.

## État des reçus

Avant cette revue, les deux gates répondaient `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review`, car le correctif `369c108` avait modifié la spec après les reçus précédents.

| Reçu | Lots | `reviewer_model` | complete | ready | errors | Round | `context_hash` | SHA-256 du fichier |
|---|---|---|---|---|---|---|---|---|
| `.reviews/spec.json` | 1 (62 sections, 17 exigences) | `claude-opus-5-5` | true | true | `[]` | 1 | `d1477804…86ca` | `68450368…5a37` |
| `.reviews/plan.json` | 2 + synthèse (122 sections, 17 exigences) | `claude-opus-5-5` | true | true | `[]` | 1 | `ee2cfa8c…c356` | `52e10505…14a2` |

- **Revue de spec :** les 17 exigences sont `covered`, avec 1 ambiguïté non critique, 3 warnings, 4 infos et une confiance de 4/5.
- **Revue de plan :** même avec un budget de 180 000 caractères, la revue se découpe en deux lots. Le premier contient tout le plan sauf Risks et Next Action ; le second contient ces deux sections, avec la spec et le plan 008 comme contexte. Une synthèse les combine, construite selon `synthesis_prompt`. Dans le lot 2, les exigences que Risks ne traite pas sont marquées `missing`, en citant les sections cherchées ; la synthèse les résout par les citations du lot 1. Le résultat : 17/17 `covered`, 1 ambiguïté non critique, 2 warnings, 3 infos, une confiance de 4/5 et 1 périmètre technique approuvé (le staging IPC privé).
- **Vérification des citations :** avant l'ingestion, chaque citation a été vérifiée comme sous-chaîne exacte de sa section, puis l'appartenance de chaque citation à ses sections d'obligation.

## Gates et progression

| Commande (`--model claude-opus-5-5 --review-max-chars 180000`) | Résultat |
|---|---|
| `validate … --progression plan` | `Progression: READY` (exit 0) |
| `validate … --pre-impl --format json` (Analyze) | exit 0 ; `findings: []`, `semantic_status: covered`, 17/17, 0 CRITICAL et 0 HIGH |
| `validate … --progression implement` | **`Progression: READY`** (exit 0) |
| `livespec delivery lane --feature 002-word-replacements` | **`Delivery lane: FULL`** ; criticality high, blast radius shared (`criticality_high`) |

Analyze n'a modifié aucun fichier : `git status` ne montrait que les reçus `.reviews/`.

## Constats

| # | Sév. | Emplacement | Constat | Correctif proposé |
|---|---|---|---|---|
| W1 | warning | `spec.md` AC-006 (l. 248), Edge Cases (l. 473), `plan.md:87` ; code : `matcher.rs:247` | **Divergence entre la spec et le code sur ZWJ/ZWNJ d'un côté CJK.** La spec limite l'exemption CJK aux « neighbouring base characters only ». ZWNJ et ZWJ sont des caractères de format (Cf), pas des caractères de base. Lue à la lettre, la spec veut donc qu'ils bloquent, puisque AC-006 en fait des caractères de mot. Le code ignore tout voisin sauf une marque combinante (`Some(c) if is_cjk_exempt => !data.is_combining_mark(c)`), et `か‍き` donne `X‍き`. Le commentaire mis à jour en `369c108` décrit ce comportement, mais aucune décision ni fixture ne le fonde. | Trancher (recommandation ci-dessous), puis aligner la spec, le plan, le code et deux fixtures. |
| W2 | warning | `spec.md` AC-004 (l. 238), AC-005 (l. 243), AC-002 | **La promesse de round-trip n'est pas universelle.** AC-004 dit que « every valid rule set round-trips », et AC-005 promet que l'export suivi d'un import replace reproduit les règles. Or l'import est plafonné à 1 000 000 d'octets, alors qu'un jeu valide peut dépasser ce plafond. Par exemple, 1000 remplacements cyrilliques de 500 caractères font environ 1,0 à 1,2 Mo. Il n'y a pas de corruption, car l'import est refusé de façon atomique. Le plan documente ce risque résiduel (`plan.md:77`). | Qualifier AC-005 (« for an exported file within the import limit »), ou faire refuser ou signaler un export au-delà du plafond. |
| W3 | warning | `spec.md` AC-007 (l. 253) | **« Inserted text includes them » n'est pas un oracle exact.** La suppression des fillers et la normalisation s'exécutent après le remplacement, et seule la correction approximative épargne les spans remplacés. Le plan (`plan.md:91`) le dit explicitement : « Protection ends there ». | Écrire dans AC-007 que la garantie est exacte pour la sortie du matcher et pour le texte de l'étape approximative, et que les nettoyages suivants s'appliquent comme à tout texte. |
| I1 | info | `spec.md` QE Analysis (l. 442, 447) et Quality Engineering (l. 452) ; `plan.md:392` | **Cohérence QE.** La criticité est la même partout (High), ce qui donne la voie full. Le vocabulaire diffère : blast radius « Every dictation output » dans QE Analysis, « Shared » dans Quality Engineering et « Shared/Cross-system » dans le plan ; risque « Silent text corruption », « Data » ou « Functional ». « Missing evidence » et « Confidence » (« nothing executed », contrat 008 « not yet specified ») sont périmés. Quality Engineering omet les gates P1 et le budget de 20 ms, sans contradiction. | Aligner QE Analysis et le plan sur Shared/Data, et rafraîchir Missing evidence. |
| I2 | info | `spec.md:445` | Le gate P1 « bulk editor with 1000 rules stays responsive » n'a pas de métrique. | Donner un budget mesurable. |
| I3 | info | `spec.md` AC-002 | L'unité « characters » n'est pas nommée. Le plan la fixe en valeurs scalaires Unicode. | La nommer dans AC-002. |
| I4 | info | `spec.md:15-17`, `plan.md:434` | Métadonnées périmées : « specification only », Status Planned, et Next Action avec `--model=gpt-6.1-sol`. | Les rafraîchir à la finalisation. |

Aucune contradiction bloquante entre la spec et le plan. Aucun AC non testable, hormis l'oracle W3 et la métrique manquante I2.

## Question ouverte : ZWJ ou ZWNJ voisin d'un côté CJK

**Recommandation : le faire bloquer**, comme une marque combinante. Le correctif tient en une ligne dans `is_side_open` : `!(data.is_combining_mark(c) || data.is_join_control(c))`.

Arguments :

1. **C'est déjà la lecture littérale de la spec.** L'exemption porte sur les « base characters only », et AC-006 classe ZWNJ et ZWJ parmi les caractères de mot. C'est le code qui s'en écarte, et le commentaire de `369c108` documente cet écart sans décision Clarify.
2. **Le principe de sûreté de la feature.** Le risque principal est la corruption silencieuse. Bloquer ne produit qu'un faux négatif, alors qu'ignorer le joiner peut couper une séquence jointe. Les décisions N3 (marque à côté d'un côté CJK) et N4 (sélecteur de variante) ont déjà retenu le faux négatif.
3. **La cohérence avec la règle UAX #29 différée.** ZWNJ (Grapheme_Cluster_Break=Extend) et ZWJ (GCB=ZWJ) ne créent pas de frontière avant eux (GB9) : ils appartiennent au cluster précédent, exactement comme une marque. La future règle par cluster les traitera comme des marques ; les faire bloquer dès maintenant évite un second changement de comportement.
4. **Un coût pratique nul.** Les moteurs ASR de Handy ne produisent pas de joiner à côté de kana, de han ou de hangul. Le seul usage courant du ZWJ, les séquences emoji, n'implique pas de côté CJK.

Étapes :

1. Enregistrer la décision avec `persist_clarification_decision`, par exemple `decision:ac-006:cjk-side-join-control-blocks`.
2. Préciser AC-006 : « a combining mark or a zero-width non-joiner or joiner immediately next to the match still blocks it on that side ».
3. Ajouter deux fixtures : `か‍き` avec la règle `か→X` (inchangé) et `き‍か` avec la même règle (inchangé).
4. Rafraîchir les reçus spec et plan.

## Recommandations

1. Trancher W1 dans Clarify (voir ci-dessus). C'est le seul écart réel entre la spec et le code.
2. Lors de la prochaine révision de spec, corriger W2 et W3 par une reformulation, sans changement de code.
3. Aligner la QE (I1). La voie reste full quoi qu'il arrive, puisque la criticité est High.
4. Garder `--review-max-chars 180000` sur toutes les gates. Les reçus sont liés à ce budget ; sans lui, les gates répondent `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review`.

## Empreintes

| Objet | SHA-256 |
|---|---|
| `spec.md` | `33ad772428c22b0c26f139587bdcb888b8a4d0a08cecbf40bc7a151de21335c5` |
| `plan.md` | `4c854f3b604098a90808bfce8b10f61ceb7837055434a1846bacc963987b5000` |
| `008-settings-cli/plan.md` (contexte) | `f95b8b53075ed8a3619f5843b3ac626efc03f9a6cb3806d9b29980ed4ca0356d` |
| `.reviews/spec.json` | `68450368e60c6ce2d9945324ad8f876b525be31be54aa900808a293d86f25a37` |
| `.reviews/plan.json` | `52e105053e530823d7b3cfae179547e4cfcac539f93f2dc36239511c614814a2` |
| `.reviews/spec.rounds.json` | `d542c4a745baa664a672b2d1e29eac2ece0388876e55e9c0d97da6f0d63aa70e` |
| `.reviews/plan.rounds.json` | `7596d153bda397d4939d9b69b4aa8f7d5707cc9e88bd943a3182980090d0e8c2` |
| `context_hash` spec | `d1477804044550eb43bba5a5c9e6be4acb82d8a90347f12279d8a4ccf6f586ca` |
| `context_hash` plan | `ee2cfa8cfd0fe5c6e983a8c03b06b8bd56b867cb490f49b13045da71b43fc356` |

Les réponses brutes sont conservées dans `raw_results` (et `synthesis` pour le plan) des reçus.
